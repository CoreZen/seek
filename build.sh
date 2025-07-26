#!/bin/bash
set -e

# Build script for creating macOS binaries for seek

# Define colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
RED='\033[0;31m'
YELLOW='\033[0;33m'
NC='\033[0m' # No Color

# Define version from Cargo.toml or use git describe
VERSION=$(grep '^version' Cargo.toml | head -n 1 | cut -d '"' -f 2)
if [ -z "$VERSION" ]; then
  VERSION=$(git describe --tags --always || echo "unknown")
fi

# Create output directory
OUTPUT_DIR="target/release-builds"
mkdir -p "$OUTPUT_DIR"

echo -e "${BLUE}Building seek v${VERSION} for macOS${NC}"
echo -e "${YELLOW}Output directory: ${OUTPUT_DIR}${NC}"

# Function to build and package for a specific target
build_target() {
  local TARGET=$1
  local BINARY_NAME=$2
  local FOLDER_NAME="seek-${VERSION}-${TARGET}"
  local OUTPUT_PATH="${OUTPUT_DIR}/${FOLDER_NAME}"

  echo -e "\n${GREEN}Building for ${TARGET}...${NC}"

  # Create target directory
  mkdir -p "${OUTPUT_PATH}"

  # Build binary
  echo -e "${YELLOW}Compiling...${NC}"
  cargo build --release --target "${TARGET}"

  # Copy binary and rename if needed
  echo -e "${YELLOW}Packaging...${NC}"
  cp "target/${TARGET}/release/seek${BINARY_NAME}" "${OUTPUT_PATH}/seek${BINARY_NAME}"

  # Copy additional files
  cp README.md "${OUTPUT_PATH}/"
  cp LICENSE "${OUTPUT_PATH}/" 2>/dev/null || echo -e "${YELLOW}Warning: LICENSE file not found${NC}"

  # Create archive
  pushd "${OUTPUT_DIR}" > /dev/null
  if [[ "${TARGET}" == *"-windows-"* ]]; then
    zip -r "${FOLDER_NAME}.zip" "${FOLDER_NAME}"
    echo -e "${GREEN}Created ${FOLDER_NAME}.zip${NC}"
  else
    tar czf "${FOLDER_NAME}.tar.gz" "${FOLDER_NAME}"
    echo -e "${GREEN}Created ${FOLDER_NAME}.tar.gz${NC}"
  fi
  popd > /dev/null

  # Calculate sha256 checksum
  if [[ "${TARGET}" == *"-windows-"* ]]; then
    shasum -a 256 "${OUTPUT_DIR}/${FOLDER_NAME}.zip" > "${OUTPUT_DIR}/${FOLDER_NAME}.zip.sha256"
  else
    shasum -a 256 "${OUTPUT_DIR}/${FOLDER_NAME}.tar.gz" > "${OUTPUT_DIR}/${FOLDER_NAME}.tar.gz.sha256"
  fi
}

# Start building
echo -e "\n${BLUE}Starting macOS build process...${NC}"

# macOS builds
build_target "x86_64-apple-darwin" ""
build_target "aarch64-apple-darwin" ""

# Create universal macOS binary (if both architectures were built)
if [ -f "${OUTPUT_DIR}/seek-${VERSION}-x86_64-apple-darwin/seek" ] && [ -f "${OUTPUT_DIR}/seek-${VERSION}-aarch64-apple-darwin/seek" ]; then
  echo -e "\n${GREEN}Creating universal macOS binary...${NC}"
  UNIVERSAL_DIR="${OUTPUT_DIR}/seek-${VERSION}-universal-apple-darwin"
  mkdir -p "${UNIVERSAL_DIR}"

  lipo -create -output "${UNIVERSAL_DIR}/seek" \
    "${OUTPUT_DIR}/seek-${VERSION}-x86_64-apple-darwin/seek" \
    "${OUTPUT_DIR}/seek-${VERSION}-aarch64-apple-darwin/seek"

  # Copy additional files
  cp README.md "${UNIVERSAL_DIR}/"
  cp LICENSE "${UNIVERSAL_DIR}/" 2>/dev/null || true

  # Create archive
  pushd "${OUTPUT_DIR}" > /dev/null
  tar czf "seek-${VERSION}-universal-apple-darwin.tar.gz" "seek-${VERSION}-universal-apple-darwin"
  popd > /dev/null

  # Calculate sha256 checksum
  shasum -a 256 "${OUTPUT_DIR}/seek-${VERSION}-universal-apple-darwin.tar.gz" > "${OUTPUT_DIR}/seek-${VERSION}-universal-apple-darwin.tar.gz.sha256"

  echo -e "${GREEN}Created universal macOS binary: ${UNIVERSAL_DIR}/seek${NC}"
fi

# Summary
echo -e "\n${BLUE}Build Summary:${NC}"
echo -e "${GREEN}Version:${NC} ${VERSION}"
echo -e "${GREEN}Output location:${NC} ${OUTPUT_DIR}"
find "${OUTPUT_DIR}" -type f \( -name "*.tar.gz" -o -name "*.zip" \) | sort | while read -r file; do
  SIZE=$(du -h "$file" | cut -f1)
  echo -e "${GREEN}Package:${NC} $(basename "$file") (${SIZE})"
  cat "${file}.sha256"
done

echo -e "\n${BLUE}All builds completed successfully!${NC}"
