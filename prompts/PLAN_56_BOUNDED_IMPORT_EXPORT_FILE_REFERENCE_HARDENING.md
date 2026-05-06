# PLAN_56_BOUNDED_IMPORT_EXPORT_FILE_REFERENCE_HARDENING.md

Use superpowers:subagent-driven-development.

Do not implement this until PLAN_55_REFS_CONFIDENCE_REASON_OUTPUT_ALIGNMENT.md is complete and green.

Goal:
Improve import/export-aware file-reference extraction without bloating the index or flooding agents with low-value references.

Scope:
Enhance import/export file-reference extraction, resolution, caps, ranking, and docs.

Do not add:
- package-manager execution
- network access
- LSP/compiler dependencies
- broad semantic resolution
- MCP
- packaging
- licensing enforcement
- telemetry
- cloud behavior
- ctags production use
- JSONL canonical storage
- `WI.md`

Product rule:
Import/export support should improve `refs`, `pack`, and `impact`. It must not create unbounded reference noise.

Phases:
- [x] Measure current file-reference counts and unresolved reasons.
- [x] Inspect current import/export extraction.
- [x] Add missing high-value import/export forms.
- [x] Add or tighten per-file/per-kind caps.
- [x] Add ranking so resolved local refs beat unresolved/heuristic refs.
- [x] Add warnings/stats for capped/noisy files.
- [x] Add regression fixtures from real patterns.
- [x] Update docs.
- [x] Run before/after metrics.
- [x] Run verification.
- [x] Commit.

Import/export forms to consider:
- JS/TS `import x from "./x"`
- JS/TS side-effect imports: `import "./x"`
- JS/TS re-exports: `export * from "./x"`, `export { x } from "./x"`
- JS/TS dynamic import with literal string: `import("./x")`
- Python relative imports: `from .foo import Bar`
- Python local module imports where resolvable
- Rust `mod foo`, `pub mod foo`
- Rust `use crate::foo::bar` where practical
- Go local module imports where `go.mod` gives local module path
- C/C++ quoted includes
- Ruby/PHP local require/include
- Shell source/dot references

Noise controls:
- Store file-level import/export edges, not one row per imported symbol by default.
- Deduplicate by source path, raw/target path, kind, and line where appropriate.
- Prefer one useful edge per import/export statement.
- Do not emit one row per imported name unless capped and justified.
- Do not index external package names as local file refs.
- Do not turn every config string into a reference.
- Do not include external URLs, mailto, data URLs, anchors, or package names as local file refs.
- Add per-file caps.
- Add per-kind caps if needed.
- Keep output deterministic.
- Rank high-confidence local edges above unresolved/heuristic edges.

Required metrics:
Before/after on the current repo:
- total records
- total file references
- file references by kind
- unresolved file references by reason
- top 10 files by file-reference count
- no-change build_index timing

Tests:
- JS/TS import and re-export local file references.
- JS/TS dynamic import with literal string.
- Python relative import local file references.
- Rust mod/pub mod local file references where practical.
- Go local module import if practical.
- C/C++ quoted include local reference and system include exclusion.
- local require/include/source references.
- dedupe behavior.
- per-file/per-kind cap behavior.
- external package/URL exclusion.
- refs/pack/impact ranking local resolved refs above unresolved noise.
- no-change build_index remains fast.

Docs:
Update:
- `docs/FILE_REFERENCES.md`
- `docs/REFERENCE_GRAPH.md`
- `docs/CONTEXT_PACKS.md`
- `docs/IMPACT_ANALYSIS.md`

Acceptance:
- import/export file references improve real output.
- index growth is bounded.
- noisy imports/config strings are controlled.
- refs/pack/impact prefer useful local evidence.
- docs explain limitations and caps.

Verification:
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo run --bin build_index`
- immediate second `cargo run --bin build_index -- --stats`
- `cargo run --bin wi -- refs build_index`
- `cargo run --bin wi -- pack build_index`
- `cargo run --bin wi -- impact build_index`
- `cargo test --test local_index -- --ignored`
- `cargo test --test real_repos -- --ignored` if `test_repos/` exists

Commit:
Bound import and export file references
