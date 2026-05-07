# PLAN_62_TARGET_PLATFORM_RELEASE_SMOKE.md

Use superpowers:subagent-driven-development.

Do not implement this until PLAN_61_REAL_REPO_EVIDENCE_STABILIZATION.md is complete and green.

Goal:
Define and execute target-platform release smoke checks so non-Linux artifacts are not treated as ready without evidence.

Context:
The feature wiring audit found the Linux local archive path works, but other target archives still need target-platform smoke before publishing.

Scope:
Release smoke/checklist hardening only. Do not add native package-manager publishing, signing/notarization, hosted services, telemetry, payment/licensing enforcement, parser changes, MCP, ctags production use, JSONL canonical storage, or `WI.md`.

Product rule:
Do not publish a target artifact until it has been smoked on a compatible target platform.

Phases:
- [x] Inspect current release scripts and archive smoke scripts.
- [x] List supported/intended targets.
- [x] Add per-target smoke checklist entries.
- [x] Add target status tracking document.
- [x] Ensure current Linux target remains verified.
- [x] Add clear “not smoked, do not publish” status for untested targets.
- [x] Add tests/checks that release docs do not imply unverified targets are ready.
- [x] Run verification.
- [x] Commit.

Target smoke requirements:
For each target archive:
- build artifact
- verify checksum sidecar
- check package contents
- smoke archive installer on target platform
- confirm all binaries report version and schema
- run `wi doctor`
- run `build_index --stats`
- run `wi <query>`
- run `wi refs`
- run `wi pack`
- run `wi impact`
- run `wi-scorecard`

Required docs:
Create or update:
- `docs/TARGET_PLATFORM_SMOKE.md`
- `docs/RELEASE_CHECKLIST.md`
- `docs/RELEASING.md`
- `docs/RC_0.1.4_HANDOFF.md` if still relevant

Acceptance:
- release docs distinguish verified Linux RC from unverified target artifacts.
- each target has a clear smoke checklist.
- unverified targets are explicitly blocked from publication.
- scripts/checks continue to pass for the verified Linux archive.
- no misleading cross-platform readiness claim remains.

Verification:
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `scripts/check-release`
- `scripts/smoke-release-archive dist/thinindex-0.1.4-x86_64-unknown-linux-gnu.tar.gz` if present
- `cargo run --bin build_index -- --stats`
- `cargo run --bin wi-scorecard`

Commit:
Add target platform release smoke checklist
