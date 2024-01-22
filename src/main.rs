mod app;
mod tui;
mod ui;
mod widgets;

use std::{env, fs, io};

use anyhow::Result;
use app::{App, EditingWhat, Mode, RebaseConfirmation};
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
    let path = {
        let mut args = env::args();
        args.next();
        args.next().unwrap()
    };
    let todo_list = fs::read_to_string(&path)?;

    let backend = CrosstermBackend::new(io::stdout());
    let terminal = Terminal::new(backend)?;
    let mut tui = Tui::new(terminal);

    tui.enter()?;
    setup_panic_hook();

    let mut app = App::new(&todo_list);
    let rebase_confirmation = run_app(&mut tui.terminal, &mut app);

    tui.reset()?;

    let todo_list = if rebase_confirmation?.0 {
        app.get_todo_list_string()?
    } else {
        String::new()
    };
    fs::write(path, todo_list)?;

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

fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> Result<RebaseConfirmation> {
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
                    KeyCode::Esc | KeyCode::Char('q') => app.ask_rebase_confirmation(),
                    KeyCode::Insert => app.insert_todo_item(),
                    KeyCode::Char('o') => app.show_original_todo_list(),
                    _ if app.todo_list.items().is_empty() => {}

                    // === Actions below are available only if the list is not empty ===
                    KeyCode::Up if key.modifiers == KeyModifiers::CONTROL => {
                        app.move_todo_item_up()
                    }
                    KeyCode::Down if key.modifiers == KeyModifiers::CONTROL => {
                        app.move_todo_item_down()
                    }
                    KeyCode::PageUp => app.todo_list.select_up(app.page_length - 1),
                    KeyCode::PageDown => app.todo_list.select_down(app.page_length - 1),
                    KeyCode::Enter => app.enter_edition(),
                    KeyCode::Delete => app.remove_todo_item(),
                    KeyCode::Char('2') => app.duplicate_todo_item(),
                    _ => app.todo_list.input(key),
                },

                Mode::Editing { what, .. } => match key.code {
                    KeyCode::Esc => app.cancel_edition(),
                    KeyCode::Enter => app.confirm_edition(),
                    KeyCode::Tab => app.switch_edition(),
                    KeyCode::Char(char) => {
                        if let EditingWhat::Command(command) = what {
                            App::select_command_by_char(command, char);
                        }
                    }
                    _ => match what {
                        EditingWhat::Command(commands) => commands.input(key),
                        EditingWhat::Parameters(parameters) => parameters.input(key),
                    },
                },

                Mode::ShowingOriginal { scroll } => match key.code {
                    KeyCode::Esc => app.mode = Mode::Main,
                    KeyCode::Up => *scroll = scroll.saturating_sub(1),
                    KeyCode::Down => *scroll = scroll.saturating_add(1),
                    _ => {}
                },

                Mode::Quitting(rebase_confirmation) => match key.code {
                    KeyCode::Esc => app.mode = Mode::Main,
                    KeyCode::Char('y') => return Ok(RebaseConfirmation(true)),
                    KeyCode::Char('n') => return Ok(RebaseConfirmation(false)),
                    KeyCode::Enter => return Ok(*rebase_confirmation.selected_item()),
                    _ => rebase_confirmation.input(key),
                },
            }
        }
    }
}
