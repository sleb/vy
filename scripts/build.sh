#!/bin/bash

# Vy Build Script
# Builds all Rust binaries and prepares them for use

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

print_step() {
    echo -e "${BLUE}[STEP]${NC} $1"
}

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    print_error "This script must be run from the vy project root directory"
    exit 1
fi

print_status "Building Vy Project..."

# Parse command line arguments
RELEASE_MODE=false
INSTALL_SYSTEM=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --release)
            RELEASE_MODE=true
            shift
            ;;
        --install)
            INSTALL_SYSTEM=true
            shift
            ;;
        -h|--help)
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --release    Build in release mode (optimized)"
            echo "  --install    Install binaries to ~/.local/bin"
            echo "  -h, --help   Show this help message"
            exit 0
            ;;
        *)
            print_error "Unknown option: $1"
            echo "Use --help for usage information"
            exit 1
            ;;
    esac
done

# Determine build mode
if [ "$RELEASE_MODE" = true ]; then
    BUILD_FLAGS="--release"
    TARGET_DIR="target/release"
    print_step "Building in release mode (optimized)..."
else
    BUILD_FLAGS=""
    TARGET_DIR="target/debug"
    print_step "Building in debug mode..."
fi

# Build all binaries
print_status "Compiling Rust workspace..."
if ! cargo build $BUILD_FLAGS --workspace; then
    print_error "Failed to build Rust binaries"
    exit 1
fi

# Check that binaries were created
BINARIES=("vy" "vy-web")
for binary in "${BINARIES[@]}"; do
    if [ ! -f "$TARGET_DIR/$binary" ]; then
        print_error "Binary $binary was not created"
        exit 1
    fi
done

print_status "✅ Successfully built all binaries!"

# Show binary locations
echo ""
print_status "Built binaries:"
for binary in "${BINARIES[@]}"; do
    echo "  • $binary: $TARGET_DIR/$binary"
done

# Install to system if requested
if [ "$INSTALL_SYSTEM" = true ]; then
    print_step "Installing binaries to ~/.local/bin..."

    # Create ~/.local/bin if it doesn't exist
    mkdir -p "$HOME/.local/bin"

    # Copy binaries
    for binary in "${BINARIES[@]}"; do
        cp "$TARGET_DIR/$binary" "$HOME/.local/bin/"
        print_status "Installed $binary to ~/.local/bin/$binary"
    done

    echo ""
    print_warning "Make sure ~/.local/bin is in your PATH:"
    echo "  export PATH=\"\$HOME/.local/bin:\$PATH\""
    echo ""
    print_status "You can now run 'vy' and 'vy-web' from anywhere!"
fi

# Build Node.js frontend if it exists
if [ -d "web" ]; then
    print_step "Building Next.js frontend..."
    cd web

    if [ ! -d "node_modules" ]; then
        print_status "Installing Node.js dependencies..."
        npm install
    fi

    if [ "$RELEASE_MODE" = true ]; then
        print_status "Building Next.js for production..."
        npm run build
    else
        print_status "Checking Next.js build..."
        npm run lint
    fi

    cd ..
    print_status "✅ Frontend build complete!"
fi

echo ""
print_status "🎉 Build complete!"
echo ""
print_status "Quick start:"
if [ "$INSTALL_SYSTEM" = true ]; then
    echo "  vy config init    # Set up configuration"
    echo "  vy chat           # Start CLI chat"
    echo "  vy web            # Start web server"
else
    echo "  $TARGET_DIR/vy config init    # Set up configuration"
    echo "  $TARGET_DIR/vy chat           # Start CLI chat"
    echo "  $TARGET_DIR/vy web            # Start web server"
fi
echo ""
print_status "For web interface:"
echo "  Terminal 1: $TARGET_DIR/vy web"
echo "  Terminal 2: cd web && npm run dev"
echo "  Then visit: http://localhost:3000"
