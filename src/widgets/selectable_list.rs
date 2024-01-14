use std::marker::PhantomData;

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::widgets::{List, ListItem, ListState};

pub trait Indexable: Clone {
    type Item;

    fn index(&self, index: usize) -> &Self::Item;
    fn len(&self) -> usize;
}

impl<T> Indexable for &[T] {
    type Item = T;

    fn index(&self, index: usize) -> &Self::Item {
        &self[index]
    }

    fn len(&self) -> usize {
        slice_len(self)
    }
}

#[inline]
fn slice_len<T>(slice: &[T]) -> usize {
    slice.len()
}

impl<T: Clone> Indexable for Vec<T> {
    type Item = T;

    fn index(&self, index: usize) -> &Self::Item {
        &self[index]
    }

    fn len(&self) -> usize {
        self.len()
    }
}

pub trait IndexableMut: Indexable {
    fn index_mut(&mut self, index: usize) -> &mut Self::Item;
}

impl<T: Clone> IndexableMut for Vec<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Item {
        &mut self[index]
    }
}

pub struct SelectableList<'a, T> {
    items: T,
    list_state: ListState,
    phantom: PhantomData<&'a ()>,
}

impl<'a, T> SelectableList<'a, T> {
    pub fn new(items: T) -> Self {
        SelectableList {
            items,
            list_state: ListState::default().with_selected(Some(0)),
            phantom: PhantomData,
        }
    }

    pub fn with_selected(mut self, index: usize) -> Self {
        self.list_state.select(Some(index));
        self
    }

    pub fn selected(&self) -> usize {
        self.list_state.selected().unwrap()
    }

    pub fn select(&mut self, index: usize) {
        self.list_state.select(Some(index));
    }
}

impl<'a, T> SelectableList<'a, T>
where
    T: Indexable + IntoIterator,
    <T as IntoIterator>::Item: Into<ListItem<'a>>,
{
    pub fn items(&self) -> &T {
        &self.items
    }

    pub fn selected_item(&self) -> &<T as Indexable>::Item {
        self.items.index(self.selected())
    }

    pub fn widget_and_state(&mut self) -> (List, &mut ListState) {
        let widget = List::new(self.items.clone());
        let state = &mut self.list_state;
        (widget, state)
    }

    pub fn select_up(&mut self, delta: usize) {
        let selected = self.selected();
        let selected = selected.saturating_sub(delta);
        self.list_state.select(Some(selected));
    }

    pub fn select_down(&mut self, delta: usize) {
        let len = self.items().len();
        let selected = self.selected();
        let selected = (selected + delta).min(len - 1);
        self.list_state.select(Some(selected));
    }

    pub fn select_last(&mut self) {
        let len = self.items().len();
        self.list_state.select(Some(len - 1));
    }

    pub fn input(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Home => self.select(0),
            KeyCode::End => self.select_last(),
            KeyCode::Up => self.select_up(1),
            KeyCode::Down => self.select_down(1),
            _ => {}
        }
    }
}

impl<'a, T> SelectableList<'a, T>
where
    T: IndexableMut + IntoIterator,
    <T as IntoIterator>::Item: Into<ListItem<'a>>,
{
    pub fn items_mut(&mut self) -> &mut T {
        &mut self.items
    }
}
