# PLAN_28_PARSER_QUERY_MAINTENANCE_GUIDE.md

Use superpowers:subagent-driven-development.

Do not implement this until PLAN_27_REAL_REPO_MANIFEST_CURATION.md is complete and green.

Progress:
- [x] Inspect parser framework, support matrix, current docs, and guardrail surfaces.
- [x] Add query/capture validation guardrails.
- [x] Add parser maintenance guide.
- [x] Add stale-doc and support/license/conformance tests.
- [x] Run required verification.
- [x] Commit with completed plan checkboxes.

Goal:
Create a maintenance guide and guardrails for adding or changing Tree-sitter query specs.

This is a contributor/maintenance hardening pass. Do not add new language support unless needed for examples. Do not add parser architecture, release packaging, license enforcement, payment behavior, telemetry, or cloud behavior.

Product rule:
Query changes must be easy to review and hard to regress.

Required docs:
Add or update:
- `docs/PARSER_MAINTENANCE.md`

Guide sections:
- parser architecture overview
- how LanguageRegistry works
- how query specs work
- normalized capture names
- capture-to-record mapping rules
- how to add a language
- how to update a language query
- how to add conformance fixtures
- how to add real-repo expected symbols
- how to run quality gates
- how to handle unsupported syntax
- how to audit grammar licenses
- what not to do

Forbidden patterns:
Document:
- no line scanners for code symbols
- no hand parsers
- no ctags parser fallback
- no broad regex parser
- no unsupported language support claims
- no grammar dependency without license entry

Guardrails:
Add tests/checks if practical:
- query files/specs use allowed normalized capture names
- every query-backed language has conformance fixture
- every query-backed language has license metadata
- unsupported captures fail tests
- stale docs check if practical

Acceptance:
- parser maintenance guide exists
- query/capture rules are documented
- normalized captures are enforced or test-visible
- contributor workflow is clear
- no behavior regressions

Verification:
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --all-targets --all-features -- -D warnings`
- ctags allowlist gate
- license audit command if configured
- parser/query validation command if added
- `cargo run --bin build_index`
- `cargo test --test local_index -- --ignored`
- `cargo test --test real_repos -- --ignored`

Report:
- changed files
- maintenance guide sections
- guardrails added
- verification commands and results
- commit hash
