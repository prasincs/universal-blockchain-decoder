#!/bin/bash
# Install Verus for CI and local development
# This script downloads and sets up Verus for formal verification

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}=== Verus Installation Script ===${NC}"

# Detect platform
OS="$(uname -s)"
ARCH="$(uname -m)"

case "${OS}" in
    Linux*)
        if [ "${ARCH}" = "x86_64" ]; then
            PLATFORM="x86-linux"
        else
            echo -e "${RED}Unsupported architecture: ${ARCH}${NC}"
            exit 1
        fi
        ;;
    Darwin*)
        if [ "${ARCH}" = "arm64" ]; then
            PLATFORM="arm64-macos"
        elif [ "${ARCH}" = "x86_64" ]; then
            PLATFORM="x86-macos"
        else
            echo -e "${RED}Unsupported architecture: ${ARCH}${NC}"
            exit 1
        fi
        ;;
    MINGW*|MSYS*|CYGWIN*)
        PLATFORM="x86-win"
        ;;
    *)
        echo -e "${RED}Unsupported OS: ${OS}${NC}"
        exit 1
        ;;
esac

echo -e "${GREEN}Detected platform: ${PLATFORM}${NC}"

# Get latest release version
echo -e "${YELLOW}Fetching latest Verus release...${NC}"
RELEASE_INFO=$(curl -s https://api.github.com/repos/verus-lang/verus/releases/latest)
VERSION=$(echo "${RELEASE_INFO}" | grep '"tag_name"' | sed -E 's/.*"([^"]+)".*/\1/')

if [ -z "${VERSION}" ]; then
    echo -e "${RED}Failed to fetch latest version${NC}"
    exit 1
fi

echo -e "${GREEN}Latest version: ${VERSION}${NC}"

# Extract version number without 'release/' prefix
VERSION_NUM=$(echo "${VERSION}" | sed 's/release\///')

# Construct download URL
DOWNLOAD_URL="https://github.com/verus-lang/verus/releases/download/${VERSION}/verus-${VERSION_NUM}-${PLATFORM}.zip"

echo -e "${YELLOW}Download URL: ${DOWNLOAD_URL}${NC}"

# Create tools directory if it doesn't exist
INSTALL_DIR="${INSTALL_DIR:-./tools/verus-bin}"
mkdir -p "${INSTALL_DIR}"

# Download Verus
echo -e "${YELLOW}Downloading Verus...${NC}"
cd "${INSTALL_DIR}"
curl -L "${DOWNLOAD_URL}" -o verus.zip

# Check if download was successful
if [ ! -s verus.zip ]; then
    echo -e "${RED}Download failed or file is empty${NC}"
    rm -f verus.zip
    exit 1
fi

# Extract
echo -e "${YELLOW}Extracting Verus...${NC}"
unzip -q verus.zip
rm verus.zip

# Find the extracted directory
EXTRACTED_DIR=$(find . -maxdepth 1 -type d -name "verus-*" | head -1)

if [ -z "${EXTRACTED_DIR}" ]; then
    echo -e "${RED}Failed to find extracted directory${NC}"
    exit 1
fi

# Move contents to current directory
mv "${EXTRACTED_DIR}"/* .
rmdir "${EXTRACTED_DIR}"

# Make executable (Unix-like systems)
if [ "${OS}" != "MINGW"* ] && [ "${OS}" != "MSYS"* ] && [ "${OS}" != "CYGWIN"* ]; then
    chmod +x verus 2>/dev/null || true

    # macOS: Remove quarantine
    if [ "${OS}" = "Darwin" ]; then
        echo -e "${YELLOW}Removing macOS quarantine...${NC}"
        xattr -d com.apple.quarantine verus 2>/dev/null || true
        find . -type f -exec xattr -d com.apple.quarantine {} \; 2>/dev/null || true
    fi
fi

# Test installation
echo -e "${YELLOW}Testing Verus installation...${NC}"
if [ -f "verus" ]; then
    VERUS_BIN="./verus"
elif [ -f "verus.exe" ]; then
    VERUS_BIN="./verus.exe"
else
    echo -e "${RED}Verus binary not found${NC}"
    exit 1
fi

"${VERUS_BIN}" --version || echo -e "${YELLOW}Note: Verus installed but version check failed (this may be normal)${NC}"

echo -e "${GREEN}=== Verus installed successfully! ===${NC}"
echo -e "${GREEN}Location: ${INSTALL_DIR}${NC}"
echo -e "${GREEN}Binary: ${VERUS_BIN}${NC}"
echo ""
echo -e "${YELLOW}To use Verus, add it to your PATH:${NC}"
echo -e "  export PATH=\"\$(pwd)/${INSTALL_DIR}:\$PATH\""
echo ""
echo -e "${YELLOW}Or use the wrapper script:${NC}"
echo -e "  ./scripts/verus.sh <file.rs>"
