# PLAN_30_QUALITY_SYSTEM_FINAL_AUDIT.md

Use superpowers:subagent-driven-development.

Do not implement this until PLAN_29_QUALITY_PLUGIN_CI_READINESS.md is complete and green.

Progress:
- [x] Inspect parser, quality, support, ctags, license, docs, release, and real-repo guardrails.
- [x] Add final parser-quality audit documentation.
- [x] Tighten docs/release checklist links for CI-safe quality gates.
- [x] Add final audit consistency tests.
- [x] Run required verification.
- [x] Commit with completed plan checkboxes.

Goal:
Audit the full parser quality system and remove drift between parser framework, support claims, quality plugin, docs, and release readiness.

This is a final audit/cleanup pass for the parser-quality track. Do not add new architecture or major product features unless required to fix a found inconsistency.

Product rule:
The parser-quality system must be coherent: claims, tests, reports, docs, and release gates must all agree.

Audit areas:
1. Parser framework:
   - no ctags parser dependency
   - no hand parser/line scanner for code symbols
   - Tree-sitter framework is source of code-symbol extraction
   - accepted extras are documented

2. Language support:
   - support matrix exists
   - support levels are accurate
   - every supported language has conformance/license/docs
   - blocked/experimental languages are not overclaimed

3. Quality plugin:
   - isolated under quality-specific modules/docs/tests
   - optional comparator does not pollute production
   - expected-symbol gates work
   - triage workflow exists
   - one-cycle runner is bounded

4. Real repos:
   - manifest schema documented
   - ignored tests run or skip clearly
   - expected symbols/patterns are used where practical
   - third-party repos are not committed

5. Ctags:
   - structural allowlist gate works
   - ctags appears only in allowed quality/comparator contexts
   - no production records use `source = "ctags"`
   - release/install docs do not require ctags

6. Licenses:
   - parser/grammar dependencies listed
   - license audit command documented
   - GPL/AGPL not introduced

7. Docs:
   - README, QUALITY docs, parser support docs, product boundary, roadmap, release docs agree
   - no stale claims about JSONL canonical storage, WI.md, ctags, or unsupported languages

Required implementation:
- Fix inconsistencies found by the audit.
- Add missing tests for discovered gaps.
- Remove stale docs.
- Tighten support-level claims.
- Tighten quality-plugin isolation if needed.
- Do not hide uncertainty.

Acceptance:
- parser quality system is internally consistent
- docs and tests agree with implementation
- ctags remains isolated as optional comparator only
- language support claims are evidence-backed
- release/package plans can rely on current quality/license state
- no major known drift remains

Verification:
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --all-targets --all-features -- -D warnings`
- ctags allowlist gate
- license audit command if configured
- quality fixture gate/report command if added
- `cargo run --bin build_index`
- `cargo run --bin wi -- build_index`
- `cargo run --bin wi -- refs build_index`
- `cargo run --bin wi -- pack build_index`
- `cargo run --bin wi -- impact build_index`
- `cargo run --bin wi-stats`
- `cargo test --test local_index -- --ignored`
- `cargo test --test real_repos -- --ignored`

Report:
- changed files
- audit findings
- fixes applied
- remaining caveats
- verification commands and results
- commit hash
