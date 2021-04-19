use std::io::{self, Write};

use crossterm::{
    cursor::Show,
    event::{read, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use tui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Layout},
    style::*,
    text::Spans,
    widgets::{Block, Borders},
    Terminal,
};

use tui_wrapper::select::*;

fn main() {
    enable_raw_mode().unwrap();
    execute!(io::stdout(), EnterAlternateScreen, EnableMouseCapture).unwrap();

    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend).unwrap();
    terminal.clear().unwrap();

    let mut select = SelectForm::new("Select");

    // let mut state = SelectState::default();
    loop {
        terminal
            .draw(|f| {
                select.update_chunk(f.size());
                select.render(f);

                // let chunk = Layout::default()
                //     .constraints([Constraint::Percentage(100)])
                //     .margin(20)
                //     .split(f.size());

                // let block = Block::default()
                //     .borders(Borders::ALL)
                //     .border_style(Style::default().fg(Color::Gray))
                //     .alignment(Alignment::Left)
                //     .title_offset(1)
                //     .title("Select");

                // let items: Vec<SelectItem> = ["Item", "Item", "Item"]
                //     .iter()
                //     .map(|item| SelectItem::new(item.to_string()))
                //     .collect();

                // let select = Select::new(items).block(block);

                // f.render_stateful_widget(select, chunk[0], &mut state);
            })
            .unwrap();

        match read().unwrap() {
            Event::Key(key) => match key.code {
                KeyCode::Char('q') => break,
                // KeyCode::Char('k') => state.select(Some(0)),
                // KeyCode::Char('j') => state.select(Some(1)),
                KeyCode::Char(_) => {}
                _ => {}
            },
            _ => {}
        }
    }
    execute!(
        io::stdout(),
        LeaveAlternateScreen,
        DisableMouseCapture,
        Show
    )
    .unwrap();
    disable_raw_mode().unwrap();
}
