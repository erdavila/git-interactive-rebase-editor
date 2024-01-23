use ratatui::{
    layout::{Constraint, Direction, Layout, Margin, Rect},
    style::{Style, Stylize},
    text::{Line, Span},
    widgets::{
        Block, Borders, Clear, ListItem, Padding, Paragraph, Scrollbar, ScrollbarOrientation,
        ScrollbarState,
    },
    Frame,
};

use crate::app::{App, Command, EditingWhat, Mode, RebaseConfirmation, TodoItem, COMMANDS};

impl<'a> From<TodoItem> for ListItem<'a> {
    fn from(item: TodoItem) -> Self {
        ListItem::new(format!("{:10} {}", item.command, item.parameters))
    }
}

impl<'a> From<&Command> for ListItem<'a> {
    fn from(command: &Command) -> Self {
        ListItem::new(command.0)
    }
}

impl<'a> From<RebaseConfirmation> for ListItem<'a> {
    fn from(conf: RebaseConfirmation) -> Self {
        ListItem::new(conf.text())
    }
}

impl RebaseConfirmation {
    fn text(&self) -> &'static str {
        match self.0 {
            true => "Yes",
            false => "No",
        }
    }
}

pub fn ui(frame: &mut Frame, app: &mut App) {
    let [todo_list_area, footer_area] = {
        let chunks = Layout::default()
            .constraints([
                Constraint::Min(COMMANDS.len() as u16),
                Constraint::Length(1),
            ])
            .split(frame.size());
        [chunks[0], chunks[1]]
    };

    let todo_items_count = app.todo_list.items().len();
    let (todo_list, todo_list_state) = app.todo_list.widget_and_state();
    let todo_list = todo_list
        .highlight_style(Style::default().reversed())
        .block(
            Block::default()
                .title(" Git Interactive Rebase ")
                .borders(Borders::ALL)
                .padding(Padding::horizontal(1)),
        );
    app.page_length = todo_list_area.height as usize - 2;
    frame.render_stateful_widget(todo_list, todo_list_area, todo_list_state);

    let scrollbar_area = todo_list_area.inner(&Margin {
        horizontal: 0,
        vertical: 1,
    });
    if let Some(mut scrollbar_state) = scrollbar_state_from_offset(
        todo_items_count,
        scrollbar_area.height as usize,
        todo_list_state.offset(),
    ) {
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight);
        frame.render_stateful_widget(scrollbar, scrollbar_area, &mut scrollbar_state);
    }

    let footer_content: &[(&[&'static str], &'static str)] = match &mut app.mode {
        Mode::Main => {
            if app.todo_list.items().is_empty() {
                &[
                    (&["INSERT"], "insert"),
                    (&["o"], "show original todo list"),
                    (&["ESC", "Q"], "quit"),
                ]
            } else {
                &[
                    (&["CTRL+↑", "CTRL+↓"], "move"),
                    (&["ENTER"], "edit"),
                    (&["INSERT"], "insert"),
                    (&["DELETE"], "remove"),
                    (&["2"], "duplicate"),
                    (&["o"], "show original todo list"),
                    (&["ESC", "Q"], "quit"),
                ]
            }
        }

        Mode::Editing {
            what: EditingWhat::Command(commands),
            ..
        } => {
            let mut cmds_area = Rect {
                x: todo_list_area.x + 3,
                y: (todo_list_state.selected().unwrap() - todo_list_state.offset()) as u16,
                width: max_command_len() as u16 + 4,
                height: COMMANDS.len() as u16 + 2,
            };
            if cmds_area.bottom() > todo_list_area.bottom() {
                cmds_area.y -= cmds_area.bottom() - todo_list_area.bottom();
            }

            let (cmds, cmds_state) = commands.widget_and_state();

            let cmds = cmds.highlight_style(Style::default().reversed()).block(
                Block::default()
                    .padding(Padding::horizontal(1))
                    .borders(Borders::ALL),
            );

            frame.render_widget(Clear, cmds_area);
            frame.render_stateful_widget(cmds, cmds_area, cmds_state);

            &[
                (&["TAB"], "edit parameters"),
                (&["ENTER"], "confirm"),
                (&["ESC"], "cancel editing"),
            ]
        }

        Mode::Editing {
            what: EditingWhat::Parameters(parameters),
            ..
        } => {
            let x = max_command_len() as u16 + 2;
            let params_area = Rect {
                x,
                y: (todo_list_state.selected().unwrap() - todo_list_state.offset()) as u16,
                width: todo_list_area.right() - x,
                height: 3,
            };

            let (widget, widget_state) = parameters.widget_and_state();
            let widget = widget.block(Block::default().borders(Borders::ALL));
            frame.render_stateful_widget(widget, params_area, widget_state);

            &[
                (&["TAB"], "edit command"),
                (&["ENTER"], "confirm"),
                (&["ESC"], "cancel editing"),
            ]
        }

        Mode::ShowingOriginal { scroll } => {
            let original_todo_area = {
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Length(5),
                        Constraint::Min(1),
                        Constraint::Length(5),
                    ])
                    .split(frame.size());

                Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([
                        Constraint::Length(7),
                        Constraint::Min(1),
                        Constraint::Length(7),
                    ])
                    .split(chunks[1])[1]
            };

            let max_scroll = (app.original_todo_list_lines.len() as u16)
                .saturating_sub(original_todo_area.height - 2 /*margins*/);
            *scroll = std::cmp::min(*scroll, max_scroll);

            let p = Paragraph::new(
                app.original_todo_list_lines
                    .iter()
                    .map(|line| Line::from(*line))
                    .collect::<Vec<_>>(),
            )
            .scroll((*scroll, 0))
            .block(
                Block::default()
                    .title("Original todo list")
                    .borders(Borders::ALL)
                    .padding(Padding::horizontal(1)),
            );

            frame.render_widget(Clear, original_todo_area);
            frame.render_widget(p, original_todo_area);

            let scrollbar_area = original_todo_area.inner(&Margin {
                horizontal: 0,
                vertical: 1,
            });
            if let Some(mut scrollbar_state) = scrollbar_state_from_offset(
                app.original_todo_list_lines.len(),
                scrollbar_area.height as usize,
                *scroll as usize,
            ) {
                let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight);
                frame.render_stateful_widget(scrollbar, scrollbar_area, &mut scrollbar_state);
            }

            &[(&["ESC"], "dismiss")]
        }

        Mode::Quitting(rebase_confirmation) => {
            const PADDING: u16 = 2;
            let question = "Proceed to rebase?";

            let dialog_width = question.len() as u16 + 2 * (PADDING + 1);
            let dialog_height = 4/*border + spacing A + question + spacing B*/ + rebase_confirmation.items().len() as u16 + 2/*spacing C + border*/;
            let dialog_area = centered_rect(dialog_width, dialog_height, frame.size());

            frame.render_widget(Block::default().borders(Borders::ALL), dialog_area);

            let dialog_inner_area = dialog_area.inner(&Margin::new(1, 1));
            frame.render_widget(Clear, dialog_inner_area);

            let [question_area, confirmation_area] = {
                let chunks = Layout::default()
                    .constraints([
                        Constraint::Length(2 /*spacing A + question*/),
                        Constraint::Min(1),
                    ])
                    .split(dialog_inner_area);
                [chunks[0], chunks[1]]
            };

            let mut question_area = question_area.inner(&Margin {
                horizontal: PADDING,
                vertical: 0,
            });
            question_area.y += 1;
            frame.render_widget(Paragraph::new(question), question_area);

            let confirmation_area = centered_rect(
                2/*left padding + right padding*/ + rebase_confirmation
                    .items()
                    .iter()
                    .map(|x| x.text().len())
                    .max()
                    .unwrap_or(0) as u16,
                rebase_confirmation.items().len() as u16,
                confirmation_area,
            );
            let (confirmation, confirmation_state) = rebase_confirmation.widget_and_state();
            let confirmation = confirmation
                .highlight_style(Style::default().reversed())
                .highlight_symbol(" " /*left padding*/);
            frame.render_stateful_widget(confirmation, confirmation_area, confirmation_state);

            &[
                (&["Y"], "quit and rebase"),
                (&["N"], "quit and don't rebase"),
                (&["ESC"], "don't quit"),
            ]
        }
    };

    let actions = footer_content.iter().map(|(keys, action)| {
        interspace(*keys)
            .map(|item| match item {
                Some(key) => Span::from(*key).style(Style::default().bold()),
                None => Span::from("/"),
            })
            .chain(std::iter::once(Span::from(format!(": {action}"))))
    });
    let footer_spans: Vec<_> = interspace(actions)
        .flat_map(|item| match item {
            Some(action) => action.collect(),
            None => vec![Span::from(" | ")],
        })
        .collect();
    let footer = Paragraph::new(ratatui::text::Line::from(footer_spans));
    frame.render_widget(footer, footer_area);
}

fn scrollbar_state_from_offset(
    content_length: usize,
    viewport_content_length: usize,
    offset: usize,
) -> Option<ScrollbarState> {
    scrollbar_position_from_offset(content_length, viewport_content_length, offset).map(
        |position| {
            ScrollbarState::new(content_length)
                .viewport_content_length(viewport_content_length)
                .position(position)
        },
    )
}

fn scrollbar_position_from_offset(
    content_length: usize,
    viewport_content_length: usize,
    offset: usize,
) -> Option<usize> {
    let max_offset = content_length.saturating_sub(viewport_content_length);
    #[allow(clippy::unnecessary_lazy_evaluations)]
    (max_offset > 0).then(|| offset * (content_length - 1) / max_offset)
}

fn max_command_len() -> usize {
    COMMANDS.iter().map(|cmd| cmd.0.len()).max().unwrap_or(0)
}

fn centered_rect(width: u16, height: u16, enclosing_rect: Rect) -> Rect {
    Rect {
        x: centered_pos(width, enclosing_rect.x, enclosing_rect.width),
        y: centered_pos(height, enclosing_rect.y, enclosing_rect.height),
        width,
        height,
    }
}

fn centered_pos(length: u16, enclosing_pos: u16, enclosing_length: u16) -> u16 {
    enclosing_pos + (enclosing_length - length) / 2
}

fn interspace<I>(items: impl IntoIterator<Item = I>) -> impl Iterator<Item = Option<I>> {
    items.into_iter().enumerate().flat_map(|(index, item)| {
        if index == 0 {
            vec![Some(item)]
        } else {
            vec![None, Some(item)]
        }
    })
}
