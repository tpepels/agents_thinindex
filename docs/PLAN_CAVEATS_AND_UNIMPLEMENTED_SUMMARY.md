# Plan Caveats And Unimplemented Summary

This document summarizes caveats, deferred work, and explicitly unimplemented
parts across the active plan series after PLAN_46. It is a planning summary, not
a new implementation plan.

Sources reviewed:

- `prompts/PLAN_46_FULL_PLAN_COMPLETION_AUDIT.md`
- active `prompts/PLAN_*.md` files
- `README.md`
- parser, quality, licensing, release, dependency, refs, pack, impact, and
  team/CI documentation under `docs/`

## Overall Status

- Active plan files are marked complete through PLAN_46.
- There are now 57 active plan files after updating PLAN_47 as the scoped
  release-distribution plan.
- PLAN_47 is intentionally not started and contains unchecked execution phases.
- No unchecked `- [ ]` boxes were found in active `prompts/PLAN_*.md` files at
  the PLAN_46 cleanup point before PLAN_47 was created.
- The duplicate PLAN_45 sequence was resolved by renaming the team/CI roadmap to
  `PLAN_45A_TEAM_CI_AND_HOSTED_VALUE_ROADMAP.md`.
- No required active plan file is missing for the observed sequence from
  PLAN_00 through PLAN_47, including lettered PLAN_11A through PLAN_11C and
  PLAN_12A through PLAN_12G.
- There is no active PLAN_11D, PLAN_11E, or monolithic
  PLAN_12_EXTENDED_LANGUAGE_PACK.
- Exact historical commit messages for PLAN_00 through PLAN_10 are not visible
  in current `git log --all`, but PLAN_46 found implementation evidence in
  code, tests, and docs and later plans verified those prerequisites.

## Relevance Review

Review date: 2026-05-01.

Implementation decision:

- No obsolete caveat was found.
- No bounded product-code fix was identified that could safely remove a caveat
  without changing the product boundary.
- The still-relevant unimplemented items are intentionally deferred, blocked by
  external credentials/local corpora, or require a dedicated product/security
  plan before implementation.
- This review updates the caveat file with explicit decisions instead of hiding
  unresolved work behind completed plan checkboxes.

Validation evidence from this review:

- `build_index` rebuilt `.dev_index/index.sqlite` successfully.
- `wi --help` lists current commands and does not list `wi ci`.
- `wi doctor` reports `overall: ok`.
- `cargo deny check licenses` reports `licenses ok`.
- `test_repos/` is not present in this checkout, so real-repo promotion claims
  cannot be strengthened from local evidence.

## PLAN_46 Re-execution Findings

Audit date: 2026-05-01.

Disk-state findings:

- `git status --short` showed this audit summary as the only untracked file
  before this pass.
- `git log --oneline -20` shows PLAN_45A, PLAN_45, PLAN_46 creation, and the
  prior PLAN_46 cleanup commits at the top of history.
- `ls prompts/PLAN_*.md | sort` showed 57 active plan files after PLAN_47 was updated.
- `ls prompts/superseded` showed the old native-parser 11-series files only
  under `prompts/superseded/`.
- `grep -n "Do not implement this until" prompts/PLAN_*.md` showed the active
  prerequisite chain remains present; PLAN_46 intentionally also contains the
  validation command text.

Active plan inventory:

- PLAN_00 through PLAN_10: complete; older exact expected commit messages are
  not visible in current `git log --all`, but implementation evidence remains
  in code, tests, and docs.
- PLAN_11A through PLAN_11C: complete; active parser backbone is Tree-sitter.
- PLAN_12A through PLAN_12G: complete; extended language work is documented by
  support levels.
- PLAN_13 through PLAN_16: complete; release/archive/signing scaffolding exists,
  but native packages and real signing remain future work.
- PLAN_17 through PLAN_30: complete; quality plugin and external-comparator
  boundary remain
  isolated to quality/reporting surfaces.
- PLAN_31 through PLAN_40: complete; dependency, refs, pack, impact, semantic
  adapter boundary, agent integration, and technical audit work are present.
- PLAN_41 through PLAN_44: complete; security/privacy, distribution hardening,
  inert licensing foundation, onboarding doctor, and product polish are present.
- PLAN_45A: complete; team/CI/hosted value remains roadmap-only.
- PLAN_45: complete; Tree-sitter real-repo convergence loop is not restarted by
  this audit.
- PLAN_46: complete; this pass is audit/cleanup/alignment only.
- PLAN_47: updated as the next scoped release-distribution implementation plan;
  not started.

Stale reference findings:

- Active prompt references to `WI.md`, JSONL, and the external-comparator
  boundary are intentional
  guardrails or historical cleanup requirements, not current product claims.
- Superseded old native-parser plan files remain under `prompts/superseded/`
  only; they are not active plans.
- The roadmap phrase "cross-platform archives and installers after the parser
  blocker is removed" was stale because Tree-sitter is already the production
  parser. It has been updated to current release/signing/licensing blockers.
- PLAN_46's old next-action sentence still described re-running Phase 1 and
  resolving the duplicate PLAN_45 fork. It has been updated to point at one
  future implementation plan.

Forbidden-surface findings:

- No current audit evidence indicates `WI.md` was reintroduced.
- No current audit evidence indicates JSONL was made canonical storage.
- No current audit evidence indicates the external tagger comparator is required
  for production indexing. Remaining external-comparator references are
  boundary, quality-comparator, or forbidden-source checks.
- No current audit evidence indicates active docs claim blocked languages are
  fully supported.
- No `test_repos/` directory exists in this checkout, so no third-party repo
  contents are staged or committed by this pass.

Recommended next implementation plan:

- `prompts/PLAN_47_RELEASE_DISTRIBUTION_COMPLETION.md`: execute one bounded
  archive-focused release hardening pass. The plan covers archive assembly,
  manifest/SBOM, checksum verification, unpack smoke checks, and honest release
  docs, while keeping native packages, real signing/notarization, GitHub Release
  publishing, package-manager distribution, update channels, source upload,
  telemetry, payments, license enforcement, hosted behavior, parser work, and
  the later documentation cleanup/indexing pass out of scope.

## Per-caveat Implementation Decisions

| Area | Still relevant? | Implemented in this pass? | Decision |
| --- | --- | --- | --- |
| PLAN_00 through PLAN_07 residual storage/test caveats | Yes | No product change needed | SQLite is already canonical. Old JSONL storage remains disposable by design, and normal tests must stay independent of local `.dev_index/` and `test_repos/`. |
| PLAN_08 through PLAN_10 packaging caveats | Yes | No | Packaging is still gated by license audit, release hardening, signing/notarization, installer maturity, and permissive dependency review. Implementing native installers or commercial packaging needs a dedicated release plan and platform-specific verification. |
| Tree-sitter syntactic extraction limit | Yes | No | This is an intentional architecture boundary. Full semantic binding, compiler/LSP analysis, macro expansion, and type resolution are not safe to add as incidental cleanup. |
| Experimental languages | Yes | No | Scala, Kotlin, Swift, Dart, and Nix stay experimental because real-repo coverage and syntax-specific evidence remain incomplete. `test_repos/` is absent, so this pass cannot honestly promote them. |
| Blocked or unsupported languages | Yes | No | Blocked entries must not be faked. Support requires a permissive grammar or approved extras path, extension mapping, query/extraction policy, fixture, notices, docs, and gates. |
| Known extraction gaps | Yes | No | The listed gaps are mostly semantic, runtime, package-manager, or language-specific resolver work. Each needs targeted tests and architecture decisions; none should be patched opportunistically. |
| Refs, pack, impact, and dependency caveats | Yes | No product change needed | Existing behavior is intentionally conservative, evidence-backed, and bounded. Removing these caveats would require semantic adapters or broader resolver guarantees. |
| Quality and real-repo caveats | Yes | No product change needed | Quality/comparator output must remain isolated. Real-repo checks remain ignored/manual because they depend on local `test_repos/`. |
| Release archive and signing gaps | Yes | No | Archive packaging and signing scaffolds exist. Real signing, notarization, native package formats, release publishing, and update channels require external tools, credentials, target platforms, and CI/release policy. |
| Licensing, Pro, hosted, and payment gaps | Yes | No | These are explicitly out of scope for the current free/local product. Implementing enforcement, payments, accounts, hosted APIs, telemetry, or source upload requires a dedicated product/security/privacy plan. |
| Security, privacy, and agent caveats | Yes | No product change needed | Local-only paths, redaction boundaries, and advisory agent behavior remain accurate. Agents cannot be forced to comply from inside this tool. |
| Plan hygiene guardrails | Yes | No product change needed | Mentions of `WI.md`, JSONL, and the external-comparator boundary remain intentional guardrails. They should not be removed unless the related architecture changes. |

Bottom line: every caveat remains relevant, but none is appropriate to implement
as unplanned cleanup. The next implementation work should start from a scoped
plan that chooses one area, defines acceptance criteria, and updates tests/docs
alongside code.

## Plan Range Summary

| Plans | Status | Caveats and unimplemented parts |
| --- | --- | --- |
| PLAN_00 through PLAN_07 | Complete | SQLite storage, refs, deterministic extraction, context commands, impact, benchmarks, agent workflow, and real-repo benchmark support are implemented. Old JSONL `.dev_index` storage is not canonical and remains disposable. Normal tests must stay independent of local `.dev_index/` and `test_repos/`. |
| PLAN_08 through PLAN_10 | Complete | Install/docs/product-boundary work is complete, but polished proprietary packaging is not claimed. The external tagger comparator is optional quality-comparator tooling only. Packaging remains gated by license audit, release hardening, signing/notarization, installer maturity, and permissive dependency review. |
| PLAN_11A through PLAN_12G | Complete | Tree-sitter is the code-symbol parser backbone, but extraction remains syntactic. Several languages are experimental or blocked. LSP/compiler/type-checker semantics are not implemented. |
| PLAN_13 through PLAN_16 | Complete | License notices, release archives, installer helpers, signing scaffolds, and CI/release checks exist. Native package formats, real signing/notarization, GitHub Release publishing, update channels, and package-manager distribution are not implemented. |
| PLAN_17 through PLAN_30 | Complete | Quality plugin, external-comparator boundary, support levels, triage, single-cycle quality loop, exports, support dashboard, manifest curation, maintenance docs, CI readiness, and quality audit are implemented. Comparator data is optional local QA data, not production ground truth. Real-repo quality checks remain ignored/manual and local. |
| PLAN_31 through PLAN_37 | Complete | Dependency graph, resolver packs, reference graph v2, dependency-aware pack/impact, test/build/config mapping, and monorepo safeguards are implemented. Dependency resolution is local and deterministic, not compiler/package-manager complete. |
| PLAN_38 | Complete | Optional semantic adapter boundary exists. Real compiler/LSP adapters are not bundled, required, or enabled by default. Semantic facts stay isolated. |
| PLAN_39 through PLAN_44 | Complete | Agent integration packs, technical audit, security/privacy docs, installer distribution hardening, inert licensing foundation, onboarding doctor, and product polish are implemented. Agent compliance remains advisory; no license enforcement, payments, telemetry, account behavior, hosted APIs, or cloud sync exists. |
| PLAN_45A | Complete | Team/CI and hosted value is a roadmap only. No `wi ci`, hosted backend, source upload, account system, payment integration, telemetry, or paid gate exists. |
| PLAN_45 | Complete | One bounded Tree-sitter real-repo convergence cycle was completed. The process intentionally stops after one cycle and at most 10 selected gaps. Future cycles require explicit human request. |
| PLAN_46 | Complete | Audit cleanup is complete. It did not implement product features. It records residual caveats and keeps stale-reference guardrails visible. |
| PLAN_47 | Not started | Archive release hardening is now the next scoped active implementation plan. It must not add native packages, real signing/notarization, GitHub Release publishing, managed update channels, parser, hosted, telemetry, payment, license-enforcement behavior, or the later documentation cleanup/indexing pass. |

## Parser And Tree-sitter Caveats

- Production code-symbol extraction uses the Tree-sitter registry, grammar
  adapters, query specs, normalized captures, and shared conformance tests.
- Tree-sitter extraction is syntactic. It does not claim semantic binding,
  compiler analysis, LSP behavior, type checking, macro expansion, runtime
  dispatch, inheritance resolution, overload resolution, or package-manager
  completeness.
- CSS, HTML, Markdown, JSON, TOML, and YAML are extras-backed deterministic
  landmarks, not Tree-sitter-backed code-symbol support.
- The external tagger comparator is optional quality-comparator tooling only. It must
  not become a production parser, fallback parser, release dependency, or source
  of production `records`/`refs`.
- Unsupported languages and formats must not be silently parsed through line
  scanning, regex extraction, external-tagger fallback, or undocumented parser paths.

### Experimental Language Support

These languages have grammar/query support but remain experimental because
real-repo coverage or documented syntax handling is incomplete:

- Scala: givens, implicits, extension handling, and real-repo coverage remain
  incomplete.
- Kotlin: interface, enum-class, extension distinctions, and real-repo coverage
  remain incomplete.
- Swift: extensions, overloads, module handling, and real-repo coverage remain
  incomplete.
- Dart: package, extension, type-alias handling, and real-repo coverage remain
  incomplete.
- Nix: exhaustive attribute/scalar extraction and real-repo coverage remain
  incomplete by design.

### Blocked Or Unsupported Language Support

Blocked entries with no support claim:

- Vue/Svelte single-file components: no approved permissive grammar/query,
  fixture, notice, or component-section adapter path.
- Objective-C/Objective-C++: no approved permissive grammar/query/fixture/notice
  path.
- SQL: no product-approved grammar/query policy for dialect differences.
- XML: no product-approved extras policy for non-noisy XML landmarks.
- Lua: no selected permissive grammar/query/fixture/notice path.
- Haskell: no selected permissive grammar/query/fixture/notice path.
- Elixir: no selected permissive grammar/query/fixture/notice path.

Languages and formats not listed in the support matrix are unsupported.

### Known Extraction Gaps

- Rust `use` records are handled through deterministic refs rather than claimed
  as full symbol extraction.
- Ruby `require` targets and shell sourced files are not runtime-resolved.
- Dynamic PHP includes, autoload behavior, and runtime namespace behavior are
  not resolved.
- C/C++ macro expansion, template instantiation, compile database semantics, and
  configured include paths are not implemented.
- C# partial-type, assembly, project-reference, and Roslyn-level resolution are
  not implemented.
- Java package visibility, inherited members, and build-system classpath
  resolution are not implemented.
- JavaScript/TypeScript runtime module, prototype, bundler, `paths`, and Babel
  alias resolution are not compiler-complete.
- Python virtual environments and package metadata are not resolved.
- JVM build metadata, Swift package/module metadata, Ruby `$LOAD_PATH`, and PHP
  autoloaders are not modeled.
- JSON/TOML/YAML scalar values are intentionally skipped by extras-backed
  extraction.

## Refs, Pack, Impact, And Dependency Caveats

- `refs`, `pack`, and `impact` use SQLite `records`, `refs`, and
  `dependencies`, but they are conservative navigation aids rather than
  exhaustive semantic engines.
- Syntax references are AST-backed observations, not proof of compiler binding.
- Dependency edges are local and deterministic. They do not invoke package
  managers, build tools, compilers, LSP servers, or network metadata.
- Resolved, ambiguous, and unresolved dependency edges are preserved with
  confidence labels instead of guessing.
- Broad text fallback remains capped and labelled heuristic.
- `wi impact <term>` includes only rows with concrete indexed file:line
  evidence, but that evidence is not a guarantee that editing the term will
  affect the file.
- `wi pack <term>` intentionally returns a bounded read set and does not dump
  full file contents.
- Test mapping is convention-based and does not run test frameworks.

## Quality And Real-repo Caveats

- Quality reports and comparator output are local QA artifacts under
  `.dev_index/quality/`; they must not be imported into production SQLite
  `records` or `refs`.
- Optional comparator output is triage evidence, not ground truth.
- Comparator-only findings must be classified as expected-symbol additions,
  fixture needs, accepted false positives, unsupported syntax, low-value noise,
  or fixed work before becoming parser requirements.
- The single-cycle quality runner is bounded. It selects at most 10 actionable
  non-comparator gaps and does not automatically start another cycle.
- Real-repo tests depend on local `test_repos/` content and remain
  ignored/manual. If `test_repos/` is missing or empty, those checks skip.
- Third-party repository contents under `test_repos/` must not be committed.
- Manifest entries should prefer expected symbols, expected patterns, absent
  symbols, thresholds, skip reasons, and unsupported syntax notes over brittle
  total record counts.

## Packaging, Release, And Licensing Caveats

- Release archives and archive install helpers exist; polished native packaging
  does not.
- Not implemented:
  - Windows MSI, MSIX, WiX, Inno Setup, Store packages, or completed
    Authenticode signing.
  - macOS `.pkg`, `.dmg`, Developer ID signing, notarization, or stapling.
  - Linux `.deb`, `.rpm`, AppImage, repository metadata, package-manager
    publishing, or completed package signing.
  - GitHub Release publishing and managed update channels.
- Signing scripts are scaffolds only. No signing certificates, private keys,
  notarization credentials, package signing keys, or release secrets are
  committed.
- Proprietary packaging remains blocked if license audit finds GPL, AGPL,
  LGPL-only, MPL-only, EPL, CDDL, unknown, no-license, custom, or
  non-commercial dependency terms without a documented future exception.
- `THIRD_PARTY_NOTICES` and `cargo deny check licenses` remain required release
  inputs.
- The external tagger comparator must not be bundled or required by release artifacts.

## Product, Pro, Hosted, And Licensing Caveats

- The licensing foundation is inert. It supports local status modeling and
  fixture-only Pro validation, but it is not cryptographic license enforcement.
- No current command is blocked by license status.
- No payment integration, account login, license server, network activation,
  telemetry, remote indexing, cloud sync, feature lockout, or hosted API exists.
- Future Pro features are candidates only. Pricing, provider, activation flow,
  local license cache shape, paid update policy, and team licensing are
  deferred decisions.
- Team/CI and hosted reports are roadmap-only. No source upload, hosted backend,
  `wi ci`, account system, or artifact upload workflow is implemented.
- Future upload-capable workflows must be opt-in, documented, redacted, and
  separable from free local command behavior.

## Security, Privacy, And Agent Caveats

- `.dev_index/`, `.dev_index/quality/`, and `test_repos/` are local-only and
  must not be committed.
- Report-like output must remain redacted when it includes potentially sensitive
  metadata.
- `wi-init` writes `AGENTS.md` and normalizes existing `CLAUDE.md`, but it does
  not create `WI.md`.
- Agent integration packs are advisory. Agents can still ignore instructions.
- `wi-stats` reports local usage and command-category audit counts from
  thinindex activity, but it cannot observe external grep, find, ls, editor
  reads, or non-thinindex file access.

## Plan Hygiene Caveats

- Active guardrails intentionally mention `WI.md`, JSONL, and the
  external-comparator boundary in "do not reintroduce" or boundary contexts.
  These mentions are not current product claims.
- JSONL quality exports are intentional report artifacts. They are not canonical
  index storage.
- Old JSONL `.dev_index` storage remains disposable and should be rebuilt by
  `build_index`.
- Superseded plans should not be revived.
- New parser support must use the existing Tree-sitter framework or documented
  extras-backed policy; do not add a second parser architecture.

## Practical Next Actions If Work Continues

1. For parser work, improve experimental or blocked language status only through
   grammar registration, extension mapping, query specs, conformance fixtures,
   license entries, docs entries, and support-matrix updates.
2. For quality work, run a new bounded quality cycle only on explicit request,
   and classify remaining gaps rather than treating raw comparator output as
   truth.
3. For release work, finish native package formats, real signing/notarization,
   release publishing, and update-channel design before claiming polished public
   distribution.
4. For product work, keep Pro/team/hosted features as candidates until there is
   evidence, privacy design, licensing design, and implementation scope.
5. For audit work, re-run the PLAN_46 inventory checks before changing plan
   status or adding new active plan files.
