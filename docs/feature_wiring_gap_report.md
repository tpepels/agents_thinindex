# Feature Wiring Gap Report

Audit date: 2026-05-07.

Audit context: this checkout is a Rust CLI repository. The worktree was already
dirty when this audit started because `docs/QUALITY.md`,
`docs/QUALITY_LOOP.md`, `tests/quality_ci_readiness.rs`, `tests/wi.rs`, and the
untracked `prompts/PLAN_60_OPTIONAL_QUALITY_CLI_DECISION.md` had in-progress
PLAN_60 changes. This report does not implement product features.

## 1. Executive summary

thinindex claims to be a local, repo-scoped navigation layer for coding
repositories. It builds `.dev_index/index.sqlite`, indexes code and format
landmarks, and exposes file:line search plus evidence-backed `refs`, `pack`,
and `impact` workflows for agents and developers.

Actually working: the core CLI loop is real, wired, and proven in this checkout.
`build_index --stats` produced a fresh no-change build with `changed files: 0`,
`records: 3450`, `refs: 3990`, `dependencies: 146`, `file references: 285`,
and `quality/comparator phases: 0`. `wi`, `wi refs`, `wi pack`, `wi impact`,
`wi doctor`, `wi-stats`, `wi-scorecard`, and `wi-init --dry-run` all ran
successfully from source. Normal Rust checks passed.

Partially wired: optional maintainer workflows such as quality reports,
comparator triage, real-repo hardening, and semantic facts have real code and
tests, but they are intentionally not part of normal user indexing/search.
Agent integration is repo-local instruction generation and advisory usage
auditing, not hard enforcement.

Stubbed or test-only: the inert licensing foundation has fixture-only Pro
status and no payment/account enforcement. Semantic adapters/facts have an
isolated model and tests but are disabled by default. MCP, hosted/team
features, native package publishing, real signing/notarization, managed updates,
telemetry, payment, account login, cloud sync, package-manager resolution, and
LSP/compiler semantics are missing or explicitly deferred.

Readiness: ready for limited dogfood as a local CLI/navigation tool. Not ready
as a production-grade distributed/commercial product because native packages,
signing/notarization, hosted/team surfaces, real license/payment flows, and
always-reproducible real-repo quality evidence are not wired.

## 2. Feature matrix

| Feature | Claimed behavior | Current status | Evidence | Gap | Next action |
| --- | --- | --- | --- | --- | --- |
| Core local indexing | Build/update repo-local SQLite index. | Real and wired | `src/bin/build_index.rs`, `src/indexer.rs`, `src/store.rs`; `cargo run --bin build_index -- --stats` passed with schema 12 data and no quality phases. | No full semantic/compiler resolution. | Keep local deterministic scope; avoid semantic overclaims. |
| SQLite storage/schema | `.dev_index/index.sqlite` is canonical storage. | Real and wired | `src/model.rs` has `INDEX_SCHEMA_VERSION: 12`; `src/store.rs` creates `meta`, `files`, `records`, `refs`, `dependencies`, `file_references`, `semantic_facts`, `usage_events`. | No migrations; schema mismatch rebuilds disposable index. | Keep schema-bearing version and rebuild tests. |
| `wi <term>` search | Return compact file:line landmarks. | Real and wired | `src/bin/wi.rs`, `src/search.rs`; `cargo run --bin wi -- build_index` returned source/test/doc landmarks. | Broad terms can match docs/plans and be noisy. | Prefer `pack`/`impact` for implementation context. |
| Auto-build/self-healing search | Missing/stale indexes rebuild once before search. | Real and wired | `src/bin/wi.rs::ensure_index_ready_once`; tests in `tests/wi.rs` and `tests/schema_version.rs`; runtime `wi` rebuilt stale index before continuing during audit. | No concurrent rebuild coordination claim. | Keep one-shot behavior and clear stderr notices. |
| `wi refs` | Show references with reason/confidence evidence. | Real and wired | `src/context.rs`, `src/refs.rs`; `cargo run --bin wi -- refs build_index` showed primary definitions, reasons, evidence, and confidence labels. | Heuristic text fallback can be noisy. | Continue precision hardening from real repo evidence. |
| `wi pack` | Return bounded read set for implementation. | Real and wired | `src/context.rs`; `cargo run --bin wi -- pack build_index` produced grouped rows with reasons/confidence. | Best-effort dependency evidence, not complete program analysis. | Keep bounded defaults and caveats. |
| `wi impact` | Return likely affected files before edits. | Real and wired | `src/context.rs`; `cargo run --bin wi -- impact build_index` produced definitions, refs, dependent files, tests, docs, config. | Not exhaustive impact analysis. | Keep "likely/plausible" wording. |
| `wi doctor` | Report setup/index/schema/binary health. | Real and wired | `src/doctor.rs`; `cargo run --bin wi -- doctor` reported `overall: ok`, schema 12, fresh index, binary/source match. | Quality state is advisory. | Keep as primary status command. |
| `wi bench` | Local benchmark report. | Real but not fully proven | `src/bin/wi.rs` dispatches `WiCommand::Bench`; `src/bench.rs`; tests in `tests/bench.rs`. | Not run in this audit; benchmark numbers are machine/repo specific. | Run for repo-specific evaluation, not as universal claim. |
| `wi-init` | Generate/normalize repo-local agent guidance. | Real and wired | `src/bin/wi-init.rs`; help shows AGENTS/Cursor/Copilot/CLAUDE behavior and `--dry-run`; tests in `tests/wi_init.rs`. | Agent compliance cannot be enforced. | Keep advisory and repo-local. |
| `wi-stats` | Show local usage and advisory workflow audit. | Real and wired | `src/bin/wi-stats.rs`, `src/stats.rs`; runtime reported 30 events and context command usage. | Cannot detect external grep/find/ls/Read. | Keep as advisory diagnostics. |
| `wi-scorecard` | Pass/warn/fail evidence for product loop. | Real and wired | `src/bin/wi-scorecard.rs`, `src/scorecard.rs`; runtime pass 9/warn 1/fail 0. | Fresh-index run cannot prove recovery dimension. | Run on stale/missing index when testing recovery. |
| Tree-sitter parser support | Supported languages use Tree-sitter extraction. | Real and wired | `Cargo.toml` tree-sitter deps; `src/tree_sitter_extraction.rs`; `tests/parser_conformance.rs`. | Syntactic only; no LSP/compiler semantics. | Keep support matrix and conformance gates. |
| Extras-backed formats | CSS/HTML/Markdown/JSON/TOML/YAML landmarks. | Real and wired | `src/extras.rs`, `docs/LANGUAGE_SUPPORT.md`, conformance tests. | Not full AST/semantic parsing. | Keep extras-backed label. |
| Dependency graph | Store deterministic local dependency edges. | Real and wired | `src/deps.rs`, `src/store.rs`, `tests/build_index.rs`. | Package manager, compiler, alias, export-map semantics are partial/missing. | Add targeted resolver hardening only from evidence. |
| File references | Store and surface local file references. | Real and wired | `src/file_refs.rs`, `src/context.rs`, `tests/file_refs.rs`; stats reported 285 file references. | Best-effort local path resolution only. | Keep unresolved reasons visible. |
| Semantic facts/adapters | Optional semantic fact table and adapter boundary. | Implemented but not wired | `src/semantic.rs`, `src/model.rs` semantic structs/table; `build_index --stats` reported `semantic facts: 0`. | Disabled by default; normal `wi` paths do not consume semantic facts. | Defer until scoped semantic-adapter plan. |
| Quality reports | Local quality summaries under `.dev_index/quality/`. | Implemented but not wired | `src/quality/export.rs`, `docs/QUALITY.md`, `tests/quality.rs`. | Not normal CLI; maintainer/test workflow only. | Keep isolated; no `wi quality` until command contract is designed. |
| Optional comparator | External tagger comparison for QA only. | Implemented but not wired | `src/quality/comparator.rs`, quality boundary tests, `docs/QUALITY.md`. | Optional external binary; not packaged; not production parser. | Keep optional/manual boundary. |
| Quality CLI | Normal `wi quality ...` workflow. | Missing | `src/wi_cli.rs` commands are search/refs/pack/impact/doctor/bench only; `wi --help` has no `wi quality`. | Quality remains tests/scripts/docs. | Explicitly defer or design bounded optional CLI later. |
| Real-repo hardening | Optional checks against local `test_repos/`. | Real but not fully proven | `tests/real_repos.rs --ignored` passed locally in 233.88s; `test_repos/` is ignored/local. | Depends on uncommitted local repos; not normal CI proof. | Keep manual; add committed synthetic mini-corpora if needed. |
| Agent integration packs | Guidance for Codex/Claude/Cursor/Copilot/generic agents. | Real but not fully proven | `docs/AGENT_INTEGRATION.md`, `integrations/agents/*`, `tests/agent_integration.rs`. | Guidance is advisory, not enforcement. | Keep claims limited to local instruction files. |
| MCP integration | Local MCP/tool wrapper. | Missing | `integrations/agents/mcp/README.md` says no MCP server/helper is bundled. | No handler/server/client config. | Separate MCP plan if needed. |
| Licensing foundation | Local/free product, inert future license model. | Stubbed/test-only | `docs/PRODUCT_BOUNDARY.md`, `docs/LICENSING.md`, tests mention fixture-only license status. | No payment/account/enforcement; Pro accepted only as local test fixture. | Do not market paid gating as implemented. |
| Hosted/team/CI value | Future team/hosted/reporting workflows. | Missing | `docs/TEAM_CI_ROADMAP.md`, `docs/CI_INTEGRATION.md`, `docs/PRODUCT_BOUNDARY.md`. | No backend, accounts, uploads, telemetry, cloud sync, CI service. | Roadmap only. |
| Install scripts | Install/uninstall source-built binaries. | Real and wired | `install.sh`, `uninstall.sh`, `tests/install_scripts.rs`. | Not rerun in audit to avoid mutating installed PATH binaries. | Keep script tests and release smoke. |
| Release archives | Package binaries/docs/scripts/notices/SBOM. | Real but not fully proven | `scripts/package-release`, `scripts/check-release`, `scripts/smoke-release-archive`, release tests. | This audit did not run full archive packaging; target-platform smoke still needed. | Run `scripts/check-release` before publication. |
| Native packages/signing/notarization | Production distribution polish. | Missing | `docs/INSTALLERS.md`, `docs/RELEASING.md`, `scripts/sign-release-artifact` scaffolding. | Requires platform tooling/credentials/policy. | Separate release/distribution plan. |
| Security/privacy boundaries | Local-only, no telemetry/source upload, redaction in reports. | Real but not fully proven | `docs/SECURITY_PRIVACY.md`, `src/privacy.rs`, quality report tests. | Redaction is not a secret scanner. | Keep `.thinindexignore` guidance. |

## 3. User-path wiring

| User path | Entry point | Data source | Core logic | State output | UI/API output | Proven by | Gap |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Explicit index build | `build_index [--stats]` in `src/bin/build_index.rs` | Repository files honoring ignore rules; existing SQLite manifest | `indexer::build_index`, Tree-sitter/extras, refs/deps/file refs | `.dev_index/index.sqlite` tables and meta schema 12 | CLI summary and optional phase timings | `cargo run --bin build_index -- --stats`, `tests/build_index.rs`, `scripts/check-build-performance` | Syntactic, not semantic/compiler complete. |
| Search | `wi <term>` in `src/bin/wi.rs` | `.dev_index/index.sqlite`; auto-rebuild if stale | `search::search`, ranking/filtering | Usage event appended to SQLite | File:line rows | `cargo run --bin wi -- build_index`, `tests/wi.rs` | Broad text can be noisy. |
| References | `wi refs <term>` | SQLite records, refs, dependencies, file refs | `context::render_refs_command` | Usage event | Primary/ref rows with reason/evidence/confidence | Runtime refs command, `tests/wi.rs`, `tests/file_refs.rs` | Heuristic rows are not semantic proof. |
| Pack | `wi pack <term>` | SQLite graph and file roles | `context::render_pack_command` | Usage event | Bounded grouped read set | Runtime pack command, `tests/wi.rs` | Best-effort context set. |
| Impact | `wi impact <term>` | SQLite graph, refs, file roles | `context::render_impact_command` | Usage event | Bounded affected-file groups | Runtime impact command, `tests/wi.rs` | Not exhaustive impact. |
| Doctor | `wi doctor` | Index metadata, repo files, binary/source state, license state | `doctor::run_doctor` | None | Health report | Runtime doctor command | Advisory quality/license status. |
| Bench | `wi bench` | Repo index, benchmark query set | `bench::run_benchmark` | No usage event for bench | Benchmark report | Tests in `tests/bench.rs`; not run in this audit | Runtime benchmark not sampled here. |
| Agent setup | `wi-init [--dry-run]` | Repo files and templates | `agent_instructions::normalize_repository_search_block`; `build_index` in normal mode | AGENTS/Cursor/Copilot, existing CLAUDE, `.thinindexignore`, `.gitignore`, index | Init/dry-run CLI report | `cargo run --bin wi-init -- --dry-run`, `tests/wi_init.rs` | Advisory instructions only. |
| Usage stats | `wi-stats` | SQLite `usage_events` | `stats::compute_windows`, workflow audit | None | Usage table/graph/audit | Runtime `wi-stats`, `tests/wi_stats.rs` | Cannot see non-`wi` tool usage. |
| Scorecard | `wi-scorecard` | Repo state and core CLI modules | `scorecard::run_scorecard` | May rebuild if needed | Pass/warn/fail report | Runtime `wi-scorecard`, `tests/scorecard.rs` | Fresh run warns recovery not observed. |
| Quality reports | Tests/quality modules, not user CLI | SQLite snapshots, manifests, optional comparator | `quality::*` modules | `.dev_index/quality/*` local reports | Test output/report files | `cargo test`, quality tests | Not wired to normal product path. |
| Release archive | `scripts/package-release`, `scripts/check-release` | Built binaries/docs/scripts/notices | Shell scripts and package-content checks | `dist/*.tar.gz`, sidecar, SBOM | Script output | Release tests; not rerun in this audit | Target-platform smoke still external. |

## 4. Core vertical slice

Main loop:

```text
repository files
  -> ignore/filter/discover
  -> Tree-sitter/extras extraction plus refs/deps/file refs
  -> SQLite persistence in .dev_index/index.sqlite
  -> wi search/refs/pack/impact output
  -> local usage stats/scorecard/doctor diagnostics
```

| Step | Status | Evidence | Gap |
| --- | --- | --- | --- |
| Repository input | Real and wired | `build_index --stats` scanned 283 files; ignore docs/templates exist. | Generated/vendor/sensitive paths require correct ignores. |
| Extraction | Real and wired | Tree-sitter deps in `Cargo.toml`; `tests/parser_conformance.rs`; extras docs and tests. | Syntactic only, no compiler/LSP/package-manager semantics. |
| Relationship processing | Real and wired | `refs: 3990`, `dependencies: 146`, `file references: 285` in runtime stats. | Best-effort heuristics and local path resolution. |
| Persistence | Real and wired | `src/store.rs` transactional save and schema validation; schema 12 runtime. | Disposable rebuild instead of migrations. |
| User output | Real and wired | Runtime `wi`, `refs`, `pack`, `impact`, `doctor`, `stats`, `scorecard` commands passed. | Output is CLI-only; no UI/API server. |
| Diagnostics/value proof | Real and wired | `wi doctor`, `wi-stats`, `wi-scorecard`, performance guard all passed. | Scorecard is advisory and context-dependent. |
| Quality loop | Implemented but not wired | `src/quality/*`, docs, tests. | Maintainer/test workflow only; no normal CLI. |

## 5. Tests: what they prove and do not prove

Tests that prove real behavior:

- `cargo test --workspace --all-targets --all-features`: 339 passed, 8 ignored.
- `tests/wi.rs`: search, refs, pack, impact, help, bounded output, stale index behavior.
- `tests/build_index.rs`: indexing, stats, performance invariants, no quality phases in normal builds.
- `tests/file_refs.rs`: resolved/unresolved file refs, dedupe, stale cleanup, CLI surfacing.
- `tests/parser_conformance.rs` and `tests/support_levels.rs`: parser support claims and fixtures.
- `tests/schema_version.rs`: schema version and stale/schema rebuild behavior.
- `tests/wi_init.rs`, `tests/agent_integration.rs`: instruction generation/normalization/dry-run docs.
- `tests/quality*.rs`: deterministic quality gates, external-comparator boundary, report export, CI separation.
- `tests/real_repos.rs --ignored`: local real-repo suite passed, but depends on local corpora.

Tests that are stubbed/test-only or may give false confidence:

- Licensing tests prove inert local fixture behavior, not real payment/account/license enforcement.
- Semantic adapter tests prove isolation/model behavior, not product semantic navigation.
- Quality/comparator tests prove optional maintainer workflows, not normal user CLI behavior.
- Release automation tests prove scripts/package checks in fixture contexts, not every target platform install.
- Real-repo ignored tests prove this checkout's local `test_repos/`, not committed reproducible corpora.

Missing integration/E2E tests:

- No end-to-end test for hosted/team/CI backend because no backend exists.
- No real MCP integration test because no MCP server/helper exists.
- No package-manager/compiler/LSP semantic resolver E2E because those features are out of scope.
- No target-platform smoke for every future archive/native package in this Linux checkout.
- No true payment/license/account E2E.

## 6. External dependencies and blockers

| Dependency | Purpose | Required for | Credentials needed? | How to test | Failure behavior | Blocker? |
| --- | --- | --- | --- | --- | --- | --- |
| Rust/Cargo toolchain | Build/test/install from source | Normal development and source install | No | `cargo test`, `cargo run --bin wi -- --help` | Build commands fail | Yes for source users. |
| Bundled SQLite via `rusqlite` | Local index database | Runtime binaries | No | `build_index`, `wi doctor` | Index open/build failure | No separate system dependency. |
| Tree-sitter crates | Parser extraction | Indexing supported languages | No | Parser conformance tests | Missing parser records/tests fail | Bundled dependency. |
| `cargo-deny` | License audit | CI/release checks | No | `cargo deny check licenses` passed | Release/check-ci fails if absent or policy fails | Tooling blocker for release checks. |
| Local `test_repos/` | Real-repo hardening | Ignored/manual tests | No | `cargo test --test real_repos -- --ignored` passed | Skips/fails depending manifest/local state | Not normal product blocker. |
| Optional external tagger comparator | Comparator QA only | Ignored/manual comparator report | No | `cargo test --test quality -- --ignored` | Skips if missing | Not product blocker. |
| Shell utilities (`tar`, `sha256sum`, etc.) | Release archive scripts | Packaging/release smoke | No | `scripts/check-release` | Script failure | Release blocker only. |
| Target OS/platforms | Installer/archive smoke | Cross-platform release confidence | No credentials, but target machines needed | Run smoke on target | Untested target remains unproven | External validation blocker. |
| Signing/notarization credentials | Future trusted distribution | Native signed releases | Yes | Future signing workflow | Not implemented | External blocker for signed production distribution. |
| Hosted/cloud/account/payment services | Future team/Pro features | Hosted/team/licensing product | Yes | None present | Missing by design | External blocker for those features only. |
| Browser/UI tooling | UI testing | Not applicable | No | No UI exists | Not applicable | No. |

Python checks were skipped because no Python package/test configuration is
present. Node checks were skipped because no `package.json` is present.

## 7. UI/API readiness, if applicable

No web UI, native UI, HTTP API, daemon, or hosted service is implemented.
Product output is CLI text. The CLI uses product-facing local SQLite data for
search, refs, pack, impact, doctor, stats, and scorecard. Diagnostics are
separated into `wi doctor`, `wi-stats`, `wi-scorecard`, and `build_index
--stats`; normal `wi` stdout remains compact.

No cross-route UI staleness applies. CLI freshness is handled by one-shot
auto-build for `wi` search/context commands and explicit schema checks in the
store/doctor/version paths.

## 8. Data/storage readiness, if applicable

State files/tables:

- `.dev_index/index.sqlite`: canonical disposable repo-local cache.
- Tables: `meta`, `files`, `records`, `refs`, `dependencies`,
  `file_references`, `semantic_facts`, `usage_events`.
- `.dev_index/quality/`: optional local quality reports, not production state.
- `.thinindexignore`, `.gitignore`, `AGENTS.md`, existing `CLAUDE.md`,
  `.cursor/rules/thinindex.mdc`, `.github/copilot-instructions.md`: repo-local
  setup/instruction state.

Readiness:

- Versioning exists through `INDEX_SCHEMA_VERSION = 12` and `meta.schema_version`.
- Schema mismatch and invalid DB paths rebuild disposable `.dev_index`.
- Index writes are transactional in `save_index_snapshot`.
- No-change and changed-file behavior is tested and runtime-checked.
- Stale records/refs/dependencies/file refs are cleaned by rebuild logic.

Gaps:

- No migration framework; compatible with disposable local cache model.
- `semantic_facts` exists but normal user paths do not consume it.
- Real-repo quality state is local-only and not committed.

## 9. Operational readiness

- Setup is documented in `README.md`, `docs/GETTING_STARTED.md`, and
  `docs/USER_DOCUMENTATION.md`.
- Startup commands are clear: `wi-init`, `build_index`, `wi doctor`, `wi`,
  `wi refs`, `wi pack`, `wi impact`.
- Errors are generally human-readable through `anyhow` contexts and `wi doctor`.
- Status/doctor command exists and returned `overall: ok`.
- Logs/metrics are local CLI diagnostics: `build_index --stats`, `wi-stats`,
  `wi-scorecard`, `wi bench`.
- Reset/recovery paths exist: `wi` auto-builds once; `build_index` rebuilds
  schema-stale/invalid indexes; `wi-init --remove` removes repo-local index.
- Packaging is covered by archive scripts, install/uninstall helpers, SBOM,
  notices, package-content checks, and release docs.

Operational gaps:

- Native packages, signed/notarized releases, package-manager publishing, and
  managed updates are not implemented.
- Full release check was not rerun for this audit; release publication should
  run `scripts/check-release`.
- Real-repo quality is manual/local and slow.

## 10. Security/safety readiness

- No telemetry, source upload, account login, payment, cloud sync, or hosted
  service is implemented.
- Secrets are not intentionally stored in config, but `.dev_index` may contain
  paths, symbols, refs, and evidence strings. Docs instruct users to ignore
  secrets/sensitive paths before indexing.
- Credentials are externalized/not used for current local workflow.
- Dangerous external behavior is limited: no package-manager execution or
  network calls in normal indexing/search; optional comparator is external and
  quality-only.
- Release scripts guard against packaging `.dev_index`, quality reports,
  `test_repos`, comparator binaries, local reports, and signing secrets.
- `wi-init --dry-run` previews repo-local changes without writing.
- Audit logs are local usage events in SQLite, not telemetry.

Gaps:

- Redaction is a safety net, not a complete secret scanner.
- Agent instruction compliance is advisory.
- Signed distribution and trusted update channels are missing.

## 11. Missing plan recommendations

```text
01_QUALITY_CLI_DECISION_OR_DEFERRAL.md
02_STABILIZE_REAL_REPO_EVIDENCE.md
03_TARGET_PLATFORM_RELEASE_SMOKE.md
04_SEMANTIC_FACT_USER_VALUE_DECISION.md
05_MCP_INTEGRATION_DECISION.md
06_NATIVE_PACKAGE_SIGNING_PLAN.md
07_LICENSE_PAYMENT_HOSTED_BOUNDARY_PLAN.md
```

`01_QUALITY_CLI_DECISION_OR_DEFERRAL.md`: either keep quality/comparator as
maintainer tests/scripts or define a bounded optional `wi quality` surface.

`02_STABILIZE_REAL_REPO_EVIDENCE.md`: reduce reliance on uncommitted local
corpora by adding committed synthetic mini-repos or a clearer manifest evidence
process.

`03_TARGET_PLATFORM_RELEASE_SMOKE.md`: run archive/install smoke on every
target platform before publishing target artifacts.

`04_SEMANTIC_FACT_USER_VALUE_DECISION.md`: decide whether semantic facts should
remain an isolated data model or become a user-facing feature.

`05_MCP_INTEGRATION_DECISION.md`: either explicitly defer MCP or implement a
local-only bounded server/helper with repo path validation and no shell escape.

`06_NATIVE_PACKAGE_SIGNING_PLAN.md`: cover native package formats,
signing/notarization credentials, and trust/update policy.

`07_LICENSE_PAYMENT_HOSTED_BOUNDARY_PLAN.md`: only if the product moves beyond
local/free mode; define accounts, payment, activation, privacy, and failure
behavior before code.

## 12. Final verdict

Ready for limited dogfood.

The local CLI product loop is implemented, wired, and proven by runtime
commands and tests: indexing, SQLite persistence, search, refs, pack, impact,
doctor, stats, scorecard, and agent instruction helpers all work locally.
However, the product is not ready for broad production distribution or hosted
commercial use because native package/signing/update channels, hosted/team
features, payment/licensing enforcement, MCP, semantic/compiler integrations,
and reproducible always-on real-repo evidence are missing or intentionally
deferred.

## Commands run and results

- `git status --short`: showed existing dirty PLAN_60-related files and
  untracked `prompts/PLAN_60_OPTIONAL_QUALITY_CLI_DECISION.md` /
  `prompts/PLAN_ORDER.md`.
- `ls`: repository contains Rust sources, docs, scripts, tests, templates,
  integrations, `.dev_index/`, and local `test_repos/`.
- `cargo fmt --all -- --check`: passed.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`:
  passed, no issues.
- `cargo test --workspace --all-targets --all-features`: passed,
  339 passed, 8 ignored.
- `scripts/check-build-performance`: passed; fixture no-change total 5 ms,
  changed files 0, parsed files 0, quality/comparator phases 0.
- `cargo deny check licenses`: passed, `licenses ok`.
- `cargo run --bin wi -- --help`: passed; lists search, refs, pack, impact,
  doctor, bench; no `wi quality`.
- `cargo run --bin wi -- --version`: passed, `wi 0.1.4 (index schema 12)`.
- `cargo run --bin build_index -- --version`: passed,
  `build_index 0.1.4 (index schema 12)`.
- `cargo run --bin build_index -- --stats`: passed; no-change build,
  `changed files: 0`, `records: 3450`, `semantic facts: 0`,
  `quality/comparator phases: 0`, `real-repo quality phases: 0`,
  `total ms: 20`.
- `cargo run --bin wi -- doctor`: passed, `overall: ok`.
- `cargo run --bin wi-init -- --help`: passed; documents repo-local writes,
  existing `CLAUDE.md` normalization, `--dry-run`, and no global config.
- `cargo run --bin wi-init -- --dry-run`: passed; reported no files changed.
- `cargo run --bin wi-stats`: passed; reported local usage windows and advisory
  workflow audit.
- `cargo run --bin wi-stats -- --version`: passed,
  `wi-stats 0.1.4 (index schema 12)`.
- `cargo run --bin wi-scorecard`: passed, summary `pass 9 / warn 1 / fail 0`.
- `cargo run --bin wi-scorecard -- --version`: passed,
  `wi-scorecard 0.1.4 (index schema 12)`.
- `cargo run --bin wi -- build_index`: passed with file:line landmarks.
- `cargo run --bin wi -- refs build_index`: passed with primary definitions,
  reasons, evidence, and confidence labels.
- `cargo run --bin wi -- pack build_index`: passed with bounded grouped read
  set and reasons/confidence.
- `cargo run --bin wi -- impact build_index`: passed with bounded likely
  affected files and reasons/confidence.
- `cargo test --test local_index -- --ignored`: passed, 1 test.
- `cargo test --test real_repos -- --ignored`: passed, 1 test, 3 filtered out,
  233.88 seconds.

Skipped:

- Python checks: no Python project/test config found.
- Node checks: no `package.json` found.
- `scripts/check-release`: not run in this audit to avoid release archive churn;
  it remains the documented pre-publication release gate.
