use crossterm::event::KeyCode::{Char, Down, Enter, Esc, Up};
use crossterm::event::{self, Event, KeyEventKind, KeyModifiers};
use ratatui::style::{Color, Modifier, Style};
use ratatui::{DefaultTerminal, Frame};
use ratatui::widgets::{Paragraph, Block};
use ratatui::layout::{Direction, Layout, Constraint};
use ratatui::text::{Line, Span};
use std::{env, io};

use crate::herdr::get_agent_list;
use crate::pi::reload_all_pi;

pub async fn run() -> io::Result<()> {
   let mut terminal = ratatui::try_init()?;

   let app_result = app(&mut terminal).await;

   let restore_result = ratatui::try_restore();

   match restore_result {
       Ok(()) => app_result,
       Err(error) => Err(error),
   }
}

#[derive(PartialEq)]
enum Screen {
    Menu,
    RunningReload,
    ResetPlaceholder,
    ReloadResult(String),
    Error(String)
}

struct AppState {
    selected: usize,
    screen: Screen
}

async fn app(terminal: &mut DefaultTerminal) -> std::io::Result<()> {
    let mut state = AppState {
        selected: 0,
        screen: Screen::Menu
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
                                        let reload_summary = reload_all_pi(&herdr_path, &agents).await;

                                        state.screen = Screen::ReloadResult(format!("{:#?}",reload_summary));
                                    },
                                    Err(error) => {
                                        state.screen = Screen::Error(format!("Failed to get agent list: {}", error));
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
                        },
                        _ => {}
                    },
                    Screen::RunningReload | Screen::ResetPlaceholder => {}
                }
            }
        }
    }
}

fn render(frame: &mut Frame, state: &AppState) {
    let [main_area, footer_area]= Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(vec![
            Constraint::Min(0),
            Constraint::Length(1)
        ])
        .areas(frame.area());

    match state.screen {
        Screen::Menu => {
            let menu_text_options = [
                "Reload all Pi",
                "Reset all Pi",
            ];

            let menu_text = menu_text_options.iter().enumerate().map(|(i, line)| {
                if i == state.selected {
                    Span::styled(format!("> {}", line), Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)).into()
                } else {
                    Span::styled(format!("  {}", line), Style::default()).into()
                }
            }).collect::<Vec<Line<'_>>>();

            let main = Paragraph::new(menu_text)
                .block(Block::bordered().title("Herdr Pi Reloader"));

            let footer = Paragraph::new("↑/k ↓/j select • Enter run • q/Esc quit");

            frame.render_widget(main, main_area);
            frame.render_widget(footer, footer_area);
        },
        Screen::RunningReload => {
            let main = Paragraph::new("Reloading Pi instances...")
                .block(Block::bordered().title("Herdr Pi Reloader"));

            let footer = Paragraph::new("Operation in progess...");

            frame.render_widget(main, main_area);
            frame.render_widget(footer, footer_area);
        },
        Screen::ResetPlaceholder => {
            let main = Paragraph::new("Resetting Pi instances...")
                .block(Block::bordered().title("Herdr Pi Reloader"));

            let footer = Paragraph::new("q/Esc quit");

            frame.render_widget(main, main_area);
            frame.render_widget(footer, footer_area);
        },
        Screen::ReloadResult(ref result) => {
            let mut lines = vec![
                Line::from("Reload Completed"),
                Line::from(""),
            ];

            lines.extend(result.lines().map(|line| Line::from(line)));

            let main = Paragraph::new(lines)
                .block(Block::bordered().title("Herdr Pi Reloader"));

            let footer = Paragraph::new("Enter/q/Esc quit");

            frame.render_widget(main, main_area);
            frame.render_widget(footer, footer_area);
        },
        Screen::Error(ref error) => {
            let main = Paragraph::new(error.as_str())
                .block(Block::bordered().title("Herdr Pi Reloader"));

            let footer = Paragraph::new("Enter/q/Esc quit");
            frame.render_widget(main, main_area);
            frame.render_widget(footer, footer_area);
        },
    }
}
