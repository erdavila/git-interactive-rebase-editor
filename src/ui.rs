use ratatui::{
    layout::{Constraint, Layout, Margin},
    style::{Style, Stylize},
    widgets::{
        Block, Borders, List, ListItem, Padding, Paragraph, Scrollbar, ScrollbarOrientation,
        ScrollbarState,
    },
    Frame,
};

use crate::app::{App, Line};

impl<'a> From<&Line> for ListItem<'a> {
    fn from(line: &Line) -> Self {
        ListItem::new(format!("{:10} {}", line.command, line.parameters))
    }
}

pub fn ui(frame: &mut Frame, app: &mut App) {
    let [lines_area, footer_area] = {
        let chunks = Layout::default()
            .constraints([Constraint::Min(10), Constraint::Length(1)])
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

    let footer = Paragraph::new("ESC/Q: quit").style(Style::default().reversed());
    frame.render_widget(footer, footer_area);
}
