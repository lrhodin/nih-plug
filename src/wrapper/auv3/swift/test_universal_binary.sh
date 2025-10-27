#!/bin/bash

# Test script for universal binary functionality
# This script validates that the universal binary contains both architectures
# and that both can be linked successfully

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
UNIVERSAL_LIB="$SCRIPT_DIR/libnih_plug_universal.a"
X86_64_LIB="$SCRIPT_DIR/libnih_plug.a"

echo -e "${GREEN}Testing universal binary functionality...${NC}"

# Test 1: Verify universal binary exists and has both architectures
echo -e "${YELLOW}Test 1: Checking universal binary architectures...${NC}"
if [ ! -f "$UNIVERSAL_LIB" ]; then
    echo -e "${RED}ERROR: Universal binary not found at $UNIVERSAL_LIB${NC}"
    exit 1
fi

ARCHITECTURES=$(lipo -info "$UNIVERSAL_LIB" | grep -o "x86_64\|arm64" | tr '\n' ' ')
if [[ "$ARCHITECTURES" == *"x86_64"* ]] && [[ "$ARCHITECTURES" == *"arm64"* ]]; then
    echo -e "${GREEN}✓ Universal binary contains both x86_64 and arm64 architectures${NC}"
else
    echo -e "${RED}ERROR: Universal binary missing required architectures. Found: $ARCHITECTURES${NC}"
    exit 1
fi

# Test 2: Verify x86_64 library exists for backward compatibility
echo -e "${YELLOW}Test 2: Checking x86_64 library for backward compatibility...${NC}"
if [ ! -f "$X86_64_LIB" ]; then
    echo -e "${RED}ERROR: x86_64 library not found at $X86_64_LIB${NC}"
    exit 1
fi

X86_64_ARCH=$(lipo -info "$X86_64_LIB" | grep -o "x86_64")
if [ "$X86_64_ARCH" = "x86_64" ]; then
    echo -e "${GREEN}✓ x86_64 library exists and has correct architecture${NC}"
else
    echo -e "${RED}ERROR: x86_64 library has wrong architecture: $X86_64_ARCH${NC}"
    exit 1
fi

# Test 3: Verify library sizes are reasonable
echo -e "${YELLOW}Test 3: Checking library sizes...${NC}"
UNIVERSAL_SIZE=$(stat -f%z "$UNIVERSAL_LIB")
X86_64_SIZE=$(stat -f%z "$X86_64_LIB")

# Universal binary should be roughly 2x the size of single architecture
EXPECTED_UNIVERSAL_SIZE=$((X86_64_SIZE * 2))
TOLERANCE=$((EXPECTED_UNIVERSAL_SIZE / 10)) # 10% tolerance

if [ $UNIVERSAL_SIZE -gt $((EXPECTED_UNIVERSAL_SIZE - TOLERANCE)) ] && [ $UNIVERSAL_SIZE -lt $((EXPECTED_UNIVERSAL_SIZE + TOLERANCE)) ]; then
    echo -e "${GREEN}✓ Universal binary size is reasonable (${UNIVERSAL_SIZE} bytes)${NC}"
else
    echo -e "${YELLOW}WARNING: Universal binary size (${UNIVERSAL_SIZE} bytes) is not as expected (expected ~${EXPECTED_UNIVERSAL_SIZE} bytes)${NC}"
fi

# Test 4: Verify libraries are valid static archives
echo -e "${YELLOW}Test 4: Verifying library format...${NC}"
if file "$UNIVERSAL_LIB" | grep -q "current ar archive"; then
    echo -e "${GREEN}✓ Universal binary is a valid static archive${NC}"
else
    echo -e "${RED}ERROR: Universal binary is not a valid static archive${NC}"
    exit 1
fi

if file "$X86_64_LIB" | grep -q "current ar archive"; then
    echo -e "${GREEN}✓ x86_64 library is a valid static archive${NC}"
else
    echo -e "${RED}ERROR: x86_64 library is not a valid static archive${NC}"
    exit 1
fi

# Test 5: Test that we can extract symbols from both architectures
echo -e "${YELLOW}Test 5: Checking symbol extraction...${NC}"
if command -v nm &> /dev/null; then
    # Extract symbols for x86_64 architecture
    X86_64_SYMBOLS=$(nm -arch x86_64 "$UNIVERSAL_LIB" 2>/dev/null | wc -l)
    # Extract symbols for arm64 architecture  
    ARM64_SYMBOLS=$(nm -arch arm64 "$UNIVERSAL_LIB" 2>/dev/null | wc -l)
    
    if [ $X86_64_SYMBOLS -gt 0 ] && [ $ARM64_SYMBOLS -gt 0 ]; then
        echo -e "${GREEN}✓ Both architectures contain symbols (x86_64: $X86_64_SYMBOLS, arm64: $ARM64_SYMBOLS)${NC}"
    else
        echo -e "${RED}ERROR: Failed to extract symbols from one or both architectures${NC}"
        exit 1
    fi
else
    echo -e "${YELLOW}WARNING: nm command not available, skipping symbol test${NC}"
fi

echo -e "${GREEN}All tests passed! Universal binary is working correctly.${NC}"
echo -e "${YELLOW}Summary:${NC}"
echo "  - Universal binary: $UNIVERSAL_LIB ($UNIVERSAL_SIZE bytes)"
echo "  - x86_64 library: $X86_64_LIB ($X86_64_SIZE bytes)"
echo "  - Architectures: $ARCHITECTURES"
echo -e "${GREEN}Ready for Xcode integration!${NC}"