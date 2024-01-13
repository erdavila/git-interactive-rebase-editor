use std::io;

use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Margin, Rect},
    style::{Color, Style, Stylize},
    widgets::{Block, Borders, List, ListState, Scrollbar, ScrollbarOrientation, ScrollbarState},
    Terminal,
};

struct App {}

fn main() -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?;
    terminal.clear()?;

    let mut app = App {};
    run_app(&mut terminal, &mut app)?;

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, _app: &mut App) -> Result<()> {
    const WITH_BORDER: bool = true;
    const LIST_HEIGHT: u16 = 10;
    const PAGE_LENGTH: usize = LIST_HEIGHT as usize - 1 - if WITH_BORDER { 2 } else { 0 };

    let mut list_state = ListState::default().with_selected(Some(0));
    let items: Vec<_> = (1..=50).map(|n| format!("{n}")).collect();

    loop {
        terminal.draw(|f| {
            let mut list = List::new(items.clone())
                .style(Style::default().fg(Color::Black).bg(Color::LightYellow))
                .highlight_symbol(">")
                .highlight_style(Style::default().on_green());
            if WITH_BORDER {
                list = list.block(
                    Block::default()
                        .title("Items")
                        .borders(Borders::ALL)
                        .green(),
                );
            }
            let area = Rect::new(1, 1, 20, LIST_HEIGHT);
            f.render_stateful_widget(list, area, &mut list_state);

            let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight);
            let area = area.inner(&Margin {
                horizontal: 0,
                vertical: if WITH_BORDER { 1 } else { 0 },
            });
            let mut scrollbar_state = ScrollbarState::new(items.len() - area.height as usize)
                .position(list_state.offset());
            f.render_stateful_widget(scrollbar, area, &mut scrollbar_state);
        })?;

        if let Event::Key(key) = event::read()? {
            if key.kind != event::KeyEventKind::Press {
                continue;
            }

            fn select<F: FnOnce(usize) -> usize>(list_state: &mut ListState, f: F) {
                let selected = list_state.selected().unwrap();
                let selected = f(selected);
                list_state.select(Some(selected));
            }

            fn select_up(list_state: &mut ListState, delta: usize) {
                select(list_state, |selected| selected.saturating_sub(delta));
            }

            let select_down = |list_state: &mut ListState, delta: usize| {
                select(list_state, |selected| {
                    (selected + delta).min(items.len() - 1)
                });
            };

            match key.code {
                KeyCode::Esc => {
                    return Ok(());
                }
                KeyCode::Up => {
                    select_up(&mut list_state, 1);
                }
                KeyCode::Down => {
                    select_down(&mut list_state, 1);
                }
                KeyCode::Home => {
                    list_state.select(Some(0));
                }
                KeyCode::End => {
                    list_state.select(Some(items.len() - 1));
                }
                KeyCode::PageUp => {
                    select_up(&mut list_state, PAGE_LENGTH);
                }
                KeyCode::PageDown => {
                    select_down(&mut list_state, PAGE_LENGTH);
                }
                _ => {}
            }
        }
    }
}
