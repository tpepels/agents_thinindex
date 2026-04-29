# PLAN_11_REMOVE_CTAGS_AND_ADD_NATIVE_PARSER.md

Use superpowers:subagent-driven-development.

Do not implement this until PLAN_00 through PLAN_10 are complete and green.

Goal:
Remove Universal Ctags completely and replace it with a bundled permissively licensed native parser path.

This pass is a licensing/product blocker for proprietary same-binary Pro packaging and Windows/macOS/Linux release artifacts.

Product rule:
thinindex must be self-contained, proprietary-package-compatible, and cross-platform. Do not depend on GPL parser tooling.

Hard requirements:
- Remove Universal Ctags integration entirely.
- Do not bundle ctags.
- Do not call ctags.
- Do not keep ctags as optional fallback.
- Do not detect ctags on PATH.
- Do not skip tests when ctags is missing.
- Do not mention ctags as an install requirement.
- New builds must work in an environment without ctags installed.

Parser strategy:
- Add a native parser backend using permissively licensed dependencies only.
- Prefer tree-sitter for supported languages.
- Use an explicit allowlist of bundled grammars.
- Each bundled parser/grammar dependency must have its license recorded.
- Keep existing extras extractors where useful for CSS/HTML/Markdown/TODOs, provided they are your own code or permissively licensed.
- Do not assume every tree-sitter grammar is acceptable; only use audited allowlisted grammars.

Initial language support:
Implement enough native extraction to preserve current practical behavior for:
- Rust
- Python
- JavaScript / JSX
- TypeScript / TSX if practical
- CSS/HTML/Markdown via existing extras or audited parser support

Extraction targets:
At minimum, preserve or replace current useful record kinds:
- function
- class
- method where practical
- struct
- enum
- trait
- module
- const/variable where practical
- imports/exports where practical
- CSS classes/ids/variables/keyframes
- HTML tags/ids/classes/data attributes
- JSX classes/data attributes/component usage
- Markdown sections/checklists/links
- TODO/FIXME

Record source:
- Native parser records should use `source = "tree_sitter"` or another explicit native source name.
- Extras should continue using `source = "extras"`.
- Newly built indexes must not contain `source = "ctags"`.

Required implementation:
1. Remove `src/ctags.rs` or leave no ctags module by the end.
2. Remove all ctags imports/calls from `src/indexer.rs` and related storage/search paths.
3. Remove `check_ctags` behavior.
4. Remove `ctags_universal` from `BuildStats` and all tests/output unless still required by a previous public contract; prefer removing it.
5. Add native parser module(s), for example:
   - `src/parser.rs`
   - `src/tree_sitter_parser.rs`
6. Integrate native parser into `build_index`.
7. Keep deterministic sorting and duplicate-location canonicalization.
8. Increment `INDEX_SCHEMA_VERSION` because extraction semantics changed.
9. Update tests and fixtures to native parser expectations.
10. Remove all `has_ctags()` skip logic from normal and ignored tests.
11. Update install/docs/help text to remove ctags requirements.
12. Add or update third-party license/notice documentation for parser dependencies.

Dependency and license constraints:
- Do not add GPL or AGPL dependencies.
- Prefer MIT/Apache-2.0/BSD/ISC/Zlib-style dependencies.
- Add a dependency/license audit note for every new parser crate/grammar.
- If a grammar license is unclear, do not bundle it.
- Do not add commercial licensing/payment behavior in this plan.

Tests:
Update existing tests so they no longer depend on ctags.

Required tests:
- `build_index` succeeds without ctags installed.
- generated index has no records with `source = "ctags"`.
- Rust fixture indexes expected symbols.
- Python fixture indexes expected symbols.
- JS/JSX/TS/TSX fixture indexes expected symbols where supported.
- CSS/HTML/Markdown extras still pass.
- duplicate path+line+col invariant still holds.
- changed/deleted/unchanged rebuild tests still pass.
- SQLite index/ref integrity tests still pass.
- `wi` search/ranking/filter tests still pass with adjusted source names.
- `wi refs`, `wi pack`, `wi impact`, and `wi bench` tests still pass.
- install/docs tests do not mention ctags as required.

Docs:
Update docs to state:
- thinindex no longer requires Universal Ctags.
- parser support is bundled through audited permissive native parser dependencies.
- Universal Ctags is not bundled and not used.
- proprietary packaging blocker related to ctags is resolved only if all ctags dependency surfaces are removed.

Update:
- README.md
- docs/ROADMAP.md if present
- docs/PRODUCT_BOUNDARY.md if present
- THIRD_PARTY_NOTICES or equivalent if present
- install/release docs if present

Instruction surfaces:
- Do not reintroduce `WI.md`.
- Keep `wi --help` as the source of truth for command syntax, filters, examples, and subcommands.
- Keep AGENTS.md and existing CLAUDE.md generation in sync with the canonical `## Repository search` block.
- AGENTS.md should be created if absent.
- CLAUDE.md should be normalized only if present; do not create CLAUDE.md.
- Repeated `wi-init` runs must not duplicate `## Repository search`.
- Remove/normalize legacy markers: `@WI.md`, `See WI.md for repository search/index usage.`, `See `WI.md` for repository search/index usage.`, and old paragraph-style Repository search blocks.
- Update tests whenever help text or canonical Repository search text changes.

Acceptance:
- ctags is fully removed from code, tests, docs, install scripts, and product requirements.
- `build_index` works without ctags.
- no new GPL/AGPL parser dependency is introduced.
- native parser/extras produce useful records for supported fixtures.
- no newly built index records have `source = "ctags"`.
- existing CLI behavior remains stable except expected source/parser naming changes.
- SQLite storage remains canonical.
- refs, pack, impact, bench, stats, and wi-init behavior remain stable.
- proprietary packaging blocker caused by required ctags usage is documented as resolved only if all ctags usage is actually removed.

Verification:
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `grep -R "ctags\\|Ctags\\|CTAGS" src tests docs README.md Cargo.toml install.sh uninstall.sh || true`
- `cargo run --bin build_index`
- `cargo run --bin wi -- build_index`
- `cargo run --bin wi -- refs build_index`
- `cargo run --bin wi -- pack build_index`
- `cargo run --bin wi -- impact build_index`
- `cargo run --bin wi-stats`
- `cargo test --test local_index -- --ignored`
- `cargo test --test real_repos -- --ignored` if `test_repos/` exists

Report:
- changed files
- removed ctags surfaces
- native parser dependencies added and their licenses
- supported languages in this pass
- known extraction gaps
- docs updated
- verification commands and results
- whether ignored local test passed
- whether ignored real-repo test ran, skipped, or failed
- commit hash
