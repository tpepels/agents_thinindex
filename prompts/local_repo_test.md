Use superpowers:subagent-driven-development.

Build and maintain a reusable SQLite index-integrity test framework that applies the same shared integrity checks to:
1. deterministic fixture/temp repo indexes during normal `cargo test`
2. the thinindex repo itself during an ignored manual test
3. real downloaded repos under `test_repos/` during an ignored manual test

Status:
This guidance has been updated for the SQLite storage architecture. It is not an active numbered plan file.

Goal:
Define index integrity checks once. Fixture, local, and real-repo tests may have separate small harnesses, but all integrity assertions must call the same shared check functions. Do not duplicate duplicate-location, required-field, source-validation, `.dev_index`, ref, or dependency assertion logic.

Required design:
- Use shared test support in `tests/common/mod.rs`.
- Shared checks must operate on records loaded from `.dev_index/index.sqlite`, not raw JSONL text.
- Expose one main shared suite function:
  - `run_named_index_integrity_checks(name: &str, snapshot: &IndexSnapshot, expected_paths: &[&str])`
- Load snapshots with:
  - `load_index_snapshot_from_sqlite(root: &Path) -> IndexSnapshot`
- Optional smaller helpers may exist, but all harnesses must call the shared suite.
- Failure messages must include the repo/check name and useful offending record/ref/dependency data.

Shared checks, written once:
1. Load SQLite records, refs, and dependency edges through store helpers.
2. Validate required record fields:
   - `path`: string, non-empty
   - `line`: integer, `>= 1`
   - `col`: integer, `>= 1`
   - `kind`: string, non-empty
   - `source`: string, non-empty
3. No duplicate record locations:
   - duplicate definition is exactly `path + line + col`
   - ignore `kind`, `name`, `source`, and `text`
4. `.dev_index` is not indexed:
   - check only structured path fields
   - do not search raw text, because evidence text may legitimately mention `.dev_index`
5. Optional expected path check:
   - if `expected_paths` is non-empty, assert each expected path substring appears in at least one record path
6. Validate ref fields, allowed ref kinds/confidence values, duplicate refs, and `.dev_index` ref paths.
7. Validate dependency fields, confidence values, duplicate dependencies, and `.dev_index` dependency paths.
8. Assert forbidden production index sources, including `source = "ctags"`, are absent.

Implementation shape:
- Keep the parsed snapshot helper:
  - `IndexSnapshot`
  - `load_index_snapshot_from_sqlite(root: &Path) -> IndexSnapshot`
- Expose and use:
  - `run_named_index_integrity_checks(name: &str, snapshot: &IndexSnapshot, expected_paths: &[&str])`

Normal fixture test:
- In `tests/build_index.rs`, keep one or more non-ignored tests that create or copy fixture repos with:
  - Markdown headings/sections
  - HTML ids/classes/data attributes
  - CSS class/variable records
  - at least one code symbol
- Run `build_index`
- Load `.dev_index/index.sqlite` through `load_index_snapshot_from_sqlite`
- Call `run_named_index_integrity_checks("fixture index integrity", &snapshot, expected_paths)`
- These tests remain part of normal `cargo test`

Thinindex local repo test:
- Keep `tests/local_index.rs`.
- Use exactly one ignored test for local repo integrity.
- Do not create several ignored tests that each rebuild the same `.dev_index`; avoid parallel-test races.
- The ignored test must:
  - remove `env!("CARGO_MANIFEST_DIR")/.dev_index`
  - call `thinindex::indexer::build_index(env!("CARGO_MANIFEST_DIR"))`
  - load rebuilt `.dev_index/index.sqlite`
  - call `run_named_index_integrity_checks("thinindex local repo", &snapshot, expected_paths)`
- Expected local paths should include current source paths that are stable and meaningful.
- This test intentionally mutates/replaces the local `.dev_index`.

Real downloaded repo tests:
- Keep `tests/real_repos.rs`.
- Use exactly one ignored test that loops through repos under `test_repos/`.
- Use root directory `test_repos/`.
- If `test_repos/` is missing, print `skipped: test_repos/ missing` and return successfully.
- If `test_repos/` exists but no repo directories are found, print `skipped: test_repos/ has no repo directories` and return successfully.
- Prefer `test_repos/MANIFEST.toml` when present.
- For each accepted repo:
  - delete that repo's `.dev_index`
  - call `thinindex::indexer::build_index(repo_path)`
  - load `repo_path/.dev_index/index.sqlite`
  - call `run_named_index_integrity_checks(repo_name, &snapshot, expected_paths)`
- Print the list of repos tested.
- Do not hardcode expected paths for arbitrary real repos unless the manifest/config declares them.
- This test must remain ignored/manual and must not affect normal `cargo test`.

Repo hygiene:
- Keep `test_repos/` ignored by git.
- Keep `test_repos/` out of the thinindex repo index through ignore rules.
- Do not commit downloaded third-party repos.

Behavioral regression test:
- Keep normal non-ignored tests proving Markdown heading aliases are canonicalized.
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
- Do not reintroduce JSONL as canonical storage.

Verification:
- Run `cargo fmt --check`
- Run `cargo test`
- Run `cargo clippy --all-targets --all-features -- -D warnings`
- Run ignored local repo test manually:
  - `cargo test --test local_index -- --ignored`
- Run ignored real repo test manually when `test_repos/` exists:
  - `cargo test --test real_repos -- --ignored`

Report:
- changed files
- verification commands and results
- whether the local ignored test passed
- whether the real-repo ignored test ran, skipped, or failed
- list of repos tested under `test_repos/`, if any
