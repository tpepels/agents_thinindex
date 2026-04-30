# PLAN_41_SECURITY_PRIVACY_AND_REPORT_REDACTION.md

Use superpowers:subagent-driven-development.

Do not implement this until PLAN_40_TECHNICAL_FINAL_AUDIT.md is complete and green.

Progress:
- [x] Phase 1: inspect reports, context output, release artifact checks, ignore rules, and quality exports
- [x] Phase 2: add privacy redaction helpers and sensitive-path warnings
- [x] Phase 3: apply redaction to command/report output and add fixture tests
- [x] Phase 4: add security/privacy documentation and release guidance
- [x] Phase 5: run required verification and commit completed Plan 41 work

Goal:
Harden security/privacy behavior for indexing, reports, quality outputs, and release artifacts.

Product rule:
A local code index should not accidentally expose secrets or sensitive project contents through reports.

Required:
- Add sensitive-file ignore guidance.
- Add report redaction policy.
- Add quality report safe/verbose modes if needed.
- Ensure reports do not dump large source text by default.
- Ensure `.dev_index` remains local/disposable.
- Ensure release artifacts do not include local indexes/reports.
- Ensure test_repos are never committed.
- Add secret-pattern warnings where practical, without becoming a secret scanner product.

Sensitive surfaces:
- quality reports
- comparator reports
- pack output
- impact output
- bench reports
- logs
- release artifacts

Tests:
- reports exclude source snippets by default where required
- redaction rules apply in fixture data
- release archive excludes `.dev_index`
- quality output paths are ignored/local
- no secrets fixture leaks into committed report

Docs:
Add/update:
- docs/SECURITY_PRIVACY.md
- README privacy note
- quality docs redaction section

Acceptance:
- report privacy policy exists
- safe defaults exist
- local index/report paths are excluded from release artifacts
- users get clear guidance

Verification:
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --all-targets --all-features -- -D warnings`
- release artifact content check if available
- `cargo run --bin build_index`
- quality report command if available

Report:
- changed files
- redaction behavior
- security/privacy docs
- verification results
- commit hash
