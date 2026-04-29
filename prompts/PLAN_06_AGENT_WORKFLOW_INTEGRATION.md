# PLAN_06_AGENT_WORKFLOW_INTEGRATION.md

Use superpowers:subagent-driven-development.

Do not implement this until PLAN_00 through PLAN_05 are complete and green.

Goal:
Make agents reliably use thinindex commands in the right order now that SQLite storage, refs, pack, impact, and benchmarks exist.

This pass updates instruction surfaces and tests only. Do not add new index storage, reference extraction, search ranking, benchmark logic, or new user-facing commands unless the plan explicitly requires a tiny compatibility adjustment.

Product rule:
Instruction text is a product contract. Keep it short, direct, and test-visible.

Current intended agent workflow:
- Run `build_index` before broad repository discovery.
- Run `wi --help` if search filters, examples, or subcommands are needed.
- Use `wi <term>` before grep/find/ls/Read to locate code.
- Use `wi pack <term>` for implementation tasks to get a compact read set.
- Use `wi impact <term>` before editing a symbol or feature area to find related tests/docs/callers.
- Read only files returned by `wi` unless the result is insufficient.
- If `wi` returns no useful result, rerun `build_index` once and retry.
- Fall back to grep/find/Read only after that retry fails.

Canonical Repository search block:
Update the canonical `## Repository search` block used by wi-init for AGENTS.md and existing CLAUDE.md to this text:

## Repository search

- Before broad repository discovery, run `build_index`.
- Run `wi --help` if you need search filters, examples, or subcommands.
- Use `wi <term>` before grep/find/ls/Read to locate code.
- For implementation work, prefer `wi pack <term>` to get a compact read set.
- Before editing a symbol or feature area, run `wi impact <term>` to find related tests/docs/callers.
- Read only files returned by `wi` unless the result is insufficient.
- If `wi` returns no useful result, rerun `build_index` once and retry.
- Fall back to grep/find/Read only after that retry fails.

Required implementation:
1. Update `src/bin/wi-init.rs`.
2. Use one canonical Repository search block constant for AGENTS.md and existing CLAUDE.md.
3. AGENTS.md behavior:
   - create AGENTS.md if absent
   - normalize existing AGENTS.md to exactly one canonical `## Repository search` block
   - do not duplicate on repeated `wi-init`
4. CLAUDE.md behavior:
   - do not create CLAUDE.md if absent
   - if CLAUDE.md exists, normalize it to exactly one canonical `## Repository search` block
   - do not insert `@WI.md`
5. Remove/normalize legacy markers:
   - `@WI.md`
   - `See WI.md for repository search/index usage.`
   - `See `WI.md` for repository search/index usage.`
   - old paragraph-style Repository search blocks
   - older bullet-list blocks that mention only `build_index` and `wi <term>` but not `wi pack` / `wi impact`
6. Clean up committed AGENTS.md and CLAUDE.md so they match the new canonical block.
7. Do not reintroduce WI.md.
8. Keep `wi --help` as the source of truth for command syntax, filters, examples, and subcommands.
9. Ensure `wi --help` already documents `refs`, `pack`, `impact`, and `bench` if those commands exist. If help is stale, update it.

Tests:
Update `tests/wi_init.rs`.

Required tests:
- AGENTS.md absent -> created with new canonical block.
- AGENTS.md old marker -> normalized.
- AGENTS.md older paragraph block -> normalized.
- AGENTS.md older bullet block without pack/impact -> normalized.
- AGENTS.md repeated run -> no duplicate `## Repository search`.
- CLAUDE.md absent -> not created.
- CLAUDE.md present -> normalized with new canonical block.
- CLAUDE.md with `@WI.md` -> replaced, not duplicated.
- rollback restores AGENTS.md and CLAUDE.md.
- no test asserts WI.md generation.
- help tests assert `wi --help` mentions `refs`, `pack`, `impact`, and `bench` if implemented.

Acceptance:
- AGENTS.md and existing CLAUDE.md use the same canonical Repository search block.
- The canonical block mentions `wi pack <term>` and `wi impact <term>`.
- `wi-init` is idempotent.
- Legacy search blocks and `@WI.md` are normalized away.
- `wi --help` remains the only help surface for detailed command syntax.
- No WI.md generation is restored.
- Existing search, refs, pack, impact, bench, and stats behavior remains unchanged.

Verification:
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo run --bin wi -- --help`
- `cargo test --test local_index -- --ignored`
- `cargo test --test real_repos -- --ignored` if `test_repos/` exists

Report:
- changed files
- final canonical Repository search block
- verification commands and results
- whether committed AGENTS.md and CLAUDE.md match
- whether ignored local test passed
- whether ignored real-repo test ran, skipped, or failed
- commit hash
