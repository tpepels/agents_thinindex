# PLAN_11A_REMOVE_CTAGS_SURFACES_AND_ADD_PARSER_BOUNDARY.md

Use superpowers:subagent-driven-development.

Do not implement this until PLAN_00 through PLAN_10 are complete and green.

Goal:
Remove Universal Ctags as an architectural dependency and add a native parser boundary.

This pass does not need to restore full language parity. It creates the parser abstraction, removes ctags command integration, removes ctags test skipping, and makes the project build/test without ctags installed.

Product rule:
thinindex must be self-contained, proprietary-package-compatible, and cross-platform. Do not depend on GPL parser tooling.

Hard requirements:
- Do not call Universal Ctags.
- Do not bundle Universal Ctags.
- Do not keep Universal Ctags as optional fallback.
- Do not detect Universal Ctags on PATH.
- Do not skip tests when ctags is missing.
- Do not mention ctags as an install requirement.
- New builds must work in an environment without ctags installed.
- Do not add GPL or AGPL dependencies.
- Do not add licensing/payment/Pro-gating behavior.
- Do not add release packaging behavior in this plan.

Parser boundary:
Add a native parser abstraction.

Suggested shape:
- `src/parser.rs`
- `src/native_parser.rs`
- `ParserBackend`
- `parse_file(path, rel_path, text) -> Vec<IndexRecord>`

The exact names may differ, but the indexing code should no longer be ctags-shaped.

Initial behavior:
- Native parser may be incomplete in this pass.
- Keep existing extras extractors for CSS/HTML/Markdown/TODO/FIXME/Makefile if they are already custom/project-owned code.
- For code symbols not yet supported by native parser, tests may be adjusted to the currently supported native behavior.
- Do not fake ctags-equivalent extraction.

Required implementation:
1. Remove `src/ctags.rs` if possible.
2. Remove all ctags imports/calls from indexer/build flow.
3. Remove `check_ctags`.
4. Remove `ctags_universal` from `BuildStats` and related tests/output.
5. Remove all `has_ctags()` skip logic from tests.
6. Add a native parser module/boundary.
7. Integrate native parser into `build_index`.
8. Keep deterministic sorting and duplicate-location canonicalization.
9. Increment `INDEX_SCHEMA_VERSION` because extraction semantics changed.
10. Update tests and fixtures to no longer depend on ctags.
11. Update docs/install/help text to remove ctags requirements.
12. Keep SQLite storage canonical.
13. Keep refs, pack, impact, bench, stats, and wi-init behavior stable as far as current extraction support allows.

Record source:
- Native parser records should use `source = "native"` initially unless a specific backend is implemented.
- Extras should continue using `source = "extras"`.
- Newly built indexes must not contain `source = "ctags"`.

Tests:
Required tests:
- `build_index` succeeds without ctags installed.
- generated index has no records with `source = "ctags"`.
- no tests skip because ctags is missing.
- CSS/HTML/Markdown/TODO extras still pass.
- duplicate path+line+col invariant still holds.
- changed/deleted/unchanged rebuild tests still pass.
- SQLite index/ref integrity tests still pass.
- `wi` search/filter tests are updated for native/extras source names.
- docs/install tests do not mention ctags as required.

Instruction surfaces:
- Do not reintroduce `WI.md`.
- Keep `wi --help` as the source of truth for command syntax, filters, examples, and subcommands.
- Keep AGENTS.md and existing CLAUDE.md generation in sync with the canonical `## Repository search` block.
- AGENTS.md should be created if absent.
- CLAUDE.md should be normalized only if present; do not create CLAUDE.md.
- Repeated `wi-init` runs must not duplicate `## Repository search`.
- Remove/normalize legacy markers: `@WI.md`, weak WI.md references, and old paragraph-style Repository search blocks.

Acceptance:
- ctags command integration is gone.
- ctags is not required to build, test, or run.
- no newly built records use `source = "ctags"`.
- native parser boundary exists.
- unsupported language gaps are documented rather than hidden.
- existing extras/search/storage/stats/wi-init behavior remains stable.
- no release packaging or licensing gates are added.

Verification:
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `grep -R "ctags\\|Ctags\\|CTAGS" src tests docs README.md Cargo.toml install.sh uninstall.sh || true`
- `cargo run --bin build_index`
- `cargo run --bin wi -- build_index`
- `cargo run --bin wi-stats`
- `cargo test --test local_index -- --ignored`
- `cargo test --test real_repos -- --ignored` if `test_repos/` exists

Report:
- changed files
- removed ctags surfaces
- native parser boundary added
- supported extraction after this pass
- known extraction gaps
- verification commands and results
- whether ignored local test passed
- whether ignored real-repo test ran, skipped, or failed
- commit hash

Phase tracking:
- [x] Remove external parser command integration and status output.
- [x] Add native parser boundary and initial native parser implementation.
- [x] Increment SQLite index schema version for extraction/source semantics.
- [x] Remove parser-tool skips and old source expectations from tests.
- [x] Update docs/install surfaces for self-contained indexing.
- [x] Run required verification.
- [x] Commit scoped PLAN_11A changes.
