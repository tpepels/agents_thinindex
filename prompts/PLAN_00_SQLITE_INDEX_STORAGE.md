# PLAN_00_SQLITE_INDEX_STORAGE.md

Use superpowers:subagent-driven-development.

Goal:
Replace the current repo-local JSONL/manifest storage with SQLite before adding references or context-pack features.

This pass changes storage only. Do not add reference graph, `wi refs`, `wi pack`, prediction, or impact analysis.

Product rule:
SQLite is the canonical `.dev_index` storage layer. Do not maintain parallel JSONL storage unless a test/export command explicitly asks for it.

Storage target:
Use one SQLite database:

- `.dev_index/index.sqlite`

Primary tables:
- `meta`
- `files`
- `records`
- `usage_events`

Future-ready table may be omitted until reference graph work:
- `refs`

Required table intent:

meta:
- key TEXT PRIMARY KEY
- value TEXT NOT NULL

Required meta keys:
- `schema_version`
- `created_at` or equivalent optional timestamp
- `updated_at` or equivalent optional timestamp

files:
- path TEXT PRIMARY KEY
- mtime_ns INTEGER NOT NULL
- size INTEGER NOT NULL

records:
- path TEXT NOT NULL
- line INTEGER NOT NULL
- col INTEGER NOT NULL
- lang TEXT NOT NULL
- kind TEXT NOT NULL
- name TEXT NOT NULL
- text TEXT NOT NULL
- source TEXT NOT NULL

records constraints/indexes:
- unique location invariant: `UNIQUE(path, line, col)`
- indexes for common lookup:
  - `name`
  - `kind`
  - `lang`
  - `path`
  - `source`

usage_events:
- timestamp INTEGER or TEXT NOT NULL
- query TEXT NOT NULL
- result_count INTEGER NOT NULL
- hit INTEGER NOT NULL
- optional fields for filters if useful:
  - kind
  - lang
  - path_filter
  - source

Dependency:
- Add SQLite support via a normal Rust SQLite crate.
- Prefer `rusqlite` with bundled SQLite unless there is an existing project reason to require system SQLite.
- Keep dependency choice simple and cross-platform.

Required implementation:
1. Add a SQLite storage module, for example `src/db.rs` or `src/sqlite_store.rs`.
2. Replace `manifest.json` and `index.jsonl` as canonical storage.
3. Replace `wi_usage.jsonl` with `usage_events` table.
4. `build_index` must:
   - open/create `.dev_index/index.sqlite`
   - initialize schema
   - reset/recreate DB on schema mismatch or malformed DB
   - discover files
   - remove records/files for deleted or changed paths
   - insert updated file metadata
   - insert new records in a transaction
   - enforce unique `path + line + col`
5. `wi` must read records from SQLite.
6. `wi` usage logging must write to SQLite.
7. `wi-stats` must read usage events from SQLite.
8. `wi-init --remove` must remove `.dev_index` as before.
9. `.dev_index` must still never index itself.
10. Existing CLI behavior and output should remain stable unless tests must change for storage paths.

Automatic old-cache rebuild:
- `build_index` must automatically reset old JSONL `.dev_index` storage and create SQLite storage.
- `wi-init` gets this behavior because it calls `build_index`.
- `wi` and `wi-stats` should not silently migrate/rebuild; they should print a clear error telling the user to run `build_index`.
- Manual `rm -rf .dev_index` remains a fallback, not the normal path.

Pre-alpha rule:
- Do not preserve backwards compatibility with old JSONL storage.
- It is acceptable to delete/rebuild old `.dev_index`.
- Remove or update tests that assert `index.jsonl`, `manifest.json`, or `wi_usage.jsonl` exist.
- Replace them with SQLite assertions.

Schema/version behavior:
- Increment `INDEX_SCHEMA_VERSION`.
- Store the schema version in SQLite `meta`.
- On missing DB, create it.
- On old/mismatched/malformed DB, reset `.dev_index` or recreate `index.sqlite`.
- Usage events may be lost on schema reset unless the existing product rule says otherwise. Make the behavior explicit in tests.

Shared integrity tests:
- Update shared integrity helpers so checks are written once and operate on loaded records, not JSONL text.
- Recommended shape:
  - `IndexSnapshot { records: Vec<IndexRecord> }`
  - `load_index_snapshot_from_sqlite(root) -> IndexSnapshot`
  - `run_named_index_integrity_checks(name: &str, snapshot: &IndexSnapshot, expected_paths: &[&str])`
- Keep checks:
  - required fields present/non-empty where applicable
  - `line >= 1`
  - `col >= 1`
  - no duplicate `path + line + col`
  - no `.dev_index/` in parsed path
  - expected path substrings if supplied
- All fixture/local/real-repo tests must call the same shared suite.

Normal tests:
- Update `tests/build_index.rs`.
- Keep fixture/temp repo integrity test in normal `cargo test`.
- Assert `.dev_index/index.sqlite` exists.
- Assert old JSONL files are not required.
- Verify changed/deleted files update records correctly.
- Verify unchanged second build skips unchanged files.
- Verify malformed/old DB resets cleanly.

Local/real repo ignored tests:
- Update `tests/local_index.rs` and `tests/real_repos.rs`.
- They must rebuild `.dev_index/index.sqlite`.
- They must call the same shared integrity suite.
- Real repo tests under `test_repos/` remain ignored/manual.

Search tests:
- Existing `wi` behavior should still pass:
  - symbol search
  - filters by kind/lang/path/source
  - limit
  - verbose output
  - CSS/HTML/Markdown extras
  - ranking tests

Old storage behavior:
- `build_index` must automatically detect old JSONL `.dev_index` storage and rebuild it as SQLite.
- `wi-init` gets this behavior because it calls `build_index`.
- `wi` and `wi-stats` must not silently rebuild; if storage is missing/outdated/old, print a clear error telling the user to run `build_index`.
- Manual `rm -rf .dev_index` remains a fallback, not the normal migration path.

Stats tests:
- Update `wi-stats` tests to use SQLite usage events.
- Keep miss logging behavior:
  - no-result search records `hit=false`
  - `result_count=0`
- Keep stats windows and recent misses tests.

AGENTS/CLAUDE/WI behavior:
- Do not reintroduce `WI.md`.
- Keep `wi --help` as the help surface.
- Keep AGENTS.md and existing CLAUDE.md canonical Repository search block behavior.

Instruction surfaces:
- Keep AGENTS.md and existing CLAUDE.md generation in sync with the canonical Repository search block.
- Do not reintroduce WI.md.
- Keep `wi --help` as the source of truth for filters/examples/subcommands.
- Update tests whenever command/help text changes.

Acceptance:
- `.dev_index/index.sqlite` is the canonical storage file.
- `build_index`, `wi`, and `wi-stats` all work from SQLite.
- No normal test depends on JSONL files.
- No duplicate record locations can be stored.
- All existing search behavior still passes.
- Shared integrity tests work for fixture, local, and real-repo harnesses.
- Old `.dev_index` JSONL state is not required or preserved.

Verification:
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test local_index -- --ignored`
- `cargo test --test real_repos -- --ignored` if `test_repos/` exists
- Manual smoke:
  - `rm -rf .dev_index`
  - `cargo run --bin build_index`
  - `cargo run --bin wi -- build_index`
  - `cargo run --bin wi -- definitely_no_such_symbol_zzzz`
  - `cargo run --bin wi-stats`

Report:
- changed files
- storage files created under `.dev_index`
- validation results
- whether ignored local test passed
- whether ignored real-repo test ran, skipped, or failed

<!-- thinindex-plan-instruction-surfaces-start -->

Instruction surfaces:
- Do not reintroduce `WI.md`.
- Keep `wi --help` as the source of truth for command syntax, filters, examples, and subcommands.
- Keep AGENTS.md and existing CLAUDE.md generation in sync with the canonical `## Repository search` block.
- AGENTS.md should be created if absent.
- CLAUDE.md should be normalized only if present; do not create CLAUDE.md.
- Repeated `wi-init` runs must not duplicate `## Repository search`.
- Remove/normalize legacy markers: `@WI.md`, `See WI.md for repository search/index usage.`, `See `WI.md` for repository search/index usage.`, and old paragraph-style Repository search blocks.
- Update tests whenever help text or canonical Repository search text changes.

<!-- thinindex-plan-instruction-surfaces-end -->

<!-- thinindex-plan-old-storage-behavior-start -->

Old storage behavior:
- `build_index` must automatically detect old JSONL `.dev_index` storage and rebuild it as SQLite.
- `wi-init` gets this behavior because it calls `build_index`.
- `wi` and `wi-stats` must not silently rebuild; if storage is missing, outdated, malformed, or old JSONL-only, print a clear error telling the user to run `build_index`.
- Manual `rm -rf .dev_index` remains a fallback, not the normal migration path.
- Do not migrate old JSONL cache files. `.dev_index` is disposable.
- It is acceptable to lose old `wi_usage.jsonl` during the SQLite migration.
- Do not delete files outside `.dev_index`.

<!-- thinindex-plan-old-storage-behavior-end -->

<!-- thinindex-plan-implementation-constraint-start -->

Implementation constraint:
- Do not begin reference graph work in this plan.
- Stop after SQLite storage, old-cache rebuild behavior, existing search/stats parity, instruction-surface consistency, and tests are green.

<!-- thinindex-plan-implementation-constraint-end -->
