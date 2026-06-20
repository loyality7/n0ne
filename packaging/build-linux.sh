#!/bin/sh
# Build and package n0ne for Linux
# Usage: sh packaging/build-linux.sh [version]
# Output: n0ne-linux-x86_64.tar.gz

set -e

VERSION="${1:-dev}"
TARGET="x86_64-unknown-linux-gnu"
DIST="dist/n0ne"

echo "==> Building n0ne $VERSION for Linux..."
cargo build --release --target "$TARGET" --bin n0ne

echo "==> Packaging..."
rm -rf dist
mkdir -p "$DIST"

cp "target/$TARGET/release/n0ne"  "$DIST/"
cp README.md                       "$DIST/"
cp LICENSE                         "$DIST/"
cp -r examples                     "$DIST/"

ARCHIVE="n0ne-linux-x86_64.tar.gz"
cd dist && tar -czf "../$ARCHIVE" n0ne
cd ..

echo ""
echo "  Done: $ARCHIVE"
echo "  Size: $(du -sh $ARCHIVE | cut -f1)"
