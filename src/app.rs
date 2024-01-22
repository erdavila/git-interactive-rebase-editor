use std::fmt::Write;

use anyhow::Result;

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
pub struct TodoItem {
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
        original_item: Option<TodoItem>,
    },
    ShowingOriginal {
        scroll: u16,
    },
    Quitting(SelectableList<'a, [RebaseConfirmation; 2]>),
}

pub struct App<'a> {
    pub todo_list: SelectableList<'a, Vec<TodoItem>>,
    pub page_length: usize,
    pub mode: Mode<'a>,
    pub original_todo_list_lines: Vec<&'a str>,
}

impl<'a> App<'a> {
    pub fn new(todo_list: &'a str) -> Self {
        let todo_list_lines: Vec<_> = todo_list.lines().collect();
        let todo_list_items = parse_todo_list(&todo_list_lines);

        App {
            todo_list: SelectableList::new(todo_list_items),
            page_length: 0,
            mode: Mode::Main,
            original_todo_list_lines: todo_list_lines,
        }
    }

    pub fn move_todo_item_up(&mut self) {
        let selected = self.todo_list.selected();
        if selected > 0 {
            self.todo_list.items_mut().swap(selected, selected - 1);
            self.todo_list.select_up(1);
        }
    }

    pub fn move_todo_item_down(&mut self) {
        let selected = self.todo_list.selected();
        if selected < self.todo_list.items().len() - 1 {
            self.todo_list.items_mut().swap(selected, selected + 1);
            self.todo_list.select_down(1);
        }
    }

    pub fn enter_edition(&mut self) {
        let original_item = self.todo_list.selected_item().clone();
        let command = original_item.command.clone();
        self.mode = Self::make_command_edition_mode(command, Some(original_item));
    }

    pub fn switch_edition(&mut self) {
        if let Mode::Editing {
            what,
            original_item,
        } = &mut self.mode
        {
            match what {
                EditingWhat::Command(command) => {
                    Self::apply_edited_command(&mut self.todo_list, command);
                    let parameters = self.todo_list.selected_item().parameters.clone();
                    self.mode =
                        Self::make_parameters_edition_mode(parameters, original_item.clone());
                }
                EditingWhat::Parameters(parameters) => {
                    Self::apply_edited_parameters(&mut self.todo_list, parameters);
                    let command = self.todo_list.selected_item().command.clone();
                    self.mode = Self::make_command_edition_mode(command, original_item.clone());
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
                    Self::apply_edited_command(&mut self.todo_list, command)
                }
                EditingWhat::Parameters(parameters) => {
                    Self::apply_edited_parameters(&mut self.todo_list, parameters)
                }
            }

            self.mode = Mode::Main;
        } else {
            unimplemented!()
        }
    }

    fn make_command_edition_mode(command: String, original_item: Option<TodoItem>) -> Mode<'a> {
        let selected_command_index = COMMANDS
            .iter()
            .position(|cmd| cmd.0 == command)
            .unwrap_or(0);

        Mode::Editing {
            what: EditingWhat::Command(
                SelectableList::new(COMMANDS.as_slice()).with_selected(selected_command_index),
            ),
            original_item,
        }
    }

    fn make_parameters_edition_mode(
        parameters: String,
        original_item: Option<TodoItem>,
    ) -> Mode<'a> {
        Mode::Editing {
            what: EditingWhat::Parameters(TextInput::new(parameters.chars())),
            original_item,
        }
    }

    fn apply_edited_command(
        todo_list: &mut SelectableList<Vec<TodoItem>>,
        command: &mut SelectableList<'_, &[Command]>,
    ) {
        todo_list.selected_item_mut().command = command.selected_item().0.to_string();
    }

    fn apply_edited_parameters(
        todo_list: &mut SelectableList<'_, Vec<TodoItem>>,
        parameters: &TextInput,
    ) {
        todo_list.selected_item_mut().parameters = parameters.content().iter().collect::<String>();
    }

    pub fn cancel_edition(&mut self) {
        if let Mode::Editing { original_item, .. } = &mut self.mode {
            match original_item {
                Some(original_item) => {
                    let item = self.todo_list.selected_item_mut();
                    std::mem::swap(item, original_item);
                }
                None => {
                    let index = self.todo_list.selected();
                    self.todo_list.items_mut().remove(index);
                }
            }
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

    pub fn insert_todo_item(&mut self) {
        let index = self.todo_list.selected();
        let command = COMMANDS[0].0.to_string();
        self.todo_list.items_mut().insert(
            index,
            TodoItem {
                command: command.clone(),
                parameters: String::new(),
            },
        );
        self.mode = Self::make_command_edition_mode(command, None);
    }

    pub fn remove_todo_item(&mut self) {
        let index = self.todo_list.selected();
        self.todo_list.items_mut().remove(index);
        if index == self.todo_list.items().len() {
            self.todo_list.select_up(1);
        }
    }

    pub fn duplicate_todo_item(&mut self) {
        let item = self.todo_list.selected_item().clone();
        let index = self.todo_list.selected();
        self.todo_list.items_mut().insert(index, item);
        self.todo_list.select_down(1);
    }

    pub fn select_command_by_char(command: &mut SelectableList<'_, &[Command]>, char: char) {
        struct Found {
            command_index: usize,
            char_pos: usize,
        }

        let mut found = None;

        for (i, Command(cmd)) in command.items().iter().enumerate() {
            if let Some(p) = cmd.chars().position(|ch| ch == char) {
                if found.is_none() || matches!(found, Some(Found { char_pos, .. }) if p < char_pos)
                {
                    found = Some(Found {
                        command_index: i,
                        char_pos: p,
                    });

                    if p == 0 {
                        break;
                    }
                }
            }
        }

        if let Some(Found { command_index, .. }) = found {
            command.select(command_index);
        }
    }

    pub fn show_original_todo_list(&mut self) {
        self.mode = Mode::ShowingOriginal { scroll: 0 };
    }

    pub fn get_todo_list_string(&self) -> Result<String> {
        format_todo_list(self.todo_list.items())
    }
}

fn parse_todo_list(todo_list_lines: &[&str]) -> Vec<TodoItem> {
    todo_list_lines
        .iter()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(|line| {
            let (command, parameters) = line.split_once(' ').unwrap_or((line, ""));
            TodoItem {
                command: command.to_string(),
                parameters: parameters.to_string(),
            }
        })
        .collect()
}

fn format_todo_list(todo_items: &[TodoItem]) -> Result<String> {
    let mut str = String::new();
    for item in todo_items {
        writeln!(&mut str, "{} {}", item.command, item.parameters)?;
    }
    Ok(str)
}
