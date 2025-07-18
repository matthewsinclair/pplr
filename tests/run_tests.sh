#!/bin/bash

# Run all BATS tests
# Usage: ./run_tests.sh [specific-test.bats]

# Get the directory where this script is located
TEST_DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$TEST_DIR"

# Set test environment
export PPLR_TEST_MODE=1
export PPLR_TEST_DATA="$TEST_DIR/fixtures"
export PPLR_ROOT="$(dirname "$TEST_DIR")"
export PPLR_DATA="$PPLR_TEST_DATA"
export PPLR_BIN_DIR="$PPLR_ROOT/bin"
export PPLR_TEMPLATE_DIR="$PPLR_ROOT/templates"
export PPLR_DIR="$PPLR_DATA"  # For backwards compatibility

# Check if bats is installed
if ! command -v bats &> /dev/null; then
    echo "Error: BATS is not installed"
    echo "Install with: brew install bats-core (macOS) or see https://github.com/bats-core/bats-core"
    exit 1
fi

# Create test fixtures directory if it doesn't exist
mkdir -p "$PPLR_TEST_DATA"

# Run tests
if [ $# -eq 0 ]; then
    echo "Running all tests..."
    bats *.bats
else
    echo "Running specific tests: $@"
    bats "$@"
fi

# Clean up test data
echo "Cleaning up test data..."
rm -rf "$PPLR_TEST_DATA"/*