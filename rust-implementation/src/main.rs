use anyhow::Result;
use clap::Parser;
use std::env;

use text_editor_rust::{
    ActionHistory, Buffer, BufferManager, EditorAction, EditorMode, EditorOps, Position,
    SafeFileManager, StatusLine, TAB_SIZE, TerminalDisplay, UndoRedoStack, MultiBuffer,
    DisplayManager, EditorOperations, FileManager, TextBuffer, UndoRedoSystem,
};

#[derive(Parser)]
#[command(name = "text-editor")]
#[command(about = "A vim-like text editor written in Rust")]
#[command(version = "1.0")]
struct Cli {
    /// Files to open
    files: Vec<String>,

    /// Start in read-only mode
    #[arg(short, long)]
    readonly: bool,

    /// Set tab size
    #[arg(long, default_value_t = 4)]
    tab_size: usize,
}

struct VimLikeEditor {
    multi_buffer: MultiBuffer<SafeFileManager>,
    editor_ops: EditorOps<MultiBuffer<SafeFileManager>>,
    display: TerminalDisplay,
    status_line: StatusLine,
    undo_system: UndoRedoStack<String>,
    action_history: ActionHistory,
    selection_start: Option<usize>,
    mode: EditorMode,
    command_buffer: String,
    readonly: bool,
}

impl VimLikeEditor {
    fn new(files: Vec<String>, readonly: bool) -> Result<Self> {
        let file_manager = SafeFileManager::new(true, 10_000_000)?; // 10MB limit
        let multi_buffer = if files.is_empty() {
            MultiBuffer::new(file_manager)
        } else {
            MultiBuffer::from_files(file_manager, files)?
        };

        let editor_ops = EditorOps::new(multi_buffer, (80, 24)); // Default size, will be updated
        let mut display = TerminalDisplay::new();
        let status_line = StatusLine::new();
        let undo_system = UndoRedoStack::new();
        let action_history = ActionHistory::new();

        // Initialize display
        display.init()?;
        let screen_size = display.get_size();

        Ok(Self {
            multi_buffer: editor_ops.buffer().clone(),
            editor_ops: EditorOps::new(multi_buffer, screen_size),
            display,
            status_line,
            undo_system,
            action_history,
            selection_start: None,
            mode: EditorMode::Edit,
            command_buffer: String::new(),
            readonly,
        })
    }

    fn run(&mut self) -> Result<()> {
        // Save initial state
        self.undo_system.save_state(self.multi_buffer.content().to_string());

        loop {
            self.render()?;

            let input = self.display.get_input()?;

            match self.mode {
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
            self.multi_buffer.content(),
            self.editor_ops.get_cursor_position(),
        )?;

        // Update and render status line
        let current_info = self.multi_buffer.get_current_buffer_info();
        let filename = current_info.map(|info| info.filename.as_str()).unwrap_or("No buffer");
        let is_modified = current_info.map(|info| info.is_modified).unwrap_or(false);

        self.status_line.update(
            filename,
            self.editor_ops.get_cursor_position(),
            self.mode,
            is_modified,
        );

        let status_text = if !self.command_buffer.is_empty() {
            format!(":{} | {}", self.command_buffer, self.status_line.format())
        } else {
            format!("{} | {}", self.multi_buffer.get_buffer_status_line(), self.status_line.format())
        };

        self.display.render_status(&status_text)?;

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
                if !self.readonly {
                    self.save_undo_state();
                    self.editor_ops.delete_char()?;
                    self.mark_modified();
                }
            }

            // Delete key
            1005 => {
                if !self.readonly {
                    self.save_undo_state();
                    let current_pos = self.editor_ops.get_cursor_position();
                    self.editor_ops.move_cursor(1, 0)?;
                    self.editor_ops.delete_char()?;
                    self.editor_ops.move_to_position(current_pos)?;
                    self.mark_modified();
                }
            }

            // Tab
            9 => {
                if !self.readonly {
                    self.save_undo_state();
                    for _ in 0..TAB_SIZE {
                        self.editor_ops.insert_char(' ')?;
                    }
                    self.mark_modified();
                }
            }

            // Enter
            10 | 13 => {
                if !self.readonly {
                    self.save_undo_state();
                    self.editor_ops.insert_char('\n')?;
                    self.mark_modified();
                }
            }

            // Escape - switch to command mode
            27 => {
                self.mode = EditorMode::Command;
                self.selection_start = None;
                self.command_buffer.clear();
            }

            // Colon - start command mode with command input
            58 if input == ':' as i32 => { // ':'
                self.mode = EditorMode::Command;
                self.command_buffer.push(':');
            }

            // Home key
            1006 => {
                let current_pos = self.editor_ops.get_cursor_position();
                self.editor_ops.move_to_position(Position::new(0, current_pos.y))?;
            }

            // End key
            1007 => {
                let current_pos = self.editor_ops.get_cursor_position();
                let line_length = self.multi_buffer.line_length(current_pos.y);
                self.editor_ops.move_to_position(Position::new(line_length, current_pos.y))?;
            }

            // Printable characters
            ch if ch >= 32 && ch <= 126 => {
                if !self.readonly {
                    self.save_undo_state();
                    self.editor_ops.insert_char(ch as u8 as char)?;
                    self.mark_modified();
                }
            }

            _ => {
                // Unknown input, ignore
            }
        }

        // Update buffer reference
        self.multi_buffer = self.editor_ops.buffer().clone();
        Ok(false) // Continue running
    }

    fn handle_command_mode_input(&mut self, input: i32) -> Result<bool> {
        match input {
            // Enter - execute command
            10 | 13 => {
                if !self.command_buffer.is_empty() {
                    if let Some(should_quit) = self.execute_command()? {
                        return Ok(should_quit);
                    }
                }
                self.command_buffer.clear();
                self.mode = EditorMode::Edit;
            }

            // Escape - cancel command
            27 => {
                self.command_buffer.clear();
                self.mode = EditorMode::Edit;
            }

            // Backspace in command buffer
            127 | 8 => {
                if !self.command_buffer.is_empty() {
                    self.command_buffer.pop();
                    if self.command_buffer.is_empty() {
                        self.mode = EditorMode::Edit;
                    }
                }
            }

            // Single character commands (when no command buffer)
            ch if self.command_buffer.is_empty() => {
                match ch as u8 as char {
                    'q' => {
                        return self.handle_quit();
                    }
                    's' => {
                        self.save_current_file()?;
                    }
                    'i' => {
                        self.mode = EditorMode::Edit;
                    }
                    'u' => {
                        self.undo()?;
                    }
                    'r' => {
                        self.redo()?;
                    }
                    'n' => {
                        self.multi_buffer.next_buffer()?;
                        self.update_editor_ops();
                    }
                    'p' => {
                        self.multi_buffer.previous_buffer()?;
                        self.update_editor_ops();
                    }
                    'h' => {
                        self.show_help()?;
                    }
                    ':' => {
                        self.command_buffer.push(':');
                    }
                    _ => {
                        // Unknown command, ignore
                    }
                }
            }

            // Add character to command buffer
            ch if ch >= 32 && ch <= 126 => {
                self.command_buffer.push(ch as u8 as char);
            }

            _ => {
                // Unknown input, ignore
            }
        }

        Ok(false)
    }

    fn execute_command(&mut self) -> Result<Option<bool>> {
        let command = self.command_buffer.trim_start_matches(':');
        let parts: Vec<&str> = command.split_whitespace().collect();

        if parts.is_empty() {
            return Ok(None);
        }

        match parts[0] {
            "q" | "quit" => {
                return Ok(Some(self.handle_quit()?));
            }
            "w" | "write" => {
                if parts.len() > 1 {
                    // Save as different filename
                    self.save_as(parts[1])?;
                } else {
                    self.save_current_file()?;
                }
            }
            "wq" => {
                self.save_current_file()?;
                return Ok(Some(true));
            }
            "e" | "edit" => {
                if parts.len() > 1 {
                    self.open_file(parts[1])?;
                }
            }
            "o" | "open" => {
                if parts.len() > 1 {
                    self.open_file(parts[1])?;
                }
            }
            "new" => {
                self.multi_buffer.new_buffer();
                self.update_editor_ops();
            }
            "bd" | "bdelete" => {
                let index = if parts.len() > 1 {
                    parts[1].parse().unwrap_or(self.multi_buffer.get_current_buffer_index())
                } else {
                    self.multi_buffer.get_current_buffer_index()
                };
                self.multi_buffer.close_buffer(index)?;
                self.update_editor_ops();
            }
            "ls" | "buffers" => {
                self.show_buffer_list()?;
            }
            "b" | "buffer" => {
                if parts.len() > 1 {
                    if let Ok(index) = parts[1].parse::<usize>() {
                        if index > 0 {
                            self.multi_buffer.switch_to_buffer(index - 1)?;
                            self.update_editor_ops();
                        }
                    }
                }
            }
            "help" => {
                self.show_help()?;
            }
            _ => {
                self.display.render_status(&format!("Unknown command: {}", command))?;
                self.display.refresh()?;
            }
        }

        Ok(None)
    }

    fn handle_quit(&mut self) -> Result<bool> {
        // Check if any buffers are modified
        let modified_buffers: Vec<_> = self.multi_buffer.list_buffers()
            .into_iter()
            .filter(|(_, info)| info.is_modified)
            .collect();

        if !modified_buffers.is_empty() {
            let msg = format!("{} file(s) modified. Save before quit? (y/n/a)", modified_buffers.len());
            self.display.render_status(&msg)?;
            self.display.refresh()?;

            let choice = self.display.get_input()?;
            match choice as u8 as char {
                'y' | 'Y' => {
                    // Save current buffer and quit
                    self.save_current_file()?;
                    return Ok(true);
                }
                'a' | 'A' => {
                    // Save all modified buffers
                    for (index, _) in modified_buffers {
                        self.multi_buffer.switch_to_buffer(index)?;
                        self.multi_buffer.save_current_buffer()?;
                    }
                    return Ok(true);
                }
                'n' | 'N' => {
                    return Ok(true); // Quit without saving
                }
                _ => {
                    return Ok(false); // Cancel quit
                }
            }
        }

        Ok(true) // No modified buffers, safe to quit
    }

    fn save_current_file(&mut self) -> Result<()> {
        if self.readonly {
            self.display.render_status("Cannot save in read-only mode")?;
            self.display.refresh()?;
            return Ok(());
        }

        self.multi_buffer.save_current_buffer()?;
        self.display.render_status("File saved")?;
        self.display.refresh()?;
        Ok(())
    }

    fn save_as(&mut self, filename: &str) -> Result<()> {
        if self.readonly {
            self.display.render_status("Cannot save in read-only mode")?;
            self.display.refresh()?;
            return Ok(());
        }

        if let Some(info) = self.multi_buffer.get_current_buffer_info_mut() {
            info.filename = filename.to_string();
            self.multi_buffer.save_current_buffer()?;
            self.display.render_status(&format!("Saved as {}", filename))?;
            self.display.refresh()?;
        }
        Ok(())
    }

    fn open_file(&mut self, filename: &str) -> Result<()> {
        match self.multi_buffer.open_file(filename) {
            Ok(_) => {
                self.update_editor_ops();
                self.display.render_status(&format!("Opened {}", filename))?;
                self.display.refresh()?;
            }
            Err(e) => {
                self.display.render_status(&format!("Error opening {}: {}", filename, e))?;
                self.display.refresh()?;
            }
        }
        Ok(())
    }

    fn show_buffer_list(&mut self) -> Result<()> {
        let buffers = self.multi_buffer.list_buffers();
        let current_index = self.multi_buffer.get_current_buffer_index();

        let mut buffer_text = String::from("Buffers:\n");
        for (index, info) in buffers {
            let marker = if index == current_index { "*" } else { " " };
            let modified = if info.is_modified { "+" } else { " " };
            buffer_text.push_str(&format!("{}{}{:3}: {}\n", marker, modified, index + 1, info.filename));
        }
        buffer_text.push_str("\nPress any key to continue...");

        self.display.clear()?;
        self.display.render_text(&buffer_text, Position::origin())?;
        self.display.refresh()?;
        self.display.get_input()?; // Wait for any key

        Ok(())
    }

    fn undo(&mut self) -> Result<()> {
        if let Some(content) = self.undo_system.undo() {
            if let Some(buffer) = self.multi_buffer.get_current_buffer_mut() {
                *buffer = Buffer::from_content(content);
                self.update_editor_ops();
            }
        }
        Ok(())
    }

    fn redo(&mut self) -> Result<()> {
        if let Some(content) = self.undo_system.redo() {
            if let Some(buffer) = self.multi_buffer.get_current_buffer_mut() {
                *buffer = Buffer::from_content(content);
                self.update_editor_ops();
            }
        }
        Ok(())
    }

    fn update_editor_ops(&mut self) {
        self.editor_ops = EditorOps::new(self.multi_buffer.clone(), self.display.get_size());
    }

    fn save_undo_state(&mut self) {
        self.undo_system.save_state(self.multi_buffer.content().to_string());
    }

    fn mark_modified(&mut self) {
        if let Some(info) = self.multi_buffer.get_current_buffer_info_mut() {
            info.is_modified = true;
        }
    }

    fn show_help(&mut self) -> Result<()> {
        let help_text = r#"
Vim-like Text Editor Help
=========================

File Operations:
  :e <file>    - Edit/open file
  :o <file>    - Open file (same as :e)
  :w           - Write/save current file
  :w <file>    - Save as different filename
  :wq          - Write and quit
  :q           - Quit (prompts if modified)

Buffer Operations:
  :new         - Create new buffer
  :ls          - List all buffers
  :b <num>     - Switch to buffer number
  :bd          - Delete current buffer
  n            - Next buffer (in command mode)
  p            - Previous buffer (in command mode)

Edit Mode:
  Arrow keys   - Move cursor
  Backspace    - Delete character before cursor
  Delete       - Delete character at cursor
  Tab          - Insert spaces
  Enter        - New line
  Escape       - Switch to command mode
  :            - Start command input

Command Mode:
  i            - Switch to edit mode
  u            - Undo
  r            - Redo
  h            - Show this help

Command-line Arguments:
  text-editor [files...]  - Open multiple files
  --readonly              - Read-only mode
  --tab-size <n>          - Set tab size

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
    let cli = Cli::parse();

    let mut editor = VimLikeEditor::new(cli.files, cli.readonly)?;
    editor.run()?;

    Ok(())
}