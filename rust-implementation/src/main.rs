use anyhow::Result;
use std::env;

use text_editor_rust::{
    ActionHistory, Buffer, EditorAction, EditorMode, EditorOps, EditorState, Position,
    SafeFileManager, StatusLine, TAB_SIZE, TerminalDisplay, UndoRedoStack,
    DisplayManager, EditorOperations, FileManager, TextBuffer, UndoRedoSystem,
};

struct TextEditor {
    state: EditorState,
    buffer: Buffer,
    editor_ops: EditorOps<Buffer>,
    display: TerminalDisplay,
    file_manager: SafeFileManager,
    status_line: StatusLine,
    undo_system: UndoRedoStack<String>,
    action_history: ActionHistory,
    selection_start: Option<usize>,
}

impl TextEditor {
    fn new(filename: Option<String>) -> Result<Self> {
        let filename = filename.unwrap_or_else(|| "untitled.txt".to_string());
        let mut state = EditorState::new(filename.clone());

        let buffer = if std::path::Path::new(&filename).exists() {
            let file_manager = SafeFileManager::new(true, 10_000_000)?; // 10MB limit
            let content = file_manager.open(&filename)?;
            Buffer::from_content(content)
        } else {
            Buffer::new()
        };

        let editor_ops = EditorOps::new(buffer.clone(), (80, 24)); // Default size, will be updated
        let mut display = TerminalDisplay::new();
        let file_manager = SafeFileManager::new(true, 10_000_000)?;
        let status_line = StatusLine::new();
        let undo_system = UndoRedoStack::new();
        let action_history = ActionHistory::new();

        // Initialize display
        display.init()?;
        let screen_size = display.get_size();

        Ok(Self {
            state,
            buffer: buffer.clone(),
            editor_ops: EditorOps::new(buffer, screen_size),
            display,
            file_manager,
            status_line,
            undo_system,
            action_history,
            selection_start: None,
        })
    }

    fn run(&mut self) -> Result<()> {
        // Save initial state
        self.undo_system.save_state(self.buffer.content().to_string());

        loop {
            self.render()?;

            let input = self.display.get_input()?;

            match self.state.mode {
                EditorMode::Edit => {
                    if self.handle_edit_mode_input(input)? {
                        break;
                    }
                }
                EditorMode::Command => {
                    if self.handle_command_mode_input(input)? {
                        break;
                    }
                }
            }
        }

        self.display.cleanup()?;
        Ok(())
    }

    fn render(&mut self) -> Result<()> {
        self.display.clear()?;

        // Render text content
        self.display.render_text(
            self.buffer.content(),
            self.editor_ops.get_cursor_position(),
        )?;

        // Update and render status line
        self.status_line.update(
            &self.state.filename,
            self.editor_ops.get_cursor_position(),
            self.state.mode,
            self.state.is_modified,
        );

        self.display.render_status(&self.status_line.format())?;

        // Move cursor to correct position
        self.display.move_cursor(self.editor_ops.get_cursor_position())?;

        self.display.refresh()?;
        Ok(())
    }

    fn handle_edit_mode_input(&mut self, input: i32) -> Result<bool> {
        match input {
            // Arrow keys
            1001 => { // Up
                self.editor_ops.move_cursor(0, -1)?;
            }
            1002 => { // Down
                self.editor_ops.move_cursor(0, 1)?;
            }
            1003 => { // Left
                self.editor_ops.move_cursor(-1, 0)?;
            }
            1004 => { // Right
                self.editor_ops.move_cursor(1, 0)?;
            }

            // Backspace
            127 | 8 => {
                self.save_undo_state();
                self.editor_ops.delete_char()?;
                self.state.is_modified = true;
                self.record_action(EditorAction::Delete {
                    position: self.buffer_position(),
                    character: ' ', // Simplified - in real impl, track actual character
                });
            }

            // Delete key
            1005 => {
                self.save_undo_state();
                let current_pos = self.editor_ops.get_cursor_position();
                self.editor_ops.move_cursor(1, 0)?;
                self.editor_ops.delete_char()?;
                self.editor_ops.move_to_position(current_pos)?;
                self.state.is_modified = true;
            }

            // Tab
            9 => {
                self.save_undo_state();
                for _ in 0..TAB_SIZE {
                    self.editor_ops.insert_char(' ')?;
                }
                self.state.is_modified = true;
            }

            // Enter
            10 | 13 => {
                self.save_undo_state();
                self.editor_ops.insert_char('\n')?;
                self.state.is_modified = true;
                self.record_action(EditorAction::Insert {
                    position: self.buffer_position(),
                    character: '\n',
                });
            }

            // Escape - switch to command mode
            27 => {
                self.state.mode = EditorMode::Command;
                self.selection_start = None;
            }

            // Home key
            1006 => {
                let current_pos = self.editor_ops.get_cursor_position();
                self.editor_ops.move_to_position(Position::new(0, current_pos.y))?;
            }

            // End key
            1007 => {
                let current_pos = self.editor_ops.get_cursor_position();
                let line_length = self.buffer.line_length(current_pos.y);
                self.editor_ops.move_to_position(Position::new(line_length, current_pos.y))?;
            }

            // Printable characters
            ch if ch >= 32 && ch <= 126 => {
                self.save_undo_state();
                self.editor_ops.insert_char(ch as u8 as char)?;
                self.state.is_modified = true;
                self.record_action(EditorAction::Insert {
                    position: self.buffer_position(),
                    character: ch as u8 as char,
                });
            }

            _ => {
                // Unknown input, ignore
            }
        }

        // Update buffer reference
        self.buffer = self.editor_ops.buffer().clone();
        Ok(false) // Continue running
    }

    fn handle_command_mode_input(&mut self, input: i32) -> Result<bool> {
        match input as u8 as char {
            'q' => {
                if self.state.is_modified {
                    // Ask user if they want to save
                    self.display.render_status("Save before quit? (y/n)")?;
                    self.display.refresh()?;
                    let choice = self.display.get_input()?;
                    if choice == 'y' as i32 || choice == 'Y' as i32 {
                        self.save_file()?;
                    }
                }
                return Ok(true); // Exit
            }

            's' => {
                self.save_file()?;
                self.display.render_status("File saved")?;
                self.display.refresh()?;
            }

            'i' => {
                self.state.mode = EditorMode::Edit;
            }

            'u' => {
                if let Some(content) = self.undo_system.undo() {
                    *self.editor_ops.buffer_mut() = Buffer::from_content(content);
                    self.buffer = self.editor_ops.buffer().clone();
                }
            }

            'r' => {
                if let Some(content) = self.undo_system.redo() {
                    *self.editor_ops.buffer_mut() = Buffer::from_content(content);
                    self.buffer = self.editor_ops.buffer().clone();
                }
            }

            'v' => {
                if self.selection_start.is_none() {
                    self.selection_start = Some(self.buffer_position());
                } else {
                    let start = self.selection_start.unwrap();
                    let end = self.buffer_position();
                    self.editor_ops.copy_selection(start.min(end), start.max(end))?;
                    self.selection_start = None;
                    self.display.render_status("Text copied")?;
                    self.display.refresh()?;
                }
            }

            'x' => {
                if let Some(start) = self.selection_start {
                    let end = self.buffer_position();
                    self.save_undo_state();
                    self.editor_ops.cut_selection(start.min(end), start.max(end))?;
                    self.buffer = self.editor_ops.buffer().clone();
                    self.selection_start = None;
                    self.state.is_modified = true;
                    self.display.render_status("Text cut")?;
                    self.display.refresh()?;
                }
            }

            'p' => {
                self.save_undo_state();
                let clipboard_content = self.editor_ops.clipboard().to_string();
                if !clipboard_content.is_empty() {
                    self.editor_ops.paste(&clipboard_content)?;
                    self.buffer = self.editor_ops.buffer().clone();
                    self.state.is_modified = true;
                    self.display.render_status("Text pasted")?;
                    self.display.refresh()?;
                }
            }

            'h' => {
                self.show_help()?;
            }

            _ => {
                // Unknown command, ignore
            }
        }

        Ok(false) // Continue running
    }

    fn save_file(&mut self) -> Result<()> {
        self.file_manager.save(&self.state.filename, self.buffer.content())?;
        self.state.is_modified = false;
        Ok(())
    }

    fn save_undo_state(&mut self) {
        self.undo_system.save_state(self.buffer.content().to_string());
    }

    fn record_action(&mut self, action: EditorAction) {
        self.action_history.record_action(action);
    }

    fn buffer_position(&self) -> usize {
        let pos = self.editor_ops.get_cursor_position();
        let mut offset = 0;
        for line_idx in 0..pos.y.min(self.buffer.line_count()) {
            offset += self.buffer.line_length(line_idx) + 1; // +1 for newline
        }
        offset + pos.x.min(self.buffer.line_length(pos.y))
    }

    fn show_help(&mut self) -> Result<()> {
        let help_text = r#"
Text Editor Help
================

Edit Mode:
- Arrow keys: Move cursor
- Backspace: Delete character before cursor
- Delete: Delete character at cursor
- Tab: Insert spaces
- Enter: New line
- Escape: Switch to command mode
- Home: Move to beginning of line
- End: Move to end of line

Command Mode:
- i: Switch to edit mode
- q: Quit (prompts to save if modified)
- s: Save file
- u: Undo
- r: Redo
- v: Start/end selection (copy)
- x: Cut selection
- p: Paste
- h: Show this help

Press any key to continue...
"#;

        self.display.clear()?;
        self.display.render_text(help_text, Position::origin())?;
        self.display.refresh()?;
        self.display.get_input()?; // Wait for any key

        Ok(())
    }
}

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    let filename = args.get(1).cloned();

    let mut editor = TextEditor::new(filename)?;
    editor.run()?;

    Ok(())
}