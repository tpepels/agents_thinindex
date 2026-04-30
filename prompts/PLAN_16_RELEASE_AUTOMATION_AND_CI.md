# PLAN_16_RELEASE_AUTOMATION_AND_CI.md

Use superpowers:subagent-driven-development.

Do not implement this until PLAN_00 through PLAN_15 are complete and green.

Goal:
Add release automation and CI checks that make thinindex releases repeatable, verified, and hard to ship with broken packaging/license surfaces.

This pass automates verification and release artifact production. Do not add new product features, parser behavior, search semantics, license enforcement, payment behavior, telemetry, or cloud services.

Product rule:
A release must be reproducible from source, pass all gates, include notices, and not bundle forbidden dependencies.

Hard requirements:
- Do not bundle Universal Ctags.
- Do not reintroduce Universal Ctags.
- Do not add GPL or AGPL dependencies.
- Do not add license/payment/Pro gating behavior.
- Do not reintroduce JSONL storage.
- Do not reintroduce `WI.md`.
- CI/release artifacts must include `THIRD_PARTY_NOTICES`.
- CI must verify all expected binaries:
  - `wi`
  - `build_index`
  - `wi-init`
  - `wi-stats`

Scope:
Add CI/release automation for:

- formatting
- tests
- clippy
- license audit
- package archive creation
- artifact content checks
- command smoke tests
- optional ignored tests where practical/manual

Preferred CI:
Use GitHub Actions if the repo already uses GitHub or has no CI.

Suggested workflows:
- `.github/workflows/ci.yml`
- `.github/workflows/release.yml`

CI workflow:
Run on push and pull request.

Required jobs:
1. format
   - `cargo fmt --check`

2. test
   - `cargo test`

3. clippy
   - `cargo clippy --all-targets --all-features -- -D warnings`

4. license audit
   - run license audit command from PLAN_12 if configured

5. smoke
   - `cargo run --bin wi -- --help`
   - `cargo run --bin wi -- --version`
   - `cargo run --bin build_index -- --version`
   - `cargo run --bin wi-init -- --version`
   - `cargo run --bin wi-stats -- --version`

6. package smoke
   - run release packaging script from PLAN_13 for current platform
   - inspect/list artifact contents
   - verify all binaries are present
   - verify `THIRD_PARTY_NOTICES` is present
   - verify `.dev_index` and `test_repos` are absent from archive
   - verify no ctags binary is present

Release workflow:
Add a manual or tag-triggered workflow if practical.

Acceptable triggers:
- `workflow_dispatch`
- tags like `v*`

Required behavior:
- build release binaries
- run the same gates as CI or depend on CI
- create release archives
- create checksums if supported
- upload artifacts to workflow artifacts
- do not publish GitHub Releases unless explicitly safe and configured

Do not require signing secrets in this plan.
Do not commit secrets.
Do not require notarization/code signing yet unless PLAN_14 already implemented it.

Cross-platform:
If practical, add matrix jobs for:

- Linux x86_64
- macOS arm64/x86_64 where GitHub Actions supports it
- Windows x86_64

If full cross-platform packaging is too much, implement current-platform CI first and document remaining platform gaps. Do not fake success for unsupported targets.

Artifact checks:
Add or reuse script checks that verify release archive contents.

Required checks:
- includes all binaries
- includes `THIRD_PARTY_NOTICES`
- includes README/install notes
- excludes `.dev_index`
- excludes `test_repos`
- excludes Universal Ctags
- archive name contains version and target

Release docs:
Update `docs/RELEASING.md`.

Required docs:
- how to run CI gates locally
- how to create release archive locally
- how to trigger release workflow
- what artifacts are expected
- what checks are required before release
- signing/notarization status
- known remaining packaging caveats

Local release check:
Add a single local command if practical, for example:

- `scripts/check-release`
- or `make release-check`

It should run:
- fmt
- test
- clippy
- license audit if configured
- help/version smoke
- packaging script
- archive content checks

Do not make it require `test_repos/`.
Do not make it require network access.

Ignored/manual tests:
Do not add ignored local/real-repo tests to mandatory CI unless CI has deterministic repos available.

If adding optional CI for real repos:
- use a small fixture or checked-in tiny sample only
- do not clone arbitrary third-party repos during normal CI
- do not depend on network

Instruction surfaces:
- Do not reintroduce `WI.md`.
- Keep `wi --help` as the source of truth for command syntax, filters, examples, and subcommands.
- Keep AGENTS.md and existing CLAUDE.md generation in sync with the canonical `## Repository search` block.
- AGENTS.md should be created if absent.
- CLAUDE.md should be normalized only if present; do not create CLAUDE.md.
- Repeated `wi-init` runs must not duplicate `## Repository search`.
- Remove/normalize legacy markers: `@WI.md`, `See WI.md for repository search/index usage.`, `See `WI.md` for repository search/index usage.`, and old paragraph-style Repository search blocks.
- Update tests whenever help text or canonical Repository search text changes.

Tests/checks:
Add focused tests or script checks for:
- CI workflow file exists if GitHub Actions is used
- release workflow file exists if added
- release-check script exists if added
- package content check catches missing binaries
- package content check catches missing `THIRD_PARTY_NOTICES`
- package content check rejects `.dev_index`
- package content check rejects `test_repos`
- package content check rejects ctags artifacts

Acceptance:
- CI workflow exists and runs core gates.
- Release automation or documented manual release flow exists.
- Package artifacts are content-checked.
- License audit is part of CI/local release checks if configured.
- Release docs are accurate.
- No ctags bundling or GPL/AGPL dependency is introduced.
- Existing CLI behavior remains stable.
- No license/payment/network behavior is added.
- Normal CI does not depend on local `.dev_index`, `test_repos/`, or network-only third-party repo clones.

Verification:
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --all-targets --all-features -- -D warnings`
- license audit command from PLAN_12, if added
- run local release-check script if added
- run release packaging script from PLAN_13 for current platform
- inspect/list archive contents
- `cargo run --bin wi -- --help`
- `cargo run --bin wi -- --version`
- `cargo run --bin build_index -- --version`
- `cargo run --bin wi-init -- --version`
- `cargo run --bin wi-stats -- --version`
- `cargo test --test local_index -- --ignored`
- `cargo test --test real_repos -- --ignored` if `test_repos/` exists

Report:
- changed files
- CI workflows added/updated
- release automation added/updated
- local release-check command if added
- artifact checks implemented
- release docs updated
- verification commands and results
- whether ignored local test passed
- whether ignored real-repo test ran, skipped, or failed
- remaining CI/release caveats
- commit hash

## Implementation tracking

- [x] Confirm PLAN_00 through PLAN_15 are complete and green.
- [x] Add archive content check script.
- [x] Add local release-check command.
- [x] Add CI workflow for core gates and package smoke.
- [x] Add release workflow for manual/tag artifact builds.
- [x] Update release docs for CI/local release automation.
- [x] Add focused tests/checks for workflows and package content gates.
- [x] Run required verification.
- [x] Commit with `Add release automation and CI gates`.
