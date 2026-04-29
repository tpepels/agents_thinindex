# PLAN_01_REFERENCE_GRAPH_FOUNDATION.md

Use superpowers:subagent-driven-development.

Goal:
Add a deterministic reference graph on top of the SQLite index storage from PLAN_00.

This pass does not add prediction, context packs, impact analysis, or new `wi` commands. It only adds the reference data model, SQLite storage, build integration, and integrity tests.

Product rule:
References must be factual and evidence-backed. Do not infer relationships without a concrete indexed occurrence or extractor result.

Prerequisite:
PLAN_00_SQLITE_INDEX_STORAGE.md must be complete and green. `.dev_index/index.sqlite` is the canonical storage file. Do not reintroduce JSONL storage.

Scope:
Add reference storage to SQLite.

Add a `refs` table to `.dev_index/index.sqlite`.

Reference model:
Add a typed model, for example:

- `ReferenceRecord`
- `from_path: String`
- `from_line: usize`
- `from_col: usize`
- `to_name: String`
- `to_kind: Option<String>`
- `ref_kind: String`
- `evidence: String`
- `source: String`

Required table fields:

- `from_path`
- `from_line`
- `from_col`
- `to_name`
- `ref_kind`
- `evidence`
- `source`

Optional field:

- `to_kind`

Allowed `ref_kind` values for this phase:

- `text_reference`
- `import`
- `markdown_link`
- `css_usage`
- `html_usage`
- `test_reference`

SQLite schema:
Add table `refs`.

Suggested columns:

- `from_path TEXT NOT NULL`
- `from_line INTEGER NOT NULL`
- `from_col INTEGER NOT NULL`
- `to_name TEXT NOT NULL`
- `to_kind TEXT`
- `ref_kind TEXT NOT NULL`
- `evidence TEXT NOT NULL`
- `source TEXT NOT NULL`

Indexes:

- `refs(to_name)`
- `refs(from_path)`
- `refs(ref_kind)`
- `refs(source)`

Duplicate reference rule:

- `UNIQUE(from_path, from_line, from_col, to_name, ref_kind)`

Build integration:
- `build_index` writes refs into SQLite.
- Existing `records`, `files`, `meta`, and `usage_events` behavior must remain stable unless schema changes require a rebuild.
- Reference generation uses discovered files and source text.
- `.dev_index` must not index itself.
- Deleted/changed files remove stale refs for those paths.
- Reset behavior removes stale refs.
- All writes should happen inside the same transaction model as index updates where practical.

Storage helpers:
Add SQLite helpers, names can vary:

- `load_refs(root) -> Vec<ReferenceRecord>`
- `save_refs(root, refs)`
- `remove_refs_for_paths(root, stale_paths)`
- `sort_refs(refs)` if sorting before insert is still useful
- query helpers for later commands may be added but are not required in this plan

Sorting/insertion must be deterministic.

Initial reference generation:
This phase may emit a minimal, conservative set of refs.

Acceptable minimum:
- create the table
- generate at least one deterministic reference type from fixture content
- prove stale refs are removed
- prove refs integrity checks work

Do not build broad substring reference extraction in this plan. That belongs in PLAN_02.

Integrity tests:
Add shared ref-integrity helpers in `tests/common/mod.rs` or the existing shared integrity module:

- load refs from SQLite
- validate required fields and types
- validate `from_line >= 1`
- validate `from_col >= 1`
- validate no `.dev_index/` paths in `from_path`
- validate duplicate reference rule

Duplicate reference definition:

- `from_path + from_line + from_col + to_name + ref_kind`

Fixture test:
Add or update a normal fixture/temp repo test that includes:

- one code symbol
- one concrete reference to that symbol or target
- one markdown link if easy
- one CSS/HTML/JSX reference if already supported by existing extractors

Run `build_index`, then assert:

- `.dev_index/index.sqlite` exists
- `refs` table exists
- ref integrity checks pass
- expected reference names appear

Local/real repo tests:
Extend ignored local/real repo integrity tests to also check refs from SQLite.

Do not make normal `cargo test` depend on local `.dev_index` or `test_repos/`.

Schema version:
Increment `INDEX_SCHEMA_VERSION` because the SQLite schema changes.

Migration behavior:
Pre-alpha rule applies. Do not preserve old SQLite DBs. On schema mismatch or malformed DB, reset/recreate `.dev_index/index.sqlite`.

Instruction surfaces:
- Keep AGENTS.md and existing CLAUDE.md generation in sync with the canonical Repository search block.
- Do not reintroduce WI.md.
- Keep `wi --help` as the source of truth for filters/examples/subcommands.
- Update tests whenever command/help text changes.

Acceptance:
- SQLite has a `refs` table
- `build_index` populates refs deterministically
- stale refs are removed on rebuild
- refs integrity checks pass for fixtures
- ignored local repo test checks refs
- ignored real repo test checks refs when `test_repos/` exists
- existing `wi` search behavior remains unchanged
- existing `wi-stats` behavior remains unchanged
- no JSONL storage is reintroduced

Verification:
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test local_index -- --ignored`
- `cargo test --test real_repos -- --ignored` if `test_repos/` exists

Report:
- changed files
- SQLite schema changes
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
