use ratatui::{
    layout::{Constraint, Layout, Margin, Rect},
    style::{Style, Stylize},
    widgets::{
        Block, Borders, Clear, ListItem, Padding, Paragraph, Scrollbar, ScrollbarOrientation,
        ScrollbarState,
    },
    Frame,
};

use crate::app::{App, Command, EditingWhat, Line, Mode, COMMANDS};

impl<'a> From<Line> for ListItem<'a> {
    fn from(line: Line) -> Self {
        ListItem::new(format!("{:10} {}", line.command, line.parameters))
    }
}

impl<'a> From<&Command> for ListItem<'a> {
    fn from(command: &Command) -> Self {
        ListItem::new(command.0)
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

    let lines_count = app.lines.items().len();
    let (lines, lines_state) = app.lines.widget_and_state();
    let lines = lines.highlight_style(Style::default().reversed()).block(
        Block::default()
            .title(" Git Interactive Rebase ")
            .borders(Borders::ALL)
            .padding(Padding::horizontal(1)),
    );
    app.page_length = lines_area.height as usize - 2;
    frame.render_stateful_widget(lines, lines_area, lines_state);

    let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight);
    let scrollbar_area = lines_area.inner(&Margin {
        horizontal: 0,
        vertical: 1,
    });
    let mut scrollbar_state =
        ScrollbarState::new(lines_count.saturating_sub(scrollbar_area.height as usize))
            .position(lines_state.offset());
    frame.render_stateful_widget(scrollbar, scrollbar_area, &mut scrollbar_state);

    let footer_text = match &mut app.mode {
        Mode::Main => "CTRL+↑/CTRL+↓: move | ENTER: edit | ESC/Q: quit",
        Mode::Editing {
            what: EditingWhat::Command(commands),
            ..
        } => {
            let mut cmds_area = Rect {
                x: lines_area.x + 3,
                y: (lines_state.selected().unwrap() - lines_state.offset()) as u16,
                width: max_command_len() as u16 + 4,
                height: COMMANDS.len() as u16 + 2,
            };
            if cmds_area.bottom() > lines_area.bottom() {
                cmds_area.y -= cmds_area.bottom() - lines_area.bottom();
            }

            let (cmds, cmds_state) = commands.widget_and_state();

            let cmds = cmds.highlight_style(Style::default().reversed()).block(
                Block::default()
                    .padding(Padding::horizontal(1))
                    .borders(Borders::ALL),
            );

            frame.render_widget(Clear, cmds_area);
            frame.render_stateful_widget(cmds, cmds_area, cmds_state);

            "TAB: edit parameters | ENTER: confirm | ESC: cancel editing"
        }
        Mode::Editing {
            what: EditingWhat::Parameters(parameters),
            ..
        } => {
            let x = max_command_len() as u16 + 2;
            let params_area = Rect {
                x,
                y: (lines_state.selected().unwrap() - lines_state.offset()) as u16,
                width: lines_area.right() - x,
                height: 3,
            };
            let params =
                Paragraph::new(parameters.clone()).block(Block::default().borders(Borders::ALL));
            frame.render_widget(Clear, params_area);
            frame.render_widget(params, params_area);

            "TAB: edit command | ENTER: confirm | ESC: cancel editing"
        }
    };

    let footer = Paragraph::new(footer_text).style(Style::default().reversed());
    frame.render_widget(footer, footer_area);
}

fn max_command_len() -> usize {
    COMMANDS.iter().map(|cmd| cmd.0.len()).max().unwrap_or(0)
}
