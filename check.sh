#!/bin/bash
# CADDY v0.2.5 - Code Quality Check Script
# Runs linting, formatting checks, and cargo check

set -e  # Exit on error

echo "=========================================="
echo "CADDY v0.2.5 Code Quality Checks"
echo "=========================================="
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to print status messages
print_status() {
    echo -e "${GREEN}[CHECK]${NC} $1"
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

# Run cargo check
print_status "Running cargo check..."
cargo check

if [ $? -eq 0 ]; then
    print_status "Cargo check passed!"
else
    print_error "Cargo check failed!"
    exit 1
fi

# Run clippy if available
if command -v cargo-clippy &> /dev/null; then
    print_status "Running clippy..."
    cargo clippy -- -D warnings

    if [ $? -eq 0 ]; then
        print_status "Clippy passed!"
    else
        print_error "Clippy found issues!"
        exit 1
    fi
else
    print_warning "Clippy not installed. Install with: rustup component add clippy"
fi

# Check formatting
if command -v cargo-fmt &> /dev/null; then
    print_status "Checking code formatting..."
    cargo fmt -- --check

    if [ $? -eq 0 ]; then
        print_status "Formatting check passed!"
    else
        print_error "Code is not properly formatted. Run: cargo fmt"
        exit 1
    fi
else
    print_warning "rustfmt not installed. Install with: rustup component add rustfmt"
fi

# Run TypeScript SDK linting if available
if command -v npm &> /dev/null; then
    if [ -d "bindings/typescript" ]; then
        print_status "Running TypeScript SDK linting..."
        cd bindings/typescript

        if [ -d "node_modules" ]; then
            npm run lint
            if [ $? -eq 0 ]; then
                print_status "TypeScript SDK linting passed!"
            else
                print_error "TypeScript SDK linting failed!"
                cd ../..
                exit 1
            fi
        else
            print_warning "TypeScript SDK dependencies not installed. Run: npm install"
        fi

        cd ../..
    fi
fi

echo ""
echo "=========================================="
print_status "All quality checks passed!"
echo "=========================================="
echo ""
