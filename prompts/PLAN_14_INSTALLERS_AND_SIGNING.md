# PLAN_14_INSTALLERS_AND_SIGNING.md

Use superpowers:subagent-driven-development.

Do not implement this until PLAN_00 through PLAN_13 are complete and green.

Goal:
Plan and add the first native installer/signing scaffolding for Windows, macOS, and Linux, without pretending full signing/notarization/repository publishing is complete.

This pass may add installer build scripts/configuration and documentation. Do not add license enforcement, payment behavior, telemetry, cloud behavior, or new product features.

Product rule:
Native installers must be explicit about what is implemented, what is unsigned, and what remains manual. Do not hide platform trust/signing limitations.

Hard requirements:
- Do not bundle Universal Ctags.
- Do not reintroduce Universal Ctags.
- Do not add GPL or AGPL dependencies.
- Do not add license/payment/Pro gating behavior.
- Do not reintroduce JSONL storage.
- Do not reintroduce `WI.md`.
- Installer artifacts must include or reference `THIRD_PARTY_NOTICES`.
- Installers must install all thinindex binaries:
  - `wi`
  - `build_index`
  - `wi-init`
  - `wi-stats`
- Installers/uninstallers must not delete repo-local `.dev_index`.

Scope:
Add installer/signing scaffolding for:

- Windows
- macOS
- Linux

Do not require every native installer format to be fully produced locally unless the platform/toolchain is available. Structure the scripts/configs so CI can build them later.

Windows:
Preferred progression:
1. zip archive from PLAN_13 remains supported
2. add PowerShell install/uninstall scripts if not already robust
3. add MSI/MSIX/WiX/Inno Setup plan or initial config only if practical

Required Windows docs:
- how to install from zip
- how to add binaries to PATH
- how to uninstall
- SmartScreen/signing caveat
- code signing is required before polished public distribution

macOS:
Preferred progression:
1. tar.gz archive from PLAN_13 remains supported
2. add install/uninstall shell behavior
3. add pkg/notarization plan or initial packaging config only if practical

Required macOS docs:
- how to install from archive
- PATH location
- unsigned binary/quarantine caveat
- signing/notarization required before polished public distribution

Linux:
Preferred progression:
1. tar.gz archive from PLAN_13 remains supported
2. add install/uninstall shell behavior
3. add deb/rpm/AppImage plan or initial packaging config only if practical

Required Linux docs:
- how to install from archive
- where binaries are installed
- how to uninstall
- package manager formats are future work unless implemented

Installer behavior:
- Install all binaries.
- Preserve existing files unless explicitly overwriting thinindex binaries.
- Print installed paths.
- Print version after install if practical.
- Do not create repo-local `.dev_index`.
- Do not run `wi-init` automatically.
- Do not mutate user repositories.
- Uninstall only installed thinindex binaries/scripts.
- Uninstall must not remove `.dev_index` from any repo.
- Uninstall must not remove user project files.

Signing:
Add documentation/scaffolding for signing, not real secrets.

Required:
- no signing keys committed
- no secrets in repo
- environment variable placeholders only if scripts need them
- clear docs for future:
  - Windows Authenticode/code signing
  - macOS Developer ID signing and notarization
  - Linux package signing/checksums

If adding scripts, they must fail clearly when required signing env vars/certs are missing.

Checksums:
Keep SHA256 checksums from PLAN_13 if implemented.
Installer docs should explain checksum verification.

Docs:
Add or update:

- `docs/INSTALLERS.md`
- `docs/RELEASING.md`
- README install section if needed

Docs must clearly distinguish:
- release archives: implemented
- native installers: implemented/scaffolded/planned
- signing/notarization: implemented or not implemented

Do not claim native installers are production-ready unless they are actually built and tested.

Tests:
Add focused tests/checks where practical.

Required tests/checks:
- install scripts mention all binaries:
  - `wi`
  - `build_index`
  - `wi-init`
  - `wi-stats`
- uninstall scripts mention all binaries
- uninstall scripts do not remove `.dev_index`
- installer docs mention `THIRD_PARTY_NOTICES`
- installer docs do not claim ctags is bundled
- installer docs do not claim signing/notarization is complete unless implemented
- no signing secrets or private keys are present in repo
- current `wi --help` remains current
- all binaries support `--version`

Forbidden:
- committing signing certificates
- committing private keys
- adding telemetry
- adding license checks
- adding network activation
- auto-running `wi-init` on arbitrary repos
- deleting `.dev_index` during install/uninstall
- bundling Universal Ctags

Instruction surfaces:
- Do not reintroduce `WI.md`.
- Keep `wi --help` as the source of truth for command syntax, filters, examples, and subcommands.
- Keep AGENTS.md and existing CLAUDE.md generation in sync with the canonical `## Repository search` block.
- AGENTS.md should be created if absent.
- CLAUDE.md should be normalized only if present; do not create CLAUDE.md.
- Repeated `wi-init` runs must not duplicate `## Repository search`.
- Remove/normalize legacy markers: `@WI.md`, `See WI.md for repository search/index usage.`, `See `WI.md` for repository search/index usage.`, and old paragraph-style Repository search blocks.
- Update tests whenever help text or canonical Repository search text changes.

Acceptance:
- installer/signing docs exist and are accurate.
- install/uninstall scripts or scaffolding are updated for all binaries.
- no installer/uninstaller deletes `.dev_index`.
- release archive flow from PLAN_13 remains intact.
- native installer status is honestly documented.
- signing/notarization status is honestly documented.
- no ctags bundling or GPL/AGPL dependency is introduced.
- existing CLI behavior remains stable.
- no license/payment/network behavior is added.

Verification:
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --all-targets --all-features -- -D warnings`
- license audit command from PLAN_12, if added
- run release packaging script from PLAN_13 for current platform
- inspect/list archive contents
- run install script in a temp install directory if supported
- run uninstall script in a temp install directory if supported
- `cargo run --bin wi -- --help`
- `cargo run --bin wi -- --version`
- `cargo run --bin build_index -- --version`
- `cargo run --bin wi-init -- --version`
- `cargo run --bin wi-stats -- --version`
- `cargo test --test local_index -- --ignored`
- `cargo test --test real_repos -- --ignored` if `test_repos/` exists

Report:
- changed files
- installer/signing scaffolding added
- docs updated
- current installer status by platform
- signing/notarization status by platform
- verification commands and results
- whether ignored local test passed
- whether ignored real-repo test ran, skipped, or failed
- remaining packaging caveats
- commit hash
