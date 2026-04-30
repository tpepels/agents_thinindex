# PLAN_29_QUALITY_PLUGIN_CI_READINESS.md

Use superpowers:subagent-driven-development.

Do not implement this until PLAN_28_PARSER_QUERY_MAINTENANCE_GUIDE.md is complete and green.

Progress:
- [x] Inspect current CI, release checks, quality docs, and normal/ignored quality tests.
- [x] Add deterministic local CI check script and workflow quality fixture job.
- [x] Document CI-safe and manual-only quality gates.
- [x] Add CI-readiness tests for scripts, workflows, and docs.
- [x] Run required verification.
- [x] Commit with completed plan checkboxes.

Goal:
Make the quality plugin and parser quality gates CI-ready without requiring external comparators, network access, or real local repos.

This pass prepares quality checks for automation. Do not add release packaging, license enforcement, payment behavior, telemetry, cloud behavior, or unrelated product features.

Product rule:
CI quality gates must be deterministic and self-contained. Optional real-repo/comparator checks remain manual or separate.

CI-ready checks:
- parser conformance fixtures
- expected-symbol fixture checks
- support-level claim checks
- ctags allowlist gate
- license audit
- quality report generation on fixture data
- no production DB pollution
- no external comparator requirement
- no `test_repos/` requirement
- no network requirement

Manual checks:
Remain manual/ignored:
- real-repo quality gate
- optional comparator quality report
- quality improvement cycle against local repos

Required implementation:
1. Add or update local CI check script if present.
2. Add quality checks to CI documentation.
3. Ensure normal `cargo test` covers deterministic quality gates.
4. Ensure ignored/manual checks are clearly documented.
5. Ensure CI cannot accidentally require ctags.
6. Ensure CI cannot accidentally require `test_repos/`.
7. Ensure reports generated in CI are small/deterministic if generated.

Docs:
Update:
- docs/QUALITY.md
- docs/QUALITY_GATES.md if present
- docs/RELEASING.md if present
- README development/checks section

Acceptance:
- quality plugin has deterministic CI-safe gates
- manual/ignored gates are clearly separated
- no external tools are required for normal CI
- no network is required
- no real repos are required
- existing verification remains stable

Verification:
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --all-targets --all-features -- -D warnings`
- ctags allowlist gate
- license audit command if configured
- quality fixture report command if added
- `cargo run --bin build_index`
- `cargo run --bin wi -- build_index`
- `cargo test --test local_index -- --ignored`
- `cargo test --test real_repos -- --ignored`

Report:
- changed files
- CI-safe quality checks
- manual-only quality checks
- docs updated
- verification commands and results
- commit hash
