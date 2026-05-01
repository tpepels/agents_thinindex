# Repository Legacy Cleanup Audit

Audit date: 2026-05-01.

This audit records one bounded repo-wide legacy cleanup and deferred-work review
after Plan 49. It did not implement product features, parser support, release
packaging behavior, licensing enforcement, payment behavior, hosted behavior,
telemetry, network activation, managed updates, account behavior, or
semantic/compiler/LSP analysis.

## Scope Reviewed

- Git status and recent commit history.
- Active `prompts/PLAN_*.md` files and superseded parser plans.
- Rust source, tests, scripts, CI workflows, release docs, parser docs, quality
  docs, user/developer docs, roadmap, and caveat summary.
- Local/generated directories visible on disk: `.dev_index/`, `dist/`, and
  `test_repos/`.
- Broad stale-surface searches for `WI.md`, JSONL, external comparator
  boundaries, old parser terms, semantic/LSP claims, release/signing claims,
  hosted/payment/telemetry/licensing claims, `test_repos/`, and TODO-style
  markers.

## Stale Items Removed

No tracked product code, scripts, active plan files, superseded plans, or
release artifacts were removed in this pass.

Reason: the audit did not find a clearly dead tracked script or generated file
that was both unreferenced and safe to delete. Local generated directories were
ignored by git rather than tracked.

## Stale Items Rewritten

- `docs/PRODUCT_BOUNDARY.md` now describes deferred native
  package/signing/update-channel behavior instead of the broader phrase
  "release installers", which could be misread now that archive install helpers
  exist.
- `docs/ROADMAP.md` now distinguishes current archive install helpers from
  unimplemented native package, signing, publishing, and update-channel
  behavior.
- `docs/README.md` now links this audit as a roadmap/handoff document.
- `docs/PLAN_CAVEATS_AND_UNIMPLEMENTED_SUMMARY.md` now records Plan 50 as the
  repo-wide legacy cleanup audit and, after Plan 51, points to a real-repo test
  readiness audit as the next bounded action.

## Legacy Surfaces Still Present

These are intentionally present and should not be treated as stale product
claims:

- `WI.md` appears in tests and plan/docs guardrails that verify it is not
  generated or reintroduced.
- JSONL appears for disposable pre-alpha cache migration notes and quality
  report detail exports, not canonical index storage.
- Universal Ctags is optional, external, not bundled, not required, and not used by production indexing.
- Ctags-related code remains isolated under quality comparator/gate modules and
  quality tests. The repository allowlist gate enforces that boundary.
- Superseded native-parser plans remain only under `prompts/superseded/` as
  historical context and are not active plans.
- `src/semantic.rs` and semantic-fact docs remain an optional adapter boundary.
  Normal `build_index`, `wi`, `wi pack`, and `wi impact` do not require or run
  compiler/LSP tooling.
- `test_repos/` exists locally in this checkout, but `git status --ignored`
  reports it as ignored and `git ls-files test_repos` returns no tracked files.
- `dist/` and `.dev_index/` exist locally from release and index validation, but
  they are ignored and untracked.

## Deferred Or Unimplemented Features

These remain real deferred items and should not be implemented as cleanup:

- Native package formats: MSI/MSIX/WiX/Inno Setup, macOS `.pkg`/`.dmg`, Linux
  `.deb`/`.rpm`/AppImage, package repositories, and package-manager publishing.
- Real signing and notarization: Authenticode, Developer ID, notarization,
  stapling, GPG/package signing, and release-token-backed publishing.
- Managed update channels and GitHub Release publishing.
- Payment handling, account behavior, hosted services, telemetry, network
  activation, source upload, and license enforcement.
- Compiler/LSP-level semantics, type checking, macro expansion, package-manager
  resolution, runtime binding, inheritance completeness, and exhaustive impact
  analysis.
- Experimental language promotion or blocked language support without the
  required grammar/extras policy, query specs, fixtures, notices, docs, and
  gates.
- Additional Tree-sitter convergence cycles unless a future prompt explicitly
  scopes one bounded cycle.

## Audit Findings By Area

| Area | Finding | Action |
| --- | --- | --- |
| Parser architecture | Production code-symbol extraction still routes through Tree-sitter framework and project-owned extras. No active native line-scanner parser was found. | No code change. |
| External comparator boundary | External comparator references are quality/report/test boundaries. Production parser/index/search/release surfaces remain guarded. | No code change. |
| Storage | `.dev_index/index.sqlite` remains canonical. JSONL references are migration or quality export notes. | No code change. |
| Release/distribution | Archive packaging and smoke checks exist. Native packages, real signing/notarization, publishing, and update channels remain future/scaffolded. | Tightened docs wording. |
| Local/generated artifacts | `.dev_index/`, `dist/`, and `test_repos/` are present locally but ignored and untracked. | No commit of local generated content. |
| Tests/quality gates | Normal tests remain independent of local corpora. Ignored real-repo tests remain explicit and local-only. | No code change. |
| Documentation | Plan 49 indexes are current. The main ambiguity found was "release installers" wording around product boundaries. | Rewrote wording and added this audit. |

## Future-plan Risks

- The language support truth audit was completed by Plan 51 in
  `docs/LANGUAGE_SUPPORT_AUDIT.md`; future language work should start from that
  evidence and avoid opportunistic parser expansion.
- Release work should stay archive-focused until a dedicated native
  package/signing/publishing plan has platform tools, credentials policy, and CI
  acceptance criteria.
- Product work should keep free/local commands independent of payment,
  activation, account, hosted, telemetry, and license-enforcement behavior.
- Semantic adapter work should stay isolated unless a future plan adds a real
  adapter with privacy, runtime, and validation rules.

## Recommended Next Action

Run a real-repo test readiness audit. It should verify ignored-test skip
behavior, manifest ergonomics, local corpus assumptions, expected-symbol
coverage gaps, and docs without committing `test_repos/` contents or adding
parser support opportunistically.
