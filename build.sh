#!/bin/bash
# CADDY v0.2.5 - Main Build Script
# This script builds the entire CADDY project including Rust core and TypeScript SDK

set -e  # Exit on error

echo "=========================================="
echo "CADDY v0.2.5 Build Script"
echo "=========================================="
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to print status messages
print_status() {
    echo -e "${GREEN}[BUILD]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    print_error "Cargo not found. Please install Rust from https://rustup.rs/"
    exit 1
fi

print_status "Rust version: $(rustc --version)"
print_status "Cargo version: $(cargo --version)"

# Clean previous builds (optional)
if [ "$1" == "--clean" ]; then
    print_status "Cleaning previous build artifacts..."
    cargo clean
    rm -rf target/
fi

# Build Rust project
print_status "Building Rust project (debug mode)..."
cargo build

if [ $? -eq 0 ]; then
    print_status "Rust debug build completed successfully!"
else
    print_error "Rust build failed!"
    exit 1
fi

# Build TypeScript SDK if Node.js is available
if command -v npm &> /dev/null; then
    print_status "Node.js version: $(node --version)"
    print_status "npm version: $(npm --version)"

    if [ -d "bindings/typescript" ]; then
        print_status "Building TypeScript SDK..."
        cd bindings/typescript

        # Install dependencies if node_modules doesn't exist
        if [ ! -d "node_modules" ]; then
            print_status "Installing TypeScript SDK dependencies..."
            npm install
        fi

        # Build TypeScript SDK
        npm run build

        if [ $? -eq 0 ]; then
            print_status "TypeScript SDK build completed successfully!"
        else
            print_error "TypeScript SDK build failed!"
            cd ../..
            exit 1
        fi

        cd ../..
    fi
else
    print_warning "Node.js not found. Skipping TypeScript SDK build."
fi

echo ""
echo "=========================================="
print_status "Build completed successfully!"
echo "=========================================="
echo ""
echo "Build artifacts:"
echo "  - Rust binary: target/debug/caddy"
echo "  - TypeScript SDK: bindings/typescript/dist/"
echo ""
