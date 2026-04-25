#!/usr/bin/env bash
set -euo pipefail

PREFIX="${PREFIX:-$HOME/.local}"
BIN_DIR="${BIN_DIR:-$PREFIX/bin}"

BINARIES=("build_index" "wi" "wi-init")

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

  Existing repositories may still contain:
    .dev_index/
    WI.md
    AGENTS.md entries that say: See WI.md for repository search/index usage.

  To clean a repo before uninstalling, run inside that repo:
    wi-init --remove

  If thinindex is already uninstalled, remove manually:
    rm -rf .dev_index
    rm -f WI.md
    # then edit AGENTS.md and remove the WI.md reference if desired
EOF

echo "done."