# PLAN_46_FULL_PLAN_COMPLETION_AUDIT.md

Use superpowers:subagent-driven-development.

Do not implement this until both completed PLAN_45 workstreams are complete and green:
- `PLAN_45_TEAM_CI_AND_HOSTED_VALUE_ROADMAP.md`
- `PLAN_45_TREE_SITTER_REAL_REPO_CONVERGENCE_LOOP.md`

Goal:
Audit and align the active plan set after PLAN_45 so the project has one coherent plan inventory, accurate prerequisite chain, honest completion status, and no stale prompts/docs/code assumptions from old JSONL storage, `WI.md`, Universal Ctags as production parser, old native-parser wording, or obsolete packaging blockers.

This is an audit, cleanup, and alignment plan. It is not a product-feature plan.

Progress:
- [ ] Phase 1: rebuild/search index and re-inventory every active `prompts/PLAN_*.md` file.
- [ ] Phase 2: resolve active plan-sequence conflicts and stale plan headings/references without reviving superseded plans.
- [ ] Phase 3: verify checkbox status against implementation, tests, docs, and git evidence.
- [ ] Phase 4: clean stale prompt/docs assumptions that conflict with current product architecture.
- [ ] Phase 5: rerun consolidated verification and commit the audit cleanup.

Hard constraints:
- Do not revive superseded plans.
- Do not renumber plans unless the audit proves the active sequence is broken.
- Do not reintroduce Universal Ctags as a production parser.
- Do not reintroduce `WI.md`.
- Do not reintroduce JSONL as canonical storage.
- Do not weaken quality gates.
- Do not hide incomplete work by marking it complete.
- Do not implement new product features.
- Do not add release packaging, signing, license enforcement, payment behavior, telemetry, cloud behavior, or hosted behavior.

Current Audit Snapshot
This snapshot was created while adding PLAN_46. Re-run Phase 1 when executing the plan.

Active plan count:
- 55 active files matched `prompts/PLAN_*.md`.
- `prompts/superseded/**` was excluded.
- No unchecked `- [ ]` boxes were found in active `PLAN_*.md` files.
- No `PLAN_46` file existed before this plan was created.

Inventory Table

| Plan | Active file | Status | Checkbox state | Implementation/commit evidence | Audit note |
|---|---|---|---|---|---|
| 00 | `PLAN_00_SQLITE_INDEX_STORAGE.md` | complete | 6/6 checked | SQLite storage exists; exact old commit message not visible in current git log | Keep complete; verify no JSONL canonical storage |
| 01 | `PLAN_01_REFERENCE_GRAPH_FOUNDATION.md` | complete | 6/6 checked | SQLite refs table/model/helpers exist; old exact commit not visible | Keep complete |
| 02 | `PLAN_02_DETERMINISTIC_REFERENCE_EXTRACTION.md` | complete | 5/5 checked | Deterministic ref extraction/tests exist; old exact commit not visible | Keep complete |
| 03 | `PLAN_03_CONTEXT_COMMANDS.md` | complete | 6/6 checked | `wi refs` and `wi pack` exist; old exact commit not visible | Keep complete |
| 04 | `PLAN_04_IMPACT_COMMAND.md` | complete | 6/6 checked | `wi impact` exists; old exact commit not visible | Keep complete |
| 05 | `PLAN_05_AGENT_VALUE_BENCHMARKS.md` | complete | 6/6 checked | `wi bench`/bench tests exist; old exact commit not visible | Keep complete |
| 06 | `PLAN_06_AGENT_WORKFLOW_INTEGRATION.md` | complete | 6/6 checked | agent instruction generation exists; old exact commit not visible | Keep complete |
| 07 | `PLAN_07_REAL_REPO_BENCHMARK_SET.md` | complete | 6/6 checked | real-repo manifest support exists; old exact commit not visible | Keep complete |
| 08 | `PLAN_08_INSTALL_AND_RELEASE_HARDENING.md` | complete | 6/6 checked | install/release tests/docs exist; old exact commit not visible | Stale ctags/native-parser blocker text must be audited |
| 09 | `PLAN_09_DOCUMENTATION_AND_PRODUCT_POSITIONING.md` | complete | 5/5 checked | README/roadmap governance exists; old exact commit not visible | Stale ctags/native-parser blocker text must be audited |
| 10 | `PLAN_10_PRICING_BOUNDARY_AND_PRO_EDITION.md` | complete | 5/5 checked | product boundary docs/tests exist; old exact commit not visible | Stale ctags/native-parser blocker text must be audited |
| 11A | `PLAN_11A_TREE_SITTER_EXTRACTION_FRAMEWORK.md` | complete | 5/5 checked | `4153735 Add Tree-sitter extraction framework` | Keep complete |
| 11B | `PLAN_11B_REPRESENTATIVE_LANGUAGE_PACK.md` | complete | 7/7 checked | `47ae249 Add representative Tree-sitter language pack` | Keep complete |
| 11C | `PLAN_11C_PARSER_CONFORMANCE_AND_DOCS.md` | complete | 7/7 checked | `4c957bd Harden Tree-sitter parser conformance and docs` | Replace stale `PLAN_12_EXTENDED_LANGUAGE_PACK` reference |
| 12A | `PLAN_12A_EXTENDED_JVM_DOTNET_LANGUAGE_PACK.md` | complete | 6/6 checked | `e7a6c83 Add extended JVM and .NET parser pack` | Keep complete |
| 12B | `PLAN_12B_EXTENDED_APP_SYSTEM_LANGUAGE_PACK.md` | complete | 6/6 checked | `9fc0675 Add extended app and system parser pack` | Keep complete |
| 12C | `PLAN_12C_EXTENDED_WEB_DOC_CONFIG_LANGUAGE_PACK.md` | complete | 6/6 checked | `fcc2c97 Add extended web doc and config parser pack` | Keep complete |
| 12D | `PLAN_12D_EXTENDED_LANGUAGE_PACK_CONFORMANCE_AND_DOCS.md` | complete | 6/6 checked | `bd6ea74 Finalize extended language pack conformance` | Keep complete |
| 12E | `PLAN_12E_REAL_REPO_LANGUAGE_HARDENING.md` | complete | 6/6 checked | `a9a6e0b Harden parser support on real repos` | Keep complete |
| 12F | `PLAN_12F_PARSER_COVERAGE_CLOSURE.md` | complete | 6/6 checked | `eb94b2d Close parser symbol coverage gaps` | Keep complete |
| 12G | `PLAN_12G_PARSER_PERFORMANCE_AND_REGRESSION_GATES.md` | complete | 6/6 checked | `1573318 Add parser performance regression gates` | Keep complete |
| 13 | `PLAN_13_LICENSE_AUDIT_AND_THIRD_PARTY_NOTICES.md` | complete | 8/8 checked | `c4da535 Add license audit and third-party notices` | Replace stale `PLAN_11A through PLAN_11E` reference |
| 14 | `PLAN_14_CROSS_PLATFORM_RELEASE_ARCHIVES.md` | complete | 9/9 checked | `f353189 Add cross-platform release archives` | Keep complete |
| 15 | `PLAN_15_INSTALLERS_AND_SIGNING.md` | complete | 9/9 checked | `4f6ccc5 Add installer and signing scaffolding` | Keep complete |
| 16 | `PLAN_16_RELEASE_AUTOMATION_AND_CI.md` | complete | 9/9 checked | `70e48db Add release automation and CI gates` | Keep complete |
| 17 | `PLAN_17_OPTIONAL_COMPARATOR_QUALITY_PLUGIN.md` | complete | 6/6 checked | `71df4ba Add optional comparator quality plugin` | Keep ctags comparator isolated |
| 18 | `PLAN_18_INDEX_QUALITY_DRIFT_PLUGIN_GATES.md` | complete | 7/7 checked | `0cba5e3 Add index quality drift gates` | Keep complete |
| 19 | `PLAN_19_CONTINUOUS_QUALITY_IMPROVEMENT_PLUGIN_LOOP.md` | complete | 7/7 checked | `6dd8809 Add continuous quality improvement loop` | Keep complete |
| 20 | `PLAN_20_SUPPORT_LEVELS_AND_LANGUAGE_CLAIMS.md` | complete | 6/6 checked | `aaa39e8 Add parser support levels and claim guards` | Keep complete |
| 21 | `PLAN_21_CTAG_ALLOWLIST_AND_FORBIDDEN_SURFACE_GATES.md` | complete | 5/5 checked | `e5b7966 Add ctags allowlist gates` | No explicit commit instruction, but commit exists |
| 22 | `PLAN_22_EXPECTED_SYMBOL_MANIFEST_EXPANSION.md` | complete | 6/6 checked | `45642a5 Expand expected symbol manifest checks` | No explicit commit instruction, but commit exists |
| 23 | `PLAN_23_COMPARATOR_TRIAGE_WORKFLOW.md` | complete | 5/5 checked | `361b69a Add comparator triage workflow` | No explicit commit instruction, but commit exists |
| 24 | `PLAN_24_SINGLE_CYCLE_QUALITY_IMPROVEMENT_RUNNER.md` | complete | 6/6 checked | `3385e83 Add single-cycle quality improvement runner` | Keep complete |
| 25 | `PLAN_25_QUALITY_PLUGIN_REPORT_EXPORTS.md` | complete | 6/6 checked | `bd308ba Add quality report exports` | JSONL here is quality export only, not storage |
| 26 | `PLAN_26_LANGUAGE_SUPPORT_DASHBOARD_DOC.md` | complete | 7/7 checked | `7d05ae6 Add generated language support dashboard` | Keep complete |
| 27 | `PLAN_27_REAL_REPO_MANIFEST_CURATION.md` | complete | 7/7 checked | `28ecca6 Curate real-repo manifest schema` | Keep local `test_repos/` uncommitted |
| 28 | `PLAN_28_PARSER_QUERY_MAINTENANCE_GUIDE.md` | complete | 6/6 checked | `f2292cb Add parser query maintenance guide` | Keep complete |
| 29 | `PLAN_29_QUALITY_PLUGIN_CI_READINESS.md` | complete | 6/6 checked | `0eb6d0d Make quality gates CI-ready` | Keep complete |
| 30 | `PLAN_30_QUALITY_SYSTEM_FINAL_AUDIT.md` | complete | 6/6 checked | `38feb14 Finalize parser quality system audit` | Keep complete |
| 31 | `PLAN_31_DEPENDENCY_GRAPH_FOUNDATION.md` | complete | 7/7 checked | `c9c7e5a Add dependency graph foundation` | Header name differs but number matches |
| 32 | `PLAN_32_IMPORT_MODULE_RESOLUTION_PACKS.md` | complete | 5/5 checked | `d87e53e Add import module resolution packs` | Keep complete |
| 33 | `PLAN_33_REFERENCE_GRAPH_V2.md` | complete | 6/6 checked | `c951ffd Upgrade reference graph evidence` | Keep complete |
| 34 | `PLAN_34_IMPACT_V2_DEPENDENCY_AWARE.md` | complete | 6/6 checked | `e834d9d Add dependency-aware impact analysis` | Header says old PLAN_33 name |
| 35 | `PLAN_35_CONTEXT_PACK_V2_DEPENDENCY_AWARE.md` | complete | 6/6 checked | `d986b4c Add dependency-aware context packs` | Keep complete |
| 36 | `PLAN_36_TEST_BUILD_CONFIG_MAPPING.md` | complete | 6/6 checked | `a59b8b9 Add test build config mappings` | Keep complete |
| 37 | `PLAN_37_MONOREPO_SCALE_AND_INCREMENTAL_INDEXING.md` | complete | 6/6 checked | `a22ef68 Add monorepo indexing safeguards` | Keep complete |
| 38 | `PLAN_38_OPTIONAL_SEMANTIC_ADAPTER_BOUNDARY.md` | complete | 6/6 checked | `9ef7da4 Add optional semantic adapter boundary` | Header says old PLAN_32 name |
| 39 | `PLAN_39_AGENT_WORKFLOW_ENFORCEMENT_AND_INTEGRATION_PACKS.md` | complete | 5/5 checked | `ccff686 Add agent workflow integration packs` | Header says old PLAN_34 name |
| 40 | `PLAN_40_TECHNICAL_FINAL_AUDIT.md` | complete | 4/4 checked | `30d05e9 Add technical final audit` | Keep complete |
| 41 | `PLAN_41_SECURITY_PRIVACY_AND_REPORT_REDACTION.md` | complete | 5/5 checked | `93c75a5 Add security privacy redaction policy` | Header says old PLAN_38 name |
| 42 | `PLAN_42_SIGNED_INSTALLER_AND_DISTRIBUTION_HARDENING.md` | complete | 5/5 checked | `4098b45 Harden signed installer distribution` | Header says old PLAN_36 name |
| 43 | `PLAN_43_PRO_LICENSING_FOUNDATION_NO_ENFORCEMENT.md` | complete | 5/5 checked | `14b82b1 Add licensing foundation without enforcement` | Header says old PLAN_35 name |
| 44 | `PLAN_44_ONBOARDING_DOCTOR_AND_PRODUCT_POLISH.md` | complete | 5/5 checked | `3d323ba Add onboarding doctor and product polish` | Header says old PLAN_39 name |
| 45A candidate | `PLAN_45_TEAM_CI_AND_HOSTED_VALUE_ROADMAP.md` | complete | 5/5 checked | `6ff426f Add team CI hosted value roadmap` | Active duplicate PLAN_45; header says old PLAN_40 name |
| 45 | `PLAN_45_TREE_SITTER_REAL_REPO_CONVERGENCE_LOOP.md` | complete | 5/5 checked | `961e527 Converge Tree-sitter real-repo quality cycle`; `bf77c71 Align Tree-sitter convergence quality state` | Active duplicate PLAN_45; recent audit cleanup exists |

Missing Plan Files
- No required active plan file is missing for the observed sequence `PLAN_00` through `PLAN_45`, with lettered `PLAN_11A` through `PLAN_11C` and `PLAN_12A` through `PLAN_12G`.
- There is no active `PLAN_11D`, `PLAN_11E`, or monolithic `PLAN_12_EXTENDED_LANGUAGE_PACK`; active references to those names are stale.
- `PLAN_46_FULL_PLAN_COMPLETION_AUDIT.md` is created by this plan-addition pass.

Duplicate Or Conflicting Plan Files
- Two active files use `PLAN_45`:
  - `PLAN_45_TEAM_CI_AND_HOSTED_VALUE_ROADMAP.md`
  - `PLAN_45_TREE_SITTER_REAL_REPO_CONVERGENCE_LOOP.md`
- Cleanup should resolve this active sequence break. Preferred cleanup is to rename the team/CI roadmap file to `PLAN_45A_TEAM_CI_AND_HOSTED_VALUE_ROADMAP.md`, update its heading, and make PLAN_46 depend on both `PLAN_45A` and `PLAN_45`. If tooling or history conventions reject lettered `45A`, document the duplicate explicitly instead of silently leaving it ambiguous.
- Several active headings are stale from older numbering and should be aligned with filenames:
  - `PLAN_34_IMPACT_V2_DEPENDENCY_AWARE.md`
  - `PLAN_38_OPTIONAL_SEMANTIC_ADAPTER_BOUNDARY.md`
  - `PLAN_39_AGENT_WORKFLOW_ENFORCEMENT_AND_INTEGRATION_PACKS.md`
  - `PLAN_41_SECURITY_PRIVACY_AND_REPORT_REDACTION.md`
  - `PLAN_42_SIGNED_INSTALLER_AND_DISTRIBUTION_HARDENING.md`
  - `PLAN_43_PRO_LICENSING_FOUNDATION_NO_ENFORCEMENT.md`
  - `PLAN_44_ONBOARDING_DOCTOR_AND_PRODUCT_POLISH.md`
  - `PLAN_45_TEAM_CI_AND_HOSTED_VALUE_ROADMAP.md`
- `PLAN_31_DEPENDENCY_GRAPH_FOUNDATION.md` has a descriptive heading that differs from the filename but keeps the same number. Normalize only if the audit chooses filename/header exactness as the rule.

Prerequisite-Chain Problems
- The main prerequisite chain is coherent through `PLAN_44`.
- Both active `PLAN_45` files depend on `PLAN_44`, so the chain forks at 45. Resolve the fork before adding further product plans.
- Stale prerequisite or cross-plan references found:
  - `PLAN_08_INSTALL_AND_RELEASE_HARDENING.md` references broad `PLAN_11` cleanup in a way that predates the rewritten `PLAN_11A` through `PLAN_11C` series.
  - `PLAN_11C_PARSER_CONFORMANCE_AND_DOCS.md` references non-existent `PLAN_12_EXTENDED_LANGUAGE_PACK`.
  - `PLAN_13_LICENSE_AUDIT_AND_THIRD_PARTY_NOTICES.md` references `PLAN_11A through PLAN_11E`, but active 11-series ends at `PLAN_11C`.
  - `PLAN_14`, `PLAN_15`, and `PLAN_16` mention release packaging from old plan numbers in a few historical implementation instructions; verify whether they now mean `PLAN_14_CROSS_PLATFORM_RELEASE_ARCHIVES.md`.

Incomplete Checkbox Phases
- None found in active `PLAN_*.md` files at snapshot time.
- Phase 1 must rerun `rg -n "^- \\[ \\]" prompts/PLAN_*.md || true` before changing any boxes.

Marked Complete Without Implementation Evidence
- None found as a hard blocker in this snapshot.
- Caveat: exact expected commit messages for `PLAN_00` through `PLAN_10` are not visible in the current `git log --all`; implementation evidence exists in code/tests/docs and later plans verified those prerequisites. PLAN_46 should either document this history gap or stop treating those old commit messages as required evidence.

Implementation Evidence With Stale Unchecked Boxes
- None found.

Stale Docs, Prompts, Or Code Assumptions
- `prompts/local_repo_test.md` is not an active `PLAN_*.md` file but still describes `.dev_index/index.jsonl` and raw JSONL shared checks. It should be updated to SQLite or moved/marked superseded.
- `PLAN_08`, `PLAN_09`, and `PLAN_10` contain old ctags/native-parser packaging blocker wording. The current architecture is Tree-sitter production parsing with Universal Ctags isolated as optional quality comparator only.
- Active plans intentionally mention `WI.md`, JSONL, and ctags in many "do not reintroduce" or boundary contexts. Do not remove these guardrails. Remove or rewrite only text that claims they are current product surfaces.
- JSONL quality exports such as `.dev_index/quality/QUALITY_REPORT_DETAILS.jsonl` are intentional report artifacts. They are not canonical index storage.
- `source = "ctags"` may appear in quality gates/tests as forbidden-source assertions. It must not appear in production records/refs.

Leftover Work By Area

Parser / Tree-sitter:
- Reconfirm Tree-sitter remains the only code-symbol parser path.
- Reconfirm no line-oriented, regex, native transitional parser, or production ctags parser path exists.
- Fix stale prompt references to monolithic/old parser plans.
- Do not expand language support unless a declared supported-language regression is found.

Quality Plugin:
- Keep Universal Ctags optional, external, and isolated to quality comparator/gate/docs/tests.
- Confirm ctags allowlist gate still passes.
- Confirm quality report JSON/JSONL artifacts do not pollute production SQLite `records` or `refs`.
- Keep comparator-only findings classified rather than promoting them to parser requirements without evidence.

Real Repos:
- Keep `test_repos/` local-only and ignored.
- Confirm local `test_repos/MANIFEST.toml` entries, if present, have accurate queries, expected symbols, expected patterns, absent symbols, thresholds, skip reasons, and unsupported syntax notes.
- Run ignored real-repo tests when `test_repos/` exists.
- Do not commit third-party repo contents.

Dependency Graph:
- Reconfirm dependency tables/edges are in SQLite and schema-versioned.
- Reconfirm import/module resolver docs match implemented confidence and ambiguity behavior.
- Verify dependency graph does not duplicate reference graph responsibilities.

Refs / Pack / Impact:
- Reconfirm `wi refs`, `wi pack`, and `wi impact` use SQLite `records`, `refs`, and dependency evidence.
- Reconfirm every impact row has a concrete file:line reason.
- Reconfirm pack/impact outputs stay deterministic, compact, deduplicated, and do not dump full files.

Packaging / Licensing:
- Reconfirm release archives exclude `.dev_index/`, `.dev_index/quality/`, `test_repos/`, build output, local reports, signing secrets, and ctags binaries.
- Reconfirm `cargo deny check licenses` policy is current.
- Update stale prompts that still describe proprietary packaging as blocked by required ctags after Tree-sitter became production parser.
- Do not add real signing, payment, license enforcement, or hosted distribution behavior.

Onboarding / Docs:
- Reconfirm `AGENTS.md`, generated `CLAUDE.md`, and `wi-init` stay aligned with the canonical Repository search block.
- Keep `wi --help` as the source of truth for filters/examples/subcommands.
- Update `prompts/local_repo_test.md` away from old JSONL index assumptions.
- Align stale active plan headings with filenames after resolving duplicate PLAN_45.

Agent Integration:
- Reconfirm `wi-stats` agent workflow audit remains local and does not imply external telemetry.
- Reconfirm docs instruct agents to use `build_index`, `wi`, `wi pack`, and `wi impact` before broad fallback search.
- Do not reintroduce `WI.md`.

Execution Phases

Phase 1 - Re-inventory:
1. Run `cargo run --bin build_index`.
2. Run `ls prompts/PLAN_*.md | sort`.
3. Count active files and check for duplicate plan numbers.
4. Run `rg -n "^- \\[[ xX]\\]" prompts/PLAN_*.md` and `rg -n "^- \\[ \\]" prompts/PLAN_*.md || true`.
5. Run `rg -n "^# PLAN_" prompts/PLAN_*.md` and compare headings to filenames.
6. Run `git log --oneline --all` and map expected commit instructions where present.

Phase 2 - Sequence and prompt cleanup:
1. Resolve the duplicate active PLAN_45 sequence using the preferred `PLAN_45A_TEAM_CI_AND_HOSTED_VALUE_ROADMAP.md` rename unless a better repository convention is documented.
2. Update stale headings to match active filenames.
3. Update stale cross-plan references:
   - `PLAN_11` broad parser cleanup -> rewritten `PLAN_11A` through `PLAN_11C` as appropriate.
   - `PLAN_11A through PLAN_11E` -> `PLAN_11A through PLAN_11C`.
   - `PLAN_12_EXTENDED_LANGUAGE_PACK` -> `PLAN_12A` through `PLAN_12G` or the exact relevant plan.
   - old release-packaging plan-number references -> current `PLAN_14` through `PLAN_16` names.
4. Do not move plans into `prompts/superseded/` unless the audit proves they are truly superseded, not merely duplicate-numbered completed work.

Phase 3 - Evidence audit:
1. For each inventory row, confirm one of: complete, partial, not started, blocked, or superseded.
2. If a plan is marked complete but lacks evidence, change only the audit status in PLAN_46 and list the gap. Do not mark it complete by assumption.
3. If implementation evidence exists but checkboxes are stale, update only the relevant plan checkboxes after verification.
4. Confirm old exact commit-message gaps for PLAN_00 through PLAN_10 are either explained by current history state or linked to recovered evidence.

Phase 4 - Stale assumption cleanup:
1. Update `prompts/local_repo_test.md` to SQLite index assumptions or mark it as superseded/local legacy guidance.
2. Update old ctags/native-parser packaging blocker text in active prompts so it reflects Tree-sitter production parsing and optional ctags comparator boundaries.
3. Preserve explicit guardrails saying not to reintroduce `WI.md`, JSONL canonical storage, or ctags production parsing.
4. Verify no current docs claim unsupported languages are fully supported.

Phase 5 - Verification and commit:
1. Run the lightweight validation commands listed below.
2. Run normal Rust verification if any source, test, script, or generated docs behavior changes.
3. Run ignored local/real-repo verification if real-repo manifest, quality gates, parser support, refs, pack, impact, or dependency evidence changed.
4. Commit only audited cleanup changes once verification is green.

Verification

Always run:
```sh
ls prompts/PLAN_*.md | sort
grep -R "Do not implement this until" prompts/PLAN_*.md
grep -R "WI.md\\|index.jsonl\\|source = \"ctags\"\\|Universal Ctags\\|ctags" prompts README.md docs src tests Cargo.toml install.sh uninstall.sh 2>/dev/null || true
rg -n "^- \\[ \\]" prompts/PLAN_*.md || true
rg -n "^# PLAN_" prompts/PLAN_*.md
```

Run if any code, test, script, generated-doc, or behavior-affecting file changes:
```sh
cargo fmt --check
cargo test
cargo clippy --all-targets --all-features -- -D warnings
cargo test --test quality_ctags_allowlist
cargo deny check licenses
cargo run --bin build_index
cargo run --bin wi -- build_index
cargo run --bin wi -- refs build_index
cargo run --bin wi -- pack build_index
cargo run --bin wi -- impact build_index
cargo run --bin wi-stats
```

Run when `test_repos/` exists and Plan 46 touches real-repo, parser, quality, dependency, refs, pack, or impact assumptions:
```sh
cargo test --test local_index -- --ignored
cargo test --test real_repos -- --ignored
cargo test --test quality_gates -- --ignored
cargo test --test quality_loop -- --ignored
```

Acceptance Criteria
- Every active plan has a clear status: complete, partial, not started, blocked, or superseded.
- All leftover work is listed or explicitly ruled out.
- Active prerequisite chain is coherent after resolving the duplicate PLAN_45 fork.
- Stale references are either fixed or explicitly classified as intentional guardrails.
- Verification requirements are consolidated.
- Next action is unambiguous: execute Phase 1, then resolve the duplicate PLAN_45 sequence and stale headings/references before any future product plan.

Commit Instructions
- If executing this plan changes only prompt/docs audit alignment, commit with:
  `Complete full plan completion audit cleanup`
- If no cleanup changes are needed after rerunning the audit, do not commit.
