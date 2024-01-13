use ratatui::widgets::ListState;

pub const COMMANDS: [&str; 12] = [
    "pick",
    "reword",
    "edit",
    "squash",
    "fixup",
    "exec",
    "break",
    "drop",
    "label",
    "reset",
    "merge",
    "update-ref",
];

pub struct Line {
    pub command: String,
    pub parameters: String,
}

pub enum Mode {
    Main,
    EditingCommand { list_state: ListState },
}

pub struct App {
    pub lines: Vec<Line>,
    pub lines_widget_state: ListState,
    pub page_length: usize,
    pub mode: Mode,
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
            mode: Mode::Main,
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

    pub fn move_up(&mut self) {
        let selected = self.selected();
        if selected > 0 {
            self.lines.swap(selected, selected - 1);
            self.select_up(1);
        }
    }

    pub fn move_down(&mut self) {
        let selected = self.selected();
        if selected < self.lines.len() - 1 {
            self.lines.swap(selected, selected + 1);
            self.select_down(1);
        }
    }

    pub fn edit_command(&mut self) {
        let selected = self.selected();
        let command = &self.lines[selected].command;
        let index = COMMANDS.iter().position(|cmd| cmd == command).unwrap_or(0);

        self.mode = Mode::EditingCommand {
            list_state: ListState::default().with_selected(Some(index)),
        };
    }

    fn selected(&self) -> usize {
        self.lines_widget_state.selected().unwrap()
    }

    pub fn select_command_up(&mut self) {
        self.modify_selected_command(|selected| selected.saturating_sub(1));
    }

    pub fn select_command_down(&mut self) {
        self.modify_selected_command(|selected| (selected + 1).min(COMMANDS.len() - 1));
    }

    fn modify_selected_command(&mut self, f: impl FnOnce(usize) -> usize) {
        if let Mode::EditingCommand { list_state } = &mut self.mode {
            let selected = list_state.selected().unwrap();
            let selected = f(selected);
            list_state.select(Some(selected));
        }
    }
}
