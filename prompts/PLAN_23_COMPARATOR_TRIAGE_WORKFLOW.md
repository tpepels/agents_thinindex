# PLAN_23_COMPARATOR_TRIAGE_WORKFLOW.md

Use superpowers:subagent-driven-development.

Do not implement this until PLAN_22_EXPECTED_SYMBOL_MANIFEST_EXPANSION.md is complete and green.

Goal:
Add a triage workflow for optional comparator-only symbols so comparator reports become actionable without treating external taggers as ground truth.

Progress:
- [x] Phase 1: inspect existing comparator gap and quality report flow
- [x] Phase 2: add isolated comparator triage model and rendering
- [x] Phase 3: add local triage report storage under `.dev_index/quality/`
- [x] Phase 4: add triage state transition tests and docs
- [x] Phase 5: run verification

This pass improves the quality plugin. Do not add parser architecture, new languages, packaging, license enforcement, payment behavior, telemetry, cloud behavior, or unrelated product commands.

Product rule:
Comparator-only symbols are leads. They must be triaged before becoming quality gates.

Triage states:
Use exactly these states:
- open
- accepted_expected_symbol
- fixture_needed
- comparator_false_positive
- unsupported_syntax
- low_value_noise
- fixed

Required implementation:
1. Add a triage model for comparator-only and thinindex-only symbols.
2. Store triage locally under `.dev_index/quality/` or another isolated quality path.
3. Do not write triage data into production index tables.
4. Add report output grouping comparator-only symbols by language/kind/path.
5. Add a workflow to promote comparator-only symbols to:
   - expected symbols
   - expected symbol patterns
   - conformance fixtures
   - documented unsupported cases
6. Add tests for triage state transitions.
7. Add docs explaining how to triage comparator-only symbols.

Hard constraints:
- ctags remains optional external comparator only.
- no normal test requires ctags.
- no production code calls ctags.
- no comparator output enters production records/refs.
- no GPL/AGPL dependency is introduced.

Failure behavior:
- Open comparator-only symbols do not fail normal gates.
- Open comparator-only symbols may fail a manual strict quality mode only if explicitly requested.
- Missing expected symbols fail once promoted.

Acceptance:
- comparator-only symbols have an explicit triage workflow
- triage data is isolated
- promotion to expected symbols/fixtures is supported or documented
- comparator disagreement is not blindly treated as thinindex failure
- existing parser/index/search/quality behavior remains stable

Verification:
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --all-targets --all-features -- -D warnings`
- ctags allowlist gate
- license audit command if configured
- `cargo run --bin build_index`
- `cargo run --bin wi -- build_index`
- `cargo test --test local_index -- --ignored`
- `cargo test --test real_repos -- --ignored`
- optional comparator test if comparator is installed

Report:
- changed files
- triage states/model
- isolated storage path
- sample triage report
- verification commands and results
- commit hash
