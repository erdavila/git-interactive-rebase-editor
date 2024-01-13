use ratatui::widgets::ListState;

pub struct Line {
    pub command: String,
    pub parameters: String,
}

pub struct App {
    pub lines: Vec<Line>,
    pub lines_widget_state: ListState,
    pub page_length: usize,
}

impl App {
    pub fn new() -> Self {
        App {
            lines: (1u32..=50)
                .map(|n| {
                    let command = match n % 3 {
                        0 => "pick",
                        1 => "edit",
                        _ => "drop",
                    }
                    .to_owned();
                    let parameters = format!("parameters {n}");
                    Line {
                        command,
                        parameters,
                    }
                })
                .collect(),
            lines_widget_state: ListState::default().with_selected(Some(0)),
            page_length: 0,
        }
    }

    pub fn select_up(&mut self, delta: usize) {
        self.modify_selected(|selected| selected.saturating_sub(delta));
    }

    pub fn select_down(&mut self, delta: usize) {
        let last_index = self.last_line_index();
        self.modify_selected(|selected| (selected + delta).min(last_index));
    }

    fn modify_selected(&mut self, f: impl FnOnce(usize) -> usize) {
        let selected = self.lines_widget_state.selected().unwrap();
        let selected = f(selected);
        self.select(selected);
    }

    pub fn select(&mut self, index: usize) {
        self.lines_widget_state.select(Some(index));
    }

    pub fn last_line_index(&self) -> usize {
        self.lines.len() - 1
    }
}
