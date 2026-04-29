# PLAN_04_IMPACT_COMMAND.md

Use superpowers:subagent-driven-development.

Do not implement this until PLAN_00, PLAN_01, PLAN_02, and PLAN_03 are complete and green.

Goal:
Add `wi impact <term>` to answer: “If I edit this symbol or area, what else should I inspect or test?”

This pass uses the SQLite index and reference graph. It must not add ML prediction. Output must be deterministic, compact, and evidence-backed.

Phase tracking:
- [x] Add CLI parsing/help for `wi impact <term>`.
- [x] Add SQLite-backed impact rendering with deterministic grouping, limits, and dedupe.
- [x] Preserve existing `wi`, `wi refs`, and `wi pack` behavior and usage logging.
- [x] Add tests for impact output, ordering, limits, missing refs, and compatibility.
- [x] Run required formatting, tests, lint, smoke command, and ignored tests.
- [x] Commit with `Add wi impact command`.

Prerequisite:
- PLAN_00_SQLITE_INDEX_STORAGE.md is complete.
- PLAN_01_REFERENCE_GRAPH_FOUNDATION.md is complete.
- PLAN_02_DETERMINISTIC_REFERENCE_EXTRACTION.md is complete.
- PLAN_03_CONTEXT_COMMANDS.md is complete.
- `.dev_index/index.sqlite` is the canonical storage file.
- SQLite contains populated `records`, `files`, `usage_events`, and `refs` tables.
- `wi refs <term>` and `wi pack <term>` are working and tested.

Product rule:
Impact output must be conservative. Do not imply certainty where only weak evidence exists. Every listed item must have a concrete file:line reason from `records` or `refs`.

CLI shape:
Add:

- `wi impact <term>`

Keep existing behavior unchanged:

- `wi <term>`
- `wi refs <term>`
- `wi pack <term>`

Existing options:
Preserve existing filters where practical:

- `-t KIND`
- `-l EXT`
- `-p PATH`
- `-s SOURCE`
- `-n N`
- `-v`
- `-r REPO`

Impact behavior:
1. Run normal search for `<term>`.
2. Select top primary match or matches.
3. Load references related to selected primary names/records from SQLite `refs`.
4. Group likely affected files by evidence type.
5. Print a compact affected-file list with reasons.

Primary matching:
- Prefer exact name matches.
- Use existing search/ranking behavior.
- Default primary limit: 3.

Output groups:
- Primary
- Likely affected tests
- Callers/importers
- Related docs
- Related config/routes/schemas if detected
- Other references

Each output row should include:
- `path:line`
- kind/ref_kind
- reason/evidence

Example shape:
Primary:
- src/service.py:12 class PromptService
  reason: exact symbol match

Likely affected tests:
- tests/test_service.py:8 test_reference PromptService
  reason: test references PromptService

Callers/importers:
- src/router.py:4 import PromptService
  reason: imports PromptService

Related docs:
- docs/API.md:42 markdown_link PromptService
  reason: docs reference PromptService

Ranking:
Prefer:

1. direct references to the primary match
2. tests referencing the primary name
3. imports/callers
4. docs references
5. related config/routes/schemas
6. UI/style references
7. fixtures/examples last

Use existing path penalties from search where practical.

Limits:
- max 3 primary matches
- max 5 tests
- max 5 callers/importers
- max 3 docs
- max 5 other references
- max 15 total non-primary output rows by default

If `-n` is supported, it may control total non-primary output rows. Keep behavior documented in help/tests.

Deduplication:
- Do not list the same file repeatedly in one group unless line-level distinction is necessary.
- Prefer one best row per file per group.
- A file may appear in multiple groups only if the evidence categories are genuinely different.
- Preserve useful file:line landmarks.

Missing refs behavior:
If primary matches exist but no impact refs exist, print a clear non-error message:

- `no impact references found for <term>`

Still show primary matches.

If no primary matches exist, preserve normal no-result behavior and usage logging.

Usage logging:
- Decide whether `wi impact` should log usage in the same usage table.
- If logging, include the subcommand in the query string or use a command field if already supported.
- Keep `wi-stats` tests passing.

Help text:
Update `wi --help` to include:

- `wi impact <term>`

Update AGENTS/CLAUDE canonical block only if necessary. Do not reintroduce `WI.md`.

Tests:
Add fixture tests for:

- `wi impact Symbol` shows primary match
- `wi impact Symbol` includes a related test when fixture has one
- `wi impact Symbol` includes importer/caller when fixture has one
- `wi impact Symbol` includes docs when fixture has one
- output includes reasons
- output respects limits
- missing refs gives a clear message
- no full-file contents are dumped
- existing `wi <term>` tests still pass
- existing `wi refs <term>` tests still pass
- existing `wi pack <term>` tests still pass
- existing `wi-stats` tests still pass

Shared integrity:
- Keep using shared index/ref integrity helpers.
- Do not duplicate integrity assertion logic.

Help/instruction update:
- Update `wi --help` to document `wi impact <term>`.
- Do not update AGENTS/CLAUDE unless the recommended agent workflow changes.
- Do not mention `WI.md`.

Real repo hardening:
- Ignored `local_index` and `real_repos` tests should still pass.
- Do not assert exact `wi impact` output for arbitrary real repos.

Acceptance:
- `wi impact <term>` gives a compact affected-file list
- every output row has a concrete file:line reason
- output is deterministic
- output is conservative and evidence-backed
- output is compact
- no full-file dumping
- existing `wi`, `wi refs`, and `wi pack` behavior remains stable
- no normal test depends on local repos or `test_repos/`
- no JSONL storage is reintroduced

Verification:
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo run --bin wi -- impact <fixture term or known local term>`
- `cargo test --test local_index -- --ignored`
- `cargo test --test real_repos -- --ignored` if repos exist

Report:
- changed files
- CLI behavior added
- output example for `wi impact`
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

<!-- thinindex-plan-help-update-start -->

Help/instruction update:
- Update `wi --help` to document `wi impact <term>`.
- Do not update AGENTS/CLAUDE unless the recommended agent workflow changes.
- Do not mention `WI.md`.

<!-- thinindex-plan-help-update-end -->
