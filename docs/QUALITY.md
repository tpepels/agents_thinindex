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

## Expected Symbols

`test_repos/MANIFEST.toml` can declare exact expected symbols:

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

## Running Gates

Normal deterministic gates run with:

```sh
cargo test
```

Ignored real-repo gates run with:

```sh
cargo test --test quality_gates -- --ignored
```

If `test_repos/` is missing or empty, the ignored gate prints a clear skip message. If a local optional Universal Ctags command is unavailable, the comparator report is skipped; Universal Ctags remains optional, external, not bundled, not required, and not used by `build_index`.

## Continuous Improvement Loop

The Check -> Plan -> Act quality loop is documented in `docs/QUALITY_LOOP.md`. It turns gate and comparator evidence into local `.dev_index/quality/QUALITY_GAPS.md` and `.dev_index/quality/QUALITY_CYCLE_01_PLAN.md` files for one bounded fix cycle.

## Support Levels

Quality gates and reports should interpret parser findings through the support levels in `src/support.rs` and `docs/PARSER_SUPPORT.md`: `supported`, `experimental`, `blocked`, and `extras-backed`. Supported languages should have passing conformance, license metadata, and real-repo checks where configured. Experimental and blocked entries must not be reported as fully supported, and extras-backed formats must stay distinct from Tree-sitter-backed code-symbol extraction.
