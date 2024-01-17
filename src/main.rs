mod app;
mod tui;
mod ui;
mod widgets;

use std::io;

use anyhow::Result;
use app::{App, EditingWhat, Mode};
use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, LeaveAlternateScreen},
};
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
    setup_panic_hook();

    let mut app = App::new();
    run_app(&mut tui.terminal, &mut app)?;

    tui.reset()?;

    Ok(())
}

fn setup_panic_hook() {
    let panic_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic| {
        disable_raw_mode().unwrap();
        execute!(io::stdout(), LeaveAlternateScreen).unwrap();

        panic_hook(panic);
    }));
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
                    KeyCode::Enter => app.enter_edition(),
                    _ => app.lines.input(key),
                },

                Mode::Editing { what, .. } => match key.code {
                    KeyCode::Esc => app.cancel_edition(),
                    KeyCode::Enter => app.confirm_edition(),
                    KeyCode::Tab => app.switch_edition(),
                    _ => match what {
                        EditingWhat::Command(commands) => commands.input(key),
                        EditingWhat::Parameters(parameters) => parameters.input(key),
                    },
                },
            }
        }
    }
}
