use crate::core::{DisplayManager, EditorError, EditorMode, Position, Result};
use pancurses::{curs_set, endwin, has_colors, init_pair, initscr, noecho, raw, start_color, Window, Input, COLOR_PAIR};
use std::collections::HashMap;

const COLOR_KEYWORD: i16 = 1;
const COLOR_NUMBER: i16 = 2;
const COLOR_STRING: i16 = 3;
const COLOR_CURSOR: i16 = 4;

pub struct TerminalDisplay {
    main_window: Option<Window>,
    status_window: Option<Window>,
    screen_size: (usize, usize),
    keywords: Vec<String>,
}

impl TerminalDisplay {
    pub fn new() -> Self {
        let keywords = vec![
            "fn", "let", "mut", "if", "else", "while", "for", "match", "struct", "enum",
            "impl", "trait", "pub", "use", "mod", "return", "break", "continue", "loop",
            "true", "false", "None", "Some", "Ok", "Err", "const", "static", "unsafe",
            "async", "await", "move", "ref", "where", "type", "as", "in"
        ].into_iter().map(|s| s.to_string()).collect();

        Self {
            main_window: None,
            status_window: None,
            screen_size: (0, 0),
            keywords,
        }
    }

    fn setup_colors(&self) -> Result<()> {
        if has_colors() {
            start_color();
            init_pair(COLOR_KEYWORD, pancurses::COLOR_BLUE, pancurses::COLOR_BLACK);
            init_pair(COLOR_NUMBER, pancurses::COLOR_CYAN, pancurses::COLOR_BLACK);
            init_pair(COLOR_STRING, pancurses::COLOR_RED, pancurses::COLOR_BLACK);
            init_pair(COLOR_CURSOR, pancurses::COLOR_BLACK, pancurses::COLOR_WHITE);
        }
        Ok(())
    }

    fn create_windows(&mut self) -> Result<()> {
        let main_win = initscr();
        let (height, width) = main_win.get_max_yx();

        if height < 2 || width < 10 {
            return Err(EditorError::Display("Terminal too small".to_string()));
        }

        self.screen_size = (width as usize, height as usize);

        // Create status window (last line)
        let status_win = main_win.subwin(1, width, height - 1, 0)
            .map_err(|_| EditorError::Display("Failed to create status window".to_string()))?;

        // Main editor window (all but last line)
        let editor_win = main_win.subwin(height - 1, width, 0, 0)
            .map_err(|_| EditorError::Display("Failed to create editor window".to_string()))?;

        self.main_window = Some(editor_win);
        self.status_window = Some(status_win);

        Ok(())
    }

    fn highlight_syntax(&self, window: &Window, text: &str, line_y: i32, cursor_pos: Option<usize>) {
        let mut x = 0;
        let mut chars = text.char_indices().peekable();

        while let Some((byte_idx, ch)) = chars.next() {
            let mut highlighted = false;

            // Check if this is the cursor position
            if let Some(cursor_x) = cursor_pos {
                if byte_idx == cursor_x {
                    window.attron(COLOR_PAIR(COLOR_CURSOR as u32));
                    window.mvaddch(line_y, x, ch);
                    window.attroff(COLOR_PAIR(COLOR_CURSOR as u32));
                    x += 1;
                    continue;
                }
            }

            // Check for keywords
            if ch.is_alphabetic() || ch == '_' {
                let word_start = byte_idx;
                let mut word_end = byte_idx + ch.len_utf8();

                // Find the end of the word
                while let Some((_, next_ch)) = chars.peek() {
                    if next_ch.is_alphanumeric() || *next_ch == '_' {
                        let (next_idx, next_ch) = chars.next().unwrap();
                        word_end = next_idx + next_ch.len_utf8();
                    } else {
                        break;
                    }
                }

                let word = &text[word_start..word_end];
                if self.keywords.contains(&word.to_string()) {
                    window.attron(COLOR_PAIR(COLOR_KEYWORD as u32));
                    window.mvaddstr(line_y, x, word);
                    window.attroff(COLOR_PAIR(COLOR_KEYWORD as u32));
                    x += word.chars().count() as i32;
                    highlighted = true;
                }
            }

            // Check for numbers
            if !highlighted && ch.is_ascii_digit() {
                window.attron(COLOR_PAIR(COLOR_NUMBER as u32));
                window.mvaddch(line_y, x, ch);
                window.attroff(COLOR_PAIR(COLOR_NUMBER as u32));
                highlighted = true;
            }

            // Check for strings
            if !highlighted && ch == '"' {
                window.attron(COLOR_PAIR(COLOR_STRING as u32));
                window.mvaddch(line_y, x, ch);

                // Continue until closing quote
                while let Some((_, next_ch)) = chars.next() {
                    x += 1;
                    window.mvaddch(line_y, x, next_ch);
                    if next_ch == '"' {
                        break;
                    }
                }
                window.attroff(COLOR_PAIR(COLOR_STRING as u32));
                highlighted = true;
            }

            if !highlighted {
                window.mvaddch(line_y, x, ch);
            }

            x += 1;
        }
    }
}

impl Default for TerminalDisplay {
    fn default() -> Self {
        Self::new()
    }
}

impl DisplayManager for TerminalDisplay {
    fn init(&mut self) -> Result<()> {
        self.create_windows()?;
        self.setup_colors()?;

        noecho();
        raw();
        curs_set(0); // Hide cursor

        if let Some(ref main_win) = self.main_window {
            main_win.keypad(true);
        }

        Ok(())
    }

    fn cleanup(&mut self) -> Result<()> {
        endwin();
        Ok(())
    }

    fn clear(&mut self) -> Result<()> {
        if let Some(ref main_win) = self.main_window {
            main_win.clear();
        }
        if let Some(ref status_win) = self.status_window {
            status_win.clear();
        }
        Ok(())
    }

    fn refresh(&mut self) -> Result<()> {
        if let Some(ref main_win) = self.main_window {
            main_win.refresh();
        }
        if let Some(ref status_win) = self.status_window {
            status_win.refresh();
        }
        Ok(())
    }

    fn render_text(&mut self, text: &str, position: Position) -> Result<()> {
        if let Some(ref main_win) = self.main_window {
            let lines: Vec<&str> = text.lines().collect();
            let (_, height) = self.screen_size;
            let editor_height = height - 1; // Subtract status bar

            for (i, line) in lines.iter().enumerate() {
                let y = i as i32;
                if y >= editor_height as i32 {
                    break;
                }

                main_win.mv(y, 0);
                main_win.clrtoeol();

                // Check if cursor is on this line
                let cursor_pos = if position.y == i {
                    Some(position.x)
                } else {
                    None
                };

                self.highlight_syntax(main_win, line, y, cursor_pos);
            }
        }
        Ok(())
    }

    fn render_status(&mut self, status: &str) -> Result<()> {
        if let Some(ref status_win) = self.status_window {
            status_win.mv(0, 0);
            status_win.clrtoeol();
            status_win.addstr(status);
        }
        Ok(())
    }

    fn get_input(&mut self) -> Result<i32> {
        if let Some(ref main_win) = self.main_window {
            match main_win.getch() {
                Some(Input::Character(ch)) => Ok(ch as i32),
                Some(Input::KeyUp) => Ok(1001),
                Some(Input::KeyDown) => Ok(1002),
                Some(Input::KeyLeft) => Ok(1003),
                Some(Input::KeyRight) => Ok(1004),
                Some(Input::KeyBackspace) => Ok(127),
                Some(Input::KeyDC) => Ok(1005), // Delete key
                Some(Input::KeyHome) => Ok(1006),
                Some(Input::KeyEnd) => Ok(1007),
                Some(Input::KeyEnter) => Ok(10), // Enter
                _ => Ok(0), // Unknown input
            }
        } else {
            Err(EditorError::Display("No main window available".to_string()))
        }
    }

    fn get_size(&self) -> (usize, usize) {
        self.screen_size
    }

    fn move_cursor(&mut self, position: Position) -> Result<()> {
        if let Some(ref main_win) = self.main_window {
            main_win.mv(position.y as i32, position.x as i32);
        }
        Ok(())
    }
}

pub struct StatusLine {
    filename: String,
    position: Position,
    mode: EditorMode,
    is_modified: bool,
}

impl StatusLine {
    pub fn new() -> Self {
        Self {
            filename: "untitled".to_string(),
            position: Position::origin(),
            mode: EditorMode::Edit,
            is_modified: false,
        }
    }

    pub fn update(&mut self, filename: &str, position: Position, mode: EditorMode, is_modified: bool) {
        self.filename = filename.to_string();
        self.position = position;
        self.mode = mode;
        self.is_modified = is_modified;
    }

    pub fn format(&self) -> String {
        let mode_str = match self.mode {
            EditorMode::Edit => "EDIT",
            EditorMode::Command => "COMMAND",
        };

        let modified_indicator = if self.is_modified { "*" } else { "" };

        format!(
            "File: {}{} | Position: {}:{} | Mode: {}",
            self.filename,
            modified_indicator,
            self.position.y + 1,
            self.position.x + 1,
            mode_str
        )
    }
}

impl Default for StatusLine {
    fn default() -> Self {
        Self::new()
    }
}