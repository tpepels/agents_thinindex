# Quality Plugins

thinindex quality plugins are isolated evaluation tools. They do not participate in production indexing, search, packaging, install, or release artifact creation.

## Boundary

- Production indexing remains Tree-sitter based.
- Comparator records are never written to production `.dev_index/index.sqlite` `records` or `refs`.
- Comparator reports are written only under `.dev_index/quality/`.
- Comparator output is not ground truth. It is a comparison signal alongside thinindex Tree-sitter output and expected-symbol manifests.
- Expected-symbol manifests and conformance fixtures are the stronger quality source when there is disagreement.

## Optional Universal Ctags Comparator

Universal Ctags is optional, external, not bundled, not required, and not used by `build_index`.

When present on a developer machine, the quality adapter can run the local command and normalize its JSON output into comparator records with path, line, optional column, kind, name, optional language, and comparator name. When absent, the comparator is skipped with `skipped: comparator not found`.

Normal tests do not require Universal Ctags. Release packages and installer scripts do not include Universal Ctags.

## Manual Run Shape

The quality layer is library tooling in this phase. A manual or ignored test run should:

1. Build the thinindex SQLite index with `cargo run --bin build_index`.
2. Load thinindex `records`.
3. Run an optional comparator adapter.
4. Compare thinindex records, comparator records, and expected symbols.
5. Write a report under `.dev_index/quality/<comparator>.txt`.

The ignored test `optional_external_ctags_comparator_generates_isolated_quality_report_or_skips` demonstrates this flow. It runs only when explicitly requested and skips cleanly if the optional command is missing.

## Report Contents

Quality reports are compact text files with:

- per-language thinindex record count
- per-language comparator record count
- matched symbols
- thinindex-only symbols
- comparator-only symbols
- expected-symbol pass/fail counts
- unknown comparator kinds
- duplicate record count
- malformed record count
- unsupported extensions

These reports are intended for parser-quality triage and must not be imported into production indexes.

## Report Exports

The quality plugin can also export stable agent-readable summaries under `.dev_index/quality/`:

- `QUALITY_REPORT.md`: compact Markdown summary for humans and agents
- `QUALITY_REPORT.json`: deterministic JSON summary with the same counts and samples
- `QUALITY_REPORT_DETAILS.jsonl`: line-delimited detail records for larger symbol/gap lists

The Markdown and JSON summaries include deterministic mode or a caller-provided timestamp, repo names, the language support matrix, records and refs by language, expected-symbol/pattern/absent checks, comparator-only and thinindex-only counts, parser error counts, unsupported extensions, gap summaries, and cycle-plan summaries when available. Summaries intentionally cap samples so agents do not need to inspect large raw comparator output. Full comparator/gap details go in `QUALITY_REPORT_DETAILS.jsonl`.

By default exports omit machine-specific absolute repo paths. Local workflows may opt into paths for local-only reports, but generated quality exports remain isolated under `.dev_index/quality/` and must not be copied into production SQLite `records` or `refs`.

Commit source code, fixtures, manifests, and docs that explain quality behavior. Keep `.dev_index/quality/` reports local unless a review explicitly asks for a small excerpt or artifact.

## Drift Gates

Quality drift gates evaluate checked-in fixture repositories in normal tests and real repositories under `test_repos/` in ignored tests. They stay in the quality module and read SQLite `records` and `refs`; they do not write comparator data back to production tables.

The normal deterministic gate uses tiny fixtures only. It fails on:

- missing expected symbols
- failing expected-symbol patterns
- found expected-absent symbols
- threshold failures
- duplicate record locations
- duplicate refs
- malformed records or refs
- `.dev_index` paths in records or refs
- `source = "ctags"` production records

Comparator-only symbols are triage data. They should be classified as expected-symbol additions, fixture/conformance additions, accepted comparator false positives, or documented unsupported syntax. They do not automatically fail deterministic gates.

## Comparator Triage

Comparator-only and thinindex-only symbols use an explicit local triage workflow before they become quality gates. Triage reports are written only under `.dev_index/quality/COMPARATOR_TRIAGE.md`; they are never written into production SQLite `records` or `refs`.

Triage states are:

- `open`
- `accepted_expected_symbol`
- `fixture_needed`
- `comparator_false_positive`
- `unsupported_syntax`
- `low_value_noise`
- `fixed`

The report groups comparator-only symbols by language, kind, and path, then lists each item with a promotion action. Use `accepted_expected_symbol` when a comparator-only symbol should become a `[[repo.expected_symbol]]` or `[[repo.expected_symbol_pattern]]`. Use `fixture_needed` when parser conformance should be expanded before changing a manifest. Use `unsupported_syntax` for documented parser gaps, `comparator_false_positive` for external-tool noise, and `low_value_noise` for findings that should stay unpromoted.

Open comparator-only symbols do not fail normal gates. A manual strict triage check may fail while any item remains `open`, but that strict mode must be explicitly requested by the quality workflow.

## Expected Symbols

`test_repos/MANIFEST.toml` is local-only real-repo quality data. Its schema and curation rules are documented in `docs/REAL_REPO_MANIFEST.md`. Active entries must include `name`, `path`, `kind`, `languages`, and `queries`; skipped entries must include `skip_reason`.

It can declare exact expected symbols:

```toml
[[repo.expected_symbol]]
language = "rs"
path = "src/indexer.rs"
kind = "function"
name = "build_index"
```

It can also declare expected symbol patterns:

```toml
[[repo.expected_symbol_pattern]]
language = "ts"
path_glob = "src/**/*.ts"
kind = "function"
name_regex = "^[A-Za-z_].*"
min_count = 20
```

Use expected-absent symbols for known false positives that must not be extracted from comments, strings, generated docs, or unsupported syntax:

```toml
[[repo.expected_absent_symbol]]
language = "py"
path = "app/example.py"
kind = "function"
name = "NotARealSymbolFromComment"
```

Optional quality thresholds are per language:

```toml
[[repo.quality_threshold]]
language = "rs"
min_records = 100
max_duplicate_locations = 0
max_malformed_records = 0
```

Avoid exact total record counts for real repositories. Prefer expected symbols, expected patterns, expected-absent symbols, and coarse minimum thresholds that reflect supported-language coverage. Failure output should include the repo, language, path, kind, expected name or pattern, and nearby records where helpful.

Choose side repos intentionally: cover supported languages and difficult syntax, avoid random clones, keep third-party repos local under ignored `test_repos/`, and add `ignore_guidance` when generated/vendor-heavy paths need local ignore rules. Remove stale repos from the manifest or mark them skipped with a clear `skip_reason`.

## Running Gates

Normal deterministic gates run with:

```sh
cargo test
```

For local CI parity, run:

```sh
scripts/check-ci
```

or:

```sh
make ci-check
```

`scripts/check-ci` runs formatting, normal tests, deterministic parser/quality fixture suites, clippy, license audit, and command smoke checks. It does not run ignored tests, does not require `test_repos/`, does not invoke optional external comparators, and does not need network access beyond whatever a caller already uses to install the Rust toolchain and cargo-deny.

## CI-Safe Gates

The CI-safe quality set is deterministic and source-controlled:

- parser conformance fixtures: `cargo test --test parser_conformance`
- support-level claim checks: `cargo test --test support_levels`
- quality report/export fixtures: `cargo test --test quality`
- expected-symbol and threshold fixtures: `cargo test --test quality_gates`
- ctags allowlist gate: covered by normal `cargo test`
- license audit: `cargo deny check licenses`

These gates use checked-in fixtures or temporary repositories. They must not read local real repos, call optional comparator commands, download side repos, or write comparator output into production SQLite tables.

## Manual-Only Gates

These checks remain ignored/manual because they depend on local repos, optional tools, or a deliberate improvement cycle:

- real-repo parser integrity: `cargo test --test real_repos -- --ignored`
- real-repo quality gates: `cargo test --test quality_gates -- --ignored`
- optional comparator quality report: `cargo test --test quality -- --ignored`
- quality improvement cycle: `cargo test --test quality_loop -- --ignored`
- real-repo benchmarks: `cargo test --test bench_repos -- --ignored`

Keep these out of normal CI unless a future workflow provisions the required local corpus explicitly.

Ignored real-repo gates run with:

```sh
cargo test --test quality_gates -- --ignored
```

If `test_repos/` is missing or empty, the ignored gate prints a clear skip message. If a local optional Universal Ctags command is unavailable, the comparator report is skipped; Universal Ctags remains optional, external, not bundled, not required, and not used by `build_index`.

## Continuous Improvement Loop

The Check -> Plan -> Act quality loop is documented in `docs/QUALITY_LOOP.md`. It turns gate and comparator evidence into local `.dev_index/quality/QUALITY_GAPS.md`, `.dev_index/quality/COMPARATOR_TRIAGE.md`, `.dev_index/quality/QUALITY_CYCLE_01_PLAN.md`, and `.dev_index/quality/QUALITY_CYCLE_01_REPORT.md` files for one bounded fix cycle.

A quality cycle is single-use by design: one execution selects at most 10 gaps, records `automatic_next_cycle_allowed = false`, writes a final report, and stops. Start another cycle only from a later explicit human request.

## Support Levels

Quality gates and reports should interpret parser findings through the support levels in `src/support.rs`, `docs/PARSER_SUPPORT.md`, and the generated dashboard in `docs/LANGUAGE_SUPPORT.md`: `supported`, `experimental`, `blocked`, and `extras-backed`. Supported languages should have passing conformance, license metadata, and real-repo checks where configured. Experimental and blocked entries must not be reported as fully supported, and extras-backed formats must stay distinct from Tree-sitter-backed code-symbol extraction.
