# Recovery Status

Audit date: 2026-05-05.

## Status

Source recovery goals are met for the current
`prompts/recovery/PLAN_ORDER.md` cycle, but an operational installed-binary
mismatch remains.

`PLAN_ORDER.md` lists RECOVERY_00 through RECOVERY_11. All recovery plan
checklists are complete, required commits exist, and the source-built core
product loop is green after the file-reference graph work.

However, PATH currently resolves thinindex commands to stale installed binaries
under `/home/tom/.local/bin`. Those binaries expect schema 11 while source now
expects schema 12. This can flip `.dev_index/index.sqlite` between schema
versions when users mix installed commands and source-built commands.

Focused follow-up prompt: `prompts/recovery/RECOVERY_NEXT.md`.

## Completed Recovery Plans

| Plan | Commit | Evidence |
| --- | --- | --- |
| RECOVERY_00_RECOVERY_SCOPE_AND_FREEZE | `417543f` | Recovery scope and freeze documented; plan order established. |
| RECOVERY_01_CURRENT_STATE_AUDIT | `f92d9e7` | Current state audit recorded behavior, bugs, value gaps, stale assumptions, and command evidence in `CURRENT_STATE.md`. |
| RECOVERY_02_CORE_TOUCHPOINT_REPAIR | `ad786f6` | Missing/stale/schema-stale `wi <query>` one-shot self-healing implemented, tested, and aligned across doctor/help/init guidance. |
| RECOVERY_03_PERFORMANCE_PROFILING_AND_BUDGETS | `1e4a46a` | Profiling identified reference extraction as the measured bottleneck; no-change fast path and reference extraction fixes landed with budgets documented. |
| RECOVERY_04_VALUE_WORKFLOWS | `a0a0d84` | Core workflows for search, refs, pack, impact, doctor, and init are documented and covered by fixture/CLI checks. |
| RECOVERY_05_AGENT_INTEGRATION_MINIMUM_USEFUL | `8b10d1d` | AGENTS, CLAUDE normalization, Cursor, Copilot, and docs were aligned with actual self-healing behavior. |
| RECOVERY_06_RELEASE_DECISION_AUDIT | `6a2baab` | Release decision audit continued recovery instead of resuming the old roadmap while later value evidence remained incomplete. |
| RECOVERY_07_PRODUCT_VALUE_SCORECARD | `0168a76` | `wi-scorecard` added compact pass/warn/fail dimensions for the core product loop, with docs and deterministic tests. |
| RECOVERY_08_ERROR_MESSAGE_AND_HELP_AUDIT | `df44185` | CLI help and error guidance were audited and repaired for current behavior, removed concepts, and actionable next steps. |
| RECOVERY_09_MINIMUM_AGENT_ACCEPTANCE_TEST | `b94c203` | Scripted acceptance test proves a small agent workflow can use `wi`, `wi refs`, `wi pack`, and `wi impact` without a manual build pre-step. |
| RECOVERY_10_RECOVERY_FINAL_AUDIT_AND_NEXT_DECISION | `aa33198` | Final audit recorded core-loop readiness and recommended real-repo/support-claim hardening before old roadmap work resumed. |
| RECOVERY_11_REAL_REPO_READINESS_AND_SUPPORT_CLAIMS | `3f3fb73` | Real-repo readiness gaps and support claims were audited; Go/PHP real-repo gaps are classified as future hardening, not support overclaims. |

## Post-Recovery Product Evidence

The file-reference graph work landed after RECOVERY_11:

| Commit | Evidence |
| --- | --- |
| `1403da6` Add file reference graph | Added SQLite `file_references`, local extraction, deterministic resolution/unresolved reasons, docs, tests, and `wi refs`/`wi pack`/`wi impact` integration. |

## Current Core Touchpoint Evidence

Commands were run from this checkout after the file-reference graph commit.

- `cargo run --bin build_index`: passed after schema reset; immediate no-change run reported `changed files: 0`, `records: 3207`.
- `cargo run --bin build_index -- --stats`: passed with `changed files: 0`, `refs: 3880`, `dependencies: 144`, `file references: 285`, and `total ms: 33`.
- `cargo run --bin wi -- build_index`: passed and returned source/test/doc landmarks.
- `cargo run --bin wi -- refs build_index`: passed and returned primary definitions plus evidence-backed references.
- `cargo run --bin wi -- pack build_index`: passed and returned a bounded read set with primary definitions, direct refs, dependencies, dependents, tests, configs, reasons, and confidence labels.
- `cargo run --bin wi -- impact build_index`: passed and returned definitions, references, dependent files, likely tests, related docs, build/config files, reasons, and confidence labels.
- `cargo run --bin wi-stats`: passed and reported local usage statistics plus an agent workflow audit.
- `cargo run --bin wi -- doctor`: passed after self-healing and reported `overall: ok`, schema version 12, fresh index, current agent instruction blocks, ignored `.dev_index/`, and free local edition behavior.
- `cargo run --bin wi -- --help`: passed and describes direct `wi <term>` use, one-shot missing/stale auto-rebuild, refs, pack, impact, and file-reference-aware context commands.
- `cargo run --bin wi-init -- --help`: passed and says it writes AGENTS/Cursor/Copilot instructions, normalizes existing CLAUDE.md, builds `.dev_index/index.sqlite`, and does not create `WI.md`.
- `cargo run --bin wi-scorecard -- --query build_index`: passed with `summary: pass 10 / warn 0 / fail 0`; it observed schema-stale recovery and measured warm query latency at 28 ms.
- `cargo test --test agent_acceptance`: passed and covers missing-index recovery, refs, pack, impact, stale recovery, and warm repeat without rebuild.
- PATH `/home/tom/.local/bin/wi doctor`: failed because it expected schema 11
  while the source-built index was schema 12; it identified the running binary
  as `/home/tom/.local/bin/wi`.
- PATH `/home/tom/.local/bin/build_index`: rebuilt this checkout using the old
  schema, proving the installed binaries can invalidate the source-built index.

## Recovery Goal Assessment

| Goal | Status | Evidence |
| --- | --- | --- |
| stale/missing index self-heals | met | RECOVERY_02 tests, `agent_acceptance`, and `wi-scorecard` schema-stale recovery evidence. |
| no-change `build_index` is fast enough | met | `build_index --stats` no-change run reported `total ms: 33`, under the documented 250 ms direct-binary budget. |
| warm `wi <query>` is fast enough | met | `wi-scorecard --query build_index` reported 28 ms, under the 150 ms budget. |
| `wi doctor`, help, and init match behavior | source met; installed blocker | Source-built `wi doctor` is ok; source `wi --help` and `wi-init --help` match behavior. PATH `wi`/`build_index` are stale and covered by `RECOVERY_NEXT.md`. |
| pack and impact are useful enough | met | Product touchpoints return bounded, grouped, evidence-backed read sets with reasons and confidence labels. |
| agent instructions match behavior | met | `wi doctor` and `wi-scorecard` report current AGENTS/CLAUDE/Cursor/Copilot instruction blocks. |
| file references improve refs/pack/impact | met | `file_references` table has 285 rows in this repo; pack/impact show `file_import` evidence such as `src/lib.rs -> src/indexer.rs`. |

## Verification Evidence

Latest audit verification:

- `cargo fmt --check`: passed.
- `cargo test`: passed, `317 passed, 8 ignored`.
- `cargo clippy --all-targets --all-features -- -D warnings`: passed.
- `cargo test --test local_index -- --ignored`: passed.
- `cargo test --test real_repos -- --ignored`: passed, `1 passed, 3 filtered out`, because local `test_repos/` exists.

## Remaining Caveats

- Concurrent schema-stale rebuild attempts are not treated as a normal product path. A parallel touchpoint run briefly exposed a schema-version mismatch while multiple `cargo run` commands competed to rebuild the disposable index. Serialized `build_index`, `wi`, `wi doctor`, and `wi-scorecard` paths recovered correctly and are the supported workflow.
- Current-repo `wi build_index` is useful but noisy because roadmap and recovery prompts mention `build_index` often. `wi pack build_index` and `wi impact build_index` remain bounded and more useful for agent read-set selection.
- Go and PHP remain supported by fixture/conformance coverage, but this checkout still lacks Go-heavy and PHP-heavy local manifest targets. That is documented as future real-repo hardening, not a support-claim blocker.
- Some local `test_repos/` side corpora remain exploratory until a future scoped plan adds stable expected-symbol or expected-pattern checks. Third-party repository contents must remain uncommitted.
- File-reference extraction is explicit and best-effort. It resolves local evidence, preserves unresolved local-looking paths, and does not claim package-manager, compiler, framework alias, LSP, or network semantics.
- Installed binaries in `/home/tom/.local/bin` are stale relative to this
  checkout. This is the only current blocker found by the post-file-reference
  audit.

## Decision

Do not resume old roadmap work until `RECOVERY_NEXT.md` is handled or the
installed-binary mismatch is explicitly accepted as an environment-only issue
with evidence.

After that focused blocker is resolved, old roadmap/product work may resume,
but only through a new scoped plan that respects the current guardrails:

- do not reintroduce Universal Ctags as a production parser;
- do not reintroduce `WI.md`;
- do not reintroduce JSONL as canonical storage;
- keep quality/comparator/real-repo checks out of normal `wi` and
  `build_index` paths;
- avoid packaging, licensing, payment, telemetry, cloud, hosted, and MCP work
  unless a future selected plan explicitly asks for it.

## Next Required Recovery Prompt

Run `prompts/recovery/RECOVERY_NEXT.md` first.

## Next Recommended Old-Roadmap Plan After RECOVERY_NEXT

There is no existing unchecked active old-roadmap `PLAN_*.md` file after
PLAN_51 and RECOVERY_11. The next old-roadmap step should be a new bounded
post-PLAN_51 plan, recommended title:

`PLAN_52_FILE_REFERENCE_GRAPH_REAL_REPO_HARDENING.md`

Recommended scope:
- validate file-reference extraction on practical local real repos without
  committing `test_repos/` contents;
- add expected file-reference patterns where stable;
- tune noisy docs/config/package path extraction only from measured evidence;
- keep `wi refs`, `wi pack`, and `wi impact` compact and useful;
- update docs for any observed limitations.

Do not use the next plan to add broad parser architecture, package-manager
execution, LSP/compiler dependencies, hosted behavior, telemetry, payment,
licensing enforcement, or release packaging.
