#!/bin/bash

# Build script for NIH-Plug AUv3 Rust static library
# This script compiles the Rust code into a static library that can be linked with the Swift extension

set -e

# Configuration
X86_64_TARGET="x86_64-apple-darwin"
ARM64_TARGET="aarch64-apple-darwin"
LIBRARY_NAME="libnih_plug.a"
UNIVERSAL_LIBRARY_NAME="libnih_plug_universal.a"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
OUTPUT_DIR="$SCRIPT_DIR"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../../../.." && pwd)"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}Building NIH-Plug AUv3 Rust static library (universal binary)...${NC}"
echo -e "${YELLOW}Script directory: $SCRIPT_DIR${NC}"
echo -e "${YELLOW}Project root: $PROJECT_ROOT${NC}"
echo -e "${YELLOW}Targets: $X86_64_TARGET, $ARM64_TARGET${NC}"

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

# Build for x86_64 (Intel Macs)
echo -e "${YELLOW}Building for x86_64 (Intel Macs)...${NC}"
cargo build --release --features auv3 --target $X86_64_TARGET

X86_64_LIBRARY_PATH="target/$X86_64_TARGET/release/$LIBRARY_NAME"
if [ ! -f "$X86_64_LIBRARY_PATH" ]; then
    echo -e "${RED}Error: x86_64 static library not found at $X86_64_LIBRARY_PATH${NC}"
    echo -e "${YELLOW}Available files in target/$X86_64_TARGET/release/:${NC}"
    ls -la "target/$X86_64_TARGET/release/" | grep -E "\.(a|dylib)$" || echo "No libraries found"
    exit 1
fi

# Build for arm64 (Apple Silicon Macs)
echo -e "${YELLOW}Building for arm64 (Apple Silicon Macs)...${NC}"
cargo build --release --features auv3 --target $ARM64_TARGET

ARM64_LIBRARY_PATH="target/$ARM64_TARGET/release/$LIBRARY_NAME"
if [ ! -f "$ARM64_LIBRARY_PATH" ]; then
    echo -e "${RED}Error: arm64 static library not found at $ARM64_LIBRARY_PATH${NC}"
    echo -e "${YELLOW}Available files in target/$ARM64_TARGET/release/:${NC}"
    ls -la "target/$ARM64_TARGET/release/" | grep -E "\.(a|dylib)$" || echo "No libraries found"
    exit 1
fi

# Create universal binary using lipo
echo -e "${YELLOW}Creating universal binary...${NC}"
lipo -create "$X86_64_LIBRARY_PATH" "$ARM64_LIBRARY_PATH" -output "$OUTPUT_DIR/$UNIVERSAL_LIBRARY_NAME"

# Verify the universal library was created
if [ -f "$OUTPUT_DIR/$UNIVERSAL_LIBRARY_NAME" ]; then
    echo -e "${GREEN}Successfully created universal binary: $UNIVERSAL_LIBRARY_NAME${NC}"
    echo -e "${YELLOW}Universal library size: $(du -h "$OUTPUT_DIR/$UNIVERSAL_LIBRARY_NAME" | cut -f1)${NC}"
    
    # Show architectures in the universal binary
    echo -e "${YELLOW}Architectures in universal binary:${NC}"
    lipo -info "$OUTPUT_DIR/$UNIVERSAL_LIBRARY_NAME"
    
    # Also copy the single-architecture library for backward compatibility
    echo -e "${YELLOW}Copying x86_64 library for backward compatibility...${NC}"
    cp "$X86_64_LIBRARY_PATH" "$OUTPUT_DIR/$LIBRARY_NAME"
    
    # Copy universal binary to build directory for Xcode linking
    echo -e "${YELLOW}Copying universal binary to build directory...${NC}"
    mkdir -p "$OUTPUT_DIR/build/Debug"
    cp "$OUTPUT_DIR/$UNIVERSAL_LIBRARY_NAME" "$OUTPUT_DIR/build/Debug/$UNIVERSAL_LIBRARY_NAME"
else
    echo -e "${RED}Error: Failed to create universal binary${NC}"
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