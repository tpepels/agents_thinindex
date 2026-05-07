# Wired and Proven Audit

Audit date: 2026-05-07.

Scope: current checkout after recovery, feature-gap audit, refs confidence
alignment, build performance guard, PLAN_55 through PLAN_65, release archive
hardening, MCP deferral, semantic deferral, and native distribution planning.
This was an audit-only pass. No feature or product fixes were made.

## Summary

- overall status: the core local agent-navigation loop is wired and proven for
  source-built use: `build_index`, `wi`, `wi refs`, `wi pack`, `wi impact`,
  `wi doctor`, `wi bench`, `wi-init --dry-run`, `wi-stats`, and
  `wi-scorecard` all ran successfully from source, and normal tests passed.
- release status: local Linux archive build/content/smoke is wired and proven
  by `scripts/check-release` and standalone `scripts/smoke-release-archive`,
  but the RC handoff document is stale. Current archive SHA256 is
  `f4f4c3ec5265bc3a56d8a1915e18f7bdc405652fa66b52eb863d28c07b72b682`;
  `docs/RC_0.1.4_HANDOFF.md` records
  `a9f6c65f1ded053541a3cddcb10ee82228bb4bc2f2d77525538a1d486f1073af`.
- dogfood status: ready for local dogfood and Linux archive RC validation with
  the checksum handoff fixed. Non-Linux target archives and native packages are
  not dogfood-ready until target-platform smoke exists.
- top unwired features: MCP, quality CLI, native packages, package-manager
  publishing, hosted/team product, telemetry, payment, license enforcement,
  managed update channels, and full semantic/LSP resolution.
- top unproven features: non-Linux target archives, native package/signing
  paths, ignored real-repo checks in this checkout because `test_repos/` is
  absent, and PATH-installed full workflow beyond version/schema agreement.
- top overclaims: stale RC handoff checksum is the only concrete active release
  overclaim found. Stale-surface search did not find active user docs claiming
  `WI.md`, JSONL canonical storage, Universal Ctags as production parser, MCP,
  payment/enforcement, hosted product, or full semantic/LSP resolution as
  implemented.
- top blockers: update `docs/RC_0.1.4_HANDOFF.md` before publication; run
  target-platform smoke before publishing any non-Linux archive.

## Classification Rules

Use exactly these statuses:

- wired_and_proven
- wired_but_unproven
- proven_but_not_wired
- partially_wired
- fixture_only
- documented_only
- intentionally_deferred
- broken
- missing

## Feature Matrix

| feature | claimed behavior | wired status | proven status | final status | user-facing entry point | evidence | tests | runtime smoke | gaps | next action |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `wi <query>` | Search repo-local landmarks and return compact file:line rows. | yes | yes | wired_and_proven | `wi <term>` | `src/bin/wi.rs`, `src/search.rs`, README command list. | `tests/wi.rs`, `tests/agent_acceptance.rs` | `cargo run --bin wi -- build_index` returned source/test/doc landmarks. | Broad current-repo terms can be noisy because plans/docs mention CLI names often. | Keep as first search path; use `pack`/`impact` for implementation read sets. |
| `wi refs` | Show references with compact reason/confidence evidence. | yes | yes | wired_and_proven | `wi refs <term>` | `src/context.rs`, `src/refs.rs`, `docs/REFERENCE_GRAPH.md`. | `tests/wi.rs`, `tests/file_refs.rs`. | `cargo run --bin wi -- refs build_index` printed primary rows, `reason`, `evidence`, and `confidence`. | Heuristic rows are still broad text fallback; semantic resolution is not claimed. | Continue precision hardening from concrete false positives. |
| `wi pack` | Return bounded implementation read set with reasons/confidence. | yes | yes | wired_and_proven | `wi pack <term>` | `src/context.rs`, `docs/CONTEXT_PACKS.md`. | `tests/wi.rs`, `tests/agent_acceptance.rs`. | `cargo run --bin wi -- pack build_index` returned 10 grouped rows. | Best-effort local evidence only. | Keep as primary agent read-set command. |
| `wi impact` | Return evidence-backed affected-file groups with reasons/confidence. | yes | yes | wired_and_proven | `wi impact <term>` | `src/context.rs`, `docs/IMPACT_ANALYSIS.md`. | `tests/wi.rs`, `tests/file_refs.rs`, `tests/agent_acceptance.rs`. | `cargo run --bin wi -- impact build_index` returned definitions, references, dependents, tests, docs, and config. | Not exhaustive semantic impact. | Keep docs caveat as plausible/evidence-backed, not exhaustive. |
| `wi doctor` | Diagnose index/schema/freshness/instructions/ignore/license/binary state. | yes | yes | wired_and_proven | `wi doctor` | `src/doctor.rs`, `src/bin/wi.rs`. | `tests/wi.rs`, `tests/schema_version.rs`. | `cargo run --bin wi -- doctor` reported `overall: ok`. | None found. | Keep as release and support smoke. |
| `wi bench` | Report local benchmark and integrity metrics. | yes | yes | wired_and_proven | `wi bench` | `src/bench.rs`, `src/bin/wi.rs`, README. | `tests/bench.rs`. | `cargo run --bin wi -- bench` reported 20/20 query hits, latencies, counts, and `integrity: ok`. | Bench is local evidence, not global performance proof. | Keep documented as local measurement. |
| `build_index` | Build/update `.dev_index/index.sqlite` with stats. | yes | yes | wired_and_proven | `build_index`, `build_index --stats` | `src/bin/build_index.rs`, `src/indexer.rs`, `src/store.rs`. | `tests/build_index.rs`, `tests/local_index.rs --ignored`. | Two `cargo run --bin build_index -- --stats` runs reported no-change `changed files: 0`, `parsed files: 0`, totals 38 ms then 25 ms. | Changed-file relationship work is incremental but conservative. | Keep performance guard; harden only with measured regressions. |
| `wi-init` | Generate/normalize repo-local agent instruction files. | yes | yes | wired_and_proven | `wi-init`, `wi-init --dry-run`, `wi-init --remove` | `src/bin/wi-init.rs`, `src/agent_instructions.rs`, `docs/AGENT_INTEGRATION.md`. | `tests/wi_init.rs`, `tests/agent_integration.rs`. | `cargo run --bin wi-init -- --dry-run` printed no-op repo-local changes and no writes. | Normal run mutates repo-local files by design; no global config writes. | Keep using dry-run in audits. |
| `wi-stats` | Show local usage stats and advisory agent workflow audit. | yes | yes | wired_and_proven | `wi-stats` | `src/bin/wi-stats.rs`, `src/stats.rs`, docs. | `tests/wi_stats.rs`. | `cargo run --bin wi-stats` reported windows, hit/miss graph, context counts, and scope caveat. | Cannot detect external grep/find/ls/Read. | Keep advisory wording. |
| `wi-scorecard` | Report pass/warn/fail evidence for the core value loop. | yes | yes | wired_and_proven | `wi-scorecard` | `src/bin/wi-scorecard.rs`, `src/scorecard.rs`, `docs/SCORECARD.md`. | `tests/scorecard.rs`, `tests/schema_version.rs`. | `cargo run --bin wi-scorecard` returned `pass 9 / warn 1 / fail 0`; warning was fresh-index-only recovery caveat. | A fresh index run does not prove recovery in that invocation. | For release evidence, run once on missing/stale index or rely on tests. |
| missing index auto-build | `wi` self-heals missing index once before search/context. | yes | yes | wired_and_proven | `wi <term>`, refs/pack/impact | `src/bin/wi.rs`, `src/indexer.rs`. | `tests/wi.rs`, `tests/agent_acceptance.rs`, `tests/schema_version.rs`. | `scripts/check-release` packaged smoke runs `wi` in a temp repo and builds automatically. | Concurrent rebuilds are not a normal supported path. | Keep serialized CLI behavior documented. |
| stale index auto-rebuild | `wi` self-heals stale index once before continuing. | yes | yes | wired_and_proven | `wi <term>` | `src/bin/wi.rs`, `src/indexer.rs`. | `tests/wi.rs`, `tests/agent_acceptance.rs`. | `wi-scorecard` warned index was already fresh; `scripts/check-release` smoke exercised temp repo build path. | This specific audit did not force a stale source checkout index by hand. | Keep stale tests as proof; add manual stale smoke only if needed. |
| schema-stale rebuild | Old/missing/invalid schema rebuilds cleanly. | yes | yes | wired_and_proven | `build_index`, `wi`, `wi-scorecard` | `src/store.rs`, `src/model.rs`, `src/binary_state.rs`. | `tests/schema_version.rs`. | Source and PATH binaries report schema 12; doctor reports schema current. | Runtime smoke did not corrupt schema manually. | Tests are sufficient; keep schema tests mandatory. |
| PATH/source schema agreement | Installed/source binaries agree and guarded write-capable paths refuse mismatch. | yes | yes | wired_and_proven | installed `wi`, `build_index`, `wi-scorecard`; source `cargo run` | `src/binary_state.rs`, `src/bin/*.rs`. | `tests/schema_version.rs`, `tests/install_scripts.rs`, `tests/installers.rs`. | PATH and source all reported `0.1.4 (index schema 12)`. | This pass ran PATH versions, not full PATH workflow. | Keep PATH version smoke; run full installed workflow before publication. |
| no-change build performance | No-change build should not parse/recompute/write and should stay fast. | yes | yes | wired_and_proven | `build_index --stats`, `scripts/check-build-performance` | `src/indexer.rs`, `docs/PERFORMANCE.md`. | `tests/build_index.rs`. | Guard: total 6 ms; repo no-change: total 25 ms, parsed 0, quality phases 0. | Timing is local-machine evidence. | Keep budget env override documented. |
| changed-file incremental rebuild | Changed/deleted files update affected records/refs/deps/file refs. | yes | yes | wired_and_proven | `build_index` | `src/indexer.rs`, `src/deps.rs`, `src/file_refs.rs`. | `tests/build_index.rs`, `tests/file_refs.rs`. | Not manually edited in this audit; tests passed. | Relationship invalidation remains conservative for target-surface changes. | Keep existing tests; add more only from measured missed updates. |
| build performance guard | CI/local guard fails on slow no-change, reparsing, quality/real-repo phases. | yes | yes | wired_and_proven | `scripts/check-build-performance` | `scripts/check-build-performance`, `.github/workflows/ci.yml`, `docs/PERFORMANCE.md`. | `tests/build_index.rs`. | `scripts/check-build-performance` passed with changed 0, parsed 0, quality phases 0. | Current repo timing is advisory; fixture is hard gate. | Keep in CI and release checks. |
| ignored path behavior | `.dev_index`, `test_repos`, generated/vendor/sensitive ignored paths excluded. | yes | yes | wired_and_proven | `build_index`, `.thinindexignore`, `.gitignore` | `src/ignore.rs`, `src/indexer.rs`, templates. | `tests/ignore_rules.rs`, `tests/build_index.rs`, `tests/security_privacy.rs`. | `build_index --stats` reported sensitive paths none; tests cover `.dev_index`/`test_repos`. | Not a secret scanner. | Keep privacy caveat. |
| unbounded output prevention | Search/context outputs are capped and grouped. | yes | yes | wired_and_proven | `wi -n`, refs/pack/impact defaults | `src/search.rs`, `src/context.rs`, `src/refs.rs`. | `tests/wi.rs`, `tests/file_refs.rs`. | Runtime refs/pack/impact outputs remained bounded. | None found. | Keep default limits stable. |
| Tree-sitter extraction | Supported code languages use bundled Tree-sitter extraction. | yes | yes | wired_and_proven | `build_index`, `wi` | `src/tree_sitter_extraction.rs`, `src/support.rs`, `Cargo.toml`. | `tests/parser_conformance.rs`, unit query tests. | `build_index --stats` indexed 3574 records. | Syntactic only; no semantic binding. | Keep query/capture conformance gates. |
| extras-backed formats | CSS/HTML/Markdown/JSON/TOML/YAML deterministic extractors. | yes | yes | wired_and_proven | `build_index`, `wi -t css_class`, etc. | `src/extras.rs`, `docs/PARSER_SUPPORT.md`. | `tests/format_conformance.rs`, `tests/wi.rs`. | Included in normal index counts and docs. | Not Tree-sitter code-symbol parsing. | Keep extras-backed support level explicit. |
| support levels | supported/experimental/blocked/extras-backed matrix governs claims. | yes | yes | wired_and_proven | docs, `wi doctor`, scorecard | `src/support.rs`, `docs/LANGUAGE_SUPPORT.md`. | `tests/support_levels.rs`. | Doctor and scorecard reported 14 supported, 5 experimental, 6 extras-backed, 7 blocked. | None found. | Keep matrix as source of truth. |
| language support matrix | Published language table matches implementation. | yes | yes | wired_and_proven | README/docs | `README.md`, `docs/PARSER_SUPPORT.md`, `src/support.rs`. | `tests/support_levels.rs`, `tests/parser_conformance.rs`. | Doctor/scorecard loaded matrix. | Real-repo proof varies by language. | Keep support-level caveats. |
| Go/PHP evidence | Go/PHP supported status has fixtures and committed synthetic evidence. | yes | yes | wired_and_proven | `build_index`, docs | `tests/fixtures/synthetic_real_repo`, docs audits. | `tests/support_levels.rs`, `tests/real_repos.rs` normal synthetic test. | `cargo test` passed committed synthetic Go/PHP context coverage. | No local `test_repos/` Go/PHP corpus in this checkout. | Add real repos only when local corpus exists. |
| expected-symbol checks | Real-repo manifest expected symbols/patterns/absent symbols are checked. | partially | yes | partially_wired | ignored/manual tests, quality gates | `src/bench.rs`, `src/quality/gate.rs`, `docs/REAL_REPO_MANIFEST.md`. | `tests/bench.rs`, `tests/quality_gates.rs`, `tests/real_repos.rs`. | Normal tests passed helper/manifest behavior. | User-facing only through ignored/manual workflows, not normal CLI. | Keep local-only manifest docs; add CLI only with a scoped plan. |
| real-repo evidence | Validate local third-party repos when present. | partially | no in this checkout | partially_wired | ignored tests | `tests/real_repos.rs`, `tests/bench_repos.rs`, docs. | ignored tests exist. | Skipped because `test_repos/` is absent. | Not reproducible from committed checkout alone. | Keep as manual; use committed synthetic evidence for CI. |
| committed synthetic evidence | Small committed corpus proves multi-language refs/context. | yes | yes | wired_and_proven | normal tests | `tests/fixtures/synthetic_real_repo`. | `tests/real_repos.rs`, `tests/support_levels.rs`. | `cargo test` passed. | Synthetic scope is narrower than real repos. | Expand only for high-value gaps. |
| symbol refs | Store and surface symbol/name references. | yes | yes | wired_and_proven | `wi refs`, pack/impact | `src/refs.rs`, `src/store.rs`, `src/context.rs`. | `tests/wi.rs`, `tests/build_index.rs`. | `wi refs build_index` printed references. | Heuristic text fallback remains best-effort. | Keep confidence labels visible. |
| file references | Store and surface local file-to-file relationships. | yes | yes | wired_and_proven | `wi refs`, pack/impact, stats | `src/file_refs.rs`, `src/store.rs`, `src/context.rs`. | `tests/file_refs.rs`, `tests/build_index.rs`. | `build_index --stats` reported 299 file refs by kind/reason. | No package-manager/export-map semantics. | Keep local evidence caveats. |
| import/export file references | Import/export/include/link file refs extracted and ranked. | yes | yes | wired_and_proven | `wi refs`, pack/impact | `src/file_refs.rs`, `src/deps.rs`. | `tests/file_refs.rs`, `tests/build_index.rs`. | Runtime refs showed `file_import indexer -> src/indexer.rs`. | Explicit local forms only. | Harden from concrete real-repo misses. |
| dependency graph | Local dependency edges feed refs/pack/impact. | yes | yes | wired_and_proven | `build_index`, pack/impact | `src/deps.rs`, `src/context.rs`, `src/store.rs`. | `tests/build_index.rs`, `tests/file_refs.rs`. | Pack/impact showed dependency/dependent file groups. | Not package-manager/compiler complete. | Keep current scope. |
| unresolved reasons | Preserve unresolved/ambiguous evidence. | yes | yes | wired_and_proven | `build_index --stats`, impact unknown groups | `src/model.rs`, `src/file_refs.rs`, `src/deps.rs`. | `tests/file_refs.rs`, `tests/build_index.rs`. | Stats reported unresolved reasons `ambiguous_match`, `external_package`, `target_not_found`. | None found. | Keep visible in diagnostics. |
| confidence/reason labels | Context rows expose reason/evidence/confidence. | yes | yes | wired_and_proven | refs/pack/impact | `src/context.rs`, `src/model.rs`, docs. | `tests/wi.rs`, `tests/file_refs.rs`. | Runtime refs/pack/impact printed reasons and confidence. | Confidence remains local evidence, not semantic proof. | Keep docs precise. |
| refs ranking | Exact/local/file-reference evidence ranks above heuristic refs. | yes | yes | wired_and_proven | `wi refs` | `src/context.rs`. | `tests/wi.rs`. | Runtime `exact_local` and `resolved` rows appeared above heuristic text refs. | Heuristic volume still depends on term. | Keep ranking tests. |
| pack ranking | Pack groups and ranks definitions, references, deps, tests, config. | yes | yes | wired_and_proven | `wi pack` | `src/context.rs`. | `tests/wi.rs`. | Runtime pack output grouped and bounded. | None found. | Keep group caps. |
| impact reasoning | Impact groups affected files with reasons/confidence. | yes | yes | wired_and_proven | `wi impact` | `src/context.rs`. | `tests/wi.rs`, `tests/file_refs.rs`. | Runtime impact output grouped definitions, refs, dependents, tests, docs, config. | Not exhaustive semantic impact. | Keep caveat. |
| semantic facts | Internal adapter facts table/model; not normal user-facing output. | no user path by design | yes as boundary | intentionally_deferred | none in normal CLI | `src/semantic.rs`, `src/model.rs`, `docs/SEMANTIC_ADAPTERS.md`. | `tests/semantic.rs`. | `build_index --stats` reported semantic facts 0; normal commands do not consume. | No real adapters or user feature. | Keep deferred until scoped semantic plan. |
| scorecard quality/value | Scorecard covers core loop dimensions. | yes | yes | wired_and_proven | `wi-scorecard` | `src/scorecard.rs`, `docs/SCORECARD.md`. | `tests/scorecard.rs`, `tests/schema_version.rs`. | Runtime pass 9 / warn 1 / fail 0. | Fresh-index warning can confuse if treated as failure. | Document acceptable warning contexts. |
| quality reports | Local quality report export and redaction. | partial user path | yes | partially_wired | tests/library/manual docs | `src/quality/export.rs`, `docs/QUALITY.md`. | `tests/quality.rs`, `tests/quality_loop.rs`. | No normal CLI smoke; quality phases 0 in normal build. | No `wi quality` command. | Keep optional/manual or design CLI later. |
| quality gates | Deterministic integrity/expected-symbol/ctags-source gates. | partial | yes | partially_wired | tests/scripts | `src/quality/gate.rs`, `src/quality/ctags_gate.rs`. | `tests/quality_gates.rs`, `tests/quality_ctags_allowlist.rs`. | `cargo test` passed; check-release includes deterministic gates via tests. | Real-repo gate is ignored/manual. | Keep normal gates deterministic. |
| optional comparator | Optional Ctags comparator for QA only. | no normal user path | yes as isolated QA | proven_but_not_wired | ignored/manual quality tests | `src/quality/comparator.rs`, `docs/QUALITY_CTAG_BOUNDARY.md`. | `tests/quality.rs`, `tests/quality_ctags_allowlist.rs`. | Normal build stats showed quality/comparator phases 0. | Not installed, packaged, or production parser. | Keep optional and isolated. |
| ctags allowlist | Prevent Ctags production/package leakage. | yes | yes | wired_and_proven | tests/package gates | `tests/quality_ctags_allowlist.rs`, `scripts/check-package-contents`. | `tests/quality_ctags_allowlist.rs`, `tests/release_automation.rs`. | `cargo test` and `scripts/check-release` passed. | None found. | Keep release gate. |
| quality CLI | Normal `wi quality` command. | no | yes as absent/deferred | intentionally_deferred | none | `src/wi_cli.rs`, `docs/QUALITY.md`. | `tests/quality_ci_readiness.rs`. | `wi --help` has no quality command. | Feature intentionally absent. | Keep deferred unless a scoped plan chooses it. |
| triage workflow | Bounded quality triage/cycle workflow. | partial/manual | yes | partially_wired | docs/tests | `src/quality/triage.rs`, `src/quality/cycle.rs`, `docs/QUALITY_LOOP.md`. | `tests/quality_loop.rs`. | No CLI smoke. | Manual maintainer workflow only. | Keep as maintainer tooling. |
| real-repo manifest | Local manifest for ignored real-repo checks. | partial/manual | yes for parser/fixture behavior | partially_wired | ignored tests/docs | `docs/REAL_REPO_MANIFEST.md`, `src/bench.rs`. | `tests/bench.rs`, `tests/real_repos.rs`. | Skipped ignored real-repo run because `test_repos/` absent. | Manual corpus required. | Keep local-only; avoid broader claims. |
| AGENTS.md | Repo-local canonical agent instructions. | yes | yes | wired_and_proven | `wi-init`, committed `AGENTS.md` | `AGENTS.md`, `src/agent_instructions.rs`. | `tests/wi_init.rs`, `tests/agent_integration.rs`. | `wi doctor` reported AGENTS current. | Advisory only. | Keep direct command wording. |
| CLAUDE.md normalization | Normalize existing CLAUDE.md; do not create when absent. | yes | yes | wired_and_proven | `wi-init` | `CLAUDE.md`, `src/bin/wi-init.rs`. | `tests/wi_init.rs`. | `wi doctor` reported CLAUDE current in this repo. | Advisory only. | Keep tests. |
| Cursor rules | Generate/normalize repo-local Cursor rule. | yes | yes | wired_and_proven | `wi-init` | `.cursor/rules/thinindex.mdc`, integrations pack. | `tests/wi_init.rs`, `tests/agent_integration.rs`. | `wi-init --dry-run` reported no-op Cursor rule. | Advisory only. | Keep repo-local. |
| Copilot instructions | Generate/normalize repo-local Copilot instructions. | yes | yes | wired_and_proven | `wi-init` | `.github/copilot-instructions.md`, integrations pack. | `tests/wi_init.rs`, `tests/agent_integration.rs`. | `wi-init --dry-run` reported no-op Copilot instructions. | Advisory only. | Keep repo-local. |
| OpenCode guidance | Use shared AGENTS.md guidance. | partially | yes as docs | partially_wired | docs/integrations | `docs/AGENT_INTEGRATION.md`. | `tests/agent_integration.rs`. | No OpenCode runtime smoke. | No OpenCode-specific config. | Keep docs as shared AGENTS guidance. |
| Codex guidance | AGENTS.md integration pack. | yes | yes | wired_and_proven | `AGENTS.md`, integrations pack | `integrations/agents/codex/AGENTS.md`. | `tests/agent_integration.rs`. | Current AGENTS validated by doctor. | Advisory only. | Keep pack in sync. |
| config helpers | Repo-local instruction helpers and dry-run. | yes | yes | wired_and_proven | `wi-init --dry-run` | `src/bin/wi-init.rs`. | `tests/wi_init.rs`. | Runtime dry-run produced no file changes. | No global config writes by design. | Keep dry-run-first docs. |
| MCP | Local MCP server/helper/config. | no | yes as absent/deferred | intentionally_deferred | none | `integrations/agents/mcp/README.md`, `docs/AGENT_INTEGRATION.md`. | `tests/agent_integration.rs`. | No MCP command installed or advertised. | Not implemented. | Keep deferred. |
| `install.sh` | Install source-built binaries. | yes | yes | wired_and_proven | `./install.sh` | `install.sh`, docs. | `tests/install_scripts.rs`. | Not run this pass to avoid mutating PATH; PATH binaries exist and versions match. | Full installed workflow not rerun this pass. | Run before publication when intentionally updating install. |
| `uninstall.sh` | Remove installed binaries only. | yes | yes | wired_and_proven | `./uninstall.sh` | `uninstall.sh`. | `tests/install_scripts.rs`. | Not run to avoid mutating user install. | Runtime uninstall not smoked this pass. | Keep tests; run in temp dirs only. |
| archive install/uninstall scripts | Archive helpers install/uninstall all five binaries without repo mutation. | yes | yes | wired_and_proven | packaged scripts | `scripts/install-archive-unix`, `scripts/windows/*.ps1`. | `tests/installers.rs`, `tests/release_automation.rs`. | `scripts/smoke-release-archive` ran packaged install-like smoke in temp repo. | Windows helper not target-smoked on Windows here. | Smoke on target platform. |
| `wi-scorecard` installed/packaged | Scorecard binary included in source install and archive. | yes | yes | wired_and_proven | install/archive | `install.sh`, `scripts/package-release`, docs. | `tests/install_scripts.rs`, `tests/release_packaging.rs`. | Package smoke ran `wi-scorecard`; PATH version reports schema 12. | Full PATH scorecard workflow not rerun this pass. | Keep archive smoke. |
| release archive contents | Archive contains expected binaries/docs/scripts and excludes local/secret/source state. | yes | yes | wired_and_proven | `scripts/package-release`, `scripts/check-package-contents` | release scripts, docs. | `tests/release_automation.rs`, `tests/release_packaging.rs`. | `scripts/check-release` passed; standalone smoke passed. | Archive rebuilt changes checksum each package pass. | Refresh handoff after final package build. |
| SBOM/notices | Archive includes SBOM and THIRD_PARTY_NOTICES. | yes | yes | wired_and_proven | release archive | `scripts/package-release`, `scripts/check-package-contents`, `THIRD_PARTY_NOTICES`. | `tests/license_audit.rs`, `tests/release_automation.rs`. | check-release package content passed. | Compact SBOM; production native SBOM format deferred. | Keep native SBOM deferral explicit. |
| RC handoff checksum | Handoff checksum should match current archive and sidecar. | yes doc path | no | broken | `docs/RC_0.1.4_HANDOFF.md` | current archive sidecar: `f4f4c3ec...`; handoff: `a9f6c65f...`. | no test currently locks handoff freshness. | `sha256sum` and sidecar match each other but not handoff. | Release blocker for handoff publication. | Regenerate/correct handoff after final archive build. |
| target-platform smoke | Do not publish target until exact archive smoked on compatible platform. | yes docs/scripts | partial | partially_wired | `docs/TARGET_PLATFORM_SMOKE.md`, smoke script | `docs/TARGET_PLATFORM_SMOKE.md`, `scripts/smoke-release-archive`. | `tests/release_automation.rs`. | Linux archive smoke passed; non-Linux targets not smoked. | Non-Linux target evidence missing. | Run per target platform before publish. |
| native packages | deb/rpm/AppImage/pkg/dmg/MSI/MSIX. | no | yes as deferred | intentionally_deferred | docs only | `docs/NATIVE_DISTRIBUTION_PLAN.md`, `docs/INSTALLERS.md`. | `tests/release_packaging.rs`, `tests/installers.rs`. | No package artifacts built. | Not implemented. | Keep deferred until platform-specific plan. |
| signing/notarization | Authenticode/Developer ID/notarization/GPG scaffold. | scaffold only | yes as scaffold/deferred | intentionally_deferred | `scripts/sign-release-artifact --dry-run` | `scripts/sign-release-artifact`, docs. | `tests/installers.rs`. | No signing run; tests prove failure without secrets. | Requires external credentials/tools/hosts. | Do not claim signed production distribution. |
| package-manager publishing | Homebrew/winget/Linux repos. | no | yes as deferred | intentionally_deferred | docs only | `docs/NATIVE_DISTRIBUTION_PLAN.md`. | `tests/release_packaging.rs`. | No publishing commands. | Not implemented. | Keep explicit deferral. |
| licensing foundation | Local/free edition and inert fixture license model. | partially | yes | fixture_only | docs/tests | `src/licensing.rs`, `docs/LICENSING.md`, `docs/PRODUCT_BOUNDARY.md`. | `tests/licensing.rs`, `tests/install_scripts.rs`. | Doctor reports free/no_license_file. | Pro accepted only by local test fixture; no enforcement. | Keep no-enforcement boundary. |
| license enforcement | Paid gates/license activation. | no | yes as absent/deferred | intentionally_deferred | none | `docs/PRODUCT_BOUNDARY.md`, `docs/LICENSING.md`. | `tests/licensing.rs`. | No runtime enforcement. | Not implemented. | Keep deferred. |
| hosted/team | Hosted service/team product. | no | yes as deferred | intentionally_deferred | docs only | `docs/TEAM_CI_ROADMAP.md`, `docs/CI_INTEGRATION.md`. | `tests/team_ci_roadmap.rs`. | No hosted runtime. | Not implemented. | Keep roadmap-only. |
| telemetry | Usage telemetry/cloud reporting. | no | yes as absent/deferred | intentionally_deferred | none | README, `docs/SECURITY_PRIVACY.md`, `docs/PRODUCT_BOUNDARY.md`. | `tests/security_privacy.rs`, `tests/team_ci_roadmap.rs`. | No telemetry commands; wi-stats is local only. | Not implemented by design. | Keep local-only claim. |
| payment | Payment/account system. | no | yes as absent/deferred | intentionally_deferred | none | `docs/PRODUCT_BOUNDARY.md`, `docs/LICENSING.md`. | `tests/licensing.rs`. | No runtime payment path. | Not implemented. | Keep deferred. |
| update channel | Managed update channel. | no | yes as deferred | intentionally_deferred | docs only | `docs/NATIVE_DISTRIBUTION_PLAN.md`, `docs/PRODUCT_BOUNDARY.md`. | `tests/release_packaging.rs`. | No update command or feed. | Not implemented. | Keep deferred. |

## Recommended Next Actions

1. Immediate blocker fix: regenerate or correct
   `docs/RC_0.1.4_HANDOFF.md` after the final `scripts/check-release` archive
   build so its SHA256 matches the current archive and `.sha256` sidecar.
2. Highest-value feature hardening item: add a reproducible committed or
   fixture-backed real-repo evidence path for the most important language/ref
   gaps so confidence does not depend on local `test_repos/`.
3. Release-readiness item: run target-platform smoke for every non-Linux archive
   before any publication claim, and record the evidence in
   `docs/TARGET_PLATFORM_SMOKE.md`.
4. Explicit deferral: keep MCP, native packages, signing/notarization,
   package-manager publishing, hosted/team product, telemetry, payment, license
   enforcement, update channels, and full semantic/LSP resolution deferred until
   scoped plans implement and verify them.

## Verification Commands And Results

- `git status --short`: clean before audit document edits.
- `git log --oneline -30`: recent history includes `be0aa2d Plan native
  package signing path`, `bc484c0 Decide MCP integration path`, and completed
  PLAN_55 through PLAN_65 commits.
- `cargo fmt --check`: passed.
- `cargo test`: passed, `343 passed, 8 ignored`.
- `cargo clippy --all-targets --all-features -- -D warnings`: passed.
- `scripts/check-build-performance`: passed; fixture no-change total 6 ms,
  changed files 0, parsed files 0, relationship recomputations 0,
  quality/comparator phases 0, real-repo quality phases 0.
- `cargo run --bin build_index -- --stats`: passed with no-change stats,
  changed files 0, parsed files 0, quality/comparator phases 0, total 38 ms.
- immediate second `cargo run --bin build_index -- --stats`: passed with
  changed files 0, parsed files 0, relationship recomputations 0, total 25 ms.
- `cargo run --bin wi -- doctor`: passed, `overall: ok`, schema 12 current,
  binary/source match.
- `cargo run --bin wi -- build_index`: passed with source/test/doc landmarks.
- `cargo run --bin wi -- refs build_index`: passed with primary rows, reason,
  evidence, and confidence labels.
- `cargo run --bin wi -- pack build_index`: passed with bounded grouped output,
  reasons, and confidence labels.
- `cargo run --bin wi -- impact build_index`: passed with grouped related-file
  output, reasons, and confidence labels.
- `cargo run --bin wi -- bench`: passed, 20 queries, 20 hits, integrity ok.
- `cargo run --bin wi-init -- --dry-run`: passed and reported no file changes.
- `cargo run --bin wi-stats`: passed and reported local usage plus advisory
  agent workflow audit.
- `cargo run --bin wi-scorecard`: passed with `pass 9 / warn 1 / fail 0`; the
  warning was that the index was already fresh, so recovery was not exercised in
  that run.
- PATH installed `wi`, `build_index`, `wi-scorecard`, `wi-stats`, and `wi-init`
  all report `0.1.4 (index schema 12)`.
- Source `cargo run --bin ... -- --version` for all five binaries reports
  `0.1.4 (index schema 12)`.
- First `scripts/check-release` attempt failed during internal `cargo test`
  because several tests could not find `target/debug/build_index`; immediately
  afterward `target/debug/build_index` existed and
  `cargo test --test build_index no_change_build_index_stats_enforces_incremental_performance_contract`
  passed. A full rerun of `scripts/check-release` then passed, including
  package content checks, checksum verification, and archive smoke.
- `scripts/smoke-release-archive
  dist/thinindex-0.1.4-x86_64-unknown-linux-gnu.tar.gz`: passed standalone.
- `cargo test --test local_index -- --ignored`: passed, 1 test.
- `cargo test --test real_repos -- --ignored`: skipped because `test_repos/`
  is absent in this checkout.
- stale-claim search found guardrail/test references to `WI.md`, JSONL, Ctags,
  hosted/payment/enforcement, and semantic/LSP limits, but no active user-doc
  claim that those are implemented current product surfaces.
