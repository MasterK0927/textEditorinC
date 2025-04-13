#!/bin/bash

echo "Creating Release Packages"
echo "============================"
echo

VERSION="1.0.0"
RELEASE_DIR="releases"
PROJECT_NAME="text-editor"

# Create release directory
mkdir -p "$RELEASE_DIR"

# Function to create source distribution
create_source_dist() {
    echo "📁 Creating source distribution..."

    local archive_name="${PROJECT_NAME}-${VERSION}-source.tar.gz"
    local temp_dir="/tmp/${PROJECT_NAME}-source"

    # Clean and prepare source
    rm -rf "$temp_dir"
    mkdir -p "$temp_dir"

    # Copy source files
    cp -r c-implementation rust-implementation "$temp_dir/"
    cp README.md verify_implementations.sh install.sh build.sh "$temp_dir/"

    # Remove build artifacts
    find "$temp_dir" -name "obj" -type d -exec rm -rf {} + 2>/dev/null || true
    find "$temp_dir" -name "target" -type d -exec rm -rf {} + 2>/dev/null || true
    find "$temp_dir" -name "your-editor" -type f -delete 2>/dev/null || true

    # Create archive
    cd /tmp
    tar -czf "${PROJECT_NAME}-${VERSION}-source.tar.gz" "${PROJECT_NAME}-source"
    mv "${PROJECT_NAME}-${VERSION}-source.tar.gz" "$(pwd)/$RELEASE_DIR/"
    cd - > /dev/null

    echo "✅ Source distribution: $RELEASE_DIR/$archive_name"
    rm -rf "$temp_dir"
}

# Function to create binary distribution (if builds are available)
create_binary_dist() {
    echo "🔧 Creating binary distribution..."

    local archive_name="${PROJECT_NAME}-${VERSION}-linux-x86_64.tar.gz"
    local temp_dir="/tmp/${PROJECT_NAME}-binary"

    # Clean temp directory
    rm -rf "$temp_dir"
    mkdir -p "$temp_dir/bin"
    mkdir -p "$temp_dir/docs"

    # Copy binaries if they exist
    if [ -f "c-implementation/your-editor" ]; then
        cp "c-implementation/your-editor" "$temp_dir/bin/teditor-c"
        echo "✅ Added C implementation"
    else
        echo "⚠️  C implementation not built"
    fi

    if [ -f "rust-implementation/target/release/text-editor" ]; then
        cp "rust-implementation/target/release/text-editor" "$temp_dir/bin/teditor"
        echo "✅ Added Rust implementation"
    else
        echo "⚠️  Rust implementation not built"
    fi

    # Copy documentation
    cp README.md "$temp_dir/docs/"
    cp c-implementation/README.md "$temp_dir/docs/README-C.md" 2>/dev/null || true
    cp rust-implementation/README.md "$temp_dir/docs/README-Rust.md" 2>/dev/null || true

    # Create install script for binary distribution
    cat > "$temp_dir/install.sh" << 'EOF'
#!/bin/bash
echo "Installing text editors..."
sudo cp bin/* /usr/local/bin/
echo "✅ Installed to /usr/local/bin/"
echo "Usage: teditor [file]  or  teditor-c [file]"
EOF
    chmod +x "$temp_dir/install.sh"

    # Create usage script
    cat > "$temp_dir/usage.txt" << 'EOF'
Text Editor Binary Distribution
==============================

Contents:
- bin/teditor      : Rust implementation (primary)
- bin/teditor-c    : C implementation
- docs/            : Documentation
- install.sh       : System installation script

Usage:
./bin/teditor filename.txt          # Run Rust version
./bin/teditor-c filename.txt        # Run C version

Commands:
:e <file>    Edit/open file
:w           Write/save
:q           Quit
:wq          Write and quit
ESC          Switch to command mode
i            Switch to edit mode

System Installation:
sudo ./install.sh
EOF

    # Create archive only if we have binaries
    if [ -f "$temp_dir/bin/teditor" ] || [ -f "$temp_dir/bin/teditor-c" ]; then
        cd /tmp
        tar -czf "${PROJECT_NAME}-${VERSION}-linux-x86_64.tar.gz" "${PROJECT_NAME}-binary"
        mv "${PROJECT_NAME}-${VERSION}-linux-x86_64.tar.gz" "$(pwd)/$RELEASE_DIR/"
        cd - > /dev/null
        echo "✅ Binary distribution: $RELEASE_DIR/$archive_name"
    else
        echo "❌ No binaries found. Build first with ./build.sh"
    fi

    rm -rf "$temp_dir"
}

# Function to create Windows cross-compilation instructions
create_windows_info() {
    cat > "$RELEASE_DIR/WINDOWS_BUILD.md" << 'EOF'
# Building for Windows

## Rust Implementation (Recommended)

### Using Windows Subsystem for Linux (WSL)
1. Install WSL2 with Ubuntu
2. Follow the Linux build instructions

### Native Windows Build
1. Install Rust: https://rustup.rs/
2. Install Visual Studio Build Tools
3. Install vcpkg and ncurses:
   ```
   vcpkg install ncurses:x64-windows
   ```
4. Build:
   ```
   cd rust-implementation
   cargo build --release
   ```

## C Implementation

### Using MinGW
1. Install MSYS2: https://www.msys2.org/
2. Install dependencies:
   ```
   pacman -S mingw-w64-x86_64-gcc mingw-w64-x86_64-ncurses
   ```
3. Build:
   ```
   cd c-implementation
   gcc -o your-editor.exe *.c -lncurses
   ```

### Using Visual Studio
1. Install Visual Studio with C++ tools
2. Install vcpkg and ncurses
3. Adapt the Makefile for MSVC

Note: ncurses support on Windows is limited. Consider using Windows Terminal
with UTF-8 support for best results.
EOF
    echo "✅ Windows build instructions: $RELEASE_DIR/WINDOWS_BUILD.md"
}

# Function to create checksums
create_checksums() {
    echo "🔐 Creating checksums..."
    cd "$RELEASE_DIR"
    for file in *.tar.gz; do
        if [ -f "$file" ]; then
            sha256sum "$file" > "${file}.sha256"
            echo "✅ Created checksum for $file"
        fi
    done
    cd - > /dev/null
}

# Main function
main() {
    echo "Creating release v$VERSION..."
    echo

    # Try to build first
    echo "🔧 Attempting to build implementations..."
    if command -v make >/dev/null && pkg-config --exists ncurses; then
        echo "Building C implementation..."
        (cd c-implementation && make clean && make) 2>/dev/null && echo "✅ C build successful" || echo "❌ C build failed"
    fi

    if command -v cargo >/dev/null; then
        echo "Building Rust implementation..."
        (cd rust-implementation && cargo build --release) 2>/dev/null && echo "✅ Rust build successful" || echo "❌ Rust build failed"
    fi

    echo

    # Create distributions
    create_source_dist
    create_binary_dist
    create_windows_info
    create_checksums

    echo
    echo "📋 Release Summary"
    echo "=================="
    echo "Release directory: $RELEASE_DIR/"
    ls -la "$RELEASE_DIR/"

    echo
    echo "🚀 Ready for distribution!"
    echo "Upload the .tar.gz files and checksums to your preferred platform:"
    echo "  - GitHub Releases"
    echo "  - GitLab Releases"
    echo "  - Package repositories"
    echo "  - Direct download server"
}

main "$@"