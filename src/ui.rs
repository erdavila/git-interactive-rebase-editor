use ratatui::{
    layout::Margin,
    style::{Color, Style, Stylize},
    widgets::{Block, Borders, List, ListItem, Scrollbar, ScrollbarOrientation, ScrollbarState},
    Frame,
};

use crate::app::{App, Line};

const WITH_BORDER: bool = true;

impl<'a> From<&Line> for ListItem<'a> {
    fn from(line: &Line) -> Self {
        ListItem::new(format!("{:10} {}", line.command, line.parameters))
    }
}

pub fn ui(frame: &mut Frame, app: &mut App) {
    let mut lines = List::new(&app.lines)
        .style(Style::default().fg(Color::Black).bg(Color::LightYellow))
        .highlight_symbol(">")
        .highlight_style(Style::default().on_green());
    if WITH_BORDER {
        lines = lines.block(
            Block::default()
                .title("Lines")
                .borders(Borders::ALL)
                .green(),
        );
    }
    let lines_area = frame.size();
    app.page_length = lines_area.height as usize - if WITH_BORDER { 2 } else { 0 };
    frame.render_stateful_widget(lines, lines_area, &mut app.lines_widget_state);

    let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight);
    let scrollbar_area = lines_area.inner(&Margin {
        horizontal: 0,
        vertical: if WITH_BORDER { 1 } else { 0 },
    });
    let mut scrollbar_state = ScrollbarState::new(
        app.lines
            .len()
            .saturating_sub(scrollbar_area.height as usize),
    )
    .position(app.lines_widget_state.offset());
    frame.render_stateful_widget(scrollbar, scrollbar_area, &mut scrollbar_state);
}
