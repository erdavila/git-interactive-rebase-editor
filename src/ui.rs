use ratatui::{
    layout::Margin,
    style::{Color, Style, Stylize},
    widgets::{Block, Borders, List, Scrollbar, ScrollbarOrientation, ScrollbarState},
    Frame,
};

use crate::app::App;

const WITH_BORDER: bool = true;

pub fn ui(frame: &mut Frame, app: &mut App) {
    let mut list = List::new(app.items.clone())
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
    let list_area = frame.size();
    app.page_length = list_area.height as usize - if WITH_BORDER { 2 } else { 0 };
    frame.render_stateful_widget(list, list_area, &mut app.list_state);

    let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight);
    let scrollbar_area = list_area.inner(&Margin {
        horizontal: 0,
        vertical: if WITH_BORDER { 1 } else { 0 },
    });
    let mut scrollbar_state = ScrollbarState::new(
        app.items
            .len()
            .saturating_sub(scrollbar_area.height as usize),
    )
    .position(app.list_state.offset());
    frame.render_stateful_widget(scrollbar, scrollbar_area, &mut scrollbar_state);
}
