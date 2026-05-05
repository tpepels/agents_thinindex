# Recovery Status

Audit date: 2026-05-05.

## Status

Recovery is complete for the current `prompts/recovery/PLAN_ORDER.md` cycle,
including the focused installed-binary follow-up.

`PLAN_ORDER.md` lists RECOVERY_00 through RECOVERY_11. All recovery plan
checklists are complete, required commits exist, the source-built core product
loop is green after the file-reference graph work, and PATH-installed binaries
now agree with the source checkout on version and schema.

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
| current recovery commit | Added schema-bearing version output, source/binary mismatch diagnosis, index-write guards, installer schema smoke checks, docs, and refreshed `/home/tom/.local/bin` binaries. |

## Current Core Touchpoint Evidence

Commands were run from this checkout after the file-reference graph commit and
the installed-binary fix.

- `which wi`: `/home/tom/.local/bin/wi`.
- `which build_index`: `/home/tom/.local/bin/build_index`.
- `wi --version`: `wi 0.1.4 (index schema 12)`.
- `build_index --version`: `build_index 0.1.4 (index schema 12)`.
- `cargo run --bin wi -- --version`: `wi 0.1.4 (index schema 12)`.
- `cargo run --bin build_index -- --version`: `build_index 0.1.4 (index schema 12)`.
- PATH `build_index`: passed after schema reset and wrote schema 12.
- PATH `wi doctor`: passed with `overall: ok`, `binary/source` ok, and binary path `/home/tom/.local/bin/wi`.
- PATH `wi build_index`: passed and returned current landmarks.
- `cargo run --bin wi -- doctor` after PATH `build_index`: passed with `overall: ok`, proving PATH build no longer downgrades the source-built index.
- `cargo run --bin build_index`: passed after PATH `build_index`; immediate no-change run reported `changed files: 0`, `records: 3244`.
- `cargo run --bin build_index -- --stats`: passed with `changed files: 0`, `refs: 3954`, `dependencies: 148`, `file references: 289`, and `total ms: 33`.
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

## Recovery Goal Assessment

| Goal | Status | Evidence |
| --- | --- | --- |
| stale/missing index self-heals | met | RECOVERY_02 tests, `agent_acceptance`, and `wi-scorecard` schema-stale recovery evidence. |
| no-change `build_index` is fast enough | met | `build_index --stats` no-change run reported `total ms: 33`, under the documented 250 ms direct-binary budget. |
| warm `wi <query>` is fast enough | met | `wi-scorecard --query build_index` reported 28 ms, under the 150 ms budget. |
| `wi doctor`, help, and init match behavior | met | Source-built and PATH `wi doctor` are ok; source and PATH version output includes schema 12; help/init docs match behavior. |
| pack and impact are useful enough | met | Product touchpoints return bounded, grouped, evidence-backed read sets with reasons and confidence labels. |
| agent instructions match behavior | met | `wi doctor` and `wi-scorecard` report current AGENTS/CLAUDE/Cursor/Copilot instruction blocks. |
| file references improve refs/pack/impact | met | `file_references` table has 289 rows in this repo; pack/impact show `file_import` evidence such as `src/lib.rs -> src/indexer.rs`. |

## Verification Evidence

Latest audit verification:

- `cargo fmt --check`: passed.
- `cargo test`: passed, `317 passed, 8 ignored`.
- `cargo clippy --all-targets --all-features -- -D warnings`: passed.
- `cargo test --test local_index -- --ignored`: passed.
- `cargo test --test real_repos -- --ignored`: passed, `1 passed, 3 filtered out`, because local `test_repos/` exists.
- PATH/version/install verification: passed for `which wi`, `which build_index`, PATH/source `--version`, PATH `wi doctor`, PATH `build_index`, PATH `wi build_index`, and source doctor/build after PATH build.

## Remaining Caveats

- Concurrent schema-stale rebuild attempts are not treated as a normal product path. A parallel touchpoint run briefly exposed a schema-version mismatch while multiple `cargo run` commands competed to rebuild the disposable index. Serialized `build_index`, `wi`, `wi doctor`, and `wi-scorecard` paths recovered correctly and are the supported workflow.
- Current-repo `wi build_index` is useful but noisy because roadmap and recovery prompts mention `build_index` often. `wi pack build_index` and `wi impact build_index` remain bounded and more useful for agent read-set selection.
- Go and PHP remain supported by fixture/conformance coverage, but this checkout still lacks Go-heavy and PHP-heavy local manifest targets. That is documented as future real-repo hardening, not a support-claim blocker.
- Some local `test_repos/` side corpora remain exploratory until a future scoped plan adds stable expected-symbol or expected-pattern checks. Third-party repository contents must remain uncommitted.
- File-reference extraction is explicit and best-effort. It resolves local evidence, preserves unresolved local-looking paths, and does not claim package-manager, compiler, framework alias, LSP, or network semantics.
- If a future schema bump lands, users must refresh installed binaries with
  `make install` or the archive installer. Current binaries now expose schema in
  `--version` and refuse index writes from a mismatched thinindex source
  checkout where practical.

## Decision

Recovery is complete. Old roadmap/product work may resume, but only through a
new scoped plan that respects the current guardrails:

- do not reintroduce Universal Ctags as a production parser;
- do not reintroduce `WI.md`;
- do not reintroduce JSONL as canonical storage;
- keep quality/comparator/real-repo checks out of normal `wi` and
  `build_index` paths;
- avoid packaging, licensing, payment, telemetry, cloud, hosted, and MCP work
  unless a future selected plan explicitly asks for it.

## Next Recommended Old-Roadmap Plan

The next old-roadmap step should be a bounded post-PLAN_51 plan:

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
