#!/bin/bash
# Shell script to run Mandrel tests for tool42 MCP server
# This script builds tool42 if needed, validates the test config, and runs the tests

set -e

# Default values
MOTH_PATH="${MOTH_PATH:-../../3rdparty/codeprism/target/release/moth}"
TEST_CONFIG="${TEST_CONFIG:-tool42-server.yaml}"
BUILD_TOOL42="${BUILD_TOOL42:-false}"
VALIDATE_ONLY="${VALIDATE_ONLY:-false}"

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --moth-path)
            MOTH_PATH="$2"
            shift 2
            ;;
        --test-config)
            TEST_CONFIG="$2"
            shift 2
            ;;
        --build-tool42)
            BUILD_TOOL42="true"
            shift
            ;;
        --validate-only)
            VALIDATE_ONLY="true"
            shift
            ;;
        -h|--help)
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --moth-path PATH      Path to moth binary (default: ../../3rdparty/codeprism/target/release/moth)"
            echo "  --test-config FILE    Test configuration file (default: tool42-server.yaml)"
            echo "  --build-tool42        Build tool42 before running tests"
            echo "  --validate-only      Only validate the configuration, don't run tests"
            echo "  -h, --help           Show this help message"
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            echo "Use --help for usage information"
            exit 1
            ;;
    esac
done

echo "=== Tool42 Mandrel Test Runner ==="
echo ""

# Check if moth binary exists
if [ ! -f "$MOTH_PATH" ]; then
    echo "ERROR: moth binary not found at: $MOTH_PATH" >&2
    echo "Please build moth first:" >&2
    echo "  cd 3rdparty/codeprism" >&2
    echo "  cargo build --release --bin moth" >&2
    exit 1
fi

echo "Using moth binary: $MOTH_PATH"

# Build tool42 if requested
if [ "$BUILD_TOOL42" = "true" ]; then
    echo "Building tool42..."
    if ! cargo build --release; then
        echo "ERROR: Failed to build tool42" >&2
        exit 1
    fi
    echo "tool42 built successfully"
fi

# Check if tool42 is available
if ! command -v tool42 &> /dev/null; then
    echo "WARNING: tool42 not found in PATH"
    echo "Make sure tool42 is installed or built"
fi

# Validate test configuration
echo ""
echo "Validating test configuration..."
if ! "$MOTH_PATH" validate "$TEST_CONFIG"; then
    echo "ERROR: Test configuration validation failed" >&2
    exit 1
fi
echo "Test configuration is valid"

if [ "$VALIDATE_ONLY" = "true" ]; then
    echo ""
    echo "Validation complete (--validate-only specified)"
    exit 0
fi

# Run tests
echo ""
echo "Running Mandrel tests..."
echo ""

if "$MOTH_PATH" run "$TEST_CONFIG"; then
    echo ""
    echo "=== All tests passed! ==="
    exit 0
else
    echo ""
    echo "=== Some tests failed ===" >&2
    exit 1
fi

