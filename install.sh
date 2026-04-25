#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

if ! command -v cargo >/dev/null 2>&1; then
  echo "error: cargo is required" >&2
  exit 1
fi

PREFIX="${PREFIX:-$HOME/.local}"
BIN_DIR="${BIN_DIR:-$PREFIX/bin}"

mkdir -p "$BIN_DIR"

cd "$ROOT"
cargo build --release

install -m 0755 target/release/build_index "$BIN_DIR/build_index"
install -m 0755 target/release/wi "$BIN_DIR/wi"
install -m 0755 target/release/wi-init "$BIN_DIR/wi-init"

if ! command -v "$BIN_DIR/build_index" >/dev/null 2>&1; then
  "$BIN_DIR/build_index" --version >/dev/null
fi

"$BIN_DIR/build_index" --version
"$BIN_DIR/wi" --version

case ":$PATH:" in
  *":$BIN_DIR:"*) ;;
  *)
    echo "warning: $BIN_DIR is not on PATH"
    echo "add this to your shell config:"
    echo "  export PATH=\"$BIN_DIR:\$PATH\""
    ;;
esac

echo "installed:"
echo "  $BIN_DIR/build_index"
echo "  $BIN_DIR/wi"