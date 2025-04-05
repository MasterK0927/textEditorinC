use crate::core::{EditorError, Result, TextBuffer};
use std::collections::VecDeque;

pub mod multi_buffer;
pub use multi_buffer::MultiBuffer;

#[derive(Debug, Clone)]
pub struct Buffer {
    content: String,
    lines: Vec<String>,
}

impl Buffer {
    pub fn new() -> Self {
        Self {
            content: String::new(),
            lines: vec![String::new()],
        }
    }

    pub fn from_content(content: String) -> Self {
        let lines: Vec<String> = if content.is_empty() {
            vec![String::new()]
        } else {
            content.lines().map(|s| s.to_string()).collect()
        };

        Self { content, lines }
    }

    fn rebuild_content(&mut self) {
        self.content = self.lines.join("\n");
    }

    fn rebuild_lines(&mut self) {
        self.lines = if self.content.is_empty() {
            vec![String::new()]
        } else {
            self.content.lines().map(|s| s.to_string()).collect()
        };
    }

    fn position_to_line_col(&self, pos: usize) -> Result<(usize, usize)> {
        let mut current_pos = 0;

        for (line_idx, line) in self.lines.iter().enumerate() {
            if current_pos + line.len() >= pos {
                return Ok((line_idx, pos - current_pos));
            }
            current_pos += line.len() + 1; // +1 for newline
        }

        Err(EditorError::CursorOutOfBounds)
    }

    fn line_col_to_position(&self, line: usize, col: usize) -> Result<usize> {
        if line >= self.lines.len() {
            return Err(EditorError::CursorOutOfBounds);
        }

        let mut pos = 0;
        for i in 0..line {
            pos += self.lines[i].len() + 1; // +1 for newline
        }

        if col > self.lines[line].len() {
            return Err(EditorError::CursorOutOfBounds);
        }

        Ok(pos + col)
    }
}

impl Default for Buffer {
    fn default() -> Self {
        Self::new()
    }
}

impl TextBuffer for Buffer {
    fn content(&self) -> &str {
        &self.content
    }

    fn length(&self) -> usize {
        self.content.len()
    }

    fn is_empty(&self) -> bool {
        self.content.is_empty() || (self.lines.len() == 1 && self.lines[0].is_empty())
    }

    fn insert(&mut self, pos: usize, ch: char) -> Result<()> {
        if pos > self.content.len() {
            return Err(EditorError::CursorOutOfBounds);
        }

        if ch == '\n' {
            let (line_idx, col) = self.position_to_line_col(pos)?;
            let current_line = &self.lines[line_idx];
            let (left, right) = current_line.split_at(col);

            self.lines[line_idx] = left.to_string();
            self.lines.insert(line_idx + 1, right.to_string());
        } else {
            let (line_idx, col) = self.position_to_line_col(pos)?;
            self.lines[line_idx].insert(col, ch);
        }

        self.rebuild_content();
        Ok(())
    }

    fn delete(&mut self, pos: usize) -> Result<()> {
        if pos >= self.content.len() {
            return Err(EditorError::CursorOutOfBounds);
        }

        let (line_idx, col) = self.position_to_line_col(pos)?;

        if col == 0 && line_idx > 0 {
            // Delete newline - merge with previous line
            let current_line = self.lines.remove(line_idx);
            self.lines[line_idx - 1].push_str(&current_line);
        } else if col > 0 {
            // Delete character in current line
            self.lines[line_idx].remove(col - 1);
        } else {
            return Err(EditorError::InvalidOperation("Cannot delete at beginning of buffer".to_string()));
        }

        self.rebuild_content();
        Ok(())
    }

    fn append(&mut self, text: &str) -> Result<()> {
        if text.is_empty() {
            return Ok(());
        }

        self.content.push_str(text);
        self.rebuild_lines();
        Ok(())
    }

    fn clear(&mut self) {
        self.content.clear();
        self.lines = vec![String::new()];
    }

    fn line_count(&self) -> usize {
        self.lines.len()
    }

    fn line_length(&self, line: usize) -> usize {
        self.lines.get(line).map(|l| l.len()).unwrap_or(0)
    }

    fn get_line(&self, line: usize) -> Option<&str> {
        self.lines.get(line).map(|s| s.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_buffer() {
        let buffer = Buffer::new();
        assert!(buffer.is_empty());
        assert_eq!(buffer.line_count(), 1);
        assert_eq!(buffer.length(), 0);
    }

    #[test]
    fn test_insert_characters() {
        let mut buffer = Buffer::new();
        buffer.insert(0, 'H').unwrap();
        buffer.insert(1, 'i').unwrap();
        assert_eq!(buffer.content(), "Hi");
        assert_eq!(buffer.line_count(), 1);
    }

    #[test]
    fn test_insert_newline() {
        let mut buffer = Buffer::new();
        buffer.insert(0, 'H').unwrap();
        buffer.insert(1, '\n').unwrap();
        buffer.insert(2, 'i').unwrap();
        assert_eq!(buffer.content(), "H\ni");
        assert_eq!(buffer.line_count(), 2);
    }

    #[test]
    fn test_delete_character() {
        let mut buffer = Buffer::from_content("Hello".to_string());
        buffer.delete(4).unwrap(); // Delete 'o'
        assert_eq!(buffer.content(), "Hell");
    }

    #[test]
    fn test_append() {
        let mut buffer = Buffer::new();
        buffer.append("Hello\nWorld").unwrap();
        assert_eq!(buffer.content(), "Hello\nWorld");
        assert_eq!(buffer.line_count(), 2);
    }
}