# PLAN_27_REAL_REPO_MANIFEST_CURATION.md

Use superpowers:subagent-driven-development.

Do not implement this until PLAN_26_LANGUAGE_SUPPORT_DASHBOARD_DOC.md is complete and green.

Progress:
- [x] Inspect manifest parser, local ignored manifest, real-repo tests, and quality docs.
- [x] Add manifest metadata fields and validation for curated entries.
- [x] Add fixture manifest validation tests independent of `test_repos/`.
- [x] Update local ignored `test_repos/MANIFEST.toml` if present.
- [x] Add real-repo manifest curation docs and links.
- [x] Run required verification.
- [x] Commit with completed plan checkboxes.

Goal:
Curate `test_repos/MANIFEST.toml` into a useful local quality corpus with expected symbols, expected patterns, repo categories, and language coverage metadata.

This pass improves local real-repo quality data. Do not commit third-party repos. Do not download repos automatically. Do not add parser architecture, release packaging, license enforcement, payment behavior, telemetry, or cloud behavior.

Product rule:
The real-repo corpus should be intentional. It must cover supported languages and known difficult syntax, not just random cloned repos.

Manifest goals:
- every supported language present in `test_repos/` has expected symbols or patterns where practical
- every repo has name/path/kind/languages/queries
- every repo can have expected paths
- every repo can have expected symbols
- every repo can have expected patterns
- unsupported/deferred languages are noted
- generated/vendor-heavy repos are marked with ignore guidance where needed

Required implementation:
1. Improve manifest schema if needed.
2. Add validation for manifest entries.
3. Add docs for choosing real repos.
4. Add docs for adding expected symbols/patterns.
5. Add local-only guidance so third-party repos are not committed.
6. Add tests using fixture manifests.
7. Keep normal tests independent of actual `test_repos/`.

Manifest fields:
Support or document:
- repo name
- repo path
- kind/category
- primary languages
- queries
- expected paths
- expected symbols
- expected symbol patterns
- expected absent symbols
- optional thresholds
- skip reason
- notes

Failure policy:
- manifest-listed non-skipped repo missing should fail ignored/manual real-repo tests
- malformed manifest should fail clearly
- normal tests should use temp/fixture manifests only

Docs:
Update quality docs with:
- how to choose side/sample repos
- what not to add
- how to keep repos local
- how to add expected symbols
- how to remove stale repos from manifest

Acceptance:
- manifest schema is documented
- manifest validation exists
- fixture tests cover manifest validation
- local real-repo manifest is cleaner and more useful if present
- normal tests do not require real repos
- existing quality gates remain stable

Verification:
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --all-targets --all-features -- -D warnings`
- ctags allowlist gate
- license audit command if configured
- `cargo test --test real_repos -- --ignored`
- quality report/gate command if added

Report:
- changed files
- manifest schema changes
- fixture manifest tests
- real-repo manifest improvements
- verification commands and results
- commit hash
