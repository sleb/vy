#!/bin/bash

# Vy Web Development Server Script
# This script starts both the Rust API server and Next.js frontend for development

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
if [ ! -f "Cargo.toml" ] || [ ! -d "web" ]; then
    print_error "This script must be run from the vy project root directory"
    exit 1
fi

print_status "Starting Vy Web Development Environment"

# Function to cleanup background processes on exit
cleanup() {
    print_warning "Shutting down development servers..."
    jobs -p | xargs -r kill
    exit 0
}

# Set up signal handling
trap cleanup SIGINT SIGTERM

# Check if vy config exists
print_step "Checking Vy configuration..."
if ! cargo run --bin vy config list > /dev/null 2>&1; then
    print_warning "Vy configuration not found or incomplete"
    print_status "Run 'cargo run --bin vy config init' first to set up your API keys"
    exit 1
fi

# Build the Rust web server
print_step "Building Rust web server..."
if ! cargo build --bin vy-web; then
    print_error "Failed to build vy-web"
    exit 1
fi

# Check if Node.js dependencies are installed
print_step "Checking Node.js dependencies..."
if [ ! -d "web/node_modules" ]; then
    print_status "Installing Node.js dependencies..."
    cd web && npm install && cd ..
fi

print_step "Starting Rust API server on http://localhost:3001..."
cargo run --bin vy web &
RUST_PID=$!

# Wait a moment for the server to start
sleep 2

# Check if the Rust server is running
if ! curl -s http://localhost:3001/health > /dev/null; then
    print_error "Rust API server failed to start"
    kill $RUST_PID 2>/dev/null || true
    exit 1
fi

print_status "✅ Rust API server is running (PID: $RUST_PID)"

print_step "Starting Next.js frontend on http://localhost:3000..."
cd web && npm run dev &
NEXTJS_PID=$!
cd ..

print_status "✅ Next.js frontend is starting (PID: $NEXTJS_PID)"

echo ""
print_status "🚀 Development servers are running!"
echo -e "  ${BLUE}Frontend:${NC} http://localhost:3000"
echo -e "  ${BLUE}API:${NC}      http://localhost:3001"
echo -e "  ${BLUE}Health:${NC}   http://localhost:3001/health"
echo ""
print_warning "Press Ctrl+C to stop all servers"

# Wait for background processes
wait
