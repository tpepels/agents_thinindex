#!/usr/bin/env bash
set -euo pipefail

PREFIX="${PREFIX:-$HOME/.local}"
BIN_DIR="${BIN_DIR:-$PREFIX/bin}"

BINARIES=("build_index" "wi" "wi-init" "wi-stats")

echo "Uninstalling from: $BIN_DIR"

for bin in "${BINARIES[@]}"; do
  TARGET="$BIN_DIR/$bin"
  if [ -f "$TARGET" ]; then
    rm -f "$TARGET"
    echo "removed: $TARGET"
  else
    echo "not found: $TARGET"
  fi
done

cat <<'EOF'

Note:
  This removed the thinindex commands only.

  It does not remove repo-local files such as:
    .dev_index/
    .thinindexignore
    AGENTS.md
    CLAUDE.md

  To remove a repo-local index, run inside that repo before uninstalling:
    wi-init --remove
EOF

echo "done."
