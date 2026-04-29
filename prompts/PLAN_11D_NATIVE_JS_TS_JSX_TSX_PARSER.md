# PLAN_11D_NATIVE_JS_TS_JSX_TSX_PARSER.md

Use superpowers:subagent-driven-development.

Do not implement this until PLAN_11A, PLAN_11B, and PLAN_11C are complete and green.

Goal:
Add native JavaScript, TypeScript, JSX, and TSX symbol extraction through the parser boundary introduced in PLAN_11A.

This pass focuses only on JS/TS/JSX/TSX extraction. Do not add packaging, licensing gates, payment behavior, or new product commands.

Product rule:
Native JS/TS parser support must be permissively licensed, bundled, deterministic, and tested without ctags.

Parser dependency:
- Prefer tree-sitter JavaScript/TypeScript grammars only if their licenses are permissive and recorded.
- Do not add GPL or AGPL dependencies.
- If any grammar license is unclear, stop and report the blocker.
- Add/update third-party notice/audit docs for every parser/grammar dependency introduced.

Extraction targets:
For JavaScript/TypeScript/JSX/TSX files, extract useful records for:
- functions
- arrow function declarations where practical
- classes
- methods where practical
- imports
- exports
- interfaces/types for TypeScript where practical
- React/component-like function names where practical
- JSX component usage where practical

Keep existing extras for:
- JSX className
- data-testid
- HTML-like tags where already supported
- CSS/HTML/Markdown unrelated extras

Record source:
- Use `source = "tree_sitter"` or the native source name chosen by the parser boundary.
- Do not emit `source = "ctags"`.

Required implementation:
1. Add JS parser support behind the native parser boundary.
2. Add TS parser support if practical.
3. Add JSX/TSX support if practical.
4. Keep line/col correct and 1-based.
5. Keep record names stable and useful for `wi <term>`.
6. Keep deterministic output order.
7. Preserve duplicate-location canonicalization.
8. Increment `INDEX_SCHEMA_VERSION` if record semantics require a rebuild.
9. Update JS/TS/JSX/TSX fixtures/tests.
10. Add focused parser fixtures if current fixtures are insufficient.
11. Update refs/pack/impact tests only where source naming or extraction behavior changes.

Tests:
Required tests:
- JS fixture indexes functions/classes/imports where supported.
- TS fixture indexes functions/classes/types/interfaces where supported.
- JSX fixture indexes component-like symbols or component usage where supported.
- TSX fixture indexes component-like symbols or component usage where supported if implemented.
- existing JSX/CSS/HTML extras still pass.
- generated index has no `source = "ctags"`.
- existing SQLite/ref/search/stats tests pass.

Docs:
- Update parser support docs to mark JS/TS/JSX/TSX native parsing support honestly.
- If TSX is not fully supported, document the gap.
- Update third-party notices/audit docs for parser dependencies.

Instruction surfaces:
- Do not reintroduce `WI.md`.
- Keep `wi --help` current if source/filter examples change.
- Keep AGENTS.md and existing CLAUDE.md canonical block behavior unchanged unless workflow changes.

Acceptance:
- JS/TS/JSX/TSX extraction no longer depends on ctags.
- useful JS/TS/JSX fixtures are indexed.
- no GPL/AGPL dependency is introduced.
- no `source = "ctags"` records are emitted.
- existing command behavior remains stable except expected native source naming.

Verification:
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo run --bin build_index`
- `cargo run --bin wi -- HeaderNavigation -l tsx` if such a fixture/local symbol exists, otherwise use a known JS/TS fixture symbol
- `cargo test --test local_index -- --ignored`
- `cargo test --test real_repos -- --ignored` if `test_repos/` exists

Report:
- changed files
- JS/TS parser dependencies and licenses
- JS/TS/JSX/TSX record kinds supported
- known extraction gaps
- verification commands and results
- whether ignored local test passed
- whether ignored real-repo test ran, skipped, or failed
- commit hash

Phase tracking:
- [x] Harden JS/TS/JSX/TSX extraction behind the native parser boundary.
- [x] Add focused JS/TS/JSX/TSX fixtures/tests for supported record kinds and `wi -l tsx`.
- [x] Document JS/TS/JSX/TSX native parser support and dependency status.
- [x] Run required verification.
- [x] Commit scoped PLAN_11D changes.
