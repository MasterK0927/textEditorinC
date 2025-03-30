# Text Editor Project

A comprehensive text editor implementation showcasing two different approaches: a modular C implementation and a modern Rust implementation with safety guarantees and SOLID design principles.

## Project Structure

```
.
├── c-implementation/          # Original C implementation (modularized)
│   ├── README.md             # C-specific documentation
│   ├── Makefile              # Build system for C version
│   ├── *.h                   # Header files (interfaces)
│   └── *.c                   # Implementation files
│
├── rust-implementation/       # Modern Rust implementation
│   ├── README.md             # Rust-specific documentation
│   ├── Cargo.toml            # Rust dependencies and build config
│   └── src/                  # Rust source code
│       ├── core.rs           # Core traits and types
│       ├── buffer/           # Memory-safe text buffer
│       ├── display/          # Terminal rendering
│       ├── editor_ops/       # Editor operations
│       ├── file_io/          # File I/O with validation
│       ├── undo/             # Type-safe undo system
│       ├── lib.rs            # Library exports
│       └── main.rs           # Main application
│
├── README.md                 # This file (project overview)
└── readme.md                 # Original detailed technical analysis
```

## Overview

This project demonstrates the evolution of software engineering practices by implementing the same text editor functionality in two different languages and paradigms:

### C Implementation (`c-implementation/`)
- **Focus**: Low-level control and performance
- **Architecture**: Modularized C with clear separation of concerns
- **Memory Management**: Manual allocation/deallocation
- **Error Handling**: Return codes and careful checking
- **Strengths**: Direct hardware control, minimal overhead
- **Challenges**: Memory safety, error-prone manual management

### Rust Implementation (`rust-implementation/`)
- **Focus**: Safety, reliability, and maintainability
- **Architecture**: SOLID principles with trait-based design
- **Memory Management**: Automatic with ownership system
- **Error Handling**: Comprehensive `Result<T>` types
- **Strengths**: Memory safety, zero-cost abstractions, excellent tooling
- **Modern Features**: Dependency injection, comprehensive testing

## Quick Start

### C Version
```bash
cd c-implementation
sudo apt install libncurses-dev  # Install dependencies
make
./your-editor [filename]
```

### Rust Version
```bash
cd rust-implementation
sudo apt install libncurses-dev  # Install dependencies
cargo run [filename]
```

## Features Comparison

| Feature | C Implementation | Rust Implementation |
|---------|------------------|---------------------|
| Text Editing | ✅ Full featured | ✅ Full featured |
| Syntax Highlighting | ✅ Basic (C keywords) | ✅ Enhanced (Rust keywords) |
| Undo/Redo | ✅ Stack-based | ✅ Type-safe with action history |
| File I/O | ✅ Basic | ✅ With validation and backups |
| Memory Safety | ⚠️ Manual management | ✅ Guaranteed by compiler |
| Error Handling | ⚠️ Return codes | ✅ Comprehensive Result types |
| Testing | ❌ Manual testing | ✅ Unit tests included |
| Modularity | ✅ Header-based | ✅ Trait-based with DI |
| Performance | ✅ Direct control | ✅ Zero-cost abstractions |

## Key Learning Points

### From C Implementation
- Low-level memory management techniques
- Manual resource handling and cleanup
- Terminal programming with ncurses
- Modular design in C using headers
- Performance optimization strategies

### From Rust Implementation
- Memory safety without garbage collection
- SOLID principles in systems programming
- Trait-based architecture and dependency injection
- Comprehensive error handling patterns
- Modern testing practices

## Architecture Highlights

### C Version - Modular Design
```c
// Clean interfaces with header files
typedef struct Buffer Buffer;
void initBuffer(Buffer *buffer);
void insertIntoBuffer(Buffer *buffer, int pos, char ch);
```

### Rust Version - Trait-Based Design
```rust
// Flexible, testable interfaces
pub trait TextBuffer {
    fn insert(&mut self, pos: usize, ch: char) -> Result<()>;
    fn delete(&mut self, pos: usize) -> Result<()>;
}

pub trait DisplayManager {
    fn render_text(&mut self, text: &str) -> Result<()>;
}
```

## Performance Characteristics

Both implementations are designed for:
- Real-time text editing performance
- Efficient memory usage patterns
- Responsive terminal interaction
- Scalability to reasonably large files

The Rust version achieves C-like performance while providing safety guarantees that prevent entire classes of bugs common in C programs.

## Development Setup

### Prerequisites
Both implementations require ncurses development headers:

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

### Building and Testing

#### C Implementation
```bash
cd c-implementation
make clean && make
./your-editor test.txt
```

#### Rust Implementation
```bash
cd rust-implementation
cargo build
cargo test          # Run unit tests
cargo run test.txt  # Run the editor
```

## Usage

Both implementations share the same interface:

### Edit Mode (Default)
- **Arrow keys**: Move cursor
- **Backspace**: Delete character before cursor
- **Delete**: Delete character at cursor
- **Tab**: Insert spaces
- **Enter**: New line
- **Escape**: Switch to command mode
- **Home/End**: Line navigation

### Command Mode
- **i**: Switch to edit mode
- **q**: Quit (prompts to save if modified)
- **s**: Save file
- **u/r**: Undo/Redo
- **v**: Start/end selection
- **x**: Cut selection
- **p**: Paste
- **h**: Show help

## Getting Started

1. **For C Development**: Start with `c-implementation/README.md`
2. **For Rust Development**: See `rust-implementation/README.md`
3. **For Comparison Study**: Build and test both versions side by side

## Contributing

This project is designed for educational purposes. When contributing:
- Maintain the clear separation between C and Rust implementations
- Ensure both versions maintain feature parity
- Include tests for new Rust features
- Update documentation for architectural changes

## Project History

This project evolved from a monolithic C implementation to demonstrate:
1. **Modularization**: Breaking down a large C file into focused modules
2. **Modern Migration**: Translating C concepts to Rust with safety improvements
3. **Architecture Evolution**: From procedural to trait-based design
4. **Safety Progression**: From manual memory management to compiler-guaranteed safety