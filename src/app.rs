use crate::widgets::{selectable_list::SelectableList, text_input::TextInput};

#[derive(Clone, Copy)]
pub struct RebaseConfirmation(pub bool);

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

pub enum EditingWhat<'a> {
    Command(SelectableList<'a, &'a [Command]>),
    Parameters(TextInput),
}

pub enum Mode<'a> {
    Main,
    Editing {
        what: EditingWhat<'a>,
        original_line: Line,
    },
    Quitting(SelectableList<'a, [RebaseConfirmation; 2]>),
}

pub struct App<'a> {
    pub lines: SelectableList<'a, Vec<Line>>,
    pub page_length: usize,
    pub mode: Mode<'a>,
}

impl<'a> App<'a> {
    pub fn new(lines: Vec<Line>) -> Self {
        App {
            lines: SelectableList::new(lines),
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

    pub fn enter_edition(&mut self) {
        let original_line = self.lines.selected_item().clone();
        let command = original_line.command.clone();
        self.mode = Self::make_command_edition_mode(command, original_line);
    }

    pub fn switch_edition(&mut self) {
        if let Mode::Editing {
            what,
            original_line,
        } = &mut self.mode
        {
            match what {
                EditingWhat::Command(command) => {
                    Self::apply_edited_command(&mut self.lines, command);
                    let parameters = self.lines.selected_item().parameters.clone();
                    self.mode =
                        Self::make_parameters_edition_mode(parameters, original_line.clone());
                }
                EditingWhat::Parameters(parameters) => {
                    Self::apply_edited_parameters(&mut self.lines, parameters);
                    let command = self.lines.selected_item().command.clone();
                    self.mode = Self::make_command_edition_mode(command, original_line.clone());
                }
            }
        } else {
            unimplemented!()
        }
    }

    pub fn confirm_edition(&mut self) {
        if let Mode::Editing { what, .. } = &mut self.mode {
            match what {
                EditingWhat::Command(command) => {
                    Self::apply_edited_command(&mut self.lines, command)
                }
                EditingWhat::Parameters(parameters) => {
                    Self::apply_edited_parameters(&mut self.lines, parameters)
                }
            }

            self.mode = Mode::Main;
        } else {
            unimplemented!()
        }
    }

    fn make_command_edition_mode(command: String, original_line: Line) -> Mode<'a> {
        let selected_command_index = COMMANDS
            .iter()
            .position(|cmd| cmd.0 == command)
            .unwrap_or(0);

        Mode::Editing {
            what: EditingWhat::Command(
                SelectableList::new(COMMANDS.as_slice()).with_selected(selected_command_index),
            ),
            original_line,
        }
    }

    fn make_parameters_edition_mode(parameters: String, original_line: Line) -> Mode<'a> {
        Mode::Editing {
            what: EditingWhat::Parameters(TextInput::new(parameters.chars())),
            original_line,
        }
    }

    fn apply_edited_command(
        lines: &mut SelectableList<Vec<Line>>,
        command: &mut SelectableList<'_, &[Command]>,
    ) {
        lines.selected_item_mut().command = command.selected_item().0.to_string();
    }

    fn apply_edited_parameters(lines: &mut SelectableList<'_, Vec<Line>>, parameters: &TextInput) {
        lines.selected_item_mut().parameters = parameters.content().iter().collect::<String>();
    }

    pub fn cancel_edition(&mut self) {
        if let Mode::Editing { original_line, .. } = &mut self.mode {
            let line = self.lines.selected_item_mut();
            std::mem::swap(line, original_line);
            self.mode = Mode::Main;
        } else {
            unimplemented!();
        }
    }

    pub fn ask_rebase_confirmation(&mut self) {
        self.mode = Mode::Quitting(SelectableList::new([
            RebaseConfirmation(true),
            RebaseConfirmation(false),
        ]));
    }

    pub fn remove_line(&mut self) {
        let index = self.lines.selected();
        self.lines.items_mut().remove(index);
        if index == self.lines.items().len() {
            self.lines.select_up(1);
        }
    }
}
