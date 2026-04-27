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
install -m 0755 target/release/wi-stats "$BIN_DIR/wi-stats"

for bin in build_index wi wi-init wi-stats; do
  if [[ ! -x "$BIN_DIR/$bin" ]]; then
    echo "error: install failed: $BIN_DIR/$bin is not executable" >&2
    exit 1
  fi
done

"$BIN_DIR/build_index" --version
"$BIN_DIR/wi" --version
"$BIN_DIR/wi-init" --version
"$BIN_DIR/wi-stats" --version

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
echo "  $BIN_DIR/wi-init"
echo "  $BIN_DIR/wi-stats"