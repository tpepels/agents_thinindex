# Documentation

Start here when you know what kind of information you need.

## Main Indexes

- [User documentation](USER_DOCUMENTATION.md): install, setup, commands, local-first model, release archives, troubleshooting, and what thinindex does not do.
- [Developer documentation](DEVELOPER_DOCUMENTATION.md): architecture, parser workflow, quality gates, release validation, invariants, tests, and active plans.
- [Project README](../README.md): short product overview, command list, storage model, support matrix, and install/release summary.
- [Roadmap](ROADMAP.md): shipped behavior, product direction, and future work.
- [Caveats and unimplemented summary](PLAN_CAVEATS_AND_UNIMPLEMENTED_SUMMARY.md): known limits, deferred work, and guardrails.

## Documentation Inventory

| Category | Documents | Purpose |
| --- | --- | --- |
| User-facing docs | [USER_DOCUMENTATION.md](USER_DOCUMENTATION.md), [GETTING_STARTED.md](GETTING_STARTED.md), [TROUBLESHOOTING.md](TROUBLESHOOTING.md), [AGENT_INTEGRATION.md](AGENT_INTEGRATION.md) | Help users set up a repo, run commands, and follow the agent-navigation workflow. |
| Developer/contributor docs | [DEVELOPER_DOCUMENTATION.md](DEVELOPER_DOCUMENTATION.md), [TECHNICAL_FINAL_AUDIT.md](TECHNICAL_FINAL_AUDIT.md), [PARSER_MAINTENANCE.md](PARSER_MAINTENANCE.md), [PERFORMANCE.md](PERFORMANCE.md) | Explain architecture, validation, parser maintenance, and performance constraints. |
| Parser and support docs | [PARSER_SUPPORT.md](PARSER_SUPPORT.md), [LANGUAGE_SUPPORT.md](LANGUAGE_SUPPORT.md), [SEMANTIC_ADAPTERS.md](SEMANTIC_ADAPTERS.md) | Track support levels and keep syntax extraction separate from semantic/compiler/LSP claims. |
| Relationship/navigation docs | [REFERENCE_GRAPH.md](REFERENCE_GRAPH.md), [DEPENDENCY_GRAPH.md](DEPENDENCY_GRAPH.md), [CONTEXT_PACKS.md](CONTEXT_PACKS.md), [IMPACT_ANALYSIS.md](IMPACT_ANALYSIS.md), [FILE_ROLES.md](FILE_ROLES.md) | Describe SQLite-backed refs, dependency evidence, context packs, impact output, and file-role mapping. |
| Quality and real-repo docs | [QUALITY.md](QUALITY.md), [QUALITY_LOOP.md](QUALITY_LOOP.md), [QUALITY_SYSTEM_AUDIT.md](QUALITY_SYSTEM_AUDIT.md), [REAL_REPO_MANIFEST.md](REAL_REPO_MANIFEST.md) | Explain deterministic gates, optional comparator boundaries, local quality loops, and ignored real-repo checks. |
| Release/distribution docs | [RELEASING.md](RELEASING.md), [INSTALLERS.md](INSTALLERS.md), [RELEASE_CHECKLIST.md](RELEASE_CHECKLIST.md), [CI_INTEGRATION.md](CI_INTEGRATION.md) | Document local release archives, archive helpers, checksums, smoke checks, CI gates, and scaffolded future distribution work. |
| Security, privacy, and product boundary docs | [SECURITY_PRIVACY.md](SECURITY_PRIVACY.md), [PRODUCT_BOUNDARY.md](PRODUCT_BOUNDARY.md), [TEAM_CI_ROADMAP.md](TEAM_CI_ROADMAP.md), [LICENSING.md](LICENSING.md) | Keep local-first, no-source-upload, no-telemetry, no-payment, and no-enforcement boundaries explicit. |
| License and notice docs | [LICENSE_AUDIT.md](LICENSE_AUDIT.md), [THIRD_PARTY_NOTICES](../THIRD_PARTY_NOTICES) | Track dependency license policy, parser grammar notices, SQLite bundling, and release notice requirements. |
| Roadmap and handoff docs | [ROADMAP.md](ROADMAP.md), [PLAN_CAVEATS_AND_UNIMPLEMENTED_SUMMARY.md](PLAN_CAVEATS_AND_UNIMPLEMENTED_SUMMARY.md), [REPO_LEGACY_CLEANUP_AUDIT.md](REPO_LEGACY_CLEANUP_AUDIT.md), active [PLAN files](../prompts/) | Preserve planning status, deferred work, and next-action guidance. These are not the main user onboarding path. |
| Stale or superseded docs | [superseded parser plans](../prompts/superseded/), [local repo test prompt](../prompts/local_repo_test.md) | Historical or local-only planning material. Do not revive superseded plans or treat local prompts as user docs. |

## High-signal Boundaries

- `wi --help` is the source of truth for command syntax, filters, examples, and subcommands.
- `.dev_index/index.sqlite` is the canonical local index cache.
- JSONL appears only in disposable legacy caches or quality report exports, not canonical storage.
- Tree-sitter-backed extraction is syntactic and does not claim compiler or LSP semantics.
- Universal Ctags is optional, external, not bundled, not required, and not used by production indexing.
- Release archives are local, source-upload-free, credential-free, and unsigned by default.
- Native packages, real signing/notarization, publishing, managed updates, hosted behavior, telemetry, payments, account behavior, network activation, and license enforcement are not shipped behavior.
