#!/usr/bin/env bash

# Runs the tests for a target and renames the report file to include the target name and the current date.
# Usage: ./run-tests.sh <target>

VALID_TARGETS=("i686-pc-windows-msvc" "x86_64-pc-windows-msvc" "x86_64-apple-darwin" "aarch64-apple-darwin" "i686-unknown-linux-gnu" "x86_64-unknown-linux-gnu" "aarch64-unknown-linux-gnu")

TARGET=$1
if [ -z "$TARGET" ]; then
    echo "Usage: ./run-tests.sh <target>"
    echo "Valid targets: ${VALID_TARGETS[*]}"
    exit 1
fi

if [[ ! " ${VALID_TARGETS[*]} " =~ ${TARGET} ]]; then
    echo "Invalid target: $TARGET"
    echo "Valid targets: ${VALID_TARGETS[*]}"
    exit 1
fi

SCRIPT_REL_PATH=$(dirname "$0")
BUILD_DIR="$SCRIPT_REL_PATH/../build"
REPORTS_DIR="$BUILD_DIR/reports"

case $TARGET in
  i686-pc-windows-msvc|x86_64-pc-windows-msvc)
    TEST_BIN_EXT=exe
    ;;
  x86_64-apple-darwin|aarch64-apple-darwin)
    TEST_BIN_EXT=zip
    ;;
  i686-unknown-linux-gnu)
    TEST_BIN_EXT=x86_32
    ;;
  x86_64-unknown-linux-gnu)
    TEST_BIN_EXT=x86_64
    ;;
  aarch64-unknown-linux-gnu)
    TEST_BIN_EXT=arm64
    ;;
esac

TEST_BINARY=gr3d-tests-$TARGET.$TEST_BIN_EXT
TEST_BINARY_PATH=$BUILD_DIR/$TEST_BINARY

echo "Creating $REPORTS_DIR"
mkdir -p "$REPORTS_DIR"

echo "Running tests for target: $TARGET"
echo "Test binary path: $TEST_BINARY_PATH"

echo "Show binary"
ls -la "$TEST_BINARY_PATH"

echo "Show build dir"
ls -la "$BUILD_DIR"

echo "$ $TEST_BINARY_PATH --headless --no-window ++ --test=determinism --target=\"$TARGET\""
$TEST_BINARY_PATH --headless ++ --test=determinism --target="$TARGET"
