use ratatui::{
    layout::{Constraint, Layout, Margin, Rect},
    style::{Style, Stylize},
    widgets::{
        Block, Borders, Clear, List, ListItem, Padding, Paragraph, Scrollbar, ScrollbarOrientation,
        ScrollbarState,
    },
    Frame,
};

use crate::app::{App, Line, Mode, COMMANDS};

impl<'a> From<&Line> for ListItem<'a> {
    fn from(line: &Line) -> Self {
        ListItem::new(format!("{:10} {}", line.command, line.parameters))
    }
}

pub fn ui(frame: &mut Frame, app: &mut App) {
    let [lines_area, footer_area] = {
        let chunks = Layout::default()
            .constraints([
                Constraint::Min(COMMANDS.len() as u16),
                Constraint::Length(1),
            ])
            .split(frame.size());
        [chunks[0], chunks[1]]
    };

    let lines = List::new(&app.lines)
        .highlight_style(Style::default().reversed())
        .block(
            Block::default()
                .title(" Git Interactive Rebase ")
                .borders(Borders::ALL)
                .padding(Padding::horizontal(1)),
        );
    app.page_length = lines_area.height as usize - 2;
    frame.render_stateful_widget(lines, lines_area, &mut app.lines_widget_state);

    let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight);
    let scrollbar_area = lines_area.inner(&Margin {
        horizontal: 0,
        vertical: 1,
    });
    let mut scrollbar_state = ScrollbarState::new(
        app.lines
            .len()
            .saturating_sub(scrollbar_area.height as usize),
    )
    .position(app.lines_widget_state.offset());
    frame.render_stateful_widget(scrollbar, scrollbar_area, &mut scrollbar_state);

    let footer_text = match &mut app.mode {
        Mode::Main => "CTRL+↑/CTRL+↓: move | ENTER: edit | ESC/Q: quit",
        Mode::EditingCommand { list_state } => {
            let mut commands_area = Rect {
                x: lines_area.x + 3,
                y: (app.lines_widget_state.selected().unwrap() - app.lines_widget_state.offset())
                    as u16,
                width: COMMANDS.iter().map(|cmd| cmd.len()).max().unwrap_or(0) as u16 + 4,
                height: COMMANDS.len() as u16 + 2,
            };
            if commands_area.bottom() > lines_area.bottom() {
                commands_area.y -= commands_area.bottom() - lines_area.bottom();
            }

            let commands = List::new(COMMANDS)
                .highlight_style(Style::default().reversed())
                .block(
                    Block::default()
                        .padding(Padding::horizontal(1))
                        .borders(Borders::ALL),
                );

            frame.render_widget(Clear, commands_area);
            frame.render_stateful_widget(commands, commands_area, list_state);

            "ESC: cancel editing"
        }
    };

    let footer = Paragraph::new(footer_text).style(Style::default().reversed());
    frame.render_widget(footer, footer_area);
}
