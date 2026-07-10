use ratatui::{DefaultTerminal, Frame};
use ratatui::widgets::{Paragraph, Block};
use ratatui::layout::{Direction, Layout, Constraint};

pub fn run() -> std::io::Result<()> {
    ratatui::run(app)
}

struct AppState {
    selected: usize
}

fn app(terminal: &mut DefaultTerminal) -> std::io::Result<()> {
    let state = AppState {
        selected: 0
    };

    loop {
        terminal.draw(|frame| render(frame, &state))?;

        if crossterm::event::read()?.is_key_press() {
            break Ok(());
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

    let menu_text_options = [
        "Reload all Pi",
        "Reset all Pi"
    ];

    let menu_text = menu_text_options.iter().enumerate().map(|(i, line)| {
        if i == state.selected {
            format!("> {}", line)
        } else {
            format!("  {}", line)
        }
    }).collect::<Vec<String>>().join("\n");

    let main = Paragraph::new(menu_text)
        .block(Block::bordered().title("Herdr Pi Reloader"));

    let footer = Paragraph::new("Press any key to quit");

    frame.render_widget(main, main_area);
    frame.render_widget(footer, footer_area);
}
