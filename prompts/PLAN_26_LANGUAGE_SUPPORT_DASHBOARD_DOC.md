# PLAN_26_LANGUAGE_SUPPORT_DASHBOARD_DOC.md

Use superpowers:subagent-driven-development.

Do not implement this until PLAN_25_QUALITY_PLUGIN_REPORT_EXPORTS.md is complete and green.

Progress:
- [x] Inspect support matrix, current parser docs, and quality export data shape.
- [x] Add deterministic language-support dashboard renderer.
- [x] Generate `docs/LANGUAGE_SUPPORT.md` from source-of-truth support data.
- [x] Add stale-check and support-claim tests.
- [x] Update README and quality docs to link the dashboard.
- [x] Run required verification.
- [x] Commit with completed plan checkboxes.

Goal:
Create a generated language-support dashboard document from the parser support matrix and quality reports.

This is documentation/reporting only. Do not add parser architecture, new languages, release packaging, license enforcement, payment behavior, telemetry, cloud behavior, or unrelated product commands.

Product rule:
Language support claims must be visible, current, and generated from checked data where practical.

Preferred output:
- `docs/LANGUAGE_SUPPORT.md`

Dashboard sections:
- summary table
- support level per language/format
- backend: tree_sitter or extras
- supported record kinds
- known gaps
- license status
- conformance status
- real-repo status
- expected-symbol coverage status
- comparator status if available
- last generated command or verification note

Support levels:
Use existing support levels:
- supported
- experimental
- blocked
- extras-backed

Hard requirements:
- Do not manually duplicate the support matrix if a source of truth exists.
- Generated docs must be deterministic.
- Tests should fail if generated dashboard is stale, if practical.
- Do not claim semantic/LSP-level analysis.
- Do not claim unsupported languages are supported.
- Do not hide blocked languages.

Tests:
- dashboard generation is deterministic
- dashboard includes every known language/format
- dashboard marks blocked/experimental correctly
- dashboard does not call extras-backed formats Tree-sitter-backed
- stale dashboard check exists if practical

Docs:
Update README/QUALITY docs to link to `docs/LANGUAGE_SUPPORT.md`.

Acceptance:
- language-support dashboard exists
- dashboard is generated or checked against source-of-truth data
- support claims are protected by tests
- no parser behavior changes are introduced
- existing quality reports/gates remain stable

Verification:
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --all-targets --all-features -- -D warnings`
- ctags allowlist gate
- license audit command if configured
- dashboard generation/check command if added
- `cargo run --bin build_index`
- `cargo run --bin wi -- build_index`
- `cargo test --test local_index -- --ignored`
- `cargo test --test real_repos -- --ignored`

Report:
- changed files
- dashboard source of truth
- generated dashboard sections
- stale-check behavior
- verification commands and results
- commit hash
