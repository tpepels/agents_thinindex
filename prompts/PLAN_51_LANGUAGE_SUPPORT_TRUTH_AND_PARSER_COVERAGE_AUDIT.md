# PLAN_51_LANGUAGE_SUPPORT_TRUTH_AND_PARSER_COVERAGE_AUDIT.md

Do not implement this until `PLAN_50_REPO_LEGACY_CLEANUP_AND_DEFERRED_WORK_AUDIT.md`
is complete and green.

## Prerequisite

Plans 11 through 12 must have established the Tree-sitter parser backbone and
language-pack support. Plans 46 through 50 must be complete and committed so the
current docs, roadmap, release boundaries, and legacy cleanup audit are the
source of truth.

This plan is a bounded truth-in-claims and parser coverage audit. It should not
add broad parser support or run a Tree-sitter convergence cycle.

## Purpose

Audit language support claims against actual implementation, tests, fixtures,
docs, and user-facing surfaces.

thinindex has broad intended language coverage, but support claims must remain
honest. Languages may be supported, experimental, blocked, extras-backed,
planned, or only partially wired. Parser work must continue through the
Tree-sitter registry, query-spec, normalized-capture, conformance-fixture, and
support-matrix architecture.

## Scope

Plan 51 covers one bounded audit pass:

- inspect parser/language source, tests, fixtures, docs, prompts, and dependency
  declarations;
- compare the intended broad language set against actual implementation;
- verify support levels and user/developer/release docs do not overclaim;
- check for forbidden parser remnants and architecture violations;
- add or update `docs/LANGUAGE_SUPPORT_AUDIT.md`;
- update docs indexes, roadmap, and caveat summary only as needed;
- add small regression checks that keep the audit aligned with support rows when
  straightforward;
- run targeted parser/language/support/conformance tests;
- run ignored real-repo tests when local `test_repos/` data exists;
- commit the audit and stop.

## Intended Language Set

Audit at least:

- Rust
- Python
- JavaScript
- TypeScript
- JSX
- TSX
- Java
- Go
- C
- C++
- Shell
- Ruby
- PHP
- C#
- Nix
- Scala
- Kotlin
- Swift
- Dart
- CSS
- HTML
- Markdown
- JSON
- TOML
- YAML

Also keep blocked languages visible when they appear in the support matrix.

## Audit Questions

For each audited language or format, determine and document:

- support level: `supported`, `experimental`, `blocked`, or `extras-backed`;
- whether grammar registration exists;
- whether extension/file mapping exists;
- whether query specs exist;
- whether normalized captures map into records;
- whether conformance fixtures exist;
- whether quality or expected-symbol tests exist;
- whether real-repo coverage exists or is deferred;
- whether user docs claim support accurately;
- whether developer docs claim support accurately;
- whether release/docs/roadmap claims are accurate;
- known caveats.

## Support-level Rules

- `supported`: implemented, fixture-tested, documented, and normal validation
  covers the core parser path.
- `experimental`: implemented enough to index useful syntax, but incomplete,
  fragile, or not sufficiently hardened.
- `blocked`: not currently implemented or blocked by missing grammar, platform,
  dependency, license, quality, or unresolved design.
- `extras-backed`: project-owned deterministic format extraction, not
  Tree-sitter-backed code-symbol parsing.

Do not downgrade quietly. If a public claim changes, explain why in the audit.

## Parser Architecture Invariants

Adding a Tree-sitter-backed language means:

1. grammar registration;
2. extension mapping;
3. Tree-sitter query specs;
4. conformance fixture;
5. license/support entry.

Adding a language must not mean:

- hand-written parser;
- line scanner;
- regex parser;
- external tagger fallback;
- copy/pasted per-language extraction loop.

If a violation exists, record it and recommend a future scoped cleanup plan
instead of attempting a broad rewrite in this audit.

## Allowed Cleanup

- Correct inaccurate support-level docs.
- Remove stale support claims from README/docs.
- Add missing caveat text where support is partial.
- Add or adjust tiny fixture expectations only if they are already present and
  clearly stale.
- Update tests only if they assert outdated support claims or can cheaply guard
  the new audit doc.
- Update `docs/PLAN_CAVEATS_AND_UNIMPLEMENTED_SUMMARY.md` and
  `docs/ROADMAP.md` if support status or next-action guidance changes.

## Non-goals

- Do not add broad new parser support.
- Do not add new languages.
- Do not run Plan 45 convergence work.
- Do not clone or commit real repositories.
- Do not introduce ctags.
- Do not use external taggers as production parsers.
- Do not implement semantic/compiler/LSP features.
- Do not weaken tests to make claims easier.

## Hard Constraints

- Do not reintroduce `WI.md`.
- Do not make JSONL canonical.
- Do not use Universal Ctags as a production parser.
- Do not call ctags from `build_index`.
- Do not bundle ctags.
- Do not make ctags required for install, build, runtime, tests, or release
  artifacts.
- Do not emit production records with `source = "ctags"`.
- Do not claim semantic/compiler/LSP-level analysis unless actually implemented.
- Do not claim unsupported or experimental languages as fully supported.
- Do not commit `test_repos/` contents.

## Implementation Steps

- [x] Phase 1: run `build_index` before broad discovery.
- [x] Phase 2: inspect git status, recent history, active plans, and parser/language file inventory.
- [x] Phase 3: search language, parser, support-level, conformance, fixture, and forbidden-parser claims.
- [x] Phase 4: compare `src/support.rs`, `src/tree_sitter_extraction.rs`, Cargo dependencies, fixtures, parser/support tests, README, parser docs, language dashboard, release/user/developer docs, and real-repo manifest docs.
- [x] Phase 5: add or update `docs/LANGUAGE_SUPPORT_AUDIT.md`.
- [x] Phase 6: correct stale support claims, if any, without adding broad parser support.
- [x] Phase 7: add small regression coverage for the audit doc if straightforward.
- [x] Phase 8: update roadmap, caveat summary, docs indexes, and this checklist honestly.
- [x] Phase 9: run required validation, commit, and stop.

## Validation Steps

- [x] `git diff --check`
- [x] `cargo fmt --check`
- [x] `cargo test`
- [x] `cargo clippy --all-targets --all-features -- -D warnings`
- [x] `cargo run --bin build_index`
- [x] `cargo run --bin wi -- --help`
- [x] `cargo run --bin wi -- doctor`
- [x] `cargo test parser`
- [x] `cargo test language`
- [x] `cargo test conformance`
- [x] `cargo test support_level`
- [x] `cargo test --test real_repos -- --ignored` if local `test_repos/` exists
- [x] `git status --short`
- [x] final `git diff --check`

## Acceptance Criteria

- `docs/LANGUAGE_SUPPORT_AUDIT.md` exists and contains a compact support table.
- Each intended language/format has a clear support level and implementation
  evidence summary.
- User-facing docs do not claim unsupported or experimental languages as fully
  supported.
- Extras-backed formats are not described as Tree-sitter-backed code-symbol
  parsers.
- Blocked languages remain visible and unclaimed.
- No parser architecture violation is introduced.
- No broad parser support, new language support, or semantic/compiler/LSP
  behavior is added.
- Local `test_repos/` contents are not committed.
- Validation passes or an inapplicable targeted command is clearly reported.

## Completion And Update Instructions

After implementation and verification:

- update this plan's checkboxes honestly;
- update `docs/ROADMAP.md` only to reflect the audit result and next truthful
  action;
- update `docs/PLAN_CAVEATS_AND_UNIMPLEMENTED_SUMMARY.md` only to reflect Plan
  51 status, corrected claims, and remaining deferred work;
- do not mark deferred parser, semantic, real-repo, release, hosted, telemetry,
  payment, or license-enforcement caveats as complete unless implemented by a
  scoped plan;
- commit with:

`Audit language support claims and parser coverage`

Stop after this one bounded audit pass.

## Final Report Requirements

- files changed
- language support table summary
- claims corrected
- parser architecture violations found, if any
- fixture/test changes, if any
- real-repo coverage status
- validation commands and results
- commit hash
- recommended next action: real-repo test readiness audit
