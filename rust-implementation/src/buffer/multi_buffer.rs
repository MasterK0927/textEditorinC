use crate::core::{BufferInfo, BufferManager, EditorError, FileManager, Position, Result, TextBuffer};
use crate::buffer::Buffer;
use std::collections::HashMap;

pub struct MultiBuffer<F: FileManager> {
    buffers: Vec<Buffer>,
    buffer_info: Vec<BufferInfo>,
    current_buffer: usize,
    file_manager: F,
    next_buffer_id: usize,
}

impl<F: FileManager> MultiBuffer<F> {
    pub fn new(file_manager: F) -> Self {
        let mut multi_buffer = Self {
            buffers: Vec::new(),
            buffer_info: Vec::new(),
            current_buffer: 0,
            file_manager,
            next_buffer_id: 0,
        };

        // Always start with at least one buffer
        multi_buffer.new_buffer();
        multi_buffer
    }

    pub fn from_files(file_manager: F, filenames: Vec<String>) -> Result<Self> {
        let mut multi_buffer = Self {
            buffers: Vec::new(),
            buffer_info: Vec::new(),
            current_buffer: 0,
            file_manager,
            next_buffer_id: 0,
        };

        if filenames.is_empty() {
            multi_buffer.new_buffer();
        } else {
            for filename in filenames {
                multi_buffer.open_file(&filename)?;
            }
        }

        Ok(multi_buffer)
    }

    pub fn get_current_buffer(&self) -> Option<&Buffer> {
        self.buffers.get(self.current_buffer)
    }

    pub fn get_current_buffer_mut(&mut self) -> Option<&mut Buffer> {
        self.buffers.get_mut(self.current_buffer)
    }

    pub fn get_current_buffer_info(&self) -> Option<&BufferInfo> {
        self.buffer_info.get(self.current_buffer)
    }

    pub fn get_current_buffer_info_mut(&mut self) -> Option<&mut BufferInfo> {
        self.buffer_info.get_mut(self.current_buffer)
    }

    pub fn save_current_buffer(&mut self) -> Result<()> {
        if let (Some(buffer), Some(info)) = (
            self.get_current_buffer(),
            self.get_current_buffer_info_mut(),
        ) {
            self.file_manager.save(&info.filename, buffer.content())?;
            info.is_modified = false;
            Ok(())
        } else {
            Err(EditorError::InvalidOperation("No current buffer".to_string()))
        }
    }

    pub fn next_buffer(&mut self) -> Result<()> {
        if self.buffers.is_empty() {
            return Err(EditorError::InvalidOperation("No buffers available".to_string()));
        }
        self.current_buffer = (self.current_buffer + 1) % self.buffers.len();
        Ok(())
    }

    pub fn previous_buffer(&mut self) -> Result<()> {
        if self.buffers.is_empty() {
            return Err(EditorError::InvalidOperation("No buffers available".to_string()));
        }
        self.current_buffer = if self.current_buffer == 0 {
            self.buffers.len() - 1
        } else {
            self.current_buffer - 1
        };
        Ok(())
    }

    pub fn find_buffer_by_name(&self, filename: &str) -> Option<usize> {
        self.buffer_info
            .iter()
            .position(|info| info.filename == filename)
    }

    pub fn get_buffer_status_line(&self) -> String {
        if let Some(info) = self.get_current_buffer_info() {
            let modified_indicator = if info.is_modified { "*" } else { "" };
            let buffer_indicator = if self.buffers.len() > 1 {
                format!(" [{}/{}]", self.current_buffer + 1, self.buffers.len())
            } else {
                String::new()
            };

            format!(
                "{}{}{}",
                info.filename,
                modified_indicator,
                buffer_indicator
            )
        } else {
            "[No buffer]".to_string()
        }
    }
}

impl<F: FileManager> BufferManager for MultiBuffer<F> {
    fn open_file(&mut self, filename: &str) -> Result<usize> {
        // Check if file is already open
        if let Some(index) = self.find_buffer_by_name(filename) {
            self.current_buffer = index;
            return Ok(index);
        }

        // Try to open the file
        let content = self.file_manager.open(filename)?;
        let buffer = Buffer::from_content(content);
        let info = BufferInfo::new(filename.to_string());

        self.buffers.push(buffer);
        self.buffer_info.push(info);

        let index = self.buffers.len() - 1;
        self.current_buffer = index;
        Ok(index)
    }

    fn new_buffer(&mut self) -> usize {
        let filename = format!("*untitled-{}", self.next_buffer_id);
        self.next_buffer_id += 1;

        let buffer = Buffer::new();
        let info = BufferInfo::new(filename);

        self.buffers.push(buffer);
        self.buffer_info.push(info);

        let index = self.buffers.len() - 1;
        self.current_buffer = index;
        index
    }

    fn switch_to_buffer(&mut self, index: usize) -> Result<()> {
        if index >= self.buffers.len() {
            return Err(EditorError::InvalidOperation(
                format!("Buffer index {} out of range", index)
            ));
        }
        self.current_buffer = index;
        Ok(())
    }

    fn close_buffer(&mut self, index: usize) -> Result<()> {
        if index >= self.buffers.len() {
            return Err(EditorError::InvalidOperation(
                format!("Buffer index {} out of range", index)
            ));
        }

        // Don't close the last buffer
        if self.buffers.len() == 1 {
            // Instead of closing, create a new empty buffer
            let buffer = Buffer::new();
            let info = BufferInfo::new("*untitled*".to_string());
            self.buffers[0] = buffer;
            self.buffer_info[0] = info;
            return Ok(());
        }

        self.buffers.remove(index);
        self.buffer_info.remove(index);

        // Adjust current buffer index
        if self.current_buffer >= index && self.current_buffer > 0 {
            self.current_buffer -= 1;
        } else if self.current_buffer >= self.buffers.len() {
            self.current_buffer = self.buffers.len() - 1;
        }

        Ok(())
    }

    fn get_current_buffer_index(&self) -> usize {
        self.current_buffer
    }

    fn get_buffer_count(&self) -> usize {
        self.buffers.len()
    }

    fn get_buffer_info(&self, index: usize) -> Option<&BufferInfo> {
        self.buffer_info.get(index)
    }

    fn list_buffers(&self) -> Vec<(usize, &BufferInfo)> {
        self.buffer_info
            .iter()
            .enumerate()
            .collect()
    }
}

// Delegate TextBuffer operations to the current buffer
impl<F: FileManager> TextBuffer for MultiBuffer<F> {
    fn content(&self) -> &str {
        self.get_current_buffer()
            .map(|b| b.content())
            .unwrap_or("")
    }

    fn length(&self) -> usize {
        self.get_current_buffer()
            .map(|b| b.length())
            .unwrap_or(0)
    }

    fn is_empty(&self) -> bool {
        self.get_current_buffer()
            .map(|b| b.is_empty())
            .unwrap_or(true)
    }

    fn insert(&mut self, pos: usize, ch: char) -> Result<()> {
        if let Some(buffer) = self.get_current_buffer_mut() {
            let result = buffer.insert(pos, ch);
            if result.is_ok() {
                if let Some(info) = self.get_current_buffer_info_mut() {
                    info.is_modified = true;
                }
            }
            result
        } else {
            Err(EditorError::InvalidOperation("No current buffer".to_string()))
        }
    }

    fn delete(&mut self, pos: usize) -> Result<()> {
        if let Some(buffer) = self.get_current_buffer_mut() {
            let result = buffer.delete(pos);
            if result.is_ok() {
                if let Some(info) = self.get_current_buffer_info_mut() {
                    info.is_modified = true;
                }
            }
            result
        } else {
            Err(EditorError::InvalidOperation("No current buffer".to_string()))
        }
    }

    fn append(&mut self, text: &str) -> Result<()> {
        if let Some(buffer) = self.get_current_buffer_mut() {
            let result = buffer.append(text);
            if result.is_ok() {
                if let Some(info) = self.get_current_buffer_info_mut() {
                    info.is_modified = true;
                }
            }
            result
        } else {
            Err(EditorError::InvalidOperation("No current buffer".to_string()))
        }
    }

    fn clear(&mut self) {
        if let Some(buffer) = self.get_current_buffer_mut() {
            buffer.clear();
            if let Some(info) = self.get_current_buffer_info_mut() {
                info.is_modified = true;
            }
        }
    }

    fn line_count(&self) -> usize {
        self.get_current_buffer()
            .map(|b| b.line_count())
            .unwrap_or(0)
    }

    fn line_length(&self, line: usize) -> usize {
        self.get_current_buffer()
            .map(|b| b.line_length(line))
            .unwrap_or(0)
    }

    fn get_line(&self, line: usize) -> Option<&str> {
        self.get_current_buffer()
            .and_then(|b| b.get_line(line))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::file_io::FileSystem;

    #[test]
    fn test_multi_buffer_creation() {
        let file_manager = FileSystem::new().unwrap();
        let multi_buffer = MultiBuffer::new(file_manager);

        assert_eq!(multi_buffer.get_buffer_count(), 1);
        assert_eq!(multi_buffer.get_current_buffer_index(), 0);
    }

    #[test]
    fn test_new_buffer() {
        let file_manager = FileSystem::new().unwrap();
        let mut multi_buffer = MultiBuffer::new(file_manager);

        multi_buffer.new_buffer();
        assert_eq!(multi_buffer.get_buffer_count(), 2);
        assert_eq!(multi_buffer.get_current_buffer_index(), 1);
    }

    #[test]
    fn test_buffer_switching() {
        let file_manager = FileSystem::new().unwrap();
        let mut multi_buffer = MultiBuffer::new(file_manager);

        multi_buffer.new_buffer();
        multi_buffer.new_buffer();

        assert_eq!(multi_buffer.get_current_buffer_index(), 2);

        multi_buffer.switch_to_buffer(0).unwrap();
        assert_eq!(multi_buffer.get_current_buffer_index(), 0);

        multi_buffer.next_buffer().unwrap();
        assert_eq!(multi_buffer.get_current_buffer_index(), 1);

        multi_buffer.previous_buffer().unwrap();
        assert_eq!(multi_buffer.get_current_buffer_index(), 0);
    }

    #[test]
    fn test_text_operations() {
        let file_manager = FileSystem::new().unwrap();
        let mut multi_buffer = MultiBuffer::new(file_manager);

        multi_buffer.insert(0, 'H').unwrap();
        multi_buffer.insert(1, 'i').unwrap();

        assert_eq!(multi_buffer.content(), "Hi");
        assert!(multi_buffer.get_current_buffer_info().unwrap().is_modified);
    }
}