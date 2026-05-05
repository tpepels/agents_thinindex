# Recovery Status

Audit date: 2026-05-05.

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
| RECOVERY_10_RECOVERY_FINAL_AUDIT_AND_NEXT_DECISION | current audit commit | This file refreshes the full recovery-cycle status and records the final recovery decision. |

## Core Touchpoint Evidence

- Missing-index CLI behavior works in a real temp repo: `target/debug/wi -r <temp repo> AlphaNeedle` printed `index database missing`, ran `build_index` once, then returned `src/service.py:1 class AlphaNeedle`.
- Stale-index CLI behavior works in a real temp repo: after editing the indexed file, `target/debug/wi -r <temp repo> BetaNeedle` printed `index is stale`, ran `build_index` once, then returned `src/service.py:1 class BetaNeedle`.
- Immediate warm repeat does not rebuild: the next `target/debug/wi -r <temp repo> BetaNeedle` returned `src/service.py:1 class BetaNeedle` with empty stderr.
- `wi doctor` in this repo reports `overall: ok`, current schema/freshness, current AGENTS/CLAUDE blocks, ignored `.dev_index/`, no required quality reports, and no required license file for the free local edition.
- `wi --help`, `wi-init`, and generated agent instructions describe direct `wi <term>` use, one-shot missing/stale auto-rebuild, `wi refs`, `wi pack`, `wi impact`, and manual `build_index` only for explicit rebuilds or recovery after auto-build failure.
- No evidence was found that normal `wi` or `build_index` runs quality/comparator/real-repo checks.
- No evidence was found that Universal Ctags is used as a production parser, `WI.md` is regenerated, or JSONL is canonical storage.

## Measured Performance

Measurements were taken locally from this checkout on 2026-05-05. `cargo run` rows include Cargo process/check overhead; direct `target/debug/*` rows are the better product CLI latency signal.

| Check | Result | Budget status |
| --- | ---: | --- |
| no-change `cargo run --bin build_index` | 0.353 s, 264 scanned, 0 changed | Cargo overhead makes this slower than the direct binary budget |
| immediate second `cargo run --bin build_index` | 0.283 s, 264 scanned, 0 changed | Cargo overhead makes this slower than the direct binary budget |
| no-change `target/debug/build_index` | 0.147 s, 264 scanned, 0 changed | under 250 ms no-change budget |
| immediate second `target/debug/build_index` | 0.090 s, 264 scanned, 0 changed | under 250 ms no-change budget |
| warm `cargo run --bin wi -- build_index` | 0.287 s wall time | Cargo overhead included |
| warm `target/debug/wi build_index` | 0.193 s wall time | output-heavy query; scorecard measured internal query latency at 28 ms |
| warm `target/debug/wi pack build_index` | 0.215 s wall time | under 250 ms pack budget |
| warm `target/debug/wi impact build_index` | 0.219 s wall time | under 250 ms impact budget |
| value scorecard `cargo run --bin wi-scorecard -- --query build_index` | pass 10 / warn 0 / fail 0 | scorecard observed stale-index recovery after audit edits |

The scorecard recovery dimension is also covered by the temp-repo CLI run and by `cargo test --test agent_acceptance`.

## Value Workflow Evidence

- `wi build_index` returns concrete source/test/doc landmarks for implementation and tests.
- `wi pack build_index` returns a bounded read set with primary definitions, direct references, dependencies, dependents, likely tests, and build/config files.
- `wi impact build_index` returns definitions, references, dependents, likely tests, related docs, and build/config files with reasons and confidence labels.
- `wi-scorecard --query build_index` reports all core value dimensions passing.
- The minimum agent acceptance workflow starts from a missing index, uses `wi`, `wi refs`, `wi pack`, and `wi impact`, checks bounded/useful output, verifies stale recovery, and verifies the immediate warm repeat does not rebuild.

## Remaining Bugs

- No blocking core-loop bug was found in this audit.
- The first attempted temp-repo CLI audit accidentally redirected stderr files into the temp repository, which correctly made the next run stale. Rerunning with verification artifacts outside the repo showed no rebuild loop.

## Remaining Value Gaps

- Current-repo `wi build_index` is useful but noisy because old roadmap/recovery plan files mention `build_index` often. `pack` and `impact` remain bounded and more useful for agent read-set selection.
- Scorecard stale/missing recovery is query-state dependent: on a fresh repo it can only report that recovery was not needed. The separate acceptance test and temp-repo CLI run cover the behavior when the scorecard does not observe it directly.
- `docs/LANGUAGE_SUPPORT_AUDIT.md` still records real-repo hardening gaps for Go-heavy and PHP-heavy local corpora. This is a claims-hardening gap, not a current overclaim.
- Optional ignored `test_repos/` workflows remain local-only and are not required for normal validation. Real-repo readiness, skip reasons, and coverage gaps should be made more explicit before old roadmap work resumes.

## Decision

Continue recovery with a new focused plan. Do not resume the old roadmap yet.

The core product loop is now usable enough for continued recovery: missing/stale indexes self-heal, normal commands do not depend on quality/comparator/real-repo work, help/instructions match behavior, pack/impact are bounded and useful, the value scorecard has no failures, and the minimum agent acceptance workflow passes. The remaining risk is evidence hardening around real-repo readiness and support-claim confidence, not a reason to add broad roadmap scope.

## Next Recommended Plan

Create and run a focused real-repo test readiness and support-claim hardening recovery plan. It should audit ignored `test_repos/` behavior, skip reasons, Go/PHP coverage gaps, optional real-repo scorecard ergonomics, and docs without committing third-party repositories or adding hosted, telemetry, packaging, payment, licensing, MCP, or cloud features.
