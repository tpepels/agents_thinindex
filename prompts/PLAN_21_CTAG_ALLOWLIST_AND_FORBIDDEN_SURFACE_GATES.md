# PLAN_21_CTAG_ALLOWLIST_AND_FORBIDDEN_SURFACE_GATES.md

Use superpowers:subagent-driven-development.

Do not implement this until PLAN_20_SUPPORT_LEVELS_AND_LANGUAGE_CLAIMS.md is complete and green.

Goal:
Replace broad grep-style ctags checks with a structural allowlist gate.

Progress:
- [x] Phase 1: inspect existing ctags mentions and old broad checks
- [x] Phase 2: add structural allowlist helpers and fixture tests
- [x] Phase 3: clean forbidden install/release/script/test surfaces
- [x] Phase 4: document the ctags boundary rule
- [x] Phase 5: run verification

This pass protects the product from accidentally reintroducing ctags as a parser/runtime/package dependency while still allowing ctags to exist as an optional isolated quality comparator.

Product rule:
ctags may exist only as optional QA comparator text/code. It must never return to production indexing, packaging, install requirements, or runtime behavior.

Allowed ctags locations:
- `src/quality/**`
- `tests/quality*`
- `tests/quality/**`
- `docs/QUALITY.md`
- `docs/QUALITY_*.md`
- docs that explicitly say ctags is optional, external, not bundled, not required, and not used by production indexing

Forbidden ctags locations:
- `src/indexer.rs`
- production parser modules
- production store/search/refs/pack/impact paths
- `build_index` implementation
- install scripts
- uninstall scripts
- release/package scripts
- normal install docs
- AGENTS/CLAUDE generated instruction text
- production SQLite records/refs
- normal tests that require ctags

Required implementation:
1. Add a script or test that scans ctags references using an allowlist.
2. The gate must fail on forbidden ctags references.
3. The gate must allow quality-plugin ctags references.
4. The gate must fail if production records contain `source = "ctags"`.
5. The gate must fail if release/package artifacts include ctags.
6. Add normal tests for the allowlist checker using fixture paths/content.
7. Add docs explaining the rule.

Do not:
- remove the optional comparator plugin
- make ctags required
- call ctags from production code
- bundle ctags
- add GPL/AGPL dependencies

Acceptance:
- ctags allowlist gate exists
- ctags comparator remains isolated
- forbidden ctags surfaces are checked
- normal tests do not require ctags
- production index cannot emit `source = "ctags"`
- release/package paths cannot include ctags
- existing parser/index/search/quality behavior remains stable

Verification:
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --all-targets --all-features -- -D warnings`
- run the ctags allowlist gate directly
- license audit command if configured
- `cargo run --bin build_index`
- `cargo run --bin wi -- build_index`
- `cargo test --test local_index -- --ignored`
- `cargo test --test real_repos -- --ignored`

Report:
- changed files
- allowlist rules
- forbidden surfaces checked
- verification commands and results
- commit hash
