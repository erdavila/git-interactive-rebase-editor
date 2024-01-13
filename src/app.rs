use crate::widgets::selectable_list::SelectableList;

pub struct Command(pub &'static str);

pub const COMMANDS: [Command; 12] = [
    Command("pick"),
    Command("reword"),
    Command("edit"),
    Command("squash"),
    Command("fixup"),
    Command("exec"),
    Command("break"),
    Command("drop"),
    Command("label"),
    Command("reset"),
    Command("merge"),
    Command("update-ref"),
];

#[derive(Clone)]
pub struct Line {
    pub command: String,
    pub parameters: String,
}

pub enum Mode<'a> {
    Main,
    Editing {
        commands: SelectableList<'a, &'a [Command]>,
    },
}

pub struct App<'a> {
    pub lines: SelectableList<'a, Vec<Line>>,
    pub page_length: usize,
    pub mode: Mode<'a>,
}

impl<'a> App<'a> {
    pub fn new() -> Self {
        App {
            lines: SelectableList::new(
                (1u32..=50)
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
            ),
            page_length: 0,
            mode: Mode::Main,
        }
    }

    pub fn move_line_up(&mut self) {
        let selected = self.lines.selected();
        if selected > 0 {
            self.lines.items_mut().swap(selected, selected - 1);
            self.lines.select_up(1);
        }
    }

    pub fn move_line_down(&mut self) {
        let selected = self.lines.selected();
        if selected < self.lines.items().len() - 1 {
            self.lines.items_mut().swap(selected, selected + 1);
            self.lines.select_down(1);
        }
    }

    pub fn edit_command(&mut self) {
        let command = &self.lines.selected_item().command;
        let index = COMMANDS
            .iter()
            .position(|cmd| cmd.0 == command)
            .unwrap_or(0);

        self.mode = Mode::Editing {
            commands: SelectableList::new(COMMANDS.as_slice()).with_selected(index),
        };
    }

    pub fn confirm_command(&mut self) {
        if let Mode::Editing { commands } = &mut self.mode {
            let selected_command = commands.selected();
            let selected_line = self.lines.selected();

            self.lines.items_mut()[selected_line].command =
                COMMANDS[selected_command].0.to_string();
            self.mode = Mode::Main;
        }
    }
}
