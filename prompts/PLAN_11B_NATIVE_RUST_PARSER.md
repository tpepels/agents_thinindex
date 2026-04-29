# PLAN_11B_NATIVE_RUST_PARSER.md

Use superpowers:subagent-driven-development.

Do not implement this until PLAN_11A is complete and green.

Goal:
Add native Rust symbol extraction through the parser boundary introduced in PLAN_11A.

This pass focuses only on Rust extraction. Do not implement Python, JS/TS, new context commands, packaging, licensing gates, or payment behavior.

Product rule:
Native parser support must be permissively licensed, bundled, deterministic, and tested without ctags.

Parser dependency:
- Prefer a tree-sitter Rust grammar only if its license is permissive and recorded.
- Do not add GPL or AGPL dependencies.
- If the grammar license is unclear, stop and report the blocker.
- Add/update third-party notice/audit docs for any parser/grammar dependency introduced.

Extraction targets:
For Rust files, extract useful records for:
- functions
- associated functions/methods where practical
- structs
- enums
- traits
- modules
- consts/statics where practical
- type aliases where practical
- imports/use items if practical

Record source:
- Use `source = "tree_sitter"` or a specific native source name chosen by PLAN_11A.
- Do not emit `source = "ctags"`.

Required implementation:
1. Add Rust parser support behind the native parser boundary.
2. Keep line/col correct and 1-based.
3. Keep record names stable and useful for `wi <term>`.
4. Keep deterministic output order.
5. Preserve duplicate-location canonicalization.
6. Increment `INDEX_SCHEMA_VERSION` if record semantics require a rebuild.
7. Update existing Rust fixtures/tests.
8. Add focused Rust parser fixtures if current fixtures are insufficient.
9. Update refs/pack/impact tests only where source naming or extraction behavior changes.

Tests:
Required tests:
- Rust fixture indexes functions.
- Rust fixture indexes structs.
- Rust fixture indexes enums.
- Rust fixture indexes traits.
- Rust fixture indexes modules where supported.
- Rust fixture indexes impl methods where supported.
- `wi build_index -l rs` or equivalent search finds expected Rust symbol.
- generated index has no `source = "ctags"`.
- local thinindex repo ignored test passes with Rust-native extraction.
- existing SQLite/ref/search/stats tests pass.

Docs:
- Update parser support docs to mark Rust native parsing as supported.
- Update third-party notices/audit docs for Rust parser dependency.

Instruction surfaces:
- Do not reintroduce `WI.md`.
- Keep `wi --help` current if source/filter examples change.
- Keep AGENTS.md and existing CLAUDE.md canonical block behavior unchanged unless workflow changes.

Acceptance:
- Rust symbol extraction no longer depends on ctags.
- Rust extraction is useful on the thinindex repo itself.
- no GPL/AGPL dependency is introduced.
- no `source = "ctags"` records are emitted.
- existing command behavior remains stable except expected native source naming.

Verification:
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo run --bin build_index`
- `cargo run --bin wi -- build_index`
- `cargo run --bin wi -- SearchOptions -l rs`
- `cargo test --test local_index -- --ignored`
- `cargo test --test real_repos -- --ignored` if `test_repos/` exists

Report:
- changed files
- Rust parser dependency and license
- Rust record kinds supported
- known Rust extraction gaps
- verification commands and results
- whether ignored local test passed
- whether ignored real-repo test ran, skipped, or failed
- commit hash

Phase tracking:
- [x] Harden Rust extraction behind the native parser boundary.
- [x] Add focused Rust fixture/tests for supported record kinds and `wi -l rs`.
- [x] Document Rust native parser support and dependency status.
- [x] Run required verification.
- [x] Commit scoped PLAN_11B changes.
