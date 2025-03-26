# Rust Text Editor

A modern text editor implementation in Rust, migrated from the original C version, following SOLID principles and Rust best practices.

## Features

- **Memory Safety**: Rust's ownership system prevents buffer overflows and memory leaks
- **Error Handling**: Comprehensive error handling using `Result<T>` and custom error types
- **Modular Architecture**: Clean separation of concerns following SOLID principles
- **Type Safety**: Strong typing prevents many runtime errors
- **Performance**: Zero-cost abstractions and efficient memory management
- **Testing**: Unit tests for all major components

## Architecture

The editor is built using dependency injection and trait-based design:

### Core Traits (SOLID Principles)

- **Single Responsibility**: Each trait has one clear purpose
- **Open/Closed**: Extensible through trait implementations
- **Liskov Substitution**: Trait objects can be used interchangeably
- **Interface Segregation**: Small, focused trait interfaces
- **Dependency Inversion**: Depends on abstractions, not concretions

### Modules

```
src/
├── core.rs              # Core traits and types
├── buffer/              # Text buffer management
├── display/             # Terminal display and rendering
├── editor_ops/          # Editor operations (cursor, edit, clipboard)
├── file_io/             # File I/O with safety checks
├── undo/                # Undo/redo system with type safety
├── lib.rs               # Library exports
└── main.rs              # Application entry point
```

## Dependencies

### System Requirements

```bash
# Ubuntu/Debian
sudo apt install libncurses-dev

# CentOS/RHEL/Fedora
sudo yum install ncurses-devel
# or
sudo dnf install ncurses-devel

# macOS
brew install ncurses
```

### Rust Dependencies

- `pancurses`: Cross-platform ncurses bindings
- `thiserror`: Ergonomic error handling
- `anyhow`: Flexible error handling for applications

## Building and Running

```bash
# Build the project
cargo build

# Run the editor
cargo run [filename]

# Run tests
cargo test

# Build optimized release version
cargo build --release
```

## Usage

### Edit Mode (Default)
- **Arrow keys**: Move cursor
- **Backspace**: Delete character before cursor
- **Delete**: Delete character at cursor
- **Tab**: Insert spaces (configurable size)
- **Enter**: New line
- **Escape**: Switch to command mode
- **Home**: Move to beginning of line
- **End**: Move to end of line

### Command Mode
- **i**: Switch to edit mode
- **q**: Quit (prompts to save if modified)
- **s**: Save file
- **u**: Undo
- **r**: Redo
- **v**: Start/end selection (copy)
- **x**: Cut selection
- **p**: Paste
- **h**: Show help

## Key Improvements Over C Version

### Memory Safety
- No manual memory management
- Automatic cleanup with RAII
- Buffer overflow protection
- No dangling pointers

### Error Handling
```rust
// Robust error handling
fn save_file(&self, filename: &str, content: &str) -> Result<()> {
    self.validate_filename(filename)?;
    self.validate_file_size(content)?;
    self.file_system.save(filename, content)?;
    Ok(())
}
```

### Type Safety
```rust
// Strong typing prevents errors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    pub x: usize,
    pub y: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditorMode {
    Edit,
    Command,
}
```

### Dependency Injection
```rust
// Flexible, testable design
pub trait DisplayManager {
    fn render_text(&mut self, text: &str, position: Position) -> Result<()>;
    fn get_input(&mut self) -> Result<i32>;
    // ...
}

pub trait FileManager {
    fn open(&self, filename: &str) -> Result<String>;
    fn save(&self, filename: &str, content: &str) -> Result<()>;
}
```

### Comprehensive Testing
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buffer_operations() {
        let mut buffer = Buffer::new();
        buffer.insert(0, 'H').unwrap();
        buffer.insert(1, 'i').unwrap();
        assert_eq!(buffer.content(), "Hi");
    }
}
```

## Configuration

The editor can be customized through constants in `core.rs`:

```rust
pub const TAB_SIZE: usize = 4;
pub const MAX_HISTORY: usize = 100;
```

## Testing

Run the test suite:

```bash
# All tests
cargo test

# Specific module tests
cargo test buffer
cargo test editor_ops

# With output
cargo test -- --nocapture
```

## Performance

- **Zero-cost abstractions**: No runtime overhead for high-level constructs
- **Efficient string handling**: Uses Rust's efficient String and str types
- **Memory pooling**: Reuses allocations where possible
- **Lazy evaluation**: Operations computed only when needed

## Safety Features

- **Input validation**: All user input is validated
- **File size limits**: Configurable maximum file sizes
- **Path sanitization**: Prevents directory traversal attacks
- **Backup creation**: Automatic backups before saving
- **Graceful error recovery**: Continues operation after non-fatal errors

## Future Enhancements

- Syntax highlighting for multiple languages
- Plugin system using dynamic loading
- Configuration file support
- Multi-buffer support (tabs)
- Search and replace functionality
- Line numbers and ruler
- Mouse support
- Clipboard integration with system clipboard

## Comparison with C Version

| Aspect | C Version | Rust Version |
|--------|-----------|--------------|
| Memory Safety | Manual management, prone to leaks/corruption | Automatic, guaranteed safety |
| Error Handling | Return codes, easily ignored | Explicit Result types, forced handling |
| Modularity | Header files, weak encapsulation | Strong module system, trait-based |
| Testing | Manual testing, no framework | Built-in test framework, easy unit tests |
| Performance | Fast, but unsafe | Fast with safety guarantees |
| Maintainability | Difficult to modify safely | Easy to extend and modify |
| Debugging | GDB, memory issues common | Excellent tooling, fewer runtime issues |

The Rust version provides the same functionality as the C version while adding memory safety, better error handling, improved modularity, and comprehensive testing capabilities.