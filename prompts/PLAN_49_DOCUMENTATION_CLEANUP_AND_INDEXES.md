# PLAN_49_DOCUMENTATION_CLEANUP_AND_INDEXES.md

Use superpowers:subagent-driven-development.

Do not implement this until `PLAN_48_RELEASE_ARCHIVE_HARDENING.md` is complete and green.

## Prerequisite

Plan 47 and Plan 48 must be complete and committed:

- `PLAN_47_RELEASE_DISTRIBUTION_COMPLETION.md`
- `PLAN_48_RELEASE_ARCHIVE_HARDENING.md`

Treat the post-Plan-48 docs and release/distribution state as the source of
truth. This plan is documentation cleanup and documentation indexing only.

## Purpose

Make the documentation set easier to browse and harder to misread.

The current docs contain accurate but distributed information about usage,
architecture, parser support, release archives, quality gates, real-repo
hardening, product boundaries, and intentionally deferred work. Plan 49 should
organize those docs into clear user and developer entry points, remove or
rewrite stale claims, and keep release/distribution wording truthful after Plans
47 and 48.

## Scope

Plan 49 is a bounded documentation pass. It covers:

- inventorying current repository documentation and classifying each doc by
  purpose;
- removing, rewriting, or clearly marking stale docs and stale claims;
- creating or updating `docs/USER_DOCUMENTATION.md`;
- creating or updating `docs/DEVELOPER_DOCUMENTATION.md`;
- creating or updating `docs/README.md` as a general docs landing page;
- keeping documentation truthful about post-Plan-48 release archive behavior;
- updating `docs/ROADMAP.md` only when needed to reflect the next truthful action;
- validating edited Markdown links with a lightweight local check;
- committing docs/plan alignment only.

## Documentation Inventory

Before editing docs, inventory current documentation and classify it into at
least these categories:

- user-facing docs;
- developer/contributor docs;
- release/distribution docs;
- roadmap/plan/handoff docs;
- stale or superseded docs;
- generated or third-party notice docs.

The inventory must include the top-level `README.md`, `docs/*.md`, active
`prompts/PLAN_*.md` files, `prompts/local_repo_test.md`, superseded prompt files
only when active docs link to them, and notice/license docs such as
`THIRD_PARTY_NOTICES`, `docs/LICENSE_AUDIT.md`, and `docs/LICENSING.md`.

Do not commit or rely on `test_repos/` contents. If local `test_repos/` exists,
note that it is local-only and out of scope for normal docs validation.

## Stale Documentation Cleanup

Remove, rewrite, or clearly mark stale documentation that implies any of the
following:

- `WI.md` is an instruction artifact;
- JSONL is canonical index storage;
- Universal Ctags is a production parser;
- Universal Ctags is bundled, required, or called by `build_index`;
- production records or refs can have `source = "ctags"`;
- old native line-scanner parsers are the intended parser backbone;
- unsupported or experimental languages are fully supported;
- thinindex has semantic/compiler/LSP-level analysis unless actually implemented;
- native installers, signing, notarization, GitHub Release publishing,
  package-manager publishing, managed updates, payment enforcement, hosted
  behavior, telemetry, network activation, or account behavior are complete
  unless actually implemented;
- `test_repos/` contents are committed or required for normal tests.

Preserve valid guardrails that say not to reintroduce `WI.md`, JSONL canonical
storage, ctags production parsing, hosted behavior, telemetry, payment handling,
license enforcement, or committed `test_repos/` content.

## User Documentation Index

Create or update:

`docs/USER_DOCUMENTATION.md`

This index is for people who want to use thinindex, not modify it. It must link
to relevant documentation for:

- what thinindex is;
- the local-first model;
- install, build, and run basics;
- core workflow:
  - `build_index`
  - `wi <term>`
  - `wi refs <term>`
  - `wi pack <term>`
  - `wi impact <term>`
  - `wi-stats`
  - `wi-init`
  - `wi doctor`
  - `wi --help`
- `.dev_index/index.sqlite`;
- what thinindex does not do;
- language support and support levels;
- release/archive distribution status;
- troubleshooting and doctor guidance;
- privacy/security posture;
- where to look next.

Do not expose internal plan sequencing as the main user path. If roadmap or plan
history is linked from the user index, label it clearly as roadmap/history.

## Developer Documentation Index

Create or update:

`docs/DEVELOPER_DOCUMENTATION.md`

This index is for people contributing to thinindex. It must link to relevant
documentation for:

- architecture overview;
- local SQLite index/storage model;
- parser architecture;
- Tree-sitter language registry, query-spec, and conformance fixture workflow;
- language support levels;
- quality gates;
- real-repo hardening approach;
- optional comparator/quality plugin boundary;
- release archive packaging and release-distribution scaffolding;
- security, privacy, and redaction notes;
- test commands;
- roadmap and active plan files;
- forbidden surfaces and invariants;
- caveats and intentionally deferred work.

## General Docs Landing Page

Create or update:

`docs/README.md`

It must link to:

- `docs/USER_DOCUMENTATION.md`;
- `docs/DEVELOPER_DOCUMENTATION.md`;
- `docs/ROADMAP.md`;
- `docs/PLAN_CAVEATS_AND_UNIMPLEMENTED_SUMMARY.md`;
- release/distribution docs;
- security/privacy docs;
- license and third-party notice docs;
- other high-value docs that help readers choose the right next document.

Keep `docs/README.md` short. It should route readers to the right doc, not
duplicate every doc.

## Link Hygiene

Use relative Markdown links that are browsable from the repository.

If the repository already has a Markdown or link-check command, use it. If no
such command exists, use a lightweight one-off local check for relative `.md`
links introduced or edited in this pass. A one-off script is acceptable; do not
add a permanent link-checking system unless it is very small, docs-only, and
clearly justified by this plan.

The link check should at minimum verify that relative Markdown links to `.md`
files in edited docs resolve on disk, allowing anchors and ignoring external
URLs, mail links, and non-Markdown assets.

## Roadmap Alignment

Update `docs/ROADMAP.md` only if needed to reflect:

- Plan 47/48 release-distribution status;
- Plan 49 documentation cleanup/indexing status;
- the next truthful action after Plan 49.

Do not claim Plan 49 is complete until it has been executed, verified, and
committed. Do not claim native packages, signing, notarization, GitHub Release
publishing, package-manager publishing, managed updates, payment/licensing
enforcement, hosted behavior, telemetry, network activation, account behavior,
or semantic/compiler/LSP-level analysis exist unless they actually exist.

## Documentation Quality Requirements

- Prefer one canonical explanation per topic.
- Avoid duplicate, conflicting explanations.
- Keep user docs focused on usage.
- Keep developer docs focused on architecture, contribution, validation, and invariants.
- Keep roadmap/plan docs clearly separate from user onboarding.
- Be explicit about support levels: `supported`, `experimental`, `blocked`, and `extras-backed`.
- Be explicit that Tree-sitter provides syntax-tree facts, not compiler/LSP semantics.
- Be explicit that ctags may appear only as an optional isolated QA comparator plugin, never production indexing.
- Be explicit that release archives are local, source-upload-free, and credential-free.
- Be explicit that native packaging/signing/notarization/publishing/update-channel work remains future or scaffolded unless actually implemented.

## Non-goals

- Do not implement product features.
- Do not change parser behavior.
- Do not add parser or language support.
- Do not add release packaging behavior.
- Do not create native installers.
- Do not add signing, notarization, publishing, update channels, telemetry,
  hosted behavior, payment handling, network activation, account behavior, or
  license enforcement.
- Do not add semantic/compiler/LSP-level analysis.
- Do not run or require `test_repos/`.
- Do not delete active plan files.
- Do not delete superseded plans unless the repository already has a clear
  convention for doing so.
- Do not remove valid technical details; move or link them from a more
  appropriate canonical location.

## Hard Constraints

- Do not reintroduce `WI.md`.
- Do not make JSONL canonical storage.
- Do not use Universal Ctags as a production parser.
- Do not call ctags from `build_index`.
- Do not bundle ctags.
- Do not make ctags required for install, build, runtime, tests, or release artifacts.
- Do not emit production records with `source = "ctags"`.
- Do not weaken parser/index quality gates.
- Do not claim semantic/compiler/LSP-level analysis unless actually implemented.
- Do not claim unsupported or experimental languages as fully supported.
- Do not add payments, hosted services, network activation, telemetry, managed updates, or license enforcement.
- Do not commit `test_repos/` contents.
- Do not implement deferred caveats opportunistically.

## Implementation Steps

- [x] Phase 1: run the current documentation inventory and classify docs by purpose.
- [x] Phase 2: audit current docs for stale or risky claims listed in this plan.
- [x] Phase 3: remove, rewrite, or clearly mark stale documentation while preserving valid guardrails.
- [x] Phase 4: create or update `docs/USER_DOCUMENTATION.md`.
- [x] Phase 5: create or update `docs/DEVELOPER_DOCUMENTATION.md`.
- [x] Phase 6: create or update `docs/README.md`.
- [x] Phase 7: update `README.md`, `docs/ROADMAP.md`, or handoff docs only where needed to point readers to the new indexes and truthful next action.
- [x] Phase 8: run link hygiene checks for edited Markdown links.
- [x] Phase 9: run required validation, update this checklist, commit, and stop.

## Validation Steps

Documentation validation:

- [x] `git diff --check`
- [x] `ls docs/*.md | sort`
- [x] `grep -RIn "WI.md\\|JSONL\\|jsonl\\|ctags\\|Ctags\\|Universal Ctags\\|source = \"ctags\"\\|semantic\\|LSP\\|installer\\|notarization\\|payment\\|license enforcement\\|test_repos\\|managed update\\|telemetry\\|hosted\\|network activation" README.md docs prompts 2>/dev/null || true`
- [x] Review the grep output and classify matches as valid guardrails, accurate caveats, or stale claims fixed by this plan.
- [x] `grep -RIn "supported\\|experimental\\|blocked\\|extras-backed" README.md docs prompts 2>/dev/null || true`
- [x] Review support-level matches and confirm no unsupported or experimental language is overclaimed.
- [x] `grep -RIn "build_index\\|wi refs\\|wi pack\\|wi impact\\|wi doctor\\|wi-stats\\|wi-init" README.md docs 2>/dev/null || true`
- [x] Confirm the user/developer indexes route readers to command docs without duplicating `wi --help` as CLI source of truth.
- [x] Run the repository's Markdown/link-check command if one exists, otherwise run a one-off relative `.md` link check for edited Markdown files.

Baseline validation:

- [x] `cargo fmt --check`
- [x] `cargo test`
- [x] `cargo clippy --all-targets --all-features -- -D warnings`

Run only if this plan changes code, scripts, generated docs, parser support,
release behavior, quality behavior, or real-repo assumptions:

- [x] Not applicable after docs-only changes: `cargo run --bin build_index`
      was run before editing for repository discovery.
- [x] Not applicable after docs-only changes: `cargo run --bin wi -- doctor`.
- [x] Not applicable after docs-only changes: `cargo deny check licenses`.
- [x] Not applicable after docs-only changes: `cargo test --test local_index -- --ignored`.
- [x] Not applicable after docs-only changes: `cargo test --test real_repos -- --ignored` if `test_repos/` exists.

Plan 49 should normally be docs-only. If the implementation pass changes only
Markdown documentation and prompt files, the ignored local/real-repo tests are
not required.

## Acceptance Criteria

- Current docs have a clear inventory and purpose classification.
- Stale or misleading docs are removed, rewritten, or explicitly marked.
- `docs/USER_DOCUMENTATION.md` exists and provides a clear user path.
- `docs/DEVELOPER_DOCUMENTATION.md` exists and provides a clear contributor path.
- `docs/README.md` exists and routes readers to user, developer, roadmap,
  release, security/privacy, license, and caveat docs.
- User docs focus on usage rather than plan sequencing.
- Developer docs focus on architecture, contribution, validation, and invariants.
- Release docs remain truthful after Plans 47 and 48.
- Support-level docs distinguish `supported`, `experimental`, `blocked`, and
  `extras-backed`.
- Tree-sitter, ctags, JSONL, WI.md, semantic/LSP, hosted, telemetry, payment,
  license-enforcement, release, and `test_repos/` boundaries remain accurate.
- Edited relative Markdown links resolve locally.
- No product behavior changes are introduced.

## Completion And Update Instructions

After implementation and verification:

- update this plan's checkboxes honestly;
- update `docs/ROADMAP.md` only to reflect the shipped documentation indexes
  and the next truthful action;
- update `docs/PLAN_CAVEATS_AND_UNIMPLEMENTED_SUMMARY.md` only if stale
  documentation caveats are resolved or reclassified;
- do not mark deferred product, parser, release, hosted, telemetry, payment, or
  license-enforcement caveats as complete unless they were actually implemented
  by a scoped plan;
- commit with:

`Add documentation cleanup and indexes`

Stop after this one scoped documentation pass. Do not start parser, release,
quality, payment, hosted, telemetry, licensing, or product-feature work.

## Final Report Requirements

- documentation inventory summary
- stale docs or claims found and fixed
- user index path and contents summary
- developer index path and contents summary
- general docs landing page path and contents summary
- files changed
- link hygiene check and result
- validation commands and results
- ignored local/real-repo test status, if applicable
- commit hash
- next recommended prompt/action

## Recommended Next Action

Choose the next scoped plan from the updated roadmap and caveat summary rather
than implementing deferred features opportunistically.
