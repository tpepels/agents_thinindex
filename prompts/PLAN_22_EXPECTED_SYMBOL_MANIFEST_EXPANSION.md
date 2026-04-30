# PLAN_22_EXPECTED_SYMBOL_MANIFEST_EXPANSION.md

Use superpowers:subagent-driven-development.

Do not implement this until PLAN_21_CTAG_ALLOWLIST_AND_FORBIDDEN_SURFACE_GATES.md is complete and green.

Goal:
Expand expected-symbol manifest coverage so real-repo quality checks measure symbol capture quality, not only parser survival.

Progress:
- [x] Phase 1: inspect current manifest support and real-repo gates
- [x] Phase 2: add expected-absent manifest model and parser support
- [x] Phase 3: wire expected symbols, patterns, and absence checks into quality gates
- [x] Phase 4: add fixture tests and docs
- [x] Phase 5: update local real-repo manifest entries where practical
- [x] Phase 6: run verification

This pass improves quality manifests and checks. Do not add parser architecture, new languages, packaging, license enforcement, payment behavior, telemetry, cloud behavior, or unrelated product commands.

Product rule:
Real-repo parser quality must be checked through expected symbols and expected patterns, not vague record counts.

Required manifest support:
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

Support expected absence where useful:

[[repo.expected_absent_symbol]]
language = "python"
path = "app/example.py"
name = "NotARealSymbolFromComment"

Required implementation:
1. Ensure manifest parsing supports explicit symbols, patterns, thresholds, and absent symbols.
2. Add targeted real-repo manifest entries for every supported language present in `test_repos/`.
3. Add fixture tests for manifest parsing/matching.
4. Add failure messages that show repo, language, path, kind, name/pattern, and nearby records if useful.
5. Add docs showing how to add expected symbols/patterns.
6. Avoid fragile exact total record counts.
7. Keep matching deterministic.

Quality policy:
- Missing explicit expected symbols fail.
- Missing expected pattern minimums fail.
- Expected absent symbols found fail.
- Comparator-only symbols do not fail unless promoted to expected symbols/patterns.
- Unsupported syntax should be documented, not silently ignored.

Do not:
- make `test_repos/` required for normal tests
- require ctags or any comparator
- weaken parser conformance
- add line-scanner fallback

Acceptance:
- expected-symbol manifest support is broad enough for real repos
- every supported language present in `test_repos/` has at least one expected-symbol or expected-pattern check where practical
- missing symbols produce actionable failures
- normal tests remain deterministic
- existing quality plugin remains isolated

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

Report:
- changed files
- manifest features added
- expected symbols/patterns added by language
- missing-symbol tests added
- verification commands and results
- commit hash
