# C Text Editor Implementation

This directory contains the original C implementation of the text editor, which has been modularized following best practices.

## Files

### Core Headers
- `editor.h` - Core data structures and constants
- `buffer.h` - Buffer management interface
- `editor_ops.h` - Editor operations interface
- `display.h` - Display and UI interface
- `file_io.h` - File I/O interface
- `undo.h` - Undo/redo system interface

### Implementation Files
- `main.c` - Main application logic
- `buffer.c` - Text buffer management
- `editor_ops.c` - Cursor movement, text editing, clipboard operations
- `display.c` - Terminal display, syntax highlighting, status bar
- `file_io.c` - File reading and writing
- `undo.c` - Undo/redo stack management

### Build System
- `Makefile` - Build configuration

## Building

### Prerequisites

Install ncurses development headers:

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

### Compilation

```bash
cd c-implementation
make clean
make
```

### Running

```bash
./your-editor [filename]
```

## Features

- Modal editing (Edit/Command modes)
- Syntax highlighting for C keywords
- Undo/redo functionality
- Copy/cut/paste operations
- File operations (open/save)
- Status bar with position and mode indicators
- Help system

## Usage

### Edit Mode (Default)
- Arrow keys: Move cursor
- Backspace: Delete character before cursor
- Delete: Delete character at cursor
- Tab: Insert spaces
- Enter: New line
- Escape: Switch to command mode
- Home: Move to beginning of line
- End: Move to end of line

### Command Mode
- `i`: Switch to edit mode
- `q`: Quit (prompts to save if modified)
- `s`: Save file
- `u`: Undo
- `r`: Redo
- `v`: Start/end selection (copy)
- `x`: Cut selection
- `p`: Paste
- `h`: Show help

## Architecture

The C implementation has been modularized into distinct modules:

1. **Buffer Management** (`buffer.h/c`)
   - Dynamic string buffer with automatic resizing
   - Insert/delete/append operations
   - Memory management

2. **Editor Operations** (`editor_ops.h/c`)
   - Cursor movement and positioning
   - Character insertion and deletion
   - Selection and clipboard operations

3. **Display System** (`display.h/c`)
   - Terminal initialization and cleanup
   - Syntax highlighting
   - Status bar rendering
   - Help system

4. **File I/O** (`file_io.h/c`)
   - File reading and writing
   - Error handling

5. **Undo System** (`undo.h/c`)
   - Undo/redo stack management
   - State preservation

## Known Limitations

- Fixed buffer size (MAX_BUFFER)
- Basic syntax highlighting (C keywords only)
- No configuration file support
- Limited error handling compared to Rust version
- Manual memory management required

## Migration to Rust

This C implementation has been migrated to Rust with significant improvements:
- Memory safety guarantees
- Better error handling
- More robust architecture
- Comprehensive testing
- Type safety

See the parent directory for the Rust implementation.