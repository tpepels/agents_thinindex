Use superpowers:subagent-driven-development.

Build a reusable index-integrity test framework that can run the same checks against both fixture/temp indexes and the developer-local `.dev_index` for this repo.

Goal:
Avoid duplicating index integrity tests. Define reusable checks once, then apply them to:
1. deterministic fixture/temp repo indexes during normal `cargo test`
2. the real local thinindex repo `.dev_index` via ignored tests

Required design:
- Create a shared test support module, probably under `tests/common/mod.rs`.
- Add reusable helpers that operate on raw `index.jsonl` text, not on a specific repo setup.
- Normal tests should generate an index from fixtures/temp repos and pass its `index.jsonl` into the shared checks.
- Local tests should read this repo’s `.dev_index/index.jsonl` and pass it into the same shared checks.
- Local tests must be `#[ignore]` because they depend on developer-local state.
- Do not make normal `cargo test` depend on the current repo’s `.dev_index`.

Required shared checks:
1. No duplicate record locations:
   - duplicate definition is exactly `path + line + col`
   - ignore `kind`, `name`, `source`, and `text` for duplicate detection
2. Required fields exist on every record:
   - `path`
   - `line`
   - `col`
   - `lang`
   - `kind`
   - `name`
   - `text`
   - `source`
3. `.dev_index` is not indexed:
   - no record path should contain `.dev_index/`
4. Optional expected path check:
   - helper should accept a list of expected path substrings and assert at least one record contains each

Implementation shape:
- Add a parsed record helper if useful, e.g.
  - `IndexJsonRecord`
  - `parse_index_jsonl(index: &str) -> Vec<...>`
- Add assertion helpers, e.g.
  - `assert_no_duplicate_locations(index: &str)`
  - `assert_required_fields(index: &str)`
  - `assert_no_dev_index_records(index: &str)`
  - `assert_index_contains_paths(index: &str, expected: &[&str])`
- Failure messages must print useful offending lines.

Normal fixture test:
- In `tests/build_index.rs`, keep or add one test that creates a temp repo with:
  - Markdown headings/sections
  - HTML ids/classes/data attributes
  - at least one code symbol
- Run `build_index`
- Read `.dev_index/index.jsonl`
- Run the shared integrity checks against it
- Keep this test non-ignored

Local repo test:
- Create `tests/local_index.rs`
- Mark tests `#[ignore]`
- Read `env!("CARGO_MANIFEST_DIR")/.dev_index/index.jsonl`
- If missing, fail with a clear message: run `build_index` from repo root first
- Run the same shared integrity checks
- Add expected local paths:
  - `src/indexer.rs`
  - `src/search.rs`
  - `src/bin/wi.rs`
  - `src/bin/wi-init.rs`
  - `src/wi_cli.rs`

Also add or keep a specific dedupe preference test:
- Build a Markdown fixture where ctags emits `section` and extras may emit `heading_*` at the same line/col.
- Assert the duplicate location check passes.
- Assert `wi Tests -l md -v` contains `kind: section`.
- Assert it does not contain `kind: heading_2` for that same heading.
- This test is normal/non-ignored.

Important:
- If implementing dedupe or changing extraction/index contents, increment `INDEX_SCHEMA_VERSION`.
- Do not remove useful fixture coverage.
- Do not add the local `.dev_index` test to normal test requirements.
- Do not duplicate the same assertion logic in multiple files; call shared helpers.

Verification:
- Run `cargo fmt`
- Run `cargo test`
- Run `cargo clippy --all-targets --all-features -- -D warnings`
- Run local ignored tests manually:
  - `build_index`
  - `cargo test --test local_index -- --ignored`

Report:
- changed files
- verification commands and results
- whether local ignored tests passed