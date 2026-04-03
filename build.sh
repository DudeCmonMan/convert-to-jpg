#!/usr/bin/env bash
set -euo pipefail

echo "=== Building Linux (x86_64) ==="
cargo build --release

echo ""
echo "=== Building Windows (x86_64) ==="
WIN_RELEASE=target/x86_64-pc-windows-gnu/release
cargo build --release --target x86_64-pc-windows-gnu

echo ""
echo "=== Build complete ==="
echo "Linux:   $(ls -lh target/release/convert-to-jpg | awk '{print $5, $NF}')"
echo "Windows: $(ls -lh $WIN_RELEASE/convert-to-jpg.exe | awk '{print $5, $NF}')"
