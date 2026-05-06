# Feature Gap Audit

Audit date: 2026-05-06.

Scope: current checkout after recovery, file-reference graph work, installed
binary/schema repair, scorecard install alignment, and RC readiness work. This is
an audit-only pass; no implementation fixes were made.

## Summary

- Overall readiness: `mostly_ready` for a Linux local archive RC, but not
  `ready` because one release handoff document has a stale archive checksum and
  a few shipped/advisory surfaces are only partially wired.
- Most important not-ready areas: stale `docs/RC_0.1.4_HANDOFF.md` checksum,
  `wi-scorecard` write-capable recovery without the same source/binary schema
  guard used by `wi` and `build_index`, manual/ignored real-repo quality gates,
  and advisory-only MCP/OpenCode integration surfaces.
- Release-impacting gaps: the current archive and sidecar verify, but the RC
  handoff document records the old SHA256. That should block publication using
  that handoff until the document is regenerated or corrected.
- Agent-value gaps: `wi refs`, `wi pack`, and `wi impact` are useful and
  evidence-backed; remaining gaps are precision/hardening rather than missing
  value. `wi refs` does not print confidence labels, and real-repo evidence is
  still local/manual rather than always-on CI evidence.

## Feature Matrix

| feature | status | evidence | files inspected | tests covering it | gaps | recommended next action |
| --- | --- | --- | --- | --- | --- | --- |
| `wi <query>` core search | ready | Source-built `cargo run --bin wi -- build_index` returned file:line landmarks; `wi` dispatches search/refs/pack/impact and logs usage. | `src/bin/wi.rs`, `src/search.rs`, `README.md` | `tests/wi.rs`, `tests/agent_acceptance.rs` | Current-repo broad terms can be noisy because prompts mention CLI names often. | Keep as-is; recommend `pack`/`impact` for implementation read sets. |
| `build_index` explicit rebuild | ready | `cargo run --bin build_index` completed after adding this audit with `changed files: 1`, `records: 3306`; the immediate second run reported `changed files: 0`, `records: 3306`. | `src/bin/build_index.rs`, `src/indexer.rs` | `tests/build_index.rs`, `tests/local_index.rs --ignored` | Relationship extraction is still global after changed files, not fully graph-incremental. | Treat no-change fast path as ready; harden graph-level incrementality later. |
| `wi doctor` | ready | `cargo run --bin wi -- doctor` reported `overall: ok`, schema 12 current, fresh index, current AGENTS/CLAUDE, and source/binary match. | `src/doctor.rs`, `src/bin/wi.rs` | `tests/wi.rs`, `tests/schema_version.rs`, `tests/install_scripts.rs` | None for current RC scope. | Keep as release smoke gate. |
| `wi-init` | mostly_ready | Help says it writes AGENTS/Cursor/Copilot, normalizes existing CLAUDE, builds index, and does not create `WI.md`; tests cover normalization. | `src/bin/wi-init.rs`, `src/agent_instructions.rs`, `docs/AGENT_INTEGRATION.md` | `tests/wi_init.rs`, `tests/agent_integration.rs` | Running it mutates repo instruction files by design, so audit used help/tests rather than rerunning init in this checkout. | Keep behavior; future dry-run would improve auditability but is not required. |
| `wi-stats` | mostly_ready | `cargo run --bin wi-stats -- --repo .` reported local usage windows and agent workflow audit. | `src/bin/wi-stats.rs`, `src/stats.rs`, `docs/AGENT_INTEGRATION.md` | `tests/wi_stats.rs`, `src/stats.rs` unit tests | Does not run `ensure_binary_matches_source`; advisory only and cannot detect external grep/find/ls/Read. | Document as advisory; add source/binary guard only if it starts writing or mutating indexes. |
| `wi-scorecard` | partially_wired | Source command ran with `pass 9 / warn 1 / fail 0`; packaged binary is included and versioned. | `src/bin/wi-scorecard.rs`, `src/scorecard.rs`, `docs/SCORECARD.md`, `install.sh`, `scripts/package-release` | `tests/scorecard.rs`, `tests/install_scripts.rs`, `tests/release_automation.rs` | It calls `build_index` during recovery but only uses `print_version_if_requested`, not `ensure_binary_matches_source`; shipped write-capable behavior has weaker schema guard than `wi`/`build_index`. | Add a focused schema-guard plan for `wi-scorecard` before relying on installed scorecard as a write path across schema bumps. |
| SQLite schema/current version | ready | `INDEX_SCHEMA_VERSION` is 12; doctor and all binaries report `0.1.4 (index schema 12)`. | `src/model.rs`, `src/store.rs`, `src/binary_state.rs` | `tests/schema_version.rs`, `tests/install_scripts.rs` | None for current schema. | Keep schema-bearing version checks in release gates. |
| missing/stale/schema-stale auto-rebuild | ready | `wi` calls one-shot rebuild before search/context commands; schema mismatch tests cover rebuild and continuation. | `src/bin/wi.rs`, `src/indexer.rs`, `src/store.rs` | `tests/wi.rs`, `tests/schema_version.rs`, `tests/agent_acceptance.rs` | Concurrent rebuild attempts are not a supported normal workflow. | Keep serialized CLI smoke checks; defer concurrency hardening. |
| incremental rebuild | mostly_ready | Changed/deleted files clean records/refs/dependencies/file refs; no-change build skips work and passed local no-change run. | `src/indexer.rs`, `src/deps.rs`, `src/file_refs.rs` | `tests/build_index.rs`, `tests/file_refs.rs` | Any changed file still triggers global relationship recomputation. | Plan a scoped dependency/ref incrementality pass only if performance data warrants it. |
| no-change build performance | ready | Local `cargo run --bin build_index` no-change run completed quickly with zero changed files; PLAN_54 recorded 12 ms direct-binary no-change runs. | `src/indexer.rs`, `prompts/PLAN_54_RELEASE_CANDIDATE_READINESS_CHECKLIST.md`, `prompts/recovery/RECOVERY_STATUS.md` | `tests/build_index.rs`, `tests/agent_acceptance.rs` | Timing depends on machine/repo size. | Keep budget checks in scorecard/RC smoke. |
| ignored paths | ready | Hard ignores include `.git`, `.dev_index`, `test_repos`; `.gitignore` and `.thinindexignore` are honored. | `src/indexer.rs`, `.thinindexignore`, `docs/SECURITY_PRIVACY.md` | `tests/ignore_rules.rs`, `tests/security_privacy.rs` | Sensitive-looking path warning is best-effort, not a secret scanner. | Keep current warnings and docs. |
| installed/source binary schema agreement | partially_wired | PATH `wi`, `build_index`, and `wi-scorecard` report schema 12; `wi`, `build_index`, and `wi-init` call `ensure_binary_matches_source`. | `src/bin/wi.rs`, `src/bin/build_index.rs`, `src/bin/wi-init.rs`, `src/bin/wi-stats.rs`, `src/bin/wi-scorecard.rs`, `src/binary_state.rs` | `tests/schema_version.rs`, `tests/install_scripts.rs` | `wi-stats` and `wi-scorecard` do not enforce source/binary agreement; `wi-scorecard` can rebuild. | Fix `wi-scorecard` first; consider `wi-stats` if read compatibility ever changes. |
| Tree-sitter parser support | ready | Tree-sitter adapters are wired into indexing; parser conformance and capture validation pass. | `src/tree_sitter_extraction.rs`, `src/indexer.rs`, `Cargo.toml` | `tests/parser_conformance.rs`, tree-sitter unit tests | None for supported fixture languages. | Keep conformance fixtures and capture-name validation required. |
| language support matrix/support levels | mostly_ready | Matrix reports 32 entries: 14 supported, 5 experimental, 6 extras-backed, 7 blocked; docs use those levels. | `src/support.rs`, `docs/LANGUAGE_SUPPORT.md`, `docs/LANGUAGE_SUPPORT_AUDIT.md` | `tests/support_levels.rs`, `tests/parser_conformance.rs`, `tests/format_conformance.rs` | Go/PHP support still lacks enough stable Go-heavy/PHP-heavy real-repo manifest evidence. | Add focused real-repo manifests for Go/PHP before raising confidence. |
| expected-symbol checks | ready | Manifest expected symbols/patterns/absent symbols and thresholds are implemented and tested. | `src/quality/gate.rs`, `src/bench.rs`, `docs/REAL_REPO_MANIFEST.md` | `tests/quality_gates.rs`, `tests/real_repos.rs` | Optional local corpora are not committed. | Keep manifests local-only; add curated expected-symbol coverage when corpora are available. |
| real-repo checks | partially_wired | `cargo test --test real_repos -- --ignored` passed locally in 238.75s with `test_repos/`; manifest support exists. | `tests/real_repos.rs`, `src/bench.rs`, `docs/REAL_REPO_MANIFEST.md`, `test_repos/` local state | `tests/real_repos.rs --ignored`, `tests/bench_repos.rs --ignored`, `tests/quality_gates.rs --ignored` | Ignored/manual, slow, and dependent on uncommitted local `test_repos/`. | Keep as manual RC evidence; do not present as always-on CI coverage. |
| quality gates | ready | Deterministic integrity, expected-symbol, absent-symbol, duplicate, and ctags-source gates are implemented. | `src/quality/gate.rs`, `src/quality/ctags_gate.rs`, `docs/QUALITY.md` | `tests/quality_gates.rs`, `tests/quality_ctags_allowlist.rs`, `tests/quality_ci_readiness.rs` | Real-repo gate variant remains ignored/manual. | Keep deterministic fixture gates in CI; manual real-repo gate before release. |
| scorecard value gate | mostly_ready | `wi-scorecard` reports search, warm latency, refs, pack, impact, doctor, init, instruction, support-claim dimensions. | `src/scorecard.rs`, `src/bin/wi-scorecard.rs`, `docs/SCORECARD.md` | `tests/scorecard.rs` | Advisory; stale/missing auto-recovery dimension warns if index was already fresh. | Keep as product-value smoke, not a sole release blocker. |
| symbol refs | mostly_ready | `wi refs build_index` returned primary definitions plus references; tree-sitter and text refs are stored. | `src/refs.rs`, `src/tree_sitter_extraction.rs`, `src/context.rs`, `src/store.rs` | `tests/wi.rs`, `tests/build_index.rs`, `src/refs.rs` tests | Text refs are heuristic and capped; exact semantic resolution remains limited. | Continue precision hardening from real-repo misses. |
| file references | mostly_ready | `file_references` model/table exists; `pack`/`impact` surface dependency/file-reference evidence; stale and duplicate behavior tested. | `src/file_refs.rs`, `src/context.rs`, `src/store.rs`, `tests/file_refs.rs` | `tests/file_refs.rs`, `tests/build_index.rs`, `tests/real_repos.rs --ignored` | Best-effort local path resolution; no package-manager/compiler/LSP/framework alias/export-map semantics. | Keep caveats explicit; add targeted resolver support only from concrete repos. |
| import/export references and dependency graph | mostly_ready | Dependency graph covers representative Rust/Python/JS/TS/Go/Dart/JVM/.NET/C/C++/Ruby/PHP/shell/Nix fixtures. | `src/deps.rs`, `tests/build_index.rs`, `src/context.rs` | `tests/build_index.rs`, `tests/file_refs.rs` | Real-world import alias/package semantics remain partial. | Harden one ecosystem at a time using manifest evidence. |
| unresolved refs/reasons | ready | Unresolved file/dependency reasons are modeled and surfaced in pack/impact unknown sections. | `src/model.rs`, `src/file_refs.rs`, `src/deps.rs`, `src/context.rs` | `tests/file_refs.rs`, `tests/build_index.rs` | None for current best-effort claim. | Keep unresolved evidence visible. |
| confidence/reason labels | partially_wired | `pack` and `impact` print reasons/confidence; `ReferenceRecord` stores confidence/reason. | `src/model.rs`, `src/context.rs` | `tests/wi.rs`, `tests/file_refs.rs` | `wi refs` prints a `reason:` line based on evidence but does not print confidence and does not clearly expose stored `ReferenceRecord.reason`. | Align `wi refs` output or narrow docs to pack/impact. |
| stale cleanup | ready | Changed/deleted files remove stale records, refs, dependencies, and file references. | `src/indexer.rs`, `src/store.rs`, `src/deps.rs`, `src/file_refs.rs` | `tests/build_index.rs`, `tests/file_refs.rs`, `tests/schema_version.rs` | None found. | Keep deterministic rebuild tests. |
| duplicate prevention | ready | References/dependencies/file refs are sorted and deduped; package checks reject forbidden artifacts. | `src/indexer.rs`, `src/deps.rs`, `src/file_refs.rs`, `scripts/check-package-contents` | `tests/build_index.rs`, `tests/file_refs.rs`, `tests/release_automation.rs` | None found. | Keep regression tests. |
| `wi refs` agent value | mostly_ready | Source command returned primary definitions and evidence-backed references. | `src/context.rs`, `src/bin/wi.rs`, `docs/AGENT_INTEGRATION.md` | `tests/wi.rs`, `tests/agent_acceptance.rs` | Lacks confidence labels; heuristic refs can be noisy. | Make confidence output consistent with pack/impact. |
| `wi pack` context ranking | ready | Source command returned bounded grouped read set with definitions, refs, dependencies, dependents, tests, config, reasons, confidence. | `src/context.rs`, `src/search.rs` | `tests/wi.rs`, `tests/agent_acceptance.rs` | None for current user-facing claim. | Keep as recommended agent read-set command. |
| `wi impact` reasoning | ready | Source command returned definitions, refs, dependent files, tests, docs, config, unresolved areas, reasons, confidence. | `src/context.rs`, `src/search.rs` | `tests/wi.rs`, `tests/agent_acceptance.rs`, `tests/file_refs.rs` | Best-effort only; cannot prove full impact. | Keep caveat as "plausible affected files" rather than exhaustive impact. |
| bounded output | ready | Defaults are search 30, refs 20, pack 10, impact 15; tests cover limits/group caps. | `src/bin/wi.rs`, `src/context.rs`, `src/search.rs` | `tests/wi.rs`, `tests/scorecard.rs` | None found. | Keep default budgets stable. |
| AGENTS.md generation | ready | Current canonical block is present and `wi doctor` validates it. | `AGENTS.md`, `src/agent_instructions.rs`, `src/doctor.rs` | `tests/wi_init.rs`, `tests/agent_integration.rs` | Cannot enforce agent behavior. | Keep wording advisory and local. |
| existing CLAUDE.md normalization | ready | Existing `CLAUDE.md` is normalized/validated; `wi-init` does not create it when absent. | `CLAUDE.md`, `src/bin/wi-init.rs`, `src/agent_instructions.rs` | `tests/wi_init.rs`, `tests/agent_integration.rs` | None for claimed behavior. | Keep normalization tests. |
| Cursor rules | ready | `.cursor/rules/thinindex.mdc` is generated/normalized and surfaced by refs. | `.cursor/rules/thinindex.mdc`, `src/bin/wi-init.rs`, `docs/AGENT_INTEGRATION.md` | `tests/wi_init.rs`, `tests/agent_integration.rs` | Advisory only. | Keep as repo-local instruction surface. |
| Copilot instructions | ready | `.github/copilot-instructions.md` is generated/normalized and surfaced by refs. | `.github/copilot-instructions.md`, `src/bin/wi-init.rs`, `docs/AGENT_INTEGRATION.md` | `tests/wi_init.rs`, `tests/agent_integration.rs` | Advisory only. | Keep as repo-local instruction surface. |
| OpenCode/Codex guidance | partially_wired | Docs say OpenCode and Codex use shared `AGENTS.md`; integration packs are advisory snippets. | `docs/AGENT_INTEGRATION.md`, `integrations/agents/codex/AGENTS.md`, `integrations/agents/generic/README.md` | `tests/agent_integration.rs` | No OpenCode-specific generated file/config; no hard enforcement. | Keep claims to shared AGENTS guidance only. |
| MCP/config helper | documented_only | `integrations/agents/mcp/README.md` is explicitly a future local-only integration plan. | `integrations/agents/mcp/README.md`, `docs/AGENT_INTEGRATION.md`, `prompts/PLAN_39_AGENT_WORKFLOW_ENFORCEMENT_AND_INTEGRATION_PACKS.md` | none beyond docs link checks | No MCP server/helper is bundled or wired. | Do not claim MCP implementation; create a separate plan if needed. |
| source install/uninstall scripts | ready | Source install covers five binaries and bundled SQLite dependency; uninstall removes commands and preserves repo-local state. | `install.sh`, `uninstall.sh`, `tests/install_scripts.rs` | `tests/install_scripts.rs` | `./install.sh` was not rerun in this audit pass to avoid mutating installed binaries; PLAN_54 ran it. | Keep install smoke in release checklist. |
| archive installers | ready | Archive install/uninstall helpers include all five binaries and do not mutate repos. | `scripts/install-archive-unix`, `scripts/uninstall-archive-unix`, `scripts/windows/install.ps1`, `scripts/windows/uninstall.ps1` | `tests/installers.rs`, `tests/release_automation.rs` | Windows scripts are packaged as scripts but not target-smoked in this Linux audit. | Smoke each target archive on its target platform. |
| release archive contents | mostly_ready | `scripts/check-release` and standalone smoke passed for current Linux archive; package-content gate enforces expected payload. | `scripts/package-release`, `scripts/check-package-contents`, `scripts/check-release`, `scripts/smoke-release-archive`, `dist/` | `tests/release_automation.rs`, `tests/release_packaging.rs` | Handoff doc checksum is stale even though archive and sidecar verify. | Regenerate/correct RC handoff before publishing. |
| SBOM/notices | ready | `scripts/check-release` validated package contents/SBOM/checksum; license tests passed in check-release. | `SBOM.md`, `THIRD_PARTY_NOTICES`, `deny.toml`, `scripts/check-package-contents` | `tests/license_audit.rs`, `tests/release_automation.rs`, `scripts/check-release` | SBOM must be regenerated after dependency/package changes. | Keep check-release as required. |
| schema-bearing `--version` output | ready | PATH and source binaries report `0.1.4 (index schema 12)`; check-release smokes all five source binaries. | `src/binary_state.rs`, `src/bin/*.rs`, `install.sh` | `tests/install_scripts.rs`, `tests/schema_version.rs` | Enforcement differs by binary as noted above. | Keep version smoke; align guards for write-capable binaries. |
| `scripts/check-release` | ready | Ran successfully, rebuilt archive, checked licenses/package contents/checksum, and smoke-tested archive. | `scripts/check-release`, `scripts/package-release`, `scripts/smoke-release-archive` | `tests/release_automation.rs` | Mutates `dist/` by design. | Keep as release gate. |
| `scripts/smoke-release-archive` | ready | Standalone smoke passed against `dist/thinindex-0.1.4-x86_64-unknown-linux-gnu.tar.gz`. | `scripts/smoke-release-archive`, `dist/` | `tests/release_automation.rs` | Linux archive only in this audit. | Run per target archive. |
| quality reports | mostly_ready | Export text/Markdown/JSON/JSONL reports are local, redacted, capped, deterministic, and isolated from production index tables. | `src/quality/export.rs`, `docs/QUALITY.md` | `tests/quality.rs`, `tests/quality_loop.rs` | Not part of normal `wi`/`build_index` workflows; mostly library/test driven. | Keep optional/manual boundary clear. |
| optional comparator plugin | partially_wired | Universal Ctags comparator is isolated, optional, skipped cleanly if missing, and not packaged. | `src/quality/comparator.rs`, `docs/QUALITY.md`, `docs/QUALITY_CTAG_BOUNDARY.md` | `tests/quality.rs`, `tests/quality_ctags_allowlist.rs` | Manual/ignored-test driven; no normal CLI workflow or required CI gate. | Keep as optional quality plugin; do not imply production parser use. |
| ctags allowlist/isolation | ready | Allowlist gates block forbidden production/package surfaces and comparator output pollution. | `src/quality/ctags_gate.rs`, `tests/quality_ctags_allowlist.rs`, `scripts/check-package-contents` | `tests/quality_ctags_allowlist.rs`, `tests/release_automation.rs` | None found. | Keep as release/package guard. |
| real-repo manifest support | partially_wired | Manifest parsing, expected symbols, expected absent symbols, and local ignored real-repo tests exist. | `src/bench.rs`, `docs/REAL_REPO_MANIFEST.md`, `tests/real_repos.rs`, `tests/bench.rs` | `tests/bench.rs`, `tests/real_repos.rs --ignored` | Depends on uncommitted `test_repos/`; not always-on CI evidence. | Keep local-only; consider synthetic committed mini-repos for stable CI. |
| triage workflow | mostly_ready | Bounded quality-cycle and comparator triage models exist with deterministic report tests. | `src/quality/triage.rs`, `src/quality/cycle.rs`, `docs/QUALITY_LOOP.md` | `tests/quality_loop.rs` | Full real-repo loop remains ignored/manual. | Keep as optional maintainer workflow. |
| README/docs product claims | mostly_ready | Active docs no longer claim `WI.md` as current, JSONL canonical storage, Ctags production indexing, four-binary archives, or old parser architecture. | `README.md`, `docs/`, `prompts/recovery/RECOVERY_STATUS.md`, `prompts/PLAN_54_RELEASE_CANDIDATE_READINESS_CHECKLIST.md` | `tests/install_scripts.rs`, `tests/support_levels.rs`, `tests/quality_system_audit.rs`, `tests/technical_final_audit.rs` | `docs/RC_0.1.4_HANDOFF.md` has stale checksum; some future/advisory surfaces could be overread as implemented if quoted out of context. | Fix handoff doc; keep advisory/future labels explicit. |
| semantic facts / adapter data model | intentionally_deferred | `semantic_facts` table/model exists and tests assert facts do not pollute normal commands. | `src/model.rs`, `src/semantic.rs`, `tests/semantic.rs` | `tests/semantic.rs` | Data model is not user-facing in normal `wi` workflows. | Keep deferred unless a scoped semantic feature plan is selected. |

## Not Wired In Yet

- MCP helper/server: `integrations/agents/mcp/README.md` is a placeholder plan
  only. No MCP server, command wrapper, installer hook, or config generator is
  bundled.
- OpenCode-specific generated config: docs route OpenCode through shared
  `AGENTS.md`; there is no OpenCode-specific file or `wi-init` writer.
- Native installers, signing, notarization, package-manager publishing, hosted
  services, telemetry, payment behavior, network activation, and license
  enforcement: documented as intentionally not included, not wired.
- Quality/comparator workflows are not normal CLI workflows. They are exposed
  through library/test/report surfaces and docs, not `wi quality` or a required
  release command.
- Semantic adapter facts have a model/table and tests, but normal `wi`,
  `refs`, `pack`, and `impact` output does not consume them.

## Partially Wired

- `wi-scorecard`: shipped and useful, but weaker than `wi`/`build_index` for
  installed/source schema enforcement while still capable of rebuilding an
  index.
- `wi-stats`: shipped and useful as an advisory read-only report, but no
  source/binary guard and no ability to prove agents avoided external tools.
- `wi refs`: useful and wired to primary/reference output, but does not print
  confidence labels and does not clearly expose stored reference reason labels
  the same way `pack` and `impact` do.
- Real-repo gates: implemented and passed locally, but ignored/manual and tied
  to uncommitted local `test_repos/`.
- Optional Ctags comparator: implemented as an isolated optional quality plugin,
  but not installed, not packaged, not required in CI/release, and not part of
  production indexing.
- Language support hardening: support levels are honest, but Go/PHP and some
  experimental languages need more stable real-repo evidence.
- Incremental indexing: file scanning and no-change fast paths are ready, but
  relationship recomputation is not fully incremental after changes.
- Archive release docs: most packaging surfaces are current, but the RC handoff
  checksum is stale relative to the current archive and sidecar.

## Not 100% Ready

- Current Linux RC archive is smoke-tested, but other target archives still need
  target-platform smoke.
- File-reference extraction is best-effort local evidence. It does not claim
  package-manager, compiler, framework alias, export-map, LSP, or network
  semantics.
- Search/reference text matching is useful but heuristic; broad current-repo
  terms can surface prompt/doc noise.
- Local real-repo evidence is strong in this checkout but not reproducible from
  the committed repository alone because third-party corpora are intentionally
  uncommitted.
- Quality reports and comparator triage are useful maintainer tools, not
  product commands for normal users.

## Overclaims

- `docs/RC_0.1.4_HANDOFF.md` overstates the current artifact metadata by
  recording SHA256
  `a3629a31ce51d4935649fdad7ae82e55cb1c853047bf873bfccb397b017c6e1e`
  while the current archive and sidecar contain
  `1509a0bfb889068ff095a331a86713dcbadfd71aba955996f1e8392073c8591b`.
- Any claim that `wi refs` exposes both confidence and reason labels would be
  too broad. `pack` and `impact` do; `refs` prints reasons/evidence but not
  confidence.
- Any claim that `wi-scorecard` has the same installed/source schema-write
  guard as `wi` and `build_index` would be too broad.
- Any claim that real-repo checks are always-on CI evidence would be too broad;
  they are ignored/manual and local-corpus dependent.
- Any claim that MCP integration is implemented would be too broad; it is an
  advisory future plan.
- No active README/docs overclaim was found for `WI.md` as current, JSONL as
  canonical storage, Universal Ctags as production indexing, old parser
  architecture, four-binary archives, or old schema.

## Release Blockers

- Block Linux RC publication using the current handoff document until
  `docs/RC_0.1.4_HANDOFF.md` is corrected or regenerated with the current
  archive SHA256. The archive itself and `.sha256` sidecar verify, but the
  handoff document is stale.
- Block any release claim that `wi-scorecard` is protected by the same
  source/binary schema guard as `wi`/`build_index` until that is true. This does
  not block shipping the archive if the scorecard is described honestly and the
  stale handoff checksum is fixed.

## Non-Blocking Caveats

- `wi-stats` is advisory only and cannot audit external grep/find/ls/file-read
  usage.
- `wi-init` writes repo-local instruction files by design; this audit relied on
  help and tests rather than rerunning it in the clean checkout.
- Optional quality/comparator/triage workflows are local maintainer tools and
  not part of normal user workflows.
- The current release smoke is Linux `x86_64-unknown-linux-gnu`; other targets
  need their own smoke.
- Native installers, signing, notarization, package-manager publishing, hosted
  services, telemetry, payment/licensing enforcement, and MCP remain
  intentionally deferred.
- Manual real-repo checks passed locally but are slow and depend on local
  third-party repositories.

## Recommended Next Plans

1. RC handoff refresh: regenerate/correct `docs/RC_0.1.4_HANDOFF.md` from the
   current archive, sidecar, and `scripts/check-release` output.
2. Scorecard schema-guard hardening: add source/binary agreement enforcement to
   `wi-scorecard` before it can rebuild indexes from an installed binary in a
   source checkout.
3. `wi refs` output alignment: decide whether to print confidence labels and
   stored reason labels, or narrow docs to say confidence is a `pack`/`impact`
   feature.
4. Real-repo evidence stabilization: add a small committed synthetic corpus or
   curated local manifest process for Go/PHP and file-reference/import
   regressions without committing third-party code.
5. Optional quality CLI decision: either keep quality/comparator as maintainer
   library/test workflows, or add a clearly optional CLI surface with the same
   isolation boundaries.
6. Target-platform release smoke: run archive smoke and installer smoke on every
   non-Linux target before publishing those archives.

## Verification Commands And Results

- `git status --short`: clean before audit edits.
- `git log --oneline -20`: recent history includes recovery, file-reference,
  installed-binary/schema, scorecard install docs, RC readiness, and RC docs
  commits.
- `cargo fmt --check`: passed.
- `cargo test`: passed, `322 passed, 8 ignored`.
- `cargo clippy --all-targets --all-features -- -D warnings`: passed.
- `cargo run --bin build_index`: passed before the audit edit with
  `changed files: 0`, `records: 3295`; passed after adding this audit with
  `changed files: 1`, `records: 3306`; immediate second run passed with
  `changed files: 0`, `records: 3306`.
- `cargo run --bin wi -- doctor`: passed, `overall: ok`, schema 12 current,
  binary/source match.
- `cargo run --bin wi -- build_index`: passed with source/test/doc landmarks.
- `cargo run --bin wi -- refs build_index`: passed with primary definitions and
  references.
- `cargo run --bin wi -- pack build_index`: passed with bounded grouped read set
  and reasons/confidence.
- `cargo run --bin wi -- impact build_index`: passed with affected-file groups
  and reasons/confidence.
- `cargo run --bin wi-scorecard`: passed with `pass 9 / warn 1 / fail 0`.
- `cargo run --bin wi-stats -- --repo .`: passed and reported local usage plus
  advisory agent workflow audit.
- `cargo run --bin wi-init -- --help`: passed; help matches current generated
  instruction surfaces.
- PATH installed checks: `/home/tom/.local/bin/wi`,
  `/home/tom/.local/bin/build_index`, and `/home/tom/.local/bin/wi-scorecard`
  report `0.1.4 (index schema 12)`.
- `scripts/check-release`: passed, generated/validated/smoked current Linux
  archive.
- `scripts/smoke-release-archive
  dist/thinindex-0.1.4-x86_64-unknown-linux-gnu.tar.gz`: passed.
- `cargo test --test local_index -- --ignored`: passed, `1 passed`.
- `cargo test --test real_repos -- --ignored`: passed locally, `1 passed`, `3
  filtered out`, finished in 238.75s.
- `sha256sum dist/thinindex-0.1.4-x86_64-unknown-linux-gnu.tar.gz`: current
  archive hash is
  `1509a0bfb889068ff095a331a86713dcbadfd71aba955996f1e8392073c8591b`.
