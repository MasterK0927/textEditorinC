#!/bin/bash

echo "Text Editor Installation Script"
echo "=================================="
echo

# Function to detect the operating system
detect_os() {
    if [[ "$OSTYPE" == "linux-gnu"* ]]; then
        if command -v apt >/dev/null 2>&1; then
            echo "ubuntu"
        elif command -v yum >/dev/null 2>&1; then
            echo "centos"
        elif command -v dnf >/dev/null 2>&1; then
            echo "fedora"
        else
            echo "linux"
        fi
    elif [[ "$OSTYPE" == "darwin"* ]]; then
        echo "macos"
    else
        echo "unknown"
    fi
}

# Function to install dependencies
install_dependencies() {
    local os=$(detect_os)
    echo "Installing dependencies for $os..."

    case $os in
        "ubuntu")
            sudo apt update
            sudo apt install -y build-essential libncurses-dev curl
            ;;
        "centos")
            sudo yum groupinstall -y "Development Tools"
            sudo yum install -y ncurses-devel curl
            ;;
        "fedora")
            sudo dnf groupinstall -y "Development Tools"
            sudo dnf install -y ncurses-devel curl
            ;;
        "macos")
            if ! command -v brew >/dev/null 2>&1; then
                echo "❌ Homebrew not found. Please install Homebrew first:"
                echo "   /bin/bash -c \"\$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)\""
                exit 1
            fi
            brew install ncurses
            ;;
        *)
            echo "❌ Unsupported operating system. Please install manually:"
            echo "   - C compiler (gcc/clang)"
            echo "   - ncurses development headers"
            echo "   - Rust (if building Rust version)"
            exit 1
            ;;
    esac
}

# Function to install Rust
install_rust() {
    if command -v cargo >/dev/null 2>&1; then
        echo "✅ Rust already installed"
    else
        echo "🦀 Installing Rust..."
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        source "$HOME/.cargo/env"
    fi
}

# Function to build C implementation
build_c() {
    echo "Building C implementation..."
    cd c-implementation

    if make clean && make; then
        echo "✅ C implementation built successfully: ./your-editor"

        # Create a symlink in /usr/local/bin if possible
        if [[ -w "/usr/local/bin" ]] || sudo -n true 2>/dev/null; then
            echo "📝 Creating system-wide link..."
            sudo ln -sf "$(pwd)/your-editor" /usr/local/bin/teditor-c
            echo "✅ Created system command: teditor-c"
        else
            echo "💡 To use globally, add to PATH or create alias:"
            echo "   echo 'alias teditor-c=\"$(pwd)/your-editor\"' >> ~/.bashrc"
        fi
    else
        echo "❌ C build failed"
        return 1
    fi

    cd ..
}

# Function to build Rust implementation
build_rust() {
    echo "🦀 Building Rust implementation..."
    cd rust-implementation

    if cargo build --release; then
        echo "✅ Rust implementation built successfully: ./target/release/text-editor"

        # Create a symlink in /usr/local/bin if possible
        if [[ -w "/usr/local/bin" ]] || sudo -n true 2>/dev/null; then
            echo "📝 Creating system-wide link..."
            sudo ln -sf "$(pwd)/target/release/text-editor" /usr/local/bin/teditor
            echo "✅ Created system command: teditor"
        else
            echo "💡 To use globally, add to PATH or create alias:"
            echo "   echo 'alias teditor=\"$(pwd)/target/release/text-editor\"' >> ~/.bashrc"
        fi
    else
        echo "❌ Rust build failed"
        return 1
    fi

    cd ..
}

# Function to create desktop entries (Linux)
create_desktop_entries() {
    if [[ "$(detect_os)" == "ubuntu" ]] || [[ "$(detect_os)" == "linux" ]]; then
        echo "🖥️ Creating desktop entries..."

        mkdir -p ~/.local/share/applications

        # C version desktop entry
        cat > ~/.local/share/applications/text-editor-c.desktop << EOF
[Desktop Entry]
Version=1.0
Type=Application
Name=Text Editor (C)
Comment=Terminal-based text editor implemented in C
Exec=teditor-c %F
Icon=text-editor
Terminal=true
Categories=Development;TextEditor;
MimeType=text/plain;text/x-c;text/x-c++;
EOF

        # Rust version desktop entry
        cat > ~/.local/share/applications/text-editor-rust.desktop << EOF
[Desktop Entry]
Version=1.0
Type=Application
Name=Text Editor (Rust)
Comment=Terminal-based text editor implemented in Rust
Exec=teditor %F
Icon=text-editor
Terminal=true
Categories=Development;TextEditor;
MimeType=text/plain;text/x-rust;text/x-c;text/x-c++;
EOF

        echo "✅ Desktop entries created"
    fi
}

# Main installation process
main() {
    echo "This script will install dependencies and build both C and Rust implementations"
    echo "of the text editor. You may be prompted for your password."
    echo
    read -p "Continue? (y/N): " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo "Installation cancelled."
        exit 0
    fi

    # Install system dependencies
    install_dependencies

    # Install Rust for Rust implementation
    install_rust

    # Build both implementations
    echo
    echo "🏗️ Building implementations..."

    c_success=false
    rust_success=false

    if build_c; then
        c_success=true
    fi

    if build_rust; then
        rust_success=true
    fi

    # Create desktop entries
    create_desktop_entries

    # Summary
    echo
    echo "Installation Summary"
    echo "======================"
    if $c_success; then
        echo "✅ C implementation: teditor-c (or ./c-implementation/your-editor)"
    else
        echo "❌ C implementation: Build failed"
    fi

    if $rust_success; then
        echo "✅ Rust implementation: teditor (or ./rust-implementation/target/release/text-editor)"
    else
        echo "❌ Rust implementation: Build failed"
    fi

    echo
    echo "Usage examples:"
    echo "  teditor filename.txt          # Rust version"
    echo "  teditor-c filename.txt        # C version"
    echo "  teditor --help                # Show help"
    echo "  teditor --readonly file.txt   # Read-only mode"
    echo
    echo "📚 See README.md for more usage information"
}

# Run main function
main "$@"