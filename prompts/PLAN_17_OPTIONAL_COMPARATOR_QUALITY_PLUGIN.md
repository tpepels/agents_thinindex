# PLAN_17_OPTIONAL_COMPARATOR_QUALITY_PLUGIN.md

Use superpowers:subagent-driven-development.

Do not implement this until PLAN_00 through PLAN_16 are complete and green.

Goal:
Add an isolated optional index-quality comparator plugin system that can compare thinindex output against external taggers such as Universal Ctags without making those tools product/runtime/build/package dependencies.

This is quality tooling only. It must stay isolated from production indexing.

Progress:
- [x] Add isolated quality comparator framework and optional external adapter.
- [x] Add quality report comparison, matching, metrics, and isolated report output.
- [x] Add normal and ignored quality tests.
- [x] Add quality documentation and ctags boundary checks.
- [x] Run required verification.
- [x] Commit completed Plan 17 changes.

Product rule:
External comparators are QA plugins, not parser backends. thinindex production indexing remains Tree-sitter based.

Isolation rule:
Implement comparator quality tooling in a clearly separated module/path.

Preferred structure:
- `src/quality/`
- `src/quality/mod.rs`
- `src/quality/comparator.rs`
- `src/quality/report.rs`
- `src/quality/manifest.rs`
- `tests/quality/` or `tests/quality_*.rs`
- `docs/QUALITY.md`

Do not mix comparator code into:
- `src/indexer.rs`
- Tree-sitter parser modules
- production search path
- production SQLite index write path
- install/release packaging scripts except explicit exclusion checks

Hard requirements:
- Do not reintroduce ctags as a parser backend.
- Do not call ctags from `build_index`.
- Do not bundle ctags.
- Do not require ctags for normal tests.
- Do not require ctags for install, build, release, packaging, or runtime operation.
- Do not emit `source = "ctags"` in production `.dev_index/index.sqlite` records.
- Comparator outputs must never be inserted into production `records` or `refs`.
- Comparator output may be written only to isolated report locations, for example `.dev_index/quality/`.
- Do not add GPL/AGPL dependencies.
- Do not add telemetry, network calls, payment behavior, or license enforcement.

Plugin boundary:
Define a comparator adapter interface, for example:
- `QualityComparator`
- `ComparatorRecord`
- `ComparatorRun`
- `QualityReport`

Comparator adapters must be optional.

Initial optional comparator:
- Universal Ctags may be supported only as an external local command if installed.
- If unavailable, report `skipped: comparator not found`.
- No normal test may require it.

Allowed ctags mentions:
Only allowed in:
- quality comparator plugin code/docs
- optional comparator tests that skip if unavailable
- docs explicitly saying ctags is optional, external, not bundled, not required, and not used by production indexing

Forbidden ctags mentions:
- production parser/indexer code
- install requirement docs
- packaging inclusion docs
- release artifact contents
- production source records
- normal test requirements

Quality comparison model:
Do not treat external comparators as ground truth.

Compare three sources:
1. thinindex Tree-sitter output
2. optional comparator output
3. expected-symbol manifests/conformance fixtures

Comparator record normalization:
Normalize external comparator output to:
- path
- line
- column if available
- kind
- name
- language if available
- comparator name

Matching:
Use conservative matching:
- normalized path
- symbol name
- compatible kind where mapping exists
- line proximity if exact line differs
- do not require exact column if comparator lacks column data

Metrics:
Report per repo/language:
- thinindex record count
- comparator record count
- matched symbols
- thinindex-only symbols
- comparator-only symbols
- expected-symbol pass/fail count
- unknown comparator kinds
- duplicate record count
- malformed record count
- unsupported extensions

Real repo input:
Use `test_repos/`.

Behavior:
- missing `test_repos/` -> skip clearly
- empty `test_repos/` -> skip clearly
- if `test_repos/MANIFEST.toml` exists, use it
- do not download repos
- do not commit third-party repos

Tests:
Normal tests:
- fake comparator adapter works
- comparator record parsing works
- kind mapping works
- matching logic works
- report includes comparator-only and thinindex-only symbols
- missing optional comparator skips cleanly
- no normal test requires ctags
- comparator output is not written to production index tables

Ignored/manual tests:
- optional external ctags comparator runs only if installed
- report is generated under isolated quality output path
- missing comparator skips, not fails
- malformed thinindex records still fail quality checks

Docs:
Add `docs/QUALITY.md`.

Must document:
- quality plugins are isolated from production indexing
- ctags is optional external comparator only
- ctags is not bundled, not required, not used by `build_index`
- comparator output is not ground truth
- expected-symbol manifests are stronger quality source
- how to run quality comparison manually
- where reports are written

Acceptance:
- quality comparator code is isolated under quality-specific modules/tests/docs
- optional comparator framework exists
- normal tests do not require ctags or external tools
- comparator reports compare thinindex/comparator/expected-symbol data
- ctags is not reintroduced as runtime/build/package dependency
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
- `cargo test --test local_index -- --ignored`
- `cargo test --test real_repos -- --ignored`
- optional comparator ignored/manual test if added

Report:
- changed files
- quality module/plugin boundary
- comparator adapters added
- allowed ctags references
- forbidden ctags references checked
- sample quality report
- verification commands and results
- commit hash
