#!/bin/bash

echo "==========================="
echo

# Check for ncurses
check_ncurses() {
    if pkg-config --exists ncurses; then
        echo "✅ ncurses found"
        return 0
    else
        echo "❌ ncurses development headers not found"
        echo "Please install ncurses development headers:"
        echo "  Ubuntu/Debian: sudo apt install libncurses-dev"
        echo "  CentOS/RHEL:   sudo yum install ncurses-devel"
        echo "  Fedora:        sudo dnf install ncurses-devel"
        echo "  macOS:         brew install ncurses"
        return 1
    fi
}

# Build C implementation
build_c() {
    echo "🔧 Building C implementation..."
    cd c-implementation || exit 1

    if make clean && make; then
        echo "✅ C build successful: ./your-editor"
        echo "   Usage: ./your-editor [filename]"
        echo "   Help:  ./your-editor --help"
        cd ..
        return 0
    else
        echo "❌ C build failed"
        cd ..
        return 1
    fi
}

# Build Rust implementation
build_rust() {
    if ! command -v cargo >/dev/null 2>&1; then
        echo "❌ Rust/Cargo not found. Install from: https://rustup.rs/"
        return 1
    fi

    echo "🦀 Building Rust implementation..."
    cd rust-implementation || exit 1

    if cargo build --release; then
        echo "✅ Rust build successful: ./target/release/text-editor"
        echo "   Usage: ./target/release/text-editor [filename]"
        echo "   Help:  ./target/release/text-editor --help"
        cd ..
        return 0
    else
        echo "❌ Rust build failed"
        cd ..
        return 1
    fi
}

# Main function
main() {
    # Check dependencies
    if ! check_ncurses; then
        echo
        echo "🚀 For automatic dependency installation, use: ./install.sh"
        exit 1
    fi

    echo
    echo "Choose build option:"
    echo "1) Build C implementation only"
    echo "2) Build Rust implementation only"
    echo "3) Build both implementations"
    echo "4) Exit"
    echo

    read -p "Enter choice (1-4): " choice

    case $choice in
        1)
            build_c
            ;;
        2)
            build_rust
            ;;
        3)
            echo "Building both implementations..."
            c_result=0
            rust_result=0

            build_c || c_result=1
            echo
            build_rust || rust_result=1

            echo
            echo "📋 Build Summary:"
            if [ $c_result -eq 0 ]; then
                echo "✅ C implementation: ./c-implementation/your-editor"
            else
                echo "❌ C implementation: Build failed"
            fi

            if [ $rust_result -eq 0 ]; then
                echo "✅ Rust implementation: ./rust-implementation/target/release/text-editor"
            else
                echo "❌ Rust implementation: Build failed"
            fi
            ;;
        4)
            echo "Build cancelled."
            exit 0
            ;;
        *)
            echo "Invalid choice. Exiting."
            exit 1
            ;;
    esac

    echo
    echo "🎉 Build complete! Run with your preferred implementation:"
    echo "   ./c-implementation/your-editor filename.txt"
    echo "   ./rust-implementation/target/release/text-editor filename.txt"
}

main "$@"