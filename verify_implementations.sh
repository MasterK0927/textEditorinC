#!/bin/bash

echo "🔍 Verifying Text Editor Implementations"
echo "========================================"
echo

# Check project structure
echo "📁 Project Structure:"
echo "├── c-implementation/    ($(find c-implementation -name "*.c" | wc -l) C files, $(find c-implementation -name "*.h" | wc -l) headers)"
echo "└── rust-implementation/ ($(find rust-implementation -name "*.rs" | wc -l) Rust files)"
echo

# Verify C implementation
echo "🔧 C Implementation Check:"
echo "─────────────────────────"
cd c-implementation

if [ -f "Makefile" ] && [ -f "main.c" ]; then
    echo "✅ C source files present"
    echo "✅ Makefile present"

    # Check for required headers
    if [ -f "editor.h" ] && [ -f "buffer.h" ] && [ -f "display.h" ]; then
        echo "✅ All header files present"
    else
        echo "❌ Missing header files"
    fi

    # Check modular structure
    modules=("buffer" "display" "editor_ops" "file_io" "undo")
    echo "📦 Modules:"
    for module in "${modules[@]}"; do
        if [ -f "${module}.c" ] && [ -f "${module}.h" ]; then
            echo "   ✅ $module module"
        else
            echo "   ❌ $module module incomplete"
        fi
    done
else
    echo "❌ C implementation incomplete"
fi

cd ..

# Verify Rust implementation
echo
echo "🦀 Rust Implementation Check:"
echo "────────────────────────────"
cd rust-implementation

if [ -f "Cargo.toml" ] && [ -f "src/main.rs" ]; then
    echo "✅ Rust project structure present"
    echo "✅ Cargo.toml present"

    # Check for core files
    if [ -f "src/core.rs" ] && [ -f "src/lib.rs" ]; then
        echo "✅ Core Rust files present"
    else
        echo "❌ Missing core Rust files"
    fi

    # Check modular structure
    modules=("buffer" "display" "editor_ops" "file_io" "undo")
    echo "📦 Modules:"
    for module in "${modules[@]}"; do
        if [ -f "src/${module}/mod.rs" ]; then
            echo "   ✅ $module module"
        else
            echo "   ❌ $module module missing"
        fi
    done

    # Check dependencies
    echo "📦 Dependencies:"
    if grep -q "pancurses" Cargo.toml; then
        echo "   ✅ pancurses (ncurses bindings)"
    fi
    if grep -q "thiserror" Cargo.toml; then
        echo "   ✅ thiserror (error handling)"
    fi
    if grep -q "anyhow" Cargo.toml; then
        echo "   ✅ anyhow (error context)"
    fi
else
    echo "❌ Rust implementation incomplete"
fi

cd ..

echo
echo "🏗️  Build Instructions:"
echo "─────────────────────────"
echo "C Implementation:"
echo "  cd c-implementation"
echo "  sudo apt install libncurses-dev  # if needed"
echo "  make"
echo "  ./your-editor [filename]"
echo "  ./your-editor --help              # Show help"
echo "  ./your-editor -r file.txt         # Read-only mode"
echo
echo "Rust Implementation:"
echo "  cd rust-implementation"
echo "  sudo apt install libncurses-dev  # if needed"
echo "  cargo build"
echo "  cargo run [filename]"
echo "  cargo run -- --help               # Show help"
echo "  cargo run -- --readonly file.txt  # Read-only mode"
echo "  cargo run -- file1.txt file2.txt  # Multiple files"
echo
echo "📚 Documentation:"
echo "─────────────────"
echo "  README.md                     - Project overview"
echo "  c-implementation/README.md    - C-specific documentation"
echo "  rust-implementation/README.md - Rust-specific documentation"
echo "  readme.md                     - Original technical analysis"
echo
echo "✨ Verification Complete!"