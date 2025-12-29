#!/bin/bash
# CADDY v0.2.5 - Test Script
# Runs all tests for the CADDY project

set -e  # Exit on error

echo "=========================================="
echo "CADDY v0.2.5 Test Suite"
echo "=========================================="
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to print status messages
print_status() {
    echo -e "${GREEN}[TEST]${NC} $1"
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

# Run Rust tests
print_status "Running Rust tests..."
cargo test

if [ $? -eq 0 ]; then
    print_status "Rust tests passed!"
else
    print_error "Rust tests failed!"
    exit 1
fi

# Run Rust tests with coverage (if requested)
if [ "$1" == "--coverage" ]; then
    if command -v cargo-tarpaulin &> /dev/null; then
        print_status "Running tests with coverage..."
        cargo tarpaulin --out Html --output-dir coverage
        print_status "Coverage report generated in coverage/"
    else
        print_warning "cargo-tarpaulin not installed. Skipping coverage."
        print_warning "Install with: cargo install cargo-tarpaulin"
    fi
fi

# Run TypeScript SDK tests if available
if command -v npm &> /dev/null; then
    if [ -d "bindings/typescript" ]; then
        print_status "Running TypeScript SDK tests..."
        cd bindings/typescript

        if [ -d "node_modules" ]; then
            npm test
            if [ $? -eq 0 ]; then
                print_status "TypeScript SDK tests passed!"
            else
                print_error "TypeScript SDK tests failed!"
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
print_status "All tests passed successfully!"
echo "=========================================="
echo ""
