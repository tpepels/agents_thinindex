# Current Thinindex State

Audit date: 2026-05-05.

This audit is evidence-only. It does not implement product fixes.

## Current Behavior Summary

- Recovery execution is now frozen to `prompts/recovery/PLAN_ORDER.md`.
- Active roadmap plans remain in `prompts/` as background only: 61 `PLAN_*.md`
  files.
- Recovery plans are present: 11 `RECOVERY_*.md` files, RECOVERY_00 through
  RECOVERY_10.
- Superseded parser plans remain available in `prompts/superseded/`: 6
  `PLAN_*.md` files.
- The current repository index is SQLite schema version 11 and currently
  contains only `tree_sitter` and `extras` production record sources. The
  current repo index has zero production records or refs with source `ctags`.
- `wi doctor` reports the current thinindex checkout as `overall: ok`.
- `wi <query>`, `wi refs`, `wi pack`, and `wi impact` work on a fresh index in
  this repository.
- `build_index` handles explicit schema/old JSONL rebuilds when it is run
  directly.
- `wi-init` creates/normalizes `AGENTS.md`, normalizes an existing `CLAUDE.md`,
  does not create `WI.md`, and builds the index.
- `wi-stats` reads local usage from SQLite and reports hit/miss and advisory
  workflow audit data.
- Optional quality/comparator code remains isolated under quality modules and
  tests. Normal `wi` and `build_index` code paths reviewed in this audit do not
  run quality/comparator/real-repo checks.

## Commands Run

- `build_index`
- `sed -n '1,220p' prompts/recovery/PLAN_ORDER.md`
- `sed -n '1,260p' prompts/recovery/RECOVERY_00_RECOVERY_SCOPE_AND_FREEZE.md`
- `sed -n '1,280p' prompts/recovery/RECOVERY_01_CURRENT_STATE_AUDIT.md`
- `git log --oneline --grep='Define recovery cycle scope' -5`
- `git log --oneline --grep='Audit current thinindex state' -5`
- `wi impact RECOVERY_01_CURRENT_STATE_AUDIT`
- `rg --files src tests docs prompts/recovery`
- `rg -n 'fn main|doctor|wi-init|wi-stats|refs|pack|impact|build_index|ctags|JSONL|jsonl|WI\\.md|CLAUDE|AGENTS|schema' src tests docs README.md Cargo.toml`
- `sed -n` reads for `src/bin/wi.rs`, `src/bin/build_index.rs`,
  `src/bin/wi-init.rs`, `src/bin/wi-stats.rs`, `src/wi_cli.rs`,
  `src/doctor.rs`, `src/agent_instructions.rs`, `src/indexer.rs`,
  `src/context.rs`, `src/search.rs`, `src/store.rs`, `src/support.rs`, and
  relevant docs/tests.
- `cargo run --bin build_index -- --stats`
- `cargo run --bin wi -- doctor`
- `cargo run --bin wi -- --help`
- `cargo run --bin wi-init -- --help`
- `cargo run --bin wi -- build_index`
- `cargo run --bin wi -- refs build_index`
- `cargo run --bin wi -- pack build_index`
- `cargo run --bin wi -- impact build_index`
- `cargo run --bin wi-stats`
- Temp-repo CLI checks for missing index, warm query, stale index,
  schema-stale index, and `wi doctor` states.
- `build_index --stats`, `wi`, `wi refs`, `wi pack`, and `wi impact` in
  `test_repos/fd`.
- `wi ctags`, `wi 'WI.md'`, `wi JSONL`, `wi native parser`
- `rg -n 'source\\s*=\\s*\"ctags\"|UniversalCtagsComparator|old_jsonl_storage_exists|index\\.jsonl|wi_usage\\.jsonl|WI\\.md|native parser' src tests README.md docs prompts/recovery prompts/superseded`
- `find`/`git` inventory commands for prompt counts, `WI.md`, ignored
  `test_repos/`, and generated local directories.
- SQLite checks for schema version and production record/ref sources.

## Failing Commands

Focused temp-repo CLI evidence:

```text
--- missing wi ---
error: index database missing; run `build_index`
next: run `build_index` in /tmp/tmp.A6UmjLAAjY
help: run `wi doctor` to inspect setup
exit:1

--- stale wi ---
error: index is stale; run `build_index`
next: run `build_index` in /tmp/tmp.A6UmjLAAjY
why: indexed files changed since the last build
help: run `wi doctor` to inspect setup
exit:1

--- schema stale wi ---
error: index schema version 999 does not match 11; run `build_index`
next: run `build_index` in /tmp/tmp.A6UmjLAAjY
help: run `wi doctor` to inspect setup
exit:1
```

These failures match the current `src/bin/wi.rs` code path: non-doctor commands
call `index_is_fresh`; stale returns a manual `build_index` error, while
missing/schema-stale indexes bubble store errors and do not rebuild.

## Performance Observations

- Current repo no-change `cargo run --bin build_index -- --stats`:
  250 scanned files, 0 changed files, 2986 records, 3641 refs, total 25066 ms.
  Reported phase timings were discover 23 ms, change detection 1 ms, parse 0
  ms, dependencies 29 ms, refs 24868 ms, save 114 ms.
- Real repo `test_repos/fd` no-change `build_index --stats`: 55 scanned files,
  0 changed files, 1135 records, 1526 refs, total 5302 ms. Reported refs phase
  was 5197 ms.
- These are audit observations only. RECOVERY_03 requires profiling before
  fixing performance.

## Confirmed Bugs

1. `wi <query>` does not self-heal a missing index. It exits and tells the user
   to run `build_index`.
2. `wi <query>` does not self-heal a stale index. It exits and tells the user
   to run `build_index`.
3. `wi <query>` does not self-heal a schema-stale index. It exits and tells the
   user to run `build_index`.
4. Help/docs/instructions still describe manual rebuild behavior. Examples:
   `wi --help`, `src/agent_instructions.rs`, `README.md`, and
   `docs/AGENT_INTEGRATION.md` tell users to run `build_index` before broad
   discovery or after stale/missing results. RECOVERY_02 must align these with
   the intended auto-rebuild behavior.
5. `wi doctor` accurately reports missing/stale/schema-stale states, but its
   next steps are manual rebuild steps and do not yet match the intended
   RECOVERY_02 behavior.
6. No-change `build_index` recomputes refs across indexable files even when 0
   files changed, producing multi-second no-op builds in the current repo and
   `test_repos/fd`.

## Confirmed Value Gaps

- `wi refs main` in `test_repos/fd` found primary definitions but no references
  for `main`, which limits usefulness for a common entry-point query.
- `wi impact main` in `test_repos/fd` produced broad source-area config/build
  heuristics and no tests or references. The command is usable but not yet
  consistently high-signal for common queries.
- `wi pack build_index` in this repo returns a compact read set, but direct
  references were empty because primary results include multiple matching
  symbols and the pack grouping favors dependency/test/config hints.
- `wi-stats` is useful locally but advisory only; it cannot detect external
  grep/find/ls/Read usage, which is disclosed in its output.

## Stale Assumptions

- The current user-facing loop assumes users and agents run `build_index`
  manually before broad discovery and after stale/missing results. The recovery
  objective now requires `wi <query>` to auto-build or auto-rebuild once and
  continue.
- `README.md` currently says `wi` does not silently rebuild a missing or stale
  index. That is true today but stale relative to RECOVERY_02's required
  behavior.
- JSONL appears only as disposable old-cache migration support and quality
  export detail naming, not as canonical storage.
- Universal Ctags appears only in optional quality comparator/boundary code,
  tests, and documentation; production indexing uses Tree-sitter plus
  project-owned extras.
- Native-parser plan references are superseded historical documents under
  `prompts/superseded/`, not active architecture.
- `test_repos/` exists locally and is ignored/untracked. Ignored real-repo
  tests should run here, but normal validation must not depend on that corpus.

## Verification

Passed:

- `cargo fmt --check`: passed.
- `cargo test`: passed, 294 passed and 7 ignored across 36 suites.
- `cargo clippy --all-targets --all-features -- -D warnings`: passed with no
  issues.
- `cargo run --bin build_index`: passed; indexed this repo with 251 scanned
  files, 2 changed files, 0 deleted files, and 2996 records.
- `cargo run --bin wi -- build_index`: passed; returned build_index landmarks.
- `cargo run --bin wi -- pack build_index`: passed; returned primary
  definitions plus dependency/test/config/doc context.
- `cargo run --bin wi -- impact build_index`: passed; returned direct
  definitions, dependent files, likely tests, docs, and build/config files.
- `cargo run --bin wi-stats`: passed; reported local usage windows and advisory
  agent workflow audit.
- `cargo test --test local_index -- --ignored`: passed, 1 passed.
- `cargo test --test real_repos -- --ignored`: applicable because
  `test_repos/` exists locally; passed, 1 passed and 3 filtered out.

## Recommended Next Plan

Proceed to `prompts/recovery/RECOVERY_02_CORE_TOUCHPOINT_REPAIR.md`.
