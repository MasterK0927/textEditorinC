use crate::core::{EditorError, EditorOperations, Position, Result, TextBuffer};
use std::collections::HashMap;

pub struct EditorOps<T: TextBuffer> {
    buffer: T,
    cursor: Position,
    clipboard: String,
    selection_start: Option<usize>,
    screen_size: (usize, usize),
}

impl<T: TextBuffer> EditorOps<T> {
    pub fn new(buffer: T, screen_size: (usize, usize)) -> Self {
        Self {
            buffer,
            cursor: Position::origin(),
            clipboard: String::new(),
            selection_start: None,
            screen_size,
        }
    }

    pub fn buffer(&self) -> &T {
        &self.buffer
    }

    pub fn buffer_mut(&mut self) -> &mut T {
        &mut self.buffer
    }

    pub fn set_screen_size(&mut self, size: (usize, usize)) {
        self.screen_size = size;
    }

    pub fn clipboard(&self) -> &str {
        &self.clipboard
    }

    pub fn has_selection(&self) -> bool {
        self.selection_start.is_some()
    }

    pub fn start_selection(&mut self) {
        self.selection_start = Some(self.position_to_buffer_offset());
    }

    pub fn clear_selection(&mut self) {
        self.selection_start = None;
    }

    pub fn get_selection_range(&self) -> Option<(usize, usize)> {
        self.selection_start.map(|start| {
            let end = self.position_to_buffer_offset();
            if start <= end {
                (start, end)
            } else {
                (end, start)
            }
        })
    }

    fn position_to_buffer_offset(&self) -> usize {
        let mut offset = 0;
        for line_idx in 0..self.cursor.y.min(self.buffer.line_count()) {
            offset += self.buffer.line_length(line_idx) + 1; // +1 for newline
        }
        offset + self.cursor.x.min(self.buffer.line_length(self.cursor.y))
    }

    fn buffer_offset_to_position(&self, offset: usize) -> Position {
        let mut current_offset = 0;

        for (line_idx, _) in (0..self.buffer.line_count()).enumerate() {
            let line_len = self.buffer.line_length(line_idx);

            if current_offset + line_len >= offset {
                return Position::new(offset - current_offset, line_idx);
            }

            current_offset += line_len + 1; // +1 for newline
        }

        // If offset is beyond buffer, return end position
        let last_line = self.buffer.line_count().saturating_sub(1);
        Position::new(self.buffer.line_length(last_line), last_line)
    }

    fn constrain_cursor(&mut self) {
        let line_count = self.buffer.line_count();
        if line_count == 0 {
            self.cursor = Position::origin();
            return;
        }

        // Constrain Y to valid lines
        if self.cursor.y >= line_count {
            self.cursor.y = line_count - 1;
        }

        // Constrain X to line length
        let line_length = self.buffer.line_length(self.cursor.y);
        if self.cursor.x > line_length {
            self.cursor.x = line_length;
        }
    }
}

impl<T: TextBuffer> EditorOperations for EditorOps<T> {
    fn insert_char(&mut self, ch: char) -> Result<()> {
        let offset = self.position_to_buffer_offset();
        self.buffer.insert(offset, ch)?;

        if ch == '\n' {
            self.cursor.y += 1;
            self.cursor.x = 0;
        } else {
            self.cursor.x += 1;
        }

        self.constrain_cursor();
        Ok(())
    }

    fn delete_char(&mut self) -> Result<()> {
        if self.cursor.x == 0 && self.cursor.y == 0 {
            return Ok(()); // Nothing to delete at start of buffer
        }

        let offset = if self.cursor.x == 0 {
            // Delete newline at beginning of line
            self.cursor.y -= 1;
            self.cursor.x = self.buffer.line_length(self.cursor.y);
            self.position_to_buffer_offset()
        } else {
            // Delete character before cursor
            self.cursor.x -= 1;
            self.position_to_buffer_offset()
        };

        self.buffer.delete(offset)?;
        self.constrain_cursor();
        Ok(())
    }

    fn move_cursor(&mut self, dx: i32, dy: i32) -> Result<()> {
        let new_x = (self.cursor.x as i32 + dx).max(0) as usize;
        let new_y = (self.cursor.y as i32 + dy).max(0) as usize;

        self.cursor = Position::new(new_x, new_y);
        self.constrain_cursor();
        Ok(())
    }

    fn move_to_position(&mut self, position: Position) -> Result<()> {
        self.cursor = position;
        self.constrain_cursor();
        Ok(())
    }

    fn get_cursor_position(&self) -> Position {
        self.cursor
    }

    fn copy_selection(&mut self, start: usize, end: usize) -> Result<String> {
        if start >= self.buffer.length() || end > self.buffer.length() || start >= end {
            return Err(EditorError::InvalidOperation("Invalid selection range".to_string()));
        }

        let content = self.buffer.content();
        let selected = content.chars().skip(start).take(end - start).collect::<String>();
        self.clipboard = selected.clone();
        Ok(selected)
    }

    fn cut_selection(&mut self, start: usize, end: usize) -> Result<String> {
        let selected = self.copy_selection(start, end)?;

        // Delete the selected text
        for _ in start..end {
            if start < self.buffer.length() {
                self.buffer.delete(start)?;
            }
        }

        // Adjust cursor position
        self.cursor = self.buffer_offset_to_position(start);
        self.constrain_cursor();

        Ok(selected)
    }

    fn paste(&mut self, text: &str) -> Result<()> {
        let offset = self.position_to_buffer_offset();

        for ch in text.chars() {
            self.buffer.insert(offset + (self.position_to_buffer_offset() - offset), ch)?;
            if ch == '\n' {
                self.cursor.y += 1;
                self.cursor.x = 0;
            } else {
                self.cursor.x += 1;
            }
        }

        self.constrain_cursor();
        Ok(())
    }
}

pub struct ClipboardManager {
    clipboard: String,
}

impl ClipboardManager {
    pub fn new() -> Self {
        Self {
            clipboard: String::new(),
        }
    }

    pub fn copy(&mut self, text: String) {
        self.clipboard = text;
    }

    pub fn paste(&self) -> &str {
        &self.clipboard
    }

    pub fn is_empty(&self) -> bool {
        self.clipboard.is_empty()
    }
}

impl Default for ClipboardManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::buffer::Buffer;

    #[test]
    fn test_insert_and_move_cursor() {
        let buffer = Buffer::new();
        let mut ops = EditorOps::new(buffer, (80, 24));

        ops.insert_char('H').unwrap();
        ops.insert_char('i').unwrap();

        assert_eq!(ops.buffer().content(), "Hi");
        assert_eq!(ops.get_cursor_position(), Position::new(2, 0));
    }

    #[test]
    fn test_delete_char() {
        let buffer = Buffer::from_content("Hello".to_string());
        let mut ops = EditorOps::new(buffer, (80, 24));

        // Move to end
        ops.move_to_position(Position::new(5, 0)).unwrap();
        ops.delete_char().unwrap();

        assert_eq!(ops.buffer().content(), "Hell");
        assert_eq!(ops.get_cursor_position(), Position::new(4, 0));
    }

    #[test]
    fn test_copy_paste() {
        let buffer = Buffer::from_content("Hello World".to_string());
        let mut ops = EditorOps::new(buffer, (80, 24));

        let copied = ops.copy_selection(0, 5).unwrap();
        assert_eq!(copied, "Hello");

        ops.move_to_position(Position::new(11, 0)).unwrap();
        ops.paste(&copied).unwrap();

        assert_eq!(ops.buffer().content(), "Hello WorldHello");
    }
}