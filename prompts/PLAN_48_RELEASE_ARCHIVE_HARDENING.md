# PLAN_48_RELEASE_ARCHIVE_HARDENING.md

Use superpowers:subagent-driven-development.

Do not implement this until `PLAN_46_FULL_PLAN_COMPLETION_AUDIT.md` is complete and green.

## Prerequisite

Plan 46 must be complete and committed. Treat the current release/distribution
state as the source of truth, including the existing Plan 47 handoff. Do not
modify `PLAN_47_RELEASE_DISTRIBUTION_COMPLETION.md` while executing this plan
unless a later human request explicitly asks for that file to change.

## Purpose

Turn the Plan 46 release-distribution recommendation into one executable,
archive-focused implementation plan.

This plan is intentionally limited to release/distribution completion that can
be implemented and verified without credentials, payment systems, hosted
behavior, network activation, external publishing, or secret-backed signing. It
does not implement Plan 47 yet; it narrows the next actionable release slice to
the existing local archive workflow.

Distribution claims must match artifacts that are actually generated, checked,
documented, and smoke-tested.

## Scope

This plan focuses on hardening the existing archive distribution path:

- current-platform release archives produced by `scripts/package-release`;
- archive install/uninstall helpers already present under `scripts/`;
- notices, compact SBOM, install notes, and release documentation bundled in the archive;
- SHA256 checksum sidecar generation and verification;
- package-content checks through `scripts/check-package-contents`;
- CI/local smoke checks that can unpack archives and run packaged binaries without secrets;
- documentation that clearly separates completed local release archives from future native packaging, signing, notarization, publishing, and update-channel work.

## Current Release Surface Inventory

Before implementation, inspect and record the current state of:

- `scripts/package-release`
- `scripts/check-package-contents`
- `scripts/check-release`
- `scripts/install-archive-unix`
- `scripts/uninstall-archive-unix`
- `scripts/windows/install.ps1`
- `scripts/windows/uninstall.ps1`
- `scripts/sign-release-artifact`
- `.github/workflows/ci.yml`
- `.github/workflows/release.yml`
- `docs/RELEASING.md`
- `docs/INSTALLERS.md`
- `docs/RELEASE_CHECKLIST.md`
- `docs/CI_INTEGRATION.md`
- `docs/LICENSE_AUDIT.md`
- `docs/LICENSING.md`
- `docs/SECURITY_PRIVACY.md`
- `THIRD_PARTY_NOTICES`
- `deny.toml`

The inventory must cover:

- existing `scripts/package-release` behavior;
- existing archive install/uninstall helper behavior;
- existing notice, SBOM, and checksum behavior;
- existing CI package smoke coverage;
- existing signing/notarization scaffolds and their dry-run or missing-tool behavior;
- existing release docs and any stale claims.

## Concrete Deliverables

Implement the smallest archive-focused release hardening slice that satisfies
the acceptance criteria. Expected deliverables include:

- archive packaging hardening for the existing `scripts/package-release` flow;
- cross-platform archive smoke coverage where feasible from the current checkout and CI environment;
- release artifact manifest/SBOM and checksum verification;
- local scripts or tests that validate release archives without secrets;
- docs that accurately describe what is shipped and what remains scaffolded;
- a clear distinction between completed local release archives and future native packaging, signing, notarization, GitHub Release publishing, package-manager distribution, or managed update channels.

If a deliverable cannot be implemented honestly in the current environment, do
not fake support. Add or update a deterministic check or documentation note that
keeps the blocker explicit, then stop within this plan.

## Explicit Non-goals

- Do not create MSI, MSIX, WiX, Inno Setup, Store, or other Windows native packages unless an existing scaffold is already present and can be generated and checked without credentials.
- Do not claim Windows Authenticode signing as completed behavior.
- Do not create macOS `.pkg` or `.dmg` distribution, Developer ID signing, notarization, or stapling as completed behavior.
- Do not create Linux `.deb`, `.rpm`, AppImage, package repository metadata, or package-manager publishing unless an existing scaffold is already present and this plan explicitly narrows to a safe, secret-free validation of that scaffold.
- Do not publish GitHub Releases.
- Do not add managed update channels or auto-update behavior.
- Do not add payment handling, hosted behavior, network activation, telemetry, account behavior, source upload, or license enforcement.
- Do not require secrets, certificates, private keys, notarization credentials, package signing keys, release tokens, or external services for normal validation.
- Do not bundle or require Universal Ctags.
- Do not use Universal Ctags as a production parser.
- Do not call external tagger tooling from `build_index`.
- Do not change parser support claims except where release docs are stale.
- Do not commit `test_repos/` contents.
- Do not implement deferred caveats opportunistically.

## Hard Constraints

- Do not reintroduce `WI.md`.
- Do not make JSONL canonical storage.
- Do not emit production records with `source = "ctags"`.
- Do not weaken parser/index quality gates.
- Do not claim semantic/compiler/LSP-level analysis unless actually implemented.
- Do not claim unsupported or experimental languages as fully supported.
- Do not make optional comparator tooling required for install, build, runtime, tests, or release artifacts.
- Do not commit signing keys, certificates, private keys, app-specific passwords, notarization profiles, package signing keys, release tokens, or other secrets.
- Keep normal `cargo test` independent of local `.dev_index/`, `test_repos/`, optional external tools, credentials, network access, and platform-specific signing tools.

## Implementation Steps

- [ ] Phase 1: run the current release surface inventory and document what already exists versus what is scaffolded.
- [ ] Phase 2: identify one archive-focused hardening slice that is possible without credentials or external publishing.
- [ ] Phase 3: harden archive assembly, manifest/SBOM content, checksum behavior, or artifact exclusion checks for that selected slice.
- [ ] Phase 4: add or update release archive smoke coverage that unpacks the artifact and runs packaged `wi --help`, `wi doctor`, and `build_index` where feasible.
- [ ] Phase 5: update release docs, installer docs, roadmap, and caveat/handoff docs so completed archive behavior and scaffolded native/signing/publishing work are clearly separated.
- [ ] Phase 6: run required validation, update this checklist, commit, and stop.

## Validation Steps

Baseline validation:

- [ ] `cargo fmt --check`
- [ ] `cargo test`
- [ ] `cargo clippy --all-targets --all-features -- -D warnings`
- [ ] `cargo run --bin build_index`
- [ ] `cargo run --bin wi -- --help`
- [ ] `cargo run --bin wi -- doctor`
- [ ] `cargo deny check licenses`
- [ ] `git diff --check`

Release validation:

- [ ] `scripts/package-release`
- [ ] `scripts/check-release`
- [ ] `scripts/check-package-contents <generated archive>`
- [ ] verify the generated `.sha256` sidecar against the generated archive using `sha256sum -c <generated archive>.sha256` or `shasum -a 256 -c <generated archive>.sha256`, depending on the host tools available.
- [ ] unpack the generated archive into a temporary directory and run packaged `wi --help`.
- [ ] unpack the generated archive into a temporary directory and run packaged `wi doctor`.
- [ ] run packaged `build_index` in a temporary repository or fixture when feasible.

If any release validation command is missing, the implementation pass must add
the command or document why that validation is not yet available.

Run ignored local/real-repo tests only if this plan changes real-repo, parser,
quality, refs, pack, impact, or dependency assumptions:

- [ ] `cargo test --test local_index -- --ignored`
- [ ] `cargo test --test real_repos -- --ignored` if `test_repos/` exists

## Acceptance Criteria

- Release archives include all intended thinindex binaries.
- Release archives include required notices, license, SBOM, and release documentation.
- Release archives include install/uninstall helper docs or scripts if currently intended.
- Checksums are produced and verifiable.
- CI or local smoke tests can unpack and run the packaged binaries.
- Docs clearly state what is complete versus scaffolded.
- Packaging remains local-first and source-upload-free.
- No Universal Ctags production dependency is introduced.
- No secret-dependent step is required for normal validation.
- Native package formats, completed signing, notarization, GitHub Release publishing, package-manager publishing, and managed update channels remain future work unless a later plan implements them.

## Completion And Update Instructions

After implementation and verification:

- update this plan's checkboxes honestly;
- update `docs/ROADMAP.md` only to reflect shipped release-archive behavior and the next truthful release-distribution action;
- update `docs/PLAN_CAVEATS_AND_UNIMPLEMENTED_SUMMARY.md` if any caveat is resolved or reclassified;
- preserve explicit docs that native packages, real signing/notarization, publishing, and update channels remain scaffolded or future work unless implemented;
- commit with:

`Advance release archive hardening`

Stop after this one scoped release-distribution pass. Do not start another
release, packaging, signing, publishing, or update-channel plan automatically.

## Final Report Requirements

- selected archive hardening slice
- why that slice was selected
- files changed
- release artifacts/checks added or updated
- package/signing/publishing/update-channel status by platform
- validation commands and results
- ignored local/real-repo test status, if applicable
- commit hash
- next recommended prompt/action
