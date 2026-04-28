Use superpowers:subagent-driven-development.

Build a reusable index-integrity test framework that applies the same shared integrity checks to:
1. deterministic fixture/temp repo indexes during normal `cargo test`
2. the thinindex repo itself during an ignored manual test
3. real downloaded repos under `test_repos/` during an ignored manual test

Goal:
Define index integrity checks once. Fixture, local, and real-repo tests may have separate small harnesses, but all integrity assertions must call the same shared check functions. Do not duplicate duplicate-location, required-field, type-validation, or `.dev_index` assertion logic.

Required design:
- Create shared test support in `tests/common/mod.rs` or a dedicated common submodule.
- Shared checks must operate on raw `index.jsonl` text plus a repo/check name.
- Expose one main shared suite function:
  - `run_named_index_integrity_checks(name: &str, index: &str, expected_paths: &[&str])`
- Optional smaller helpers may exist, but all harnesses must call the shared suite.
- Failure messages must include the repo/check name and useful offending JSON lines.

Shared checks, written once:
1. Parse every non-empty line as JSON.
2. Validate required fields exist and have expected types:
   - `path`: string, non-empty
   - `line`: integer, `>= 1`
   - `col`: integer, `>= 1`
   - `lang`: string
   - `kind`: string, non-empty
   - `name`: string
   - `text`: string
   - `source`: string, non-empty
3. No duplicate record locations:
   - duplicate definition is exactly parsed `path + line + col`
   - ignore `kind`, `name`, `source`, and `text`
4. `.dev_index` is not indexed:
   - check only the parsed `path` field
   - do not search raw JSON text, because `text` may legitimately mention `.dev_index`
5. Optional expected path check:
   - if `expected_paths` is non-empty, assert each expected path substring appears in at least one parsed `path`

Implementation shape:
- Add a parsed record helper if useful:
  - `IndexJsonRecord`
  - `parse_index_jsonl(name: &str, index: &str) -> Vec<...>`
- Add assertion helpers if useful, but expose and use:
  - `run_named_index_integrity_checks(name: &str, index: &str, expected_paths: &[&str])`

Normal fixture test:
- In `tests/build_index.rs`, keep or add one non-ignored test that creates a temp repo with:
  - Markdown headings/sections
  - HTML ids/classes/data attributes
  - CSS class/variable records
  - at least one code symbol
- Run `build_index`
- Read `.dev_index/index.jsonl`
- Call `run_named_index_integrity_checks("fixture index integrity", &index, expected_paths)`
- This test remains part of normal `cargo test`

Thinindex local repo test:
- Keep/create `tests/local_index.rs`
- Use exactly one ignored test for local repo integrity.
- Do not create several ignored tests that each rebuild the same `.dev_index`; avoid parallel-test races.
- The ignored test must:
  - remove `env!("CARGO_MANIFEST_DIR")/.dev_index`
  - call `thinindex::indexer::build_index(env!("CARGO_MANIFEST_DIR"))`
  - read rebuilt `.dev_index/index.jsonl`
  - call `run_named_index_integrity_checks("thinindex local repo", &index, expected_paths)`
- Expected local paths:
  - `src/indexer.rs`
  - `src/search.rs`
  - `src/bin/wi.rs`
  - `src/bin/wi-init.rs`
  - `src/wi_cli.rs`
- This test intentionally mutates/replaces the local `.dev_index`.

Real downloaded repo tests:
- Create `tests/real_repos.rs`
- Use exactly one ignored test that loops through repos under `test_repos/`.
- Use root directory `test_repos/`.
- If `test_repos/` is missing, print `skipped: test_repos/ missing` and return successfully.
- If `test_repos/` exists but no repo directories are found, print `skipped: test_repos/ has no repo directories` and return successfully.
- Accept immediate child directories as repo roots if they contain `.git` OR at least one recognizable project marker:
  - `Cargo.toml`
  - `package.json`
  - `pyproject.toml`
  - `go.mod`
  - `.gitignore`
  - `src/`
- For each accepted repo:
  - delete that repo’s `.dev_index`
  - call `thinindex::indexer::build_index(repo_path)`
  - read `repo_path/.dev_index/index.jsonl`
  - call `run_named_index_integrity_checks(repo_name, &index, &[])`
- Print the list of repos tested.
- Do not hardcode expected paths for arbitrary real repos unless a per-repo manifest/config is introduced.
- This test must remain ignored/manual and must not affect normal `cargo test`.

Repo hygiene:
- Add `test_repos/` to `.gitignore` if not already ignored.
- Add `test_repos/` to `.thinindexignore` if the thinindex repo should not index downloaded external repos as part of its own local index.
- Do not commit downloaded third-party repos.

Behavioral regression test:
- Keep or add a normal non-ignored test proving Markdown heading aliases are canonicalized.
- This is separate from the generic integrity framework, though it may also call the shared integrity suite.
- It should assert:
  - `wi Tests -l md -v` contains `kind: section`
  - it does not contain `kind: heading_2` for that same heading
- Keep the duplicate-location invariant unchanged.

Important:
- If changing extraction/index contents, increment `INDEX_SCHEMA_VERSION`.
- Do not remove useful fixture coverage.
- Do not duplicate assertion logic across fixture/local/real-repo harnesses.
- Do not weaken the duplicate-location invariant.
- Normal `cargo test` must not require `test_repos/` or a pre-existing developer-local `.dev_index`.

Verification:
- Run `cargo fmt`
- Run `cargo test`
- Run `cargo clippy --all-targets --all-features -- -D warnings`
- Run ignored local repo test manually:
  - `cargo test --test local_index -- --ignored`
- Run ignored real repo test manually:
  - `cargo test --test real_repos -- --ignored`

Report:
- changed files
- verification commands and results
- whether the local ignored test passed
- whether the real-repo ignored test ran, skipped, or failed
- list of repos tested under `test_repos/`, if any