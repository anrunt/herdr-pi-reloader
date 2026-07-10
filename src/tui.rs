use ratatui::{DefaultTerminal, Frame};
use ratatui::widgets::{Paragraph, Block};
use ratatui::layout::{Direction, Layout, Constraint};

pub fn run() -> std::io::Result<()> {
    ratatui::run(app)
}

fn app(terminal: &mut DefaultTerminal) -> std::io::Result<()> {
    loop {
        terminal.draw(render)?;

        if crossterm::event::read()?.is_key_press() {
            break Ok(());
        }
    }
}

fn render(frame: &mut Frame) {
    let [main_area, footer_area]= Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(vec![
            Constraint::Min(0),
            Constraint::Length(1)
        ])
        .areas(frame.area());


    let main = Paragraph::new("TUI placeholder")
        .block(Block::bordered().title("Herdr Pi Reloader"));

    let footer = Paragraph::new("Press any key to quit");

    frame.render_widget(main, main_area);
    frame.render_widget(footer, footer_area);
}
