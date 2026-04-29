# PLAN_03_CONTEXT_COMMANDS.md

Use superpowers:subagent-driven-development.

Goal:
Add user-facing commands that turn the SQLite reference graph into useful agent read guidance.

Add:

- `wi refs <term>`
- `wi pack <term>`

Do not add ML prediction. Output must be deterministic, compact, and explainable.

Phase tracking:
- [x] Add CLI parsing/help for `wi refs <term>` and `wi pack <term>`.
- [x] Add SQLite-backed refs/read-pack rendering with deterministic limits and dedupe.
- [x] Preserve existing `wi <term>` behavior and usage logging.
- [x] Add tests for refs/pack output, ordering, limits, missing refs, and compatibility.
- [x] Run required formatting, tests, lint, smoke commands, and ignored tests.
- [x] Commit with `Add wi refs and pack context commands`.

Prerequisite:
PLAN_00_SQLITE_INDEX_STORAGE.md, PLAN_01_REFERENCE_GRAPH_FOUNDATION.md, and PLAN_02_DETERMINISTIC_REFERENCE_EXTRACTION.md must be complete and green. `.dev_index/index.sqlite` is the canonical storage file and contains populated `records`, `files`, `usage_events`, and `refs` tables.

Product rule:
The output should help an agent decide what to read next without dumping file contents.

CLI shape:
Support subcommands if simple:

- `wi refs <term>`
- `wi pack <term>`

Keep existing `wi <term>` behavior unchanged.

Existing options:
Preserve existing filters where practical:

- `-t KIND`
- `-l EXT`
- `-p PATH`
- `-s SOURCE`
- `-n N`
- `-v`
- `-r REPO`

Do not break existing `wi <term>` tests.

`wi refs <term>` behavior:
1. Run normal search for `<term>`.
2. Select top primary match or matches.
3. Load matching references from SQLite `refs`.
4. Show references related to the matched names/records.

Primary matching:
- Prefer exact name matches.
- Use the existing search/ranking path.
- Default primary limit: 3.

Reference matching:
- Match refs by `to_name` against selected primary record names.
- If useful and deterministic, also match refs by exact queried term.
- Do not emit unrelated broad substring refs.

Output groups:

- Primary
- References

Reference rows should include:

- `from_path:from_line`
- `ref_kind`
- `to_name`
- compact reason/evidence

Example shape:

Primary:
- src/service.py:12 class PromptService

References:
- tests/test_service.py:8 test_reference PromptService
  reason: PromptService
- src/router.py:4 import PromptService
  reason: from service import PromptService

Limits:
- max 3 primary matches by default
- max 20 refs total by default
- `-n` may control refs limit if practical; otherwise document current behavior in help/tests

`wi pack <term>` behavior:
Produce a compact suggested read set.

Groups:

- Primary
- Tests
- Callers/importers
- Docs
- Related UI/style/config

Suggested limits:

- max 3 primary
- max 3 tests
- max 3 callers/importers
- max 2 docs
- max 10 total suggested reads

Output must include reasons.

Example reason formats:

- `reason: exact symbol match`
- `reason: test_reference to PromptService`
- `reason: import reference`
- `reason: markdown link/reference`
- `reason: css/html usage`

Do not output full file contents.

Deduplication:
- Do not list the same file repeatedly in `wi pack` unless line-level distinction is necessary.
- Prefer one best row per file per group.
- Preserve useful line landmarks.

Ranking:
Initial deterministic ranking:

1. exact primary matches
2. production source references
3. import/caller references
4. tests
5. docs
6. UI/style/config references
7. fixtures/examples last

Use existing path penalties from search if possible.

Missing refs behavior:
If no refs are found, print a clear non-error message, for example:

- `no references found for <term>`

Still show primary matches if they exist.

If no primary matches are found, preserve normal no-result behavior and usage logging.

Usage logging:
- Existing `wi <term>` usage logging must remain.
- Decide whether `wi refs` and `wi pack` should log usage in the same usage table.
- If they log usage, include the full subcommand/query in the query string or add a command field if already supported.
- Keep `wi-stats` tests passing.

Help text:
Update `wi --help` to include:

- `wi refs <term>`
- `wi pack <term>`

Update AGENTS/CLAUDE canonical block only if necessary. Do not reintroduce `WI.md`.

Tests:
Add fixture tests for:

- `wi refs Symbol` includes known import/test reference
- `wi refs Symbol` shows primary match
- `wi refs Symbol` respects deterministic ordering
- `wi pack Symbol` includes primary symbol and related test
- `wi pack Symbol` includes importer/caller where fixture provides one
- `wi pack Symbol` does not dump file contents
- output respects limits
- missing refs gives a clear message
- existing `wi <term>` tests still pass
- existing filter tests still pass
- existing `wi-stats` tests still pass

Shared integrity:
- Keep using shared index/ref integrity helpers.
- Do not duplicate integrity assertion logic.

Real repo hardening:
- Ignored `local_index` and `real_repos` tests should still pass.
- Do not assert exact `wi refs` or `wi pack` output for arbitrary real repos.

Help/instruction update:
- Update `wi --help` to document `wi refs <term>` and `wi pack <term>`.
- Consider updating the AGENTS/CLAUDE canonical block only if `wi pack <term>` becomes the recommended default agent workflow.
- Do not mention `WI.md`.

Acceptance:
- agents can run `wi pack <term>` to get a small read plan
- agents can run `wi refs <term>` to inspect factual references
- references are factual and cite file:line landmarks
- output is deterministic
- output is compact and reasoned
- no full-file dumping
- existing `wi <term>` behavior remains stable
- no normal test depends on local repos or `test_repos/`
- no JSONL storage is reintroduced

Verification:
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo run --bin wi -- refs <fixture term or known local term>`
- `cargo run --bin wi -- pack <fixture term or known local term>`
- `cargo test --test local_index -- --ignored`
- `cargo test --test real_repos -- --ignored` if repos exist

Report:
- changed files
- CLI behavior added
- output examples for `wi refs` and `wi pack`
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
- Update `wi --help` to document `wi refs <term>` and `wi pack <term>`.
- Consider updating the AGENTS/CLAUDE canonical block only if `wi pack <term>` becomes the recommended default agent workflow.
- Do not mention `WI.md`.

<!-- thinindex-plan-help-update-end -->
