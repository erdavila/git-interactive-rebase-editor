use std::io::Write;

use anyhow::Result;
use crossterm::{
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::Backend};

pub struct Tui<B: Backend> {
    pub terminal: Terminal<B>,
}

impl<B: Backend + Write> Tui<B> {
    pub fn new(terminal: Terminal<B>) -> Self {
        Tui { terminal }
    }

    pub fn enter(&mut self) -> Result<()>
    where
        <B as Backend>::Error: Send + Sync + 'static,
    {
        enable_raw_mode()?;
        execute!(self.terminal.backend_mut(), EnterAlternateScreen)?;
        self.terminal.hide_cursor()?;
        self.terminal.clear()?;

        Ok(())
    }

    pub fn reset(&mut self) -> Result<()>
    where
        <B as Backend>::Error: Send + Sync + 'static,
    {
        disable_raw_mode()?;
        execute!(self.terminal.backend_mut(), LeaveAlternateScreen)?;
        self.terminal.show_cursor()?;

        Ok(())
    }
}
