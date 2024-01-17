use std::cmp::max;

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    prelude::{Buffer, Rect},
    style::{Style, Styled, Stylize},
    widgets::{Block, StatefulWidget, Widget},
};

pub struct TextInputState {
    offset: usize,
    cursor_position: usize,
}
impl TextInputState {
    fn adjust(&mut self, content_len: usize, width: usize) {
        // Fix cursor away from content end
        if self.cursor_position > content_len {
            self.cursor_position = content_len;
        }

        // Fix exceeding offset
        let used_len = max(content_len, self.cursor_position + 1);
        if used_len - self.offset < width {
            self.offset = used_len.saturating_sub(width);
        }

        // Fix cursor outside the area, at left
        if self.cursor_position < self.offset {
            self.offset = self.cursor_position;
        }

        // Fix cursor outside the area, at right
        if self.cursor_position - self.offset >= width {
            self.offset = self.cursor_position - width + 1;
        }
    }
}

pub struct TextInput {
    content: Vec<char>,
    state: TextInputState,
}
impl TextInput {
    pub fn new(content: impl IntoIterator<Item = char>) -> Self {
        let content: Vec<_> = content.into_iter().collect();
        let cursor_position = content.len();
        TextInput {
            content,
            state: TextInputState {
                offset: 0,
                cursor_position,
            },
        }
    }

    pub fn content(&self) -> &[char] {
        &self.content
    }

    pub fn widget_and_state(&mut self) -> (TextInputWidget, &mut TextInputState) {
        let widget = TextInputWidget {
            content: &self.content,
            block: None,
            style: Style::default(),
        };

        (widget, &mut self.state)
    }

    pub fn move_prev(&mut self) {
        if self.state.cursor_position > 0 {
            self.state.cursor_position -= 1;
        }
    }

    pub fn move_next(&mut self) {
        if self.state.cursor_position < self.content.len() {
            self.state.cursor_position += 1;
        }
    }

    pub fn move_begin(&mut self) {
        self.state.cursor_position = 0;
    }

    pub fn move_end(&mut self) {
        self.state.cursor_position = self.content.len()
    }

    pub fn insert(&mut self, char: char) {
        self.content.insert(self.state.cursor_position, char);
        self.state.cursor_position += 1;
    }

    pub fn delete(&mut self) {
        if self.state.cursor_position < self.content.len() {
            self.content.remove(self.state.cursor_position);
        }
    }

    pub fn delete_prev(&mut self) {
        if self.state.cursor_position > 0 {
            self.state.cursor_position -= 1;
            self.content.remove(self.state.cursor_position);
        }
    }

    pub fn input(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Left => self.move_prev(),
            KeyCode::Right => self.move_next(),
            KeyCode::Home => self.move_begin(),
            KeyCode::End => self.move_end(),
            KeyCode::Char(char) => self.insert(char),
            KeyCode::Delete => self.delete(),
            KeyCode::Backspace => self.delete_prev(),
            _ => {}
        }
    }
}

pub struct TextInputWidget<'a> {
    content: &'a [char],
    block: Option<Block<'a>>,
    style: Style,
}
impl<'a> TextInputWidget<'a> {
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }

    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }
}
impl<'a> Styled for TextInputWidget<'a> {
    type Item = TextInputWidget<'a>;

    fn style(&self) -> Style {
        self.style
    }

    fn set_style(self, style: Style) -> Self::Item {
        self.style(style)
    }
}
impl<'a> StatefulWidget for TextInputWidget<'a> {
    type State = TextInputState;

    fn render(mut self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        buf.set_style(area, Style::reset());
        buf.set_style(area, self.style);
        let content_area = match self.block.take() {
            Some(b) => {
                let inner_area = b.inner(area);
                b.render(area, buf);
                inner_area
            }
            None => area,
        };

        if content_area.height < 1 || content_area.width < 1 {
            return;
        }

        state.adjust(self.content.len(), content_area.width as usize);

        // This assumes that each char occupies exactly one cell
        let string: String = self
            .content
            .iter()
            .skip(state.offset)
            .chain(std::iter::repeat(&' '))
            .take(content_area.width as usize)
            .collect();

        buf.set_string(content_area.x, content_area.y, string, Style::default());
        buf.get_mut(
            content_area.x + state.cursor_position as u16 - state.offset as u16,
            content_area.y,
        )
        .set_style(Style::default().reversed().slow_blink());
    }
}

#[cfg(test)]
mod tests {
    use super::TextInputState;

    macro_rules! assert_adjust {
        (len=$len:expr, w=$width:expr; off=$off_in:expr, pos=$pos_in:expr => off=$off_out:expr, pos=$pos_out:expr) => {
            let mut state = TextInputState {
                offset: $off_in,
                cursor_position: $pos_in,
            };

            state.adjust($len, $width);

            assert_eq!(state.offset, $off_out, "wrong offset");
            assert_eq!(state.cursor_position, $pos_out, "wrong cursor_position");
        };
    }

    #[test]
    fn state_adjust() {
        assert_adjust!(len=0, w=1; off=0, pos=0 => off=0, pos=0);
        assert_adjust!(len=3, w=5; off=0, pos=2 => off=0, pos=2);

        // Cursor away from content end
        // [x x x  ]    =>   [x x x  ]
        //          ^               ^
        assert_adjust!(len=3, w=4; off=0, pos=5 => off=0, pos=3);
        assert_adjust!(len=3, w=4; off=0, pos=4 => off=0, pos=3);
        assert_adjust!(len=3, w=4; off=0, pos=3 => off=0, pos=3);

        // Exceeding offset - cursor on content
        // x x[x x  ]   =>   x[x x x]
        //     ^                 ^
        assert_adjust!(len=4, w=3; off=3, pos=2 => off=1, pos=2);
        assert_adjust!(len=4, w=3; off=2, pos=2 => off=1, pos=2);
        assert_adjust!(len=4, w=3; off=1, pos=2 => off=1, pos=2);

        // Exceeding offset - cursor right after content
        // x x[x    ]   =>   x[x x  ]
        //       ^                 ^
        assert_adjust!(len=3, w=3; off=3, pos=3 => off=1, pos=3);
        assert_adjust!(len=3, w=3; off=2, pos=3 => off=1, pos=3);
        assert_adjust!(len=3, w=3; off=1, pos=3 => off=1, pos=3);

        // Cursor outside the area, at left
        // x x[x x]x   =>   x[x x]x x
        //   ^                ^
        assert_adjust!(len=5, w=2; off=3, pos=1 => off=1, pos=1);
        assert_adjust!(len=5, w=2; off=2, pos=1 => off=1, pos=1);
        assert_adjust!(len=5, w=2; off=1, pos=1 => off=1, pos=1);

        // Cursor outside the area, at right
        // x[x x x]    =>  x x[x x  ]
        //         ^               ^
        assert_adjust!(len=4, w=3; off=0, pos=4 => off=2, pos=4);
        assert_adjust!(len=4, w=3; off=1, pos=4 => off=2, pos=4);
        assert_adjust!(len=4, w=3; off=2, pos=4 => off=2, pos=4);
    }
}
