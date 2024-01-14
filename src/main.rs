mod app;
mod tui;
mod ui;
mod widgets;

use std::io;

use anyhow::Result;
use app::{App, Mode};
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
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

        if let Event::Key(mut key) = event::read()? {
            if key.kind != event::KeyEventKind::Press {
                continue;
            }

            if let KeyCode::Char(mut char) = key.code {
                char.make_ascii_lowercase();
                key.code = KeyCode::Char(char);
            }

            match &mut app.mode {
                Mode::Main => match key.code {
                    KeyCode::Esc | KeyCode::Char('q') => {
                        return Ok(());
                    }
                    KeyCode::Up if key.modifiers == KeyModifiers::CONTROL => app.move_line_up(),
                    KeyCode::Down if key.modifiers == KeyModifiers::CONTROL => app.move_line_down(),
                    KeyCode::PageUp => app.lines.select_up(app.page_length - 1),
                    KeyCode::PageDown => app.lines.select_down(app.page_length - 1),
                    KeyCode::Enter => app.edit_command(),
                    _ => app.lines.input(key),
                },
                Mode::Editing { commands, .. } => {
                    match key.code {
                        KeyCode::Esc => {
                            // TODO: temporary implementation
                            app.mode = Mode::Main;
                        }
                        _ => commands.input(key),
                    }
                }
            }
        }
    }
}
