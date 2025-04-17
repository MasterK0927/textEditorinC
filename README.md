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

### Automatic Installation (Recommended)
```bash
# Clone the repository
git clone <repository-url>
cd textEditorinC

# Run the installation script (installs dependencies and builds both versions)
./install.sh

# Use the editors
teditor filename.txt      # Rust version (primary)
teditor-c filename.txt    # C version
```

### 🔧 Manual Build
```bash
# For interactive build process
./build.sh

# Or build manually:
# C Version
cd c-implementation
sudo apt install libncurses-dev  # Install dependencies
make
./your-editor [filename]

# Rust Version
cd rust-implementation
sudo apt install libncurses-dev  # Install dependencies
cargo build --release
./target/release/text-editor [filename]
```

### Manual Installation
After building, install system-wide manually:
```bash
# Build first
./build.sh

# Install C version
sudo cp c-implementation/your-editor /usr/local/bin/teditor-c

# Install Rust version
sudo cp rust-implementation/target/release/text-editor /usr/local/bin/teditor

# Make executable (if needed)
sudo chmod +x /usr/local/bin/teditor*

# Now use anywhere
teditor filename.txt
teditor-c filename.txt
```

### Pre-built Releases
Download pre-built binaries from the releases section:
```bash
# Extract and install
tar -xzf text-editor-1.0.0-linux-x86_64.tar.gz
cd text-editor-binary
sudo ./install.sh
```

## Features Comparison

| Feature | C Implementation | Rust Implementation |
|---------|------------------|---------------------|
| Text Editing | ✅ Full featured | ✅ Full featured |
| Syntax Highlighting | ✅ Basic (C keywords) | ✅ Enhanced (Rust keywords) |
| Undo/Redo | ✅ Stack-based | ✅ Type-safe with action history |
| File I/O | ✅ Basic | ✅ With validation and backups |
| Vim Commands | ✅ :e, :w, :q, :wq | ✅ Full vim-like command set |
| Multi-buffer | ❌ Single file | ✅ Multiple files with switching |
| Read-only Mode | ✅ Command-line flag | ✅ Command-line flag |
| Command-line Args | ✅ Basic file opening | ✅ Advanced with clap |
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

#### Quick Build
```bash
./build.sh          # Interactive build script
```

#### C Implementation
```bash
cd c-implementation
make clean && make
./your-editor test.txt
```

#### Rust Implementation
```bash
cd rust-implementation
cargo build --release
cargo test          # Run unit tests
./target/release/text-editor test.txt
```

#### Creating Release Packages
```bash
./create-release.sh  # Creates distributable packages
```

## Usage

Both implementations now support vim-like commands:

### Edit Mode (Default)
- **Arrow keys**: Move cursor
- **Backspace**: Delete character before cursor
- **Delete**: Delete character at cursor
- **Tab**: Insert spaces
- **Enter**: New line
- **Escape**: Switch to command mode
- **Home/End**: Line navigation
- **:**: Start vim command input

### Command Mode
- **i**: Switch to edit mode
- **q**: Quit (prompts to save if modified)
- **s**: Save file
- **u/r**: Undo/Redo
- **v**: Start/end selection
- **x**: Cut selection
- **p**: Paste
- **h**: Show help
- **:**: Start vim command input

### Vim-like Commands
- **:e <file>**: Edit/open file
- **:o <file>**: Open file (same as :e)
- **:w**: Write/save current file
- **:w <file>**: Save as different filename
- **:wq**: Write and quit
- **:q**: Quit
- **:new**: Create new buffer (Rust only)
- **:ls**: List all buffers (Rust only)
- **:b <num>**: Switch to buffer number (Rust only)
- **:bd**: Delete current buffer (Rust only)

## Getting Started

1. **For C Development**: Start with `c-implementation/README.md`
2. **For Rust Development**: See `rust-implementation/README.md`
3. **For Comparison Study**: Build and test both versions side by side

## Distribution & Deployment

### Available Scripts
- **`./install.sh`** - Complete installation with dependencies
- **`./build.sh`** - Interactive build process
- **`./create-release.sh`** - Create distributable packages
- **`./verify_implementations.sh`** - Verify project structure

### System Integration
After installation, the editors are available as:
- **`teditor`** - Rust implementation (primary command)
- **`teditor-c`** - C implementation

### Cross-Platform Support
- **Linux**: Full support (tested on Ubuntu/Debian)
- **macOS**: Supported with Homebrew
- **Windows**: Available via WSL or native build (see releases)

### Package Formats
The `create-release.sh` script generates:
- Source distribution (`text-editor-1.0.0-source.tar.gz`)
- Binary distribution (`text-editor-1.0.0-linux-x86_64.tar.gz`)
- Windows build instructions
- SHA256 checksums for verification

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