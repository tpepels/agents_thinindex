#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

if ! command -v cargo >/dev/null 2>&1; then
  echo "error: cargo is required" >&2
  exit 1
fi

PREFIX="${PREFIX:-$HOME/.local}"
BIN_DIR="${BIN_DIR:-$PREFIX/bin}"
EXPECTED_INDEX_SCHEMA="$(sed -n 's/^pub const INDEX_SCHEMA_VERSION: u32 = \([0-9][0-9]*\);$/\1/p' "$ROOT/src/model.rs")"

if [[ -z "$EXPECTED_INDEX_SCHEMA" ]]; then
  echo "error: failed to read expected index schema from $ROOT/src/model.rs" >&2
  exit 1
fi

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

for bin in build_index wi wi-init wi-stats; do
  version_output="$("$BIN_DIR/$bin" --version)"
  echo "$version_output"
  if [[ "$version_output" != *"index schema $EXPECTED_INDEX_SCHEMA"* ]]; then
    echo "error: installed $BIN_DIR/$bin did not report expected index schema $EXPECTED_INDEX_SCHEMA" >&2
    exit 1
  fi
done

for bin in build_index wi wi-init wi-stats; do
  active_path="$(command -v "$bin" 2>/dev/null || true)"
  if [[ -n "$active_path" && "$active_path" != "$BIN_DIR/$bin" ]]; then
    echo "warning: PATH resolves $bin to $active_path, not $BIN_DIR/$bin"
  fi
done

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
