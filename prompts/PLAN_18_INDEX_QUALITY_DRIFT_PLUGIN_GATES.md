# PLAN_18_INDEX_QUALITY_DRIFT_PLUGIN_GATES.md

Use superpowers:subagent-driven-development.

Do not implement this until PLAN_17_OPTIONAL_COMPARATOR_QUALITY_PLUGIN.md is complete and green.

Goal:
Add isolated quality-plugin drift gates so parser/index quality cannot silently regress.

This extends the quality plugin layer from PLAN_17. It must remain separate from production indexing and release packaging.

Progress:
- [x] Add quality-only drift gate model and deterministic report rendering.
- [x] Extend manifest support for expected quality thresholds.
- [x] Add normal deterministic gate tests.
- [x] Add ignored real-repo gate coverage with optional comparator report.
- [x] Update quality docs.
- [x] Run required verification.
- [x] Commit completed Plan 18 changes.

Product rule:
Quality gates should protect expected symbols, parser coverage, and comparator insights without turning optional comparators into dependencies.

Isolation rule:
Keep drift-gate code under quality-specific modules/tests/docs.

Preferred structure:
- `src/quality/`
- `tests/quality_*.rs`
- `docs/QUALITY.md`
- optional `docs/QUALITY_GATES.md`

Do not put drift-gate logic into:
- production index write path
- Tree-sitter parser extraction path, except reusable read-only helpers
- package/install scripts except exclusion checks
- AGENTS/CLAUDE instruction generation

Hard requirements:
- Do not make ctags a required dependency.
- Do not call ctags from `build_index`.
- Do not bundle ctags.
- Do not emit `source = "ctags"` in production records.
- Do not insert comparator output into production SQLite records/refs.
- Do not add GPL/AGPL dependencies.
- Do not add telemetry, network calls, payment behavior, or license enforcement.

Gate levels:
Implement two quality gate levels.

1. Normal deterministic gate:
- runs in normal `cargo test`
- uses fixtures and checked-in tiny repos only
- no ctags
- no external comparator
- no `test_repos/`
- no network
- fails on expected-symbol loss, duplicate records, malformed records, malformed refs, `.dev_index` indexing, or `source = "ctags"`

2. Ignored/manual real-repo gate:
- runs against `test_repos/`
- uses `test_repos/MANIFEST.toml` if present
- includes expected symbols/patterns
- includes optional comparator report if available
- prints quality report
- fails on declared supported-language expected-symbol loss or integrity failures

Optional comparator behavior:
- Comparator-only symbols do not automatically fail.
- Comparator-only symbols must be triaged into:
  - expected-symbol additions
  - fixture/conformance additions
  - accepted comparator false positives
  - documented unsupported syntax

Expected-symbol manifest:
Support explicit expected symbols:

[[repo.expected_symbol]]
language = "rust"
path = "src/indexer.rs"
kind = "function"
name = "build_index"

Support expected symbol patterns:

[[repo.expected_symbol_pattern]]
language = "typescript"
path_glob = "src/**/*.ts"
kind = "function"
name_regex = "^[A-Za-z_].*"
min_count = 20

Support optional thresholds:

[[repo.quality_threshold]]
language = "rust"
min_records = 100
max_duplicate_locations = 0
max_malformed_records = 0

Avoid fragile exact total record counts unless fixture repos are tiny and deterministic.

Quality report:
Generate or print compact deterministic reports:
- repo name/path
- languages checked
- expected symbols checked
- expected symbols missing
- expected patterns checked
- expected patterns failing
- records by language
- refs by language
- duplicates
- malformed records
- unsupported extensions
- comparator-only symbols if comparator available
- thinindex-only symbols if comparator available

Tests:
Normal tests:
- expected-symbol manifest parser
- expected-symbol matching
- expected-symbol-pattern matching
- threshold matching
- report ordering is deterministic
- missing expected symbol message is actionable
- comparator-only symbol does not fail normal deterministic gate
- normal tests do not require ctags or `test_repos/`
- production DB never receives comparator records

Ignored/manual tests:
- real-repo quality gate
- optional comparator quality report if comparator exists
- expected-symbol checks against `test_repos/MANIFEST.toml`

Docs:
Update quality docs:
- how quality gates work
- how to add expected symbols
- how to add expected symbol patterns
- how to interpret comparator-only/thinindex-only symbols
- how to run ignored real-repo quality gates
- ctags remains optional external comparator only

Acceptance:
- normal quality drift gate exists and is isolated
- ignored real-repo quality gate exists
- expected-symbol and pattern checks are supported
- comparator insight remains optional
- failure messages identify missing symbols clearly
- parser quality cannot silently regress in fixtures
- real-repo quality regressions are visible and actionable
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
- `cargo test --test local_index -- --ignored`
- `cargo test --test real_repos -- --ignored`
- optional comparator ignored/manual test if comparator is installed

Report:
- changed files
- quality plugin drift-gate boundary
- expected-symbol manifest support
- normal quality gate behavior
- ignored real-repo quality gate behavior
- comparator integration behavior
- sample drift report
- verification commands and results
- commit hash
