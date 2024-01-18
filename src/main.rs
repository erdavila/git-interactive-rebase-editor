mod app;
mod tui;
mod ui;
mod widgets;

use std::{
    env,
    fs::File,
    io::{self, BufRead, BufReader, BufWriter, Write},
};

use anyhow::Result;
use app::{App, EditingWhat, Line, Mode, RebaseConfirmation};
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
    let lines = read_lines(&path)?;

    let backend = CrosstermBackend::new(io::stdout());
    let terminal = Terminal::new(backend)?;
    let mut tui = Tui::new(terminal);

    tui.enter()?;
    setup_panic_hook();

    let mut app = App::new(lines);
    let rebase_confirmation = run_app(&mut tui.terminal, &mut app);

    tui.reset()?;

    let lines: &[Line] = if rebase_confirmation?.0 {
        app.lines.items()
    } else {
        &[]
    };
    save_lines(lines, &path)?;

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
                    KeyCode::Up if key.modifiers == KeyModifiers::CONTROL => app.move_line_up(),
                    KeyCode::Down if key.modifiers == KeyModifiers::CONTROL => app.move_line_down(),
                    KeyCode::PageUp => app.lines.select_up(app.page_length - 1),
                    KeyCode::PageDown => app.lines.select_down(app.page_length - 1),
                    KeyCode::Enter => app.enter_edition(),
                    KeyCode::Delete => app.remove_line(),
                    KeyCode::Char('2') => app.duplicate_line(),
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

fn read_lines(path: &str) -> Result<Vec<Line>> {
    let file = BufReader::new(File::open(path)?);

    let mut lines = Vec::new();
    for line in file.lines() {
        let line = line?;
        let line = line.trim();

        if line.starts_with('#') || line.is_empty() {
            continue;
        }

        let (command, parameters) = line.split_once(' ').unwrap_or((line, ""));
        let line = Line {
            command: command.to_string(),
            parameters: parameters.to_string(),
        };

        lines.push(line);
    }

    Ok(lines)
}

fn save_lines(lines: &[Line], path: &str) -> Result<()> {
    let mut file = BufWriter::new(File::create(path)?);
    for line in lines {
        writeln!(file, "{} {}", line.command, line.parameters)?;
    }
    Ok(())
}
