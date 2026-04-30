# PLAN_31_MODULE_AND_DEPENDENCY_RESOLUTION_FOUNDATION.md

Use superpowers:subagent-driven-development.

Do not implement this until PLAN_30_QUALITY_SYSTEM_FINAL_AUDIT.md is complete and green.

Progress:
- [x] Inspect refs, store, indexer, schema, docs, and integrity-test surfaces.
- [x] Add dependency graph model and SQLite storage.
- [x] Add local dependency extraction and resolution foundation.
- [x] Add fixture tests for resolved, unresolved, stale, and duplicate edges.
- [x] Document dependency graph scope and boundaries.
- [x] Run required verification.
- [x] Commit with completed plan checkboxes.

Goal:
Add a local module/dependency resolution foundation so refs, pack, and impact can move beyond raw symbol matching.

Product rule:
Impact quality depends on knowing how files relate. Tree-sitter syntax facts are not enough.

Scope:
Build a generic dependency graph layer for local repositories.

Required:
- Add a dependency graph model separate from parser records.
- Capture file-to-file relationships where they can be resolved locally.
- Support imports/includes/requires/use statements from existing Tree-sitter records.
- Keep unresolved imports explicit.
- Do not add external LSP/compiler dependencies in this plan.
- Do not add network access.
- Do not add package-manager install behavior.
- Do not add payment/licensing/cloud behavior.

Initial ecosystems:
- Rust modules/use where practical
- Python imports where practical
- JS/TS imports where practical
- Go imports where practical
- Java package/import where practical
- C/C++ includes where practical
- Ruby/PHP requires/includes where practical
- Shell source/dot includes where practical

Data model:
Add tables or data structures for:
- source file
- target file if resolved
- import/module string
- dependency kind
- language
- confidence
- unresolved reason

Tests:
- fixture dependency graph tests per representative ecosystem
- unresolved imports are recorded, not silently dropped
- stale dependency cleanup on changed/deleted files
- no duplicate dependency edges
- refs/pack/impact continue to work

Docs:
Document:
- dependency graph is local and best-effort
- unresolved imports are expected
- no semantic compiler/LSP guarantees yet

Acceptance:
- dependency graph exists
- resolved/unresolved imports are queryable internally
- refs/pack/impact remain stable
- no external runtime dependencies are introduced

Verification:
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo run --bin build_index`
- `cargo run --bin wi -- impact build_index`
- `cargo test --test local_index -- --ignored`
- `cargo test --test real_repos -- --ignored`

Report:
- changed files
- dependency graph model
- supported ecosystems
- unresolved import behavior
- verification results
- commit hash
