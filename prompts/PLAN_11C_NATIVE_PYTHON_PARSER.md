# PLAN_11C_NATIVE_PYTHON_PARSER.md

Use superpowers:subagent-driven-development.

Do not implement this until PLAN_11A and PLAN_11B are complete and green.

Goal:
Add native Python symbol extraction through the parser boundary introduced in PLAN_11A.

This pass focuses only on Python extraction. Do not implement JS/TS, packaging, licensing gates, payment behavior, or new product commands.

Product rule:
Native Python parser support must be permissively licensed, bundled, deterministic, and tested without ctags.

Parser dependency:
- Prefer a tree-sitter Python grammar only if its license is permissive and recorded.
- Do not add GPL or AGPL dependencies.
- If the grammar license is unclear, stop and report the blocker.
- Add/update third-party notice/audit docs for any parser/grammar dependency introduced.

Extraction targets:
For Python files, extract useful records for:
- module-level functions
- classes
- methods
- async functions/methods
- imports where practical
- constants/variables where practical and not noisy

Record source:
- Use `source = "tree_sitter"` or the native source name chosen by the parser boundary.
- Do not emit `source = "ctags"`.

Required implementation:
1. Add Python parser support behind the native parser boundary.
2. Keep line/col correct and 1-based.
3. Keep record names stable and useful for `wi <term>`.
4. Keep deterministic output order.
5. Preserve duplicate-location canonicalization.
6. Increment `INDEX_SCHEMA_VERSION` if record semantics require a rebuild.
7. Update Python fixtures/tests.
8. Add focused Python parser fixtures if current fixtures are insufficient.
9. Update refs/pack/impact tests only where source naming or extraction behavior changes.

Tests:
Required tests:
- Python fixture indexes classes.
- Python fixture indexes functions.
- Python fixture indexes methods.
- Python fixture indexes async functions if supported.
- Python fixture indexes imports if implemented.
- `wi PromptService -l py` or equivalent search finds expected Python symbol.
- generated index has no `source = "ctags"`.
- existing SQLite/ref/search/stats tests pass.

Docs:
- Update parser support docs to mark Python native parsing as supported.
- Update third-party notices/audit docs for Python parser dependency.

Instruction surfaces:
- Do not reintroduce `WI.md`.
- Keep `wi --help` current if source/filter examples change.
- Keep AGENTS.md and existing CLAUDE.md canonical block behavior unchanged unless workflow changes.

Acceptance:
- Python symbol extraction no longer depends on ctags.
- Python extraction is useful on realistic Python fixture/project layouts.
- no GPL/AGPL dependency is introduced.
- no `source = "ctags"` records are emitted.
- existing command behavior remains stable except expected native source naming.

Verification:
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo run --bin build_index`
- `cargo run --bin wi -- PromptService -l py` if such a fixture/local symbol exists, otherwise use a known Python test symbol
- `cargo test --test local_index -- --ignored`
- `cargo test --test real_repos -- --ignored` if `test_repos/` exists

Report:
- changed files
- Python parser dependency and license
- Python record kinds supported
- known Python extraction gaps
- verification commands and results
- whether ignored local test passed
- whether ignored real-repo test ran, skipped, or failed
- commit hash

Phase tracking:
- [x] Harden Python extraction behind the native parser boundary.
- [x] Add focused Python fixture/tests for supported record kinds and `wi -l py`.
- [x] Document Python native parser support and dependency status.
- [x] Run required verification.
- [x] Commit scoped PLAN_11C changes.
