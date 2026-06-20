#!/bin/sh
# n0ne installer — https://github.com/loyality7/n0ne
# Usage: curl -fsSL https://raw.githubusercontent.com/loyality7/n0ne/main/scripts/install.sh | sh

set -e

REPO="loyality7/n0ne"
INSTALL_DIR="/usr/local/bin"
BIN_NAME="n0ne"

# ── Detect OS and architecture ────────────────────────────────────────────────
OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
  Linux)  PLATFORM="linux" ;;
  Darwin) PLATFORM="macos" ;;
  *)
    echo "error: unsupported OS: $OS"
    echo "  n0ne supports Linux and macOS."
    exit 1
    ;;
esac

case "$ARCH" in
  x86_64)  ARCH_TAG="x86_64" ;;
  aarch64|arm64) ARCH_TAG="aarch64" ;;
  *)
    echo "error: unsupported architecture: $ARCH"
    exit 1
    ;;
esac

# ── Check / install clang ─────────────────────────────────────────────────────
check_clang() {
  command -v clang >/dev/null 2>&1
}

install_clang() {
  echo "==> clang not found — installing..."
  if command -v apt-get >/dev/null 2>&1; then
    sudo apt-get update -q && sudo apt-get install -y clang
  elif command -v brew >/dev/null 2>&1; then
    brew install llvm
  elif command -v pacman >/dev/null 2>&1; then
    sudo pacman -S --noconfirm clang
  elif command -v dnf >/dev/null 2>&1; then
    sudo dnf install -y clang
  elif command -v apk >/dev/null 2>&1; then
    sudo apk add clang
  else
    echo "error: could not install clang automatically."
    echo "  Please install clang manually and re-run this script."
    echo "  Ubuntu/Debian:  sudo apt install clang"
    echo "  macOS:          brew install llvm"
    exit 1
  fi
}

if ! check_clang; then
  install_clang
fi

echo "==> clang found: $(command -v clang)"

# ── Fetch latest release tag ──────────────────────────────────────────────────
echo "==> Fetching latest n0ne release..."
LATEST=$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" \
  | grep '"tag_name"' | head -1 | sed 's/.*"tag_name": "\(.*\)".*/\1/')

if [ -z "$LATEST" ]; then
  echo "error: could not fetch latest release from GitHub."
  exit 1
fi

echo "==> Latest release: $LATEST"

# ── Download binary ───────────────────────────────────────────────────────────
ARTIFACT="n0ne-${PLATFORM}-${ARCH_TAG}.tar.gz"
URL="https://github.com/${REPO}/releases/download/${LATEST}/${ARTIFACT}"

TMP_DIR="$(mktemp -d)"
TMP_FILE="${TMP_DIR}/${ARTIFACT}"

echo "==> Downloading ${ARTIFACT}..."
curl -fsSL --progress-bar -o "$TMP_FILE" "$URL"

echo "==> Extracting..."
tar -xzf "$TMP_FILE" -C "$TMP_DIR"

# ── Install binary ────────────────────────────────────────────────────────────
BINARY="${TMP_DIR}/n0ne/${BIN_NAME}"

if [ ! -f "$BINARY" ]; then
  echo "error: binary not found in archive."
  exit 1
fi

chmod +x "$BINARY"

if [ -w "$INSTALL_DIR" ]; then
  mv "$BINARY" "${INSTALL_DIR}/${BIN_NAME}"
else
  sudo mv "$BINARY" "${INSTALL_DIR}/${BIN_NAME}"
fi

rm -rf "$TMP_DIR"

# ── Done ──────────────────────────────────────────────────────────────────────
echo ""
echo "  n0ne ${LATEST} installed successfully!"
echo ""
echo "  Get started:"
echo "    echo 'task main\\n    print(\"hello world\")' > hello.n0"
echo "    n0ne run hello.n0"
echo ""
