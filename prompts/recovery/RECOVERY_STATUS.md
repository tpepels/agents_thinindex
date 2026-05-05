# Recovery Status

Audit date: 2026-05-05.

## Status

Recovery is complete for the current `prompts/recovery/PLAN_ORDER.md` cycle,
the focused installed-binary follow-up, the post-recovery PLAN_52
file-reference real-repo hardening pass, the PLAN_53 scorecard install/docs
alignment pass, and the PLAN_54 release-candidate readiness checklist.

RC status: `ready_with_caveats` for the current local release archive path.
No core search/index/install/package blocker remains for the local archive RC.

`PLAN_ORDER.md` lists RECOVERY_00 through RECOVERY_11. All recovery plan
checklists are complete, required commits exist, the source-built core product
loop is green after the file-reference graph and real-repo hardening work, and
PATH-installed binaries now agree with the source checkout on version and
schema.

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
| `319caa0` Fix installed binary schema mismatch | Added schema-bearing version output, source/binary mismatch diagnosis, index-write guards, installer schema smoke checks, docs, and refreshed `/home/tom/.local/bin` binaries. |
| `7a0c3b6` Harden file references on real repos | Hardened Markdown fragment/query resolution, HTML `srcset`, CSS/SCSS `@import`, Sass partials, `.csproj` project-file paths, and noisy config filtering; added fixture coverage, real-repo reporting, docs, and PLAN_52 execution notes. |
| `5b1899b` Align scorecard install docs | `wi-scorecard` is installed, packaged, schema-version-smoked, listed in installer/release docs, included in archive package content checks, and shipped with `docs/SCORECARD.md`. |
| this commit Assess release candidate readiness | README archive-binary count was corrected; installed/source schema agreement, no-change build speed, installed self-healing, scorecard, release archive contents/exclusions, SBOM/checksum, archive smoke, and local/real-repo verification are green. |

## Current Core Touchpoint Evidence

Commands were run from this checkout after PLAN_52.

- `which wi`: `/home/tom/.local/bin/wi`.
- `which build_index`: `/home/tom/.local/bin/build_index`.
- `wi --version`: `wi 0.1.4 (index schema 12)`.
- `build_index --version`: `build_index 0.1.4 (index schema 12)`.
- `cargo run --bin wi -- --version`: `wi 0.1.4 (index schema 12)`.
- `cargo run --bin build_index -- --version`: `build_index 0.1.4 (index schema 12)`.
- `cargo run --bin build_index`: passed with `changed files: 1`, `records: 3264` after adding PLAN_53 and updating this status.
- Immediate `cargo run --bin build_index -- --stats`: passed with `changed files: 0`, `refs: 3960`, `dependencies: 148`, `file references: 287`, and `total ms: 35`.
- `cargo run --bin wi -- build_index`: passed and returned source/test/doc landmarks.
- `cargo run --bin wi -- refs build_index`: passed and returned primary definitions plus evidence-backed references.
- `cargo run --bin wi -- pack build_index`: passed and returned a bounded read set with primary definitions, direct refs, dependencies, dependents, tests, configs, reasons, and confidence labels.
- `cargo run --bin wi -- impact build_index`: passed and returned definitions, references, dependent files, likely tests, related docs, build/config files, reasons, and confidence labels.
- `cargo run --bin wi -- doctor`: passed with `overall: ok`, schema version 12, fresh index, current agent instruction blocks, ignored `.dev_index/`, and free local edition behavior.
- `cargo run --bin wi-stats`: passed and reported local usage statistics plus an agent workflow audit with recorded refs/pack/impact usage.
- `cargo run --bin wi -- --help`: passed and describes direct `wi <term>` use, one-shot missing/stale auto-rebuild, refs, pack, impact, and file-reference-aware context commands.
- `cargo run --bin wi-init -- --help`: passed and says it writes AGENTS/Cursor/Copilot instructions, normalizes existing CLAUDE.md, builds `.dev_index/index.sqlite`, and does not create `WI.md`.
- `cargo run --bin wi -- -r test_repos/web-50projects pack createBoxes`: passed and surfaced `3d-boxes-background/index.html:18 file_script 3d-boxes-background/script.js`.
- `cargo run --bin wi -- --help`: passed and says `wi pack` includes useful local file references and `wi impact` includes reverse file references where available.
- `cargo run --bin wi-init -- --help`: passed and says `wi-init` does not create `WI.md`.

## Recovery Goal Assessment

| Goal | Status | Evidence |
| --- | --- | --- |
| stale/missing index self-heals | met | RECOVERY_02 tests, `agent_acceptance`, and `wi-scorecard` schema-stale recovery evidence. |
| no-change `build_index` is fast enough | met | `build_index --stats` no-change run reported `total ms: 35`, under the documented 250 ms direct-binary budget. |
| warm `wi <query>` is fast enough | met | `wi-scorecard --query build_index` reported 28 ms, under the 150 ms budget. |
| `wi doctor`, help, and init match behavior | met | Source-built and PATH `wi doctor` are ok; source and PATH version output includes schema 12; help/init docs match behavior. |
| pack and impact are useful enough | met | Product touchpoints return bounded, grouped, evidence-backed read sets with reasons and confidence labels. |
| agent instructions match behavior | met | `wi doctor` and `wi-scorecard` report current AGENTS/CLAUDE/Cursor/Copilot instruction blocks. |
| file references improve refs/pack/impact | met | PLAN_52 fixture and real-repo checks are green; this repo has 287 file references; `web-50projects` has 113 file references with 102 resolved, and `wi pack createBoxes` surfaces the HTML `file_script` edge from `index.html` to `script.js`. |

## Verification Evidence

Latest audit verification:

- `cargo fmt --check`: passed.
- `cargo test`: passed, `322 passed, 8 ignored`.
- `cargo clippy --all-targets --all-features -- -D warnings`: passed.
- `cargo test --test local_index -- --ignored`: passed.
- `cargo test --test real_repos -- --ignored`: passed, `1 passed, 3 filtered out`, because local `test_repos/` exists.
- PATH/version/install verification: passed for `which wi`, `which build_index`, PATH/source `--version`, PATH `build_index`, PATH `wi build_index`, and source doctor/build after PATH build.
- PLAN_53 install/archive verification: passed for `./install.sh`,
  `which wi-scorecard`, `wi-scorecard --version`, `scripts/check-release`, and
  standalone `scripts/smoke-release-archive dist/thinindex-0.1.4-x86_64-unknown-linux-gnu.tar.gz`.
- PLAN_54 RC verification: passed for PATH/source version and schema checks,
  installed `wi doctor`, `build_index --stats` no-change runs (`total ms: 12`),
  installed missing/stale temp-repo self-healing, installed `wi build_index`,
  `wi refs build_index`, `wi pack build_index`, `wi impact build_index`,
  installed `wi-scorecard` (`pass 10 / warn 0 / fail 0`), `./install.sh`,
  `cargo fmt --check`, `cargo test`, clippy, `scripts/check-release`,
  standalone release archive smoke, `cargo test --test local_index -- --ignored`,
  and `cargo test --test real_repos -- --ignored`.

## Remaining Caveats

- Concurrent schema-stale rebuild attempts are not treated as a normal product path. A parallel touchpoint run briefly exposed a schema-version mismatch while multiple `cargo run` commands competed to rebuild the disposable index. Serialized `build_index`, `wi`, `wi doctor`, and `wi-scorecard` paths recovered correctly and are the supported workflow.
- Current-repo `wi build_index` is useful but noisy because roadmap and recovery prompts mention `build_index` often. `wi pack build_index` and `wi impact build_index` remain bounded and more useful for agent read-set selection.
- Go and PHP remain supported by fixture/conformance coverage, but this checkout still lacks Go-heavy and PHP-heavy local manifest targets. That is documented as future real-repo hardening, not a support-claim blocker.
- Some local `test_repos/` side corpora remain exploratory until a future scoped plan adds stable expected-symbol or expected-pattern checks. Third-party repository contents must remain uncommitted.
- File-reference extraction is explicit and best-effort. It resolves local evidence, preserves unresolved local-looking paths, and does not claim package-manager, compiler, framework alias, LSP, root-relative web-base, package export-map, or network semantics.
- RC smoke has been performed for the current `x86_64-unknown-linux-gnu`
  archive. Any additional target archive still needs its own target-platform
  smoke before publishing.
- Native installers, signing, notarization, package-manager publishing, hosted
  services, telemetry, and payment/licensing enforcement remain intentionally
  unimplemented and documented as scaffolded or future work. This does not
  block the local archive RC.
- If a future schema bump lands, users must refresh installed binaries with
  `make install` or the archive installer. Current binaries now expose schema in
  `--version` and refuse index writes from a mismatched thinindex source
  checkout where practical.

## Decision

Recovery is complete enough to cut a local archive release candidate. Active
roadmap prompt files are checked complete through PLAN_54, and no high-priority
core search/index/install/package touchpoint is currently broken or slow.

Decision: `ready_with_caveats`. Proceed to target-platform RC smoke/tag prep, or
resume future roadmap work only through a new scoped plan that respects the
current guardrails:

- do not reintroduce Universal Ctags as a production parser;
- do not reintroduce `WI.md`;
- do not reintroduce JSONL as canonical storage;
- keep quality/comparator/real-repo checks out of normal `wi` and
  `build_index` paths;
- avoid packaging, licensing, payment, telemetry, cloud, hosted, and MCP work
  unless a future selected plan explicitly asks for it.

## Next Recommended Action

Run the release checklist target-platform smoke for any archive target being
published, then tag/package the RC from a clean worktree. Create a narrow
follow-up plan only if target-platform RC smoke finds a concrete blocker.
