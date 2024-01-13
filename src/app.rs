use ratatui::widgets::ListState;

pub struct App {
    pub items: Vec<String>,
    pub list_state: ListState,
    pub page_length: usize,
}

impl App {
    pub fn new() -> Self {
        App {
            items: (1..=50).map(|n| format!("{n}")).collect(),
            list_state: ListState::default().with_selected(Some(0)),
            page_length: 0,
        }
    }

    pub fn select_up(&mut self, delta: usize) {
        self.modify_selected(|selected| selected.saturating_sub(delta));
    }

    pub fn select_down(&mut self, delta: usize) {
        let last_index = self.last_item_index();
        self.modify_selected(|selected| (selected + delta).min(last_index));
    }

    fn modify_selected(&mut self, f: impl FnOnce(usize) -> usize) {
        let selected = self.list_state.selected().unwrap();
        let selected = f(selected);
        self.select(selected);
    }

    pub fn select(&mut self, index: usize) {
        self.list_state.select(Some(index));
    }

    pub fn last_item_index(&self) -> usize {
        self.items.len() - 1
    }
}
