# PLAN_65_NATIVE_PACKAGE_SIGNING_PLAN.md

Use superpowers:subagent-driven-development.

Do not implement this until PLAN_64_MCP_INTEGRATION_DECISION.md is complete and green.

Goal:
Define a concrete native package/signing/notarization plan for production distribution without pretending it is implemented.

Context:
The feature wiring audit found native package publishing, real signing/notarization, trusted update channels, and package-manager distribution are missing or scaffolded.

Scope:
Planning, docs, and checks only unless a tiny scaffold fix is required. Do not implement payment/licensing enforcement, hosted services, telemetry, parser changes, MCP, ctags production use, JSONL canonical storage, or `WI.md`.

Product rule:
Distribution claims must distinguish archive RC readiness from signed/native production distribution readiness.

Phases:
- [x] Inspect current install, archive, signing, release, and installer docs/scripts.
- [x] Inventory target formats.
- [x] Identify credentials required for signing/notarization.
- [x] Identify platform machines required for smoke.
- [x] Define package-manager publishing path separately from archive publishing.
- [x] Add explicit production distribution blockers.
- [x] Add a staged distribution roadmap.
- [x] Add tests/docs checks to prevent overclaiming signed/native readiness.
- [x] Run verification.
- [x] Commit.

Decision:
Keep production native distribution deferred while preserving the archive RC
path. Current release readiness means unsigned local archives with checksum
sidecars, SBOM/notices, content checks, and target-platform smoke. Native
packages, real signing/notarization, Homebrew, winget, repository publishing,
managed update channels, and rollback policy require later platform-specific
plans with external credentials, compatible smoke machines, and package-manager
review paths.

Target areas:
- Linux archive
- Linux deb/rpm/AppImage or explicit deferral
- macOS archive/pkg and notarization
- Windows zip/MSI/MSIX and signing
- Homebrew
- winget
- checksums
- SBOM/notices
- update channels
- rollback/uninstall

Required docs:
Create or update:
- `docs/NATIVE_DISTRIBUTION_PLAN.md`
- `docs/RELEASING.md`
- `docs/INSTALLERS.md`
- `docs/RELEASE_CHECKLIST.md`
- `docs/PRODUCT_BOUNDARY.md` if needed

Acceptance:
- archive RC readiness is not confused with production signed/native distribution.
- signing/notarization credentials are explicitly external blockers.
- target-platform smoke requirements are clear.
- package-manager publishing is clearly deferred or scoped.
- docs do not claim native package readiness prematurely.

Verification:
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --all-targets --all-features -- -D warnings`
- docs/governance tests if present
- `scripts/check-release` if release docs/scripts changed
- `cargo run --bin wi-scorecard`

Commit:
Plan native package signing path
