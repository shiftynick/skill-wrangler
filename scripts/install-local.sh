#!/usr/bin/env bash
# Build Skill Wrangler release and install to ~/.local/bin
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

echo "Building release (npm run tauri build)..."
npm run tauri build

BINARY_NAME="skill-wrangler"
RELEASE_BINARY="$REPO_ROOT/src-tauri/target/release/$BINARY_NAME"

if [[ ! -f "$RELEASE_BINARY" ]]; then
  echo "Release binary not found at $RELEASE_BINARY" >&2
  exit 1
fi

INSTALL_DIR="${HOME}/.local/bin"
DEST="$INSTALL_DIR/$BINARY_NAME"

mkdir -p "$INSTALL_DIR"
cp -f "$RELEASE_BINARY" "$DEST"
chmod +x "$DEST"

echo "Installed Skill Wrangler to $DEST"
