# PLAN_47_RELEASE_DISTRIBUTION_COMPLETION.md

Use superpowers:subagent-driven-development.

Do not implement this until PLAN_46_FULL_PLAN_COMPLETION_AUDIT.md is complete and green.

Goal:
Complete the next bounded release-distribution layer without changing parser behavior, search semantics, licensing enforcement, payment behavior, hosted behavior, telemetry, or source-upload boundaries.

This plan exists because PLAN_46 identified release distribution as the next coherent implementation area after the full plan audit. It must stay focused on distribution readiness and must not turn into product, parser, or hosted-service work.

Product rule:
Distribution claims must match artifacts that are actually generated, checked, and documented. Do not claim native packages, signing, notarization, publishing, or update channels are complete until they are implemented and verified on the relevant platform.

Progress:
- [ ] Phase 1: audit current release archive, installer helper, signing scaffold, CI, license audit, and package-content state.
- [ ] Phase 2: choose one bounded distribution slice that can be verified in the current environment.
- [ ] Phase 3: implement only that selected slice with explicit platform/status documentation.
- [ ] Phase 4: add or update package, signing, publishing, or update-channel tests/checks for that slice.
- [ ] Phase 5: update docs, roadmap, and handoff notes with truthful distribution status and next action.
- [ ] Phase 6: run required verification, commit, and stop.

Hard constraints:
- Do not reintroduce `WI.md`.
- Do not make JSONL canonical.
- Do not use any external tagger as a production parser.
- Do not bundle optional external comparator binaries.
- Do not make optional comparator tooling required for install, build, runtime, release, or tests.
- Do not weaken parser/index quality gates.
- Do not add parser support or parser extraction rules.
- Do not claim semantic/compiler/LSP-level analysis.
- Do not claim unsupported or experimental languages as fully supported.
- Do not add payment handling, hosted behavior, network activation, telemetry, source upload, or license enforcement.
- Do not commit signing keys, certificates, private keys, app-specific passwords, notarization profiles, package signing keys, release tokens, or other secrets.
- Do not commit `test_repos/` contents.
- Do not implement deferred caveats outside this release-distribution scope.

Prerequisite audit:
Before implementation, inspect:

- `docs/ROADMAP.md`
- `docs/PLAN_CAVEATS_AND_UNIMPLEMENTED_SUMMARY.md`
- `docs/RELEASING.md`
- `docs/INSTALLERS.md`
- `docs/RELEASE_CHECKLIST.md`
- `docs/SECURITY_PRIVACY.md`
- `docs/LICENSE_AUDIT.md`
- `docs/LICENSING.md`
- `scripts/package-release`
- `scripts/check-package-contents`
- `scripts/check-release`
- `scripts/sign-release-artifact`
- `.github/workflows/`
- `THIRD_PARTY_NOTICES`
- `deny.toml`

Current known state:
- Release archives are implemented through `scripts/package-release`.
- Archive content checks are implemented through `scripts/check-package-contents`.
- Local release checks are implemented through `scripts/check-release`.
- Unix archive install/uninstall helpers exist.
- Windows archive install/uninstall helpers exist.
- Signing/notarization support is scaffolded through `scripts/sign-release-artifact`.
- Native package formats are not implemented.
- Real signing/notarization is not complete.
- Release publishing and update channels are not implemented.
- The current tool remains local/free with no license enforcement, payments, accounts, telemetry, source upload, or hosted backend.

Allowed bounded slices:
Choose exactly one slice for this plan execution. Do not implement multiple slices unless they are inseparable and still small.

1. Windows native package slice:
   - add a real, reproducible Windows native package path only if it can be generated and checked in the current environment;
   - otherwise add tested validation/docs that keep Windows native packaging honestly scaffolded.

2. macOS native package slice:
   - add a real, reproducible macOS package or disk-image path only if it can be generated and checked in the current environment;
   - otherwise add tested validation/docs that keep macOS native packaging honestly scaffolded.

3. Linux native package slice:
   - add one real, reproducible Linux package format such as `.deb`, `.rpm`, or AppImage only if it can be generated and checked in the current environment;
   - otherwise add tested validation/docs that keep Linux native packaging honestly scaffolded.

4. Real signing/notarization slice:
   - wire a real signing or notarization path only when required tools and secrets are supplied by environment or secure CI;
   - dry-run and missing-secret behavior must remain explicit and testable without secrets.

5. Release publishing slice:
   - add a manual release-publishing workflow only if artifact generation, artifact checks, notices, checksums, and credential boundaries are clear;
   - do not publish automatically from normal CI.

6. Update-channel boundary slice:
   - document and test update-channel metadata boundaries without adding auto-update behavior unless a later plan explicitly scopes it.

If none of these slices can be implemented honestly in the current environment, do not fake support. Update docs/tests to make the blocker explicit, commit the audit alignment, and stop.

Implementation requirements:
- Preserve existing release archive behavior unless the selected slice explicitly changes it.
- Keep release artifacts assembled from explicit files only.
- Ensure release artifacts exclude:
  - `.dev_index/`
  - `.dev_index/quality/`
  - `test_repos/`
  - build output junk
  - local reports
  - local benchmark output
  - signing secrets or secret-like files
  - optional external comparator binaries
- Keep `THIRD_PARTY_NOTICES` in all release artifacts.
- Keep `SBOM.md` or equivalent artifact inventory accurate.
- Keep checksum behavior accurate for generated artifacts.
- Keep docs honest about unsigned/scaffolded platforms.
- Keep normal `cargo test` independent of local `.dev_index/`, `test_repos/`, optional external tools, credentials, network access, and platform-specific signing tools.

Documentation updates:
Update as needed:

- `README.md`
- `docs/ROADMAP.md`
- `docs/RELEASING.md`
- `docs/INSTALLERS.md`
- `docs/RELEASE_CHECKLIST.md`
- `docs/SECURITY_PRIVACY.md`
- `docs/PLAN_CAVEATS_AND_UNIMPLEMENTED_SUMMARY.md`

Do not claim a package/signing/publishing/update-channel capability unless the corresponding artifact or workflow is generated, checked, and documented.

Tests/checks:
Add or update focused tests or script checks for the selected slice:

- generated artifact exists when the slice claims generation;
- generated artifact has expected name, version, target/platform, and extension;
- generated artifact includes all expected binaries;
- generated artifact includes notices, install notes, and inventory/SBOM data;
- generated artifact excludes local caches, quality reports, test repos, source checkout bulk, build output, comparator binaries, and secret-like material;
- signing dry-run/missing-secret behavior is deterministic if signing is in scope;
- release-publishing workflow is manual or tag-scoped and does not publish from normal CI if publishing is in scope;
- docs do not overclaim unimplemented platforms.

Verification:
Run at minimum:

- `cargo fmt --check`
- `cargo test`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo deny check licenses`
- `scripts/check-ci`
- `scripts/check-release`
- `cargo run --bin build_index`
- `cargo run --bin wi -- --help`
- `cargo run --bin wi -- doctor`
- `git diff --check`

Also run any selected-slice commands, for example:

- package generation command for the selected native package format;
- package content inspection for the generated artifact;
- signing dry-run for the selected platform;
- release-publishing workflow lint/dry-run if available.

Run ignored local/real-repo tests only if this plan changes real-repo, parser, quality, refs, pack, impact, or dependency assumptions:

- `cargo test --test local_index -- --ignored`
- `cargo test --test real_repos -- --ignored` if `test_repos/` exists

Acceptance:
- Exactly one bounded distribution slice is selected and reported.
- Implemented artifacts or workflows are generated, checked, and documented.
- Unsupported package/signing/publishing/update-channel paths remain clearly scaffolded or blocked.
- Existing local/free CLI behavior remains stable.
- No parser, search, hosted, telemetry, payment, or license-enforcement behavior is added.
- No secrets or `test_repos/` contents are committed.
- Verification passes.
- The next recommended action is clear.

Commit instructions:
After verification passes, commit with:

`Advance release distribution completion`

Final response:
- selected distribution slice
- why that slice was selected
- files changed
- artifacts/workflows added or updated
- package/signing/publishing/update-channel status by platform
- verification commands and results
- ignored local/real-repo test status, if applicable
- commit hash
- next recommended prompt/action
