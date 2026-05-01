# PLAN_50_REPO_LEGACY_CLEANUP_AND_DEFERRED_WORK_AUDIT.md

Do not implement this until `PLAN_49_DOCUMENTATION_CLEANUP_AND_INDEXES.md` is
complete and green.

## Prerequisite

Plans 46 through 49 must be complete and committed:

- `PLAN_46_FULL_PLAN_COMPLETION_AUDIT.md`
- `PLAN_47_RELEASE_DISTRIBUTION_COMPLETION.md`
- `PLAN_48_RELEASE_ARCHIVE_HARDENING.md`
- `PLAN_49_DOCUMENTATION_CLEANUP_AND_INDEXES.md`

Treat the post-Plan-49 repository state as the source of truth. This plan is a
bounded repo-wide legacy cleanup and deferred-work audit. It may update docs,
plan checkboxes, and audit handoff notes, but it must not implement deferred
product features opportunistically.

## Purpose

thinindex has accumulated work across storage, parser replacement, quality
plugins, real-repo hardening, release archives, installer/signing scaffolds,
licensing boundaries, agent integration, and documentation indexing. Plan 50
checks whether stale code, docs, scripts, generated artifacts, old parser
assumptions, or unclear deferred features remain after that sequence.

The goal is to make stale surfaces explicit, remove or rewrite obvious stale
documentation, and record remaining legacy/deferred items in one concise audit.

## Scope

Plan 50 covers one bounded audit/cleanup pass:

- inventory likely stale or legacy surfaces across code, docs, scripts, tests,
  prompts, CI, release files, and local/generated directories;
- remove or rewrite obviously stale documentation;
- delete or quarantine obviously dead generated artifacts or scripts only when
  safe, unreferenced, and verified;
- identify unimplemented or deferred features clearly;
- create or update `docs/REPO_LEGACY_CLEANUP_AUDIT.md`;
- update `docs/README.md`, `docs/ROADMAP.md`, and
  `docs/PLAN_CAVEATS_AND_UNIMPLEMENTED_SUMMARY.md` only as needed to point to
  the new audit and the next truthful action;
- commit audit/alignment changes only.

## Audit Categories

Check these surfaces and either clean them up or classify why they remain:

1. Forbidden legacy surfaces:
   - `WI.md` references;
   - JSONL as canonical storage;
   - Universal Ctags as production parser;
   - ctags called by `build_index`;
   - bundled or required ctags;
   - production records with `source = "ctags"`;
   - old native line-scanner parser architecture;
   - regex/parser fallback language extraction loops;
   - unsupported languages claimed as supported;
   - compiler/LSP/semantic claims not implemented;
   - payment, hosted, telemetry, activation, managed-update, or
     license-enforcement claims not implemented.
2. Stale release/distribution surfaces:
   - claims that native packages are complete when only archive packaging exists;
   - claims that signing, notarization, publishing, or update channels are
     complete when they are scaffolded or future work;
   - generated `dist/` artifacts accidentally tracked;
   - release scripts that no longer match docs;
   - docs that imply secrets or external services are required for normal
     validation.
3. Stale parser/language surfaces:
   - old parser implementations that violate the Tree-sitter architecture
     invariant;
   - stale language support claims;
   - query fixtures not reflected in docs;
   - support-level mismatch between code, tests, and docs;
   - unimplemented languages listed as supported.
4. Stale test/real-repo surfaces:
   - `test_repos/` contents accidentally tracked;
   - normal tests depending on local corpora;
   - ignored tests that are stale or impossible to run;
   - manifest docs that do not match the actual schema;
   - quality gates that reference missing scripts or commands.
5. Dead or misleading docs/scripts:
   - superseded but unmarked docs;
   - duplicated content that conflicts with canonical docs;
   - internal history presented as user-facing workflow;
   - unreferenced scripts with no plausible current use.

## Allowed Cleanup

- Remove or rewrite stale docs.
- Move still-valid content into canonical docs.
- Mark historical/superseded material clearly.
- Delete clearly dead generated artifacts or accidental local output.
- Delete clearly unused scripts only when they are not referenced by docs, CI,
  tests, scripts, or active plans and validation passes.
- Update `docs/ROADMAP.md` and
  `docs/PLAN_CAVEATS_AND_UNIMPLEMENTED_SUMMARY.md` to reflect truthful current
  state.
- Create or update `docs/REPO_LEGACY_CLEANUP_AUDIT.md`.

## Non-goals

- Do not add new product features.
- Do not add parser or language support.
- Do not run a Tree-sitter convergence cycle.
- Do not implement release packaging, native installers, signing, publishing,
  update channels, payments, hosted behavior, telemetry, account behavior,
  network activation, or license enforcement.
- Do not implement semantic/compiler/LSP analysis.
- Do not rewrite large architecture without a separate scoped plan.
- Do not delete active plan files.
- Do not delete superseded plans unless the repository already has a clear
  convention for doing so.
- Do not commit `test_repos/` contents.

## Hard Constraints

- Do not reintroduce `WI.md`.
- Do not make JSONL canonical.
- Do not use Universal Ctags as a production parser.
- Do not call ctags from `build_index`.
- Do not bundle ctags.
- Do not make ctags required for install, build, runtime, tests, or release
  artifacts.
- Do not emit production records with `source = "ctags"`.
- Do not weaken parser/index quality gates.
- Do not claim semantic/compiler/LSP-level analysis unless actually implemented.
- Do not claim unsupported or experimental languages as fully supported.
- Do not add payments, hosted services, telemetry, network activation, managed
  updates, account behavior, or license enforcement.

## Implementation Steps

- [x] Phase 1: run `build_index` before broad discovery.
- [x] Phase 2: inspect git status, recent history, active plans, superseded plans,
      and broad repository file inventory.
- [x] Phase 3: run broad stale-surface searches for legacy parser, storage,
      release, licensing, hosted, telemetry, and TODO-style markers.
- [x] Phase 4: inspect high-value user, developer, roadmap, caveat, release,
      parser, quality, and script surfaces.
- [x] Phase 5: clean up or rewrite obvious stale docs without changing product
      behavior.
- [x] Phase 6: delete or quarantine only clearly safe generated/dead artifacts,
      if any are tracked or staged.
- [x] Phase 7: create or update `docs/REPO_LEGACY_CLEANUP_AUDIT.md`.
- [x] Phase 8: update roadmap, caveat summary, docs index, and this checklist
      honestly.
- [x] Phase 9: run required validation, commit, and stop.

## Validation Steps

- [x] `git diff --check`
- [x] `cargo fmt --check`
- [x] `cargo test`
- [x] `cargo clippy --all-targets --all-features -- -D warnings`
- [x] `cargo run --bin build_index`
- [x] `cargo run --bin wi -- --help`
- [x] `cargo run --bin wi -- doctor`
- [x] `scripts/check-release` if release docs/scripts changed
- [x] `cargo deny check licenses` if dependency/license/release docs changed
- [x] `git status --short`
- [x] final `git diff --check`

## Acceptance Criteria

- `docs/REPO_LEGACY_CLEANUP_AUDIT.md` exists and summarizes stale items removed,
  stale items rewritten, remaining legacy/deferred surfaces, future-plan risks,
  and the recommended next action.
- Any stale docs changed by this pass are more truthful after the edit.
- No product behavior, parser behavior, release packaging behavior, licensing
  enforcement, payment, hosted, telemetry, network activation, managed update,
  account, or semantic/compiler/LSP behavior is added.
- Ignored local/generated directories such as `.dev_index/`, `dist/`, and
  `test_repos/` are not committed.
- Active plans and docs point to one truthful next action.
- Validation passes.

## Completion And Update Instructions

After implementation and verification:

- update this plan's checkboxes honestly;
- update `docs/ROADMAP.md` only to reflect the audit result and next truthful
  action;
- update `docs/PLAN_CAVEATS_AND_UNIMPLEMENTED_SUMMARY.md` only to reflect the
  Plan 50 audit status and remaining deferred work;
- do not mark deferred product, parser, release, hosted, telemetry, payment, or
  license-enforcement caveats as complete unless implemented by a scoped plan;
- commit with:

`Clean up legacy surfaces and audit deferred work`

Stop after this one bounded cleanup/audit pass.

## Final Report Requirements

- files changed
- stale items removed
- stale items rewritten
- legacy/deferred items still present
- items intentionally left for future scoped plans
- validation commands and results
- commit hash
- recommended next action: language support truth audit
