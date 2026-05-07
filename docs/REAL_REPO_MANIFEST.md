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

## Readiness Classification

Classify each local corpus entry before treating it as support evidence:

- `hardened`: active repo with stable `queries` plus exact expected symbols, expected patterns, expected-absent symbols, or thresholds that exercise the claimed language/format.
- `exploratory`: active repo with useful local coverage but no stable expected-symbol checks yet. Keep a `notes` field explaining what syntax or product risk it explores before promoting findings.
- `skipped`: documented repo that is absent, stale, too noisy, duplicated by another corpus, or intentionally deferred. Set `skip = true` and `skip_reason`.
- `out of scope`: local third-party clone under `test_repos/` that is not part of the manifest. Do not use it as support evidence until it is added, skipped, or documented by a scoped change.

Supported languages can remain supported when fixture and conformance evidence is valid, even if local real-repo evidence is still incomplete. Record that separately as a real-repo hardening gap. Current known examples are Go and PHP: both are fixture-backed supported languages, but this checkout does not currently have a Go-heavy or PHP-heavy manifest target.

## Committed Synthetic Evidence

Normal tests also include committed synthetic corpora for high-risk evidence
that should not depend on third-party local clones. The current stable corpus is
`tests/fixtures/synthetic_real_repo/`. It covers Go, PHP, TypeScript
import/export references, resolved file references, expected symbols,
expected-absent symbols, and `wi`/`wi refs`/`wi pack`/`wi impact` smoke queries
through the normal `cargo test` path.

Treat this synthetic evidence as stronger than ad hoc fixtures but narrower than
real third-party repository evidence. It proves selected product slices are
stable in CI; it does not replace ignored local `test_repos/` checks for
broader syntax diversity. Support docs should distinguish:

- conformance fixtures: one-file parser and extras checks;
- committed synthetic evidence: stable mini-repo product slices in normal tests;
- local real-repo evidence: ignored/manual checks over uncommitted
  `test_repos/MANIFEST.toml` entries.

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
cargo test --test real_repos -- --ignored --nocapture
cargo test --test quality_gates -- --ignored
cargo test --test quality_loop -- --ignored
```

Use `--nocapture` for the real-repo check when running it interactively. The test prints the selected manifest repos, per-repo parser coverage, expected-symbol counts, unsupported extension gaps, and aggregate coverage, but Cargo hides that progress without `--nocapture` on passing tests. A full local corpus can take several minutes because the test removes each repo's `.dev_index/` and rebuilds from scratch.

Malformed manifests fail during loading. Active entries missing `name`, `path`, `kind`, `languages`, or non-empty `queries` fail clearly. Nested expected-symbol sections fail if required fields such as `name`, `name_regex`, `min_count`, or threshold `language` are missing.
