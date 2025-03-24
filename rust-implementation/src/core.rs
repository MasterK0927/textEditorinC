use thiserror::Error;

#[derive(Error, Debug)]
pub enum EditorError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Buffer operation failed: {0}")]
    Buffer(String),
    #[error("Cursor out of bounds")]
    CursorOutOfBounds,
    #[error("Display error: {0}")]
    Display(String),
    #[error("Invalid operation: {0}")]
    InvalidOperation(String),
}

pub type Result<T> = std::result::Result<T, EditorError>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    pub x: usize,
    pub y: usize,
}

impl Position {
    pub fn new(x: usize, y: usize) -> Self {
        Self { x, y }
    }

    pub fn origin() -> Self {
        Self { x: 0, y: 0 }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditorMode {
    Edit,
    Command,
}

impl Default for EditorMode {
    fn default() -> Self {
        EditorMode::Edit
    }
}

pub trait TextBuffer {
    fn content(&self) -> &str;
    fn length(&self) -> usize;
    fn is_empty(&self) -> bool;
    fn insert(&mut self, pos: usize, ch: char) -> Result<()>;
    fn delete(&mut self, pos: usize) -> Result<()>;
    fn append(&mut self, text: &str) -> Result<()>;
    fn clear(&mut self);
    fn line_count(&self) -> usize;
    fn line_length(&self, line: usize) -> usize;
    fn get_line(&self, line: usize) -> Option<&str>;
}

pub trait UndoRedoSystem<T: Clone> {
    fn save_state(&mut self, state: T);
    fn undo(&mut self) -> Option<T>;
    fn redo(&mut self) -> Option<T>;
    fn can_undo(&self) -> bool;
    fn can_redo(&self) -> bool;
    fn clear(&mut self);
}

pub trait FileManager {
    fn open(&self, filename: &str) -> Result<String>;
    fn save(&self, filename: &str, content: &str) -> Result<()>;
}

pub trait DisplayManager {
    fn init(&mut self) -> Result<()>;
    fn cleanup(&mut self) -> Result<()>;
    fn clear(&mut self) -> Result<()>;
    fn refresh(&mut self) -> Result<()>;
    fn render_text(&mut self, text: &str, position: Position) -> Result<()>;
    fn render_status(&mut self, status: &str) -> Result<()>;
    fn get_input(&mut self) -> Result<i32>;
    fn get_size(&self) -> (usize, usize);
    fn move_cursor(&mut self, position: Position) -> Result<()>;
}

pub trait EditorOperations {
    fn insert_char(&mut self, ch: char) -> Result<()>;
    fn delete_char(&mut self) -> Result<()>;
    fn move_cursor(&mut self, dx: i32, dy: i32) -> Result<()>;
    fn move_to_position(&mut self, position: Position) -> Result<()>;
    fn get_cursor_position(&self) -> Position;
    fn copy_selection(&mut self, start: usize, end: usize) -> Result<String>;
    fn cut_selection(&mut self, start: usize, end: usize) -> Result<String>;
    fn paste(&mut self, text: &str) -> Result<()>;
}

pub struct EditorState {
    pub cursor: Position,
    pub scroll_offset: usize,
    pub filename: String,
    pub mode: EditorMode,
    pub is_modified: bool,
}

impl EditorState {
    pub fn new(filename: String) -> Self {
        Self {
            cursor: Position::origin(),
            scroll_offset: 0,
            filename,
            mode: EditorMode::default(),
            is_modified: false,
        }
    }
}

pub const TAB_SIZE: usize = 4;
pub const MAX_HISTORY: usize = 100;