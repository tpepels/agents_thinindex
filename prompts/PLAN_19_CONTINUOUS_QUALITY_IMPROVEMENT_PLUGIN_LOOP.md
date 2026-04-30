# PLAN_19_CONTINUOUS_QUALITY_IMPROVEMENT_PLUGIN_LOOP.md

Use superpowers:subagent-driven-development.

Do not implement this until PLAN_18_INDEX_QUALITY_DRIFT_PLUGIN_GATES.md is complete and green.

Goal:
Add an isolated Check → Plan → Act quality-improvement loop that repeatedly improves parser/index quality using fixture, real-repo, expected-symbol, and optional comparator evidence.

This is a bounded quality workflow, not an endless vague loop.

Progress:
- [x] Add quality gap and cycle plan model.
- [x] Add gap collection from quality gate and comparator evidence.
- [x] Add deterministic report and bounded plan rendering/writing.
- [x] Add normal and ignored quality loop tests.
- [x] Add quality loop documentation.
- [x] Run required verification.
- [x] Commit completed Plan 19 changes.

Product rule:
Quality improvement must be evidence-driven. Every parser/index improvement should come from a failing check, missed expected symbol, real-repo gap, or comparator insight that becomes a fixture or manifest expectation.

Isolation rule:
Keep the quality loop inside the quality plugin layer.

Preferred structure:
- `src/quality/`
- `tests/quality_*.rs`
- `docs/QUALITY.md`
- `docs/QUALITY_LOOP.md`
- generated/local reports under `.dev_index/quality/`

Do not put loop logic into:
- production parser/indexer code except actual fixes driven by the loop
- install/release packaging scripts
- AGENTS/CLAUDE generation
- production SQLite records/refs

Hard requirements:
- Do not make ctags required.
- Do not call ctags from `build_index`.
- Do not bundle ctags.
- Do not emit `source = "ctags"` in production records.
- Do not add GPL/AGPL dependencies.
- Do not add telemetry, network calls, payment behavior, or license enforcement.
- Do not use line-oriented fallback parsers for code symbols.
- Do not weaken existing quality gates.

Cycle model:
Implement/document a bounded Check → Plan → Act cycle.

Check:
- run normal quality gate
- run ignored real-repo quality gate when `test_repos/` exists
- run optional comparator report when comparator is available
- collect missing expected symbols
- collect failing expected patterns
- collect parser panics/errors
- collect supported-language zero-record cases
- collect comparator-only symbols
- collect noisiest/slowest files if performance data exists

Plan:
- write or update a local quality gap report, for example `.dev_index/quality/QUALITY_GAPS.md`
- group gaps by language, syntax construct, severity, and evidence source
- select a bounded batch of fixable gaps
- generate a targeted cycle plan, for example `.dev_index/quality/QUALITY_CYCLE_01_PLAN.md`

Act:
- implement selected parser/query/fixture/manifest fixes
- add conformance fixtures for each fixed parser miss where practical
- add expected-symbol manifest entries for important real-repo symbols where practical
- rerun gates
- commit fixes

Cycle limits:
- run exactly one quality cycle per execution
- do not automatically start a second cycle
- max 10 gaps per cycle by default
- prioritize high-impact/high-confidence parser misses
- prefer supported-language missed symbols over comparator noise
- do not mix unrelated architecture changes
- stop when remaining gaps are documented unsupported cases, comparator false positives, or low-value noise

Quality target:
Do not use the word “perfect” in docs or output.

Use measurable closure criteria:
- no missing manifest expected symbols
- no failing expected symbol patterns
- no duplicate locations
- no malformed records/refs
- no parser panics
- no supported language emits zero useful records on real files without documented reason
- comparator-only symbols are triaged
- every parser fix gets a regression fixture where practical
- quality report improves or remains stable between runs

Optional command:
Add a command only if it fits cleanly and remains isolated, for example:
- `wi quality check`
- `wi quality report`
- `wi quality plan`

If adding commands:
- keep them clearly under a quality namespace
- do not pollute normal `wi <term>` behavior
- do not require ctags
- update `wi --help`

If adding commands is too invasive, implement scripts/tests/docs only.

Reports:
Quality reports should include:
- gap id
- repo/path
- language
- symbol/kind/pattern
- evidence source
- severity
- suggested fix type
- status: open/fixed/unsupported/false-positive
- fixture added: yes/no
- manifest added: yes/no

Docs:
Add/update `docs/QUALITY_LOOP.md`:
- how to run check
- how to read gaps
- how to select a cycle
- how to add fixtures
- how to add manifest expected symbols
- how to treat comparator-only symbols
- how to stop a cycle
- what remains unsupported

Tests:
Normal tests:
- quality gap model
- gap grouping
- cycle plan generation from fake report data
- triage status handling
- deterministic output ordering
- no ctags requirement
- no production DB pollution

Ignored/manual tests:
- full quality loop check/report against `test_repos/`
- optional comparator integration if comparator exists

Acceptance:
- quality improvement loop is isolated
- Check → Plan → Act workflow exists
- quality gaps are reported with actionable evidence
- cycle plans are bounded
- fixed parser misses require regression fixtures where practical
- comparator-only symbols are triaged, not blindly accepted
- no ctags runtime/build/package dependency is introduced
- production indexes never contain `source = "ctags"`
- existing parser/index/search/refs/pack/impact/bench/stats/wi-init behavior remains stable

Verification:
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --all-targets --all-features -- -D warnings`
- license audit command if configured
- targeted ctags gate:
  - verify ctags appears only in quality comparator docs/code/tests or explicit “not bundled/not required” docs
- `cargo run --bin build_index`
- `cargo run --bin wi -- build_index`
- `cargo run --bin wi -- refs build_index`
- `cargo run --bin wi -- pack build_index`
- `cargo run --bin wi -- impact build_index`
- `cargo run --bin wi-stats`
- quality command/script if added
- `cargo test --test local_index -- --ignored`
- `cargo test --test real_repos -- --ignored`
- optional comparator ignored/manual test if comparator is installed

Report:
- changed files
- quality loop boundary
- Check → Plan → Act workflow shape
- gap report format
- cycle plan format
- tests added
- sample quality gap report
- verification commands and results
- commit hash
