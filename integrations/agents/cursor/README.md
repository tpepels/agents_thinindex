# Cursor Rules

`wi-init` creates `.cursor/rules/thinindex.mdc` in the repository. The rule is local to the repo, advisory, and idempotent across repeated init runs.

The generated rule tells Cursor to run `wi <term>` directly before blind repository discovery, use `wi refs` for broad reference searches, use `wi pack` before implementation, use `wi impact` before edits, and rely on `wi --help` for command details. Missing or stale indexes self-heal once before the search continues.

Run `wi-init --dry-run` to preview the repo-local rule before writing files. No global Cursor settings, network calls, telemetry, or hosted services are configured.
