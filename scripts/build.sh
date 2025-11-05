#!/bin/bash

# Build script for the Rust to UdonSharp integration project

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Default values
BUILD_TYPE="dev"
TARGET_DIR="target"
WASM_TARGET="wasm32-unknown-unknown"

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --release)
            BUILD_TYPE="release"
            shift
            ;;
        --wasm)
            BUILD_WASM=true
            shift
            ;;
        --target-dir)
            TARGET_DIR="$2"
            shift 2
            ;;
        --help)
            echo "Usage: $0 [OPTIONS]"
            echo "Options:"
            echo "  --release      Build in release mode"
            echo "  --wasm         Build WASM targets for UdonSharp"
            echo "  --target-dir   Specify target directory"
            echo "  --help         Show this help message"
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            exit 1
            ;;
    esac
done

echo -e "${GREEN}Building Rust to UdonSharp Integration...${NC}"

# Build the workspace
if [[ "$BUILD_TYPE" == "release" ]]; then
    echo -e "${YELLOW}Building in release mode...${NC}"
    cargo build --release --workspace
else
    echo -e "${YELLOW}Building in development mode...${NC}"
    cargo build --workspace
fi

# Build WASM targets if requested
if [[ "$BUILD_WASM" == "true" ]]; then
    echo -e "${YELLOW}Building WASM targets...${NC}"
    
    # Check if wasm32-unknown-unknown target is installed
    if ! rustup target list --installed | grep -q "$WASM_TARGET"; then
        echo -e "${YELLOW}Installing $WASM_TARGET target...${NC}"
        rustup target add "$WASM_TARGET"
    fi
    
    # Build WASM profile
    if [[ "$BUILD_TYPE" == "release" ]]; then
        cargo build --target "$WASM_TARGET" --profile wasm-release
    else
        cargo build --target "$WASM_TARGET" --profile wasm-dev
    fi
    
    # Run wasm-opt if available
    if command -v wasm-opt &> /dev/null; then
        echo -e "${YELLOW}Optimizing WASM with wasm-opt...${NC}"
        find target/"$WASM_TARGET"/wasm-release -name "*.wasm" -exec wasm-opt -Os {} -o {} \; 2>/dev/null || true
        find target/"$WASM_TARGET"/wasm-dev -name "*.wasm" -exec wasm-opt -O1 {} -o {} \; 2>/dev/null || true
    else
        echo -e "${YELLOW}wasm-opt not found, skipping WASM optimization${NC}"
    fi
    
    echo -e "${GREEN}WASM build completed!${NC}"
fi

echo -e "${GREEN}Build completed successfully!${NC}"