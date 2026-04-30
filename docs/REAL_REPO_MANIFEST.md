# Real-Repo Manifest Curation

`test_repos/MANIFEST.toml` is local-only quality data for real-repo parser hardening. It tells ignored tests which local clones to index, which queries to smoke, and which expected symbols or patterns must stay present.

Do not commit third-party repository contents. Keep `test_repos/` ignored and local. Commit only source changes, fixtures, docs, and tests that improve thinindex itself.

## Choosing Repos

Pick repos intentionally:

- Prefer small or medium repositories with clear source layout.
- Cover supported languages and extras-backed formats from `docs/LANGUAGE_SUPPORT.md`.
- Include known difficult syntax only when it has a clear expected symbol, pattern, absent symbol, threshold, or note.
- Prefer repos with stable filenames and declarations over generated-heavy samples.
- Avoid repos dominated by vendor, dependency, generated, minified, build, or lockfile output.

When a useful repo has noisy generated/vendor paths, add local ignore rules in that cloned repo and record `ignore_guidance` in the manifest.

Run `build_index --stats` inside large or noisy local clones before tightening expected symbols. The compact stats report surfaces large skipped files, broad size warnings, and phase timings without writing bulky snapshots.

## Required Repo Fields

Every active repo entry must include:

```toml
[[repo]]
name = "local-project"
path = "local-project"
kind = "rust-cli"
languages = ["rs"]
queries = ["build_index", "Config", "main"]
```

Field meanings:

- `name`: stable label used in reports.
- `path`: directory under `test_repos/`; `.` means the parent project.
- `kind`: category such as `rust-cli`, `python-web`, `typescript-react`, or `fixture-corpus`.
- `languages`: primary language or format ids exercised by the repo.
- `queries`: deterministic smoke queries for `wi`, `wi refs`, `wi pack`, and `wi impact` checks.

Optional repo fields:

- `description`: short human description.
- `expected_paths`: path substrings that must appear in indexed records.
- `expected_symbols`: simple legacy symbol-name checks.
- `expected_symbol_patterns`: simple legacy regex checks.
- `notes`: local curation context, deferred syntax, or unsupported language notes.
- `ignore_guidance`: generated/vendor/minified paths that should be ignored locally if they dominate output.
- `skip = true`: keep an entry documented without requiring the clone.
- `skip_reason`: required when `skip = true`.

Skipped repos are not loaded and their path does not need to exist. Non-skipped repos must exist; ignored real-repo tests fail clearly when a manifest-listed active repo is missing.

## Expected Symbols

Use exact expected symbols for important declarations:

```toml
[[repo.expected_symbol]]
language = "rs"
path = "src/indexer.rs"
kind = "function"
name = "build_index"
```

Use expected patterns for broader coverage:

```toml
[[repo.expected_symbol_pattern]]
language = "ts"
path_glob = "src/**/*.ts"
kind = "function"
name_regex = "^[A-Za-z_].*"
min_count = 20
```

Use expected-absent symbols for false positives that must not be extracted:

```toml
[[repo.expected_absent_symbol]]
language = "py"
path = "app/example.py"
kind = "function"
name = "NotARealSymbolFromComment"
```

Use quality thresholds sparingly:

```toml
[[repo.quality_threshold]]
language = "rs"
min_records = 100
max_duplicate_locations = 0
max_malformed_records = 0
```

Avoid exact total record counts for real repositories. Prefer expected symbols, expected patterns, expected-absent symbols, and coarse thresholds tied to supported-language coverage.

## Removing Stale Repos

Remove or skip stale entries when a local clone is gone, renamed, too noisy, or no longer useful:

```toml
[[repo]]
name = "old-project"
path = "old-project"
skip = true
skip_reason = "removed from local corpus; replacement repo covers the same syntax"
```

If an active repo remains in the manifest, keep its expected symbols and patterns current. Do not leave a non-skipped missing path in the manifest.

## Validation

Normal tests use temporary fixture manifests and do not require `test_repos/`. Ignored/manual checks validate local real repos:

```sh
cargo test --test real_repos -- --ignored
cargo test --test quality_gates -- --ignored
cargo test --test quality_loop -- --ignored
```

Malformed manifests fail during loading. Active entries missing `name`, `path`, `kind`, `languages`, or non-empty `queries` fail clearly. Nested expected-symbol sections fail if required fields such as `name`, `name_regex`, `min_count`, or threshold `language` are missing.
