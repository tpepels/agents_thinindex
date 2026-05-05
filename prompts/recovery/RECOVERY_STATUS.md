# Recovery Status

Audit date: 2026-05-05.

## Completed Recovery Plans

| Plan | Commit | Evidence |
| --- | --- | --- |
| RECOVERY_00_RECOVERY_SCOPE_AND_FREEZE | `417543f` | Recovery scope and freeze documented; plan order established. |
| RECOVERY_01_CURRENT_STATE_AUDIT | `f92d9e7` | Current state audit recorded in `prompts/recovery/CURRENT_STATE.md`. |
| RECOVERY_02_CORE_TOUCHPOINT_REPAIR | `ad786f6` | Missing/stale/schema-stale `wi` self-healing implemented and tested. |
| RECOVERY_03_PERFORMANCE_PROFILING_AND_BUDGETS | `1e4a46a` | Performance bottleneck profiled, fixed, and documented. |
| RECOVERY_04_VALUE_WORKFLOWS | `a0a0d84` | Core value workflows defined, tested, and documented in `VALUE_WORKFLOWS.md`. |
| RECOVERY_05_AGENT_INTEGRATION_MINIMUM_USEFUL | `8b10d1d` | AGENTS/CLAUDE/Cursor/Copilot instruction surfaces updated and tested. |

## Core Touchpoint Evidence

- Missing-index CLI behavior works: `wi -r <temp repo> AlphaNeedle` printed `index database missing`, ran `build_index` once, then returned `src/lib.rs:1 function AlphaNeedle`.
- Stale-index CLI behavior works: after editing the temp repo, `wi -r <temp repo> BetaNeedle` printed `index is stale`, ran `build_index` once, then returned `src/lib.rs:2 function BetaNeedle`.
- Immediate warm repeat does not rebuild: the second `wi -r <temp repo> BetaNeedle` returned `src/lib.rs:2 function BetaNeedle` without the rebuild message.
- `wi doctor` in this repo reports `overall: ok`, current schema/freshness, current AGENTS/CLAUDE blocks, ignored `.dev_index/`, and local-free license state.
- `wi --help` documents auto-build/auto-rebuild, manual `build_index` only for explicit rebuilds or indexing errors, filters, examples, and subcommands.
- `wi-init` generated repo-local `.cursor/rules/thinindex.mdc` and `.github/copilot-instructions.md` in RECOVERY_05 and repeated runs normalized without duplicate Repository search blocks.

## Measured Performance

Measurements were taken locally from this checkout on 2026-05-05.

| Check | Result | Budget status |
| --- | ---: | --- |
| stale/current-repo `cargo run --bin build_index` after one changed recovery file | 4.595 s, 258 scanned, 1 changed | under 6 s stale budget |
| immediate second `cargo run --bin build_index` | 0.108 s, 258 scanned, 0 changed | under 250 ms no-change budget |
| warm `target/debug/wi build_index` | 0.070 s | under 150 ms warm search budget |
| warm `target/debug/wi refs build_index` | 0.091 s | under 200 ms refs budget |
| warm `target/debug/wi pack build_index` | 0.128 s | under 250 ms pack budget |
| warm `target/debug/wi impact build_index` | 0.125 s | under 250 ms impact budget |
| real-repo `wi pack main -r test_repos/fd -n 8` through `cargo run` | 0.183 s | useful and bounded |
| real-repo `wi impact main -r test_repos/fd -n 8` through `cargo run` | 0.180 s | useful and bounded |

`cargo run` timings include Cargo process overhead. Direct `target/debug/wi` timings are the better product CLI latency signal.

## Value Workflow Evidence

- `wi build_index` returns concrete file:line landmarks for implementation and tests.
- `wi refs build_index` returns primary definitions plus docs/config/text references.
- `wi pack build_index` returns a compact read set with primary definitions, direct references, dependencies, dependents, likely tests, config/build files, and unresolved hints.
- `wi impact build_index` returns likely affected files with reasons and confidence labels.
- On `test_repos/fd`, `wi pack main -n 8` returned entry points, dependencies such as `src/cli.rs`, `src/dir_entry.rs`, and `src/error.rs`, and build/config context.
- On `test_repos/fd`, `wi doctor` accurately reported a fresh index while flagging missing `AGENTS.md` and missing `.dev_index/` ignore setup.

## Remaining Bugs

- No blocking core-loop bug was found in this audit.
- No evidence was found that normal `wi` or `build_index` runs quality/comparator/real-repo checks.
- No evidence was found that Universal Ctags is used as a production parser, `WI.md` is regenerated, or JSONL is canonical storage.

## Remaining Value Gaps

- Real-repo reference evidence is query-dependent. `main` in `test_repos/fd` has useful pack/impact context but no direct references because it is an entry point.
- `test_repos/fd` is not initialized with agent instructions, so `wi doctor -r test_repos/fd` correctly reports missing `AGENTS.md` and `.dev_index/` ignore setup.
- `docs/LANGUAGE_SUPPORT_AUDIT.md` still records real-repo hardening gaps for Go-heavy and PHP-heavy local corpora. This is a claims-hardening gap, not a current overclaim.
- Recovery plans RECOVERY_07 through RECOVERY_10 remain incomplete, so the old roadmap should stay paused.

## Decision

Continue recovery with the remaining focused plans. Do not resume the old roadmap yet.

The core product loop is now usable enough for continued recovery: missing/stale indexes self-heal, warm commands meet documented budgets, value workflows return bounded evidence, and agent instruction surfaces are current. The remaining work is evidence and acceptance hardening, not a reason to cut scope or block all progress.

## Next Recommended Plan

Run `prompts/recovery/RECOVERY_07_PRODUCT_VALUE_SCORECARD.md` next.
