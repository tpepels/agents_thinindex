# PLAN_36_SIGNED_INSTALLER_AND_DISTRIBUTION_HARDENING.md

Use superpowers:subagent-driven-development.

Do not implement this until PLAN_41_SECURITY_PRIVACY_AND_REPORT_REDACTION.md is complete and green.

Goal:
Harden commercial distribution paths for Windows, macOS, and Linux.

Product rule:
Distribution must be explicit about what is signed, unsigned, notarized, packaged, or only scaffolded.

Required:
- Review existing release archives/installers/signing plans.
- Add or harden installer scripts/configs.
- Add artifact content checks.
- Add signing/notarization scaffolding without committing secrets.
- Add checksum/SBOM/notice inclusion if not already present.
- Ensure ctags is not bundled.
- Ensure THIRD_PARTY_NOTICES is included.
- Ensure no `.dev_index` or `test_repos` in artifacts.

Platform targets:
- Windows zip/MSI/MSIX or documented scaffold
- macOS tar/pkg/notarization scaffold
- Linux tar/deb/rpm/AppImage scaffold where practical

Hard constraints:
- no signing keys committed
- no secrets committed
- no ctags bundled
- no GPL/AGPL surprise dependencies
- no network requirement for normal tests

Tests/checks:
- all expected binaries included
- notices included
- license/docs included
- forbidden paths excluded
- artifact names include version/target
- checksums generated if supported

Docs:
Update:
- docs/RELEASING.md
- docs/INSTALLERS.md
- README install section if needed

Acceptance:
- distribution flow is clear
- artifact checks exist
- signing status is honest
- package contents are safe and audited

Verification:
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --all-targets --all-features -- -D warnings`
- license audit command if configured
- release packaging script
- archive content inspection
- install/uninstall smoke in temp dir if supported

Report:
- changed files
- platform status
- artifact checks
- signing/notarization status
- verification results
- commit hash
