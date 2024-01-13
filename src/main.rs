mod app;
mod tui;
mod ui;

use std::io;

use anyhow::Result;
use app::App;
use crossterm::event::{self, Event, KeyCode};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    Terminal,
};
use tui::Tui;

use crate::ui::ui;

fn main() -> Result<()> {
    let backend = CrosstermBackend::new(io::stdout());
    let terminal = Terminal::new(backend)?;
    let mut tui = Tui::new(terminal);

    tui.enter()?;

    let mut app = App::new();
    run_app(&mut tui.terminal, &mut app)?;

    tui.reset()?;

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> Result<()> {
    loop {
        terminal.draw(|f| {
            ui(f, app);
        })?;

        if let Event::Key(key) = event::read()? {
            if key.kind != event::KeyEventKind::Press {
                continue;
            }

            match key.code {
                KeyCode::Esc => {
                    return Ok(());
                }
                KeyCode::Up => app.select_up(1),
                KeyCode::Down => app.select_down(1),
                KeyCode::Home => app.select(0),
                KeyCode::End => app.select(app.last_item_index()),
                KeyCode::PageUp => app.select_up(app.page_length - 1),
                KeyCode::PageDown => app.select_down(app.page_length - 1),
                _ => {}
            }
        }
    }
}
