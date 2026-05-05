# RECOVERY_NEXT_INSTALLED_BINARY_SCHEMA_MISMATCH.md

Use superpowers:subagent-driven-development.

Do not resume old roadmap work until this focused recovery prompt is complete
or explicitly declined with evidence.

Goal:
Fix the operational mismatch where PATH-installed thinindex binaries are stale
relative to the source checkout and can rebuild `.dev_index/index.sqlite` with
an older schema.

Observed evidence from the 2026-05-05 recovery audit:
- Source-built `cargo run --bin wi -- doctor` expects schema 12 and reports ok
  after source-built self-healing.
- PATH resolves `wi`, `build_index`, `wi-init`, and `wi-stats` to
  `/home/tom/.local/bin/*`.
- `/home/tom/.local/bin/wi doctor` reports `schema version 12 does not match 11`
  and identifies the running binary as `/home/tom/.local/bin/wi`.
- `/home/tom/.local/bin/build_index` rebuilt this checkout's `.dev_index` using
  the older schema, after which source-built commands had to rebuild it again.

Scope:
- Audit how local release/install/copy flows update binaries under
  `/home/tom/.local/bin` or another PATH location.
- Make the installed binaries and source checkout agree on schema version and
  help text.
- Add a focused smoke check that catches stale installed binaries or documents
  an explicit manual reinstall step when an install path is outside repo
  control.
- Update docs/help/troubleshooting if users are expected to reinstall after a
  schema bump.
- Keep the fix local and practical.

Hard constraints:
- Do not implement unrelated roadmap plans.
- Do not add packaging, signing, licensing, payment, telemetry, cloud, hosted,
  MCP, or update-channel behavior.
- Do not reintroduce Universal Ctags as a production parser.
- Do not reintroduce `WI.md`.
- Do not reintroduce JSONL as canonical storage.
- Do not weaken parser/index/quality gates.
- Do not make normal `wi` or `build_index` run quality/comparator/real-repo
  checks.

Acceptance:
- PATH `wi doctor` and source-built `cargo run --bin wi -- doctor` agree on the
  current schema after the fix or the mismatch is explicitly documented as a
  manual reinstall requirement with a clear command.
- PATH `wi --help` includes the same current core behavior as source help,
  including file-reference-aware pack/impact wording.
- Running PATH `build_index` followed by source `wi doctor` does not flip the
  repo between schema versions.
- Running source `build_index` followed by PATH `wi doctor` does not report an
  older expected schema.
- Any required reinstall command is tested or manually verified.

Verification:
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo run --bin build_index`
- `cargo run --bin wi -- doctor`
- `cargo run --bin wi -- --help`
- `cargo run --bin wi -- pack build_index`
- `cargo run --bin wi -- impact build_index`
- PATH `wi doctor`
- PATH `wi --help`
- PATH `build_index`
- source `cargo run --bin wi -- doctor` after PATH `build_index`

Commit:
`Fix installed binary schema mismatch`
