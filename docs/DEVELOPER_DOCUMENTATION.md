# Developer Documentation

This index is for people contributing to thinindex. It focuses on architecture,
validation, invariants, and where to make changes.

## Architecture Overview

- [Project README](../README.md): product shape, command list, storage model, parser support matrix, install/release summary.
- [Technical final audit](TECHNICAL_FINAL_AUDIT.md): relationship/navigation architecture checkpoint.
- [Repository legacy cleanup audit](REPO_LEGACY_CLEANUP_AUDIT.md): stale-surface review and deferred-work handoff.
- [Roadmap](ROADMAP.md): shipped behavior, future work, and product boundaries.
- [Caveats and unimplemented summary](PLAN_CAVEATS_AND_UNIMPLEMENTED_SUMMARY.md): deferred work and known limits.

Core relationship docs:

- [Reference graph](REFERENCE_GRAPH.md)
- [Dependency graph](DEPENDENCY_GRAPH.md)
- [Context packs](CONTEXT_PACKS.md)
- [Impact analysis](IMPACT_ANALYSIS.md)
- [File roles](FILE_ROLES.md)
- [Semantic adapter boundary](SEMANTIC_ADAPTERS.md)

## Storage And Indexing

The canonical local cache is `.dev_index/index.sqlite`. Old JSONL index caches
are disposable migration input only, not current storage.

Start with:

- [Performance](PERFORMANCE.md)
- [Security and privacy](SECURITY_PRIVACY.md)
- [Technical final audit](TECHNICAL_FINAL_AUDIT.md)

## Parser Architecture

Production code-symbol extraction uses the Tree-sitter framework: registry,
strategy/adapter wiring, query specs, normalized captures, conformance fixtures,
support levels, and notice/license coverage.

Adding a Tree-sitter-backed language means updating grammar registration,
extension mapping, query specs, conformance fixtures, support/docs entries, and
license/notice entries. Do not add hand-written line scanners or a second code
parser architecture.

Read:

- [Parser support levels](PARSER_SUPPORT.md)
- [Parser maintenance](PARSER_MAINTENANCE.md)
- [Language support dashboard](LANGUAGE_SUPPORT.md)
- [Language support audit](LANGUAGE_SUPPORT_AUDIT.md): claim-vs-implementation evidence, real-repo coverage gaps, and parser architecture findings.
- [License audit](LICENSE_AUDIT.md)

## Quality And Real-repo Hardening

Quality tooling is isolated from production indexing. Optional comparator
output is local QA evidence and must never populate production SQLite `records`
or `refs`.

Universal Ctags is optional, external, not bundled, not required, and not used by production indexing.

Read:

- [Quality plugins](QUALITY.md)
- [Quality loop](QUALITY_LOOP.md)
- [Quality system audit](QUALITY_SYSTEM_AUDIT.md)
- [Real-repo manifest](REAL_REPO_MANIFEST.md)

`test_repos/` is local-only and ignored. Normal `cargo test` must not depend on
local cloned repositories.

## Release And Distribution

The completed release path is local archive packaging plus validation. Native
package formats, real signing/notarization, GitHub Release publishing,
package-manager distribution, and managed update channels remain future or
scaffolded work.

Read:

- [Releasing](RELEASING.md)
- [Installers and signing](INSTALLERS.md)
- [Release checklist](RELEASE_CHECKLIST.md)
- [CI integration](CI_INTEGRATION.md)
- [Licensing foundation](LICENSING.md)
- [License audit](LICENSE_AUDIT.md)

## Security, Privacy, And Product Boundaries

- [Security and privacy](SECURITY_PRIVACY.md): local caches, redaction, release artifact exclusions.
- [Product boundary](PRODUCT_BOUNDARY.md): free/local core and deferred paid/hosted work.
- [Team CI roadmap](TEAM_CI_ROADMAP.md): roadmap-only team/CI/hosted ideas with no-source-upload constraints.

Do not add payment handling, hosted behavior, network activation, account
behavior, telemetry, managed updates, or license enforcement without a scoped
future plan.

## Test Commands

Normal validation:

```bash
cargo fmt --check
cargo test
cargo clippy --all-targets --all-features -- -D warnings
cargo run --bin build_index
cargo run --bin wi -- doctor
cargo deny check licenses
```

Release validation:

```bash
scripts/package-release
scripts/check-package-contents <archive>
scripts/smoke-release-archive <archive>
scripts/check-release
```

Manual/local-only checks:

```bash
cargo test --test local_index -- --ignored
cargo test --test real_repos -- --ignored
cargo test --test bench_repos -- --ignored
```

Run ignored real-repo checks only when local `test_repos/` data exists and the
change touches parser, quality, refs, pack, impact, dependency, benchmark, or
real-repo assumptions.

## Active Plans And Invariants

Active plan files live under `../prompts/PLAN_*.md`. Superseded parser plans
remain under `../prompts/superseded/` as history and should not be revived.

Key invariants:

- do not reintroduce `WI.md`;
- do not make JSONL canonical storage;
- do not use ctags as a production parser;
- do not emit production records with `source = "ctags"`;
- do not claim unsupported or experimental languages as fully supported;
- do not claim compiler/LSP-level semantics unless implemented;
- do not commit `test_repos/` contents.

See [Plan 46 audit](../prompts/PLAN_46_FULL_PLAN_COMPLETION_AUDIT.md) and
[Plan 49](../prompts/PLAN_49_DOCUMENTATION_CLEANUP_AND_INDEXES.md) for current
plan hygiene and documentation cleanup requirements.
