pub mod core;
pub mod buffer;
pub mod display;
pub mod editor_ops;
pub mod file_io;
pub mod undo;

pub use core::*;
pub use buffer::Buffer;
pub use display::{TerminalDisplay, StatusLine};
pub use editor_ops::{EditorOps, ClipboardManager};
pub use file_io::{FileSystem, SafeFileManager};
pub use undo::{UndoRedoStack, ActionHistory, EditorAction};