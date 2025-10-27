#!/bin/bash

# Build script for NIH-Plug AUv3 Rust static library
# This script compiles the Rust code into a static library that can be linked with the Swift extension

set -e

# Configuration
RUST_TARGET="x86_64-apple-darwin"
LIBRARY_NAME="libnih_plug.a"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
OUTPUT_DIR="$SCRIPT_DIR"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../../../.." && pwd)"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}Building NIH-Plug AUv3 Rust static library...${NC}"
echo -e "${YELLOW}Script directory: $SCRIPT_DIR${NC}"
echo -e "${YELLOW}Project root: $PROJECT_ROOT${NC}"

# Change to the project root directory
cd "$PROJECT_ROOT"

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    echo -e "${RED}Error: Not in NIH-plug project root directory${NC}"
    echo -e "${YELLOW}Current directory: $(pwd)${NC}"
    echo -e "${YELLOW}Looking for Cargo.toml...${NC}"
    ls -la | grep -i cargo || echo "No Cargo.toml found"
    exit 1
fi

# Build the static library
echo -e "${YELLOW}Compiling Rust code...${NC}"
cargo build --release --features auv3 --target $RUST_TARGET

# Find the generated library
LIBRARY_PATH="target/$RUST_TARGET/release/$LIBRARY_NAME"

if [ ! -f "$LIBRARY_PATH" ]; then
    echo -e "${RED}Error: Static library not found at $LIBRARY_PATH${NC}"
    echo -e "${YELLOW}Available files in target/$RUST_TARGET/release/:${NC}"
    ls -la "target/$RUST_TARGET/release/" | grep -E "\.(a|dylib)$" || echo "No libraries found"
    exit 1
fi

# Copy the library to the Swift directory
echo -e "${YELLOW}Copying library to Swift directory...${NC}"
cp "$LIBRARY_PATH" "$OUTPUT_DIR/"

# Verify the library was copied
if [ -f "$OUTPUT_DIR/$LIBRARY_NAME" ]; then
    echo -e "${GREEN}Successfully built and copied $LIBRARY_NAME${NC}"
    echo -e "${YELLOW}Library size: $(du -h "$OUTPUT_DIR/$LIBRARY_NAME" | cut -f1)${NC}"
else
    echo -e "${RED}Error: Failed to copy library${NC}"
    exit 1
fi

# Generate C header file using cbindgen (if available)
if command -v cbindgen &> /dev/null; then
    echo -e "${YELLOW}Generating C header file...${NC}"
    cbindgen --config cbindgen.toml --crate nih-plug --output "$OUTPUT_DIR/nih_plug_auv3.h" || {
        echo -e "${YELLOW}Warning: cbindgen failed, using manual header${NC}"
    }
else
    echo -e "${YELLOW}cbindgen not available, using manual header${NC}"
fi

echo -e "${GREEN}Build complete!${NC}"
echo -e "${YELLOW}Next steps:${NC}"
echo "1. Open NIHPlugAUv3.xcodeproj in Xcode"
echo "2. Build the project (Cmd+B)"
echo "3. Test the extension in Logic Pro or GarageBand"