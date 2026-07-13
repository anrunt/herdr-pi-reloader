use crossterm::event::KeyCode::{Char, Down, Enter, Esc, Up};
use crossterm::event::{self, Event, KeyEventKind, KeyModifiers};
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::{DefaultTerminal, Frame};
use std::{env, io};

use crate::herdr::get_agent_list;
use crate::pi::{ReloadSummary, reload_all_pi};
use crate::render::{
    render_error, render_menu, render_reload_result, render_reset_placeholder,
    render_running_reload,
};

pub async fn run() -> io::Result<()> {
    let mut terminal = ratatui::try_init()?;

    let app_result = app(&mut terminal).await;

    let restore_result = ratatui::try_restore();

    match restore_result {
        Ok(()) => app_result,
        Err(error) => Err(error),
    }
}

enum Screen {
    Menu,
    RunningReload,
    ResetPlaceholder,
    ReloadResult(ReloadSummary),
    Error(String),
}

struct AppState {
    selected: usize,
    screen: Screen,
}

async fn app(terminal: &mut DefaultTerminal) -> std::io::Result<()> {
    let mut state = AppState {
        selected: 0,
        screen: Screen::Menu,
    };

    loop {
        terminal.draw(|frame| render(frame, &state))?;

        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    Char('q') | Esc => break Ok(()),
                    Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => break Ok(()),
                    _ => {}
                }

                match &state.screen {
                    Screen::Menu => match key.code {
                        Char('j') | Down => state.selected = 1,
                        Char('k') | Up => state.selected = 0,
                        Enter => {
                            if state.selected == 0 {
                                state.screen = Screen::RunningReload;
                                terminal.draw(|frame| render(frame, &state))?;

                                let herdr_path_result = env::var("HERDR_BIN_PATH");

                                let herdr_path = match herdr_path_result {
                                    Ok(path) => path,
                                    Err(_error) => String::from("herdr"),
                                };

                                match get_agent_list(&herdr_path).await {
                                    Ok(agents) => {
                                        let reload_summary =
                                            reload_all_pi(&herdr_path, &agents).await;

                                        state.screen = Screen::ReloadResult(reload_summary);
                                    }
                                    Err(error) => {
                                        state.screen = Screen::Error(format!(
                                            "Failed to get agent list: {}",
                                            error
                                        ));
                                    }
                                };
                            } else {
                                state.screen = Screen::ResetPlaceholder;
                            }
                        }
                        _ => {}
                    },
                    Screen::ReloadResult(_) | Screen::Error(_) => match key.code {
                        Enter => {
                            break Ok(());
                        }
                        _ => {}
                    },
                    Screen::RunningReload | Screen::ResetPlaceholder => {}
                }
            }
        }
    }
}

fn render(frame: &mut Frame, state: &AppState) {
    let [main_area, footer_area] = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(vec![Constraint::Min(0), Constraint::Length(1)])
        .areas(frame.area());

    match &state.screen {
        Screen::Menu => render_menu(frame, main_area, footer_area, state.selected),
        Screen::RunningReload => render_running_reload(frame, main_area, footer_area),
        Screen::ResetPlaceholder => render_reset_placeholder(frame, main_area, footer_area),
        Screen::ReloadResult(result) => render_reload_result(frame, main_area, footer_area, result),
        Screen::Error(error) => render_error(frame, main_area, footer_area, error),
    }
}

