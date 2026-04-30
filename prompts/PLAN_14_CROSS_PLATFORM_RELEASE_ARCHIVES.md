# PLAN_14_CROSS_PLATFORM_RELEASE_ARCHIVES.md

Use superpowers:subagent-driven-development.

Do not implement this until PLAN_00 through PLAN_13 are complete and green.

Goal:
Add reproducible cross-platform release archive packaging for thinindex.

This pass creates release archives only. Do not add native installers, signing, notarization, license enforcement, payment behavior, telemetry, cloud behavior, or new product features.

Product rule:
Release archives must be boring, explicit, and license-compliant. They must not bundle GPL/AGPL dependencies or Universal Ctags.

Hard requirements:
- Do not bundle Universal Ctags.
- Do not proceed if any active Universal Ctags code/test/install requirement remains or if the Tree-sitter grammar license audit is incomplete.
- Do not reintroduce Universal Ctags.
- Do not add GPL or AGPL dependencies.
- Do not add license/payment/Pro gating behavior.
- Do not reintroduce JSONL storage.
- Do not reintroduce `WI.md`.
- Release artifacts must include third-party notices.
- Release artifacts must include all thinindex binaries.

Release artifact targets:
Support archive creation for the current platform first, and structure the script so CI can build additional targets later.

Expected archive names:

- `thinindex-<version>-<target>.tar.gz` for Unix-like targets
- `thinindex-<version>-<target>.zip` for Windows targets

Target examples:
- `x86_64-unknown-linux-gnu`
- `aarch64-unknown-linux-gnu`
- `x86_64-apple-darwin`
- `aarch64-apple-darwin`
- `x86_64-pc-windows-msvc`

Do not require all cross-compilation targets to work locally in this pass unless the toolchain is already configured.

Archive contents:
Each release archive should contain:

- `wi` or `wi.exe`
- `build_index` or `build_index.exe`
- `wi-init` or `wi-init.exe`
- `wi-stats` or `wi-stats.exe`
- `README.md` or a release install/readme file
- `THIRD_PARTY_NOTICES`
- `LICENSE` if a license file exists
- short install notes

Do not include:
- `.dev_index`
- `test_repos`
- target build directory junk
- source checkout
- Universal Ctags
- generated local benchmark outputs unless explicitly intended

Script:
Add a packaging script, for example:

- `scripts/package-release`
- or `scripts/package_release.sh`

Required behavior:
- build release binaries
- collect binaries into a staging directory
- copy required docs/notices
- create archive under `dist/`
- print artifact path
- fail clearly if required files are missing
- clean/recreate staging directory deterministically
- not require network access

Optional:
- support `--target <triple>`
- support `--version <version>`
- support `--out-dir dist`
- support `--dry-run`

Versioning:
Use Cargo package version by default.

The script may derive version from:
- `cargo metadata`
- `Cargo.toml`
- `cargo pkgid`

Keep it simple and testable.

Windows notes:
If running on non-Windows, do not require local Windows archive creation unless target binaries already exist.

The script should be structured so Windows archive support can be tested by path/extension logic without needing Windows execution.

Checksums:
Add checksum generation if simple:

- SHA256 file beside each archive

Example:
- `thinindex-0.2.0-x86_64-unknown-linux-gnu.tar.gz.sha256`

If checksums are added, tests should verify they are created.

Tests:
Add tests for packaging logic without relying on global installation or network.

Acceptable test styles:
- script text/static tests
- temp staging tests if script supports dry-run or fake binary dir
- current-platform smoke test if fast enough

Required tests/checks:
- package script exists
- package script references all binaries:
  - `wi`
  - `build_index`
  - `wi-init`
  - `wi-stats`
- package script includes `THIRD_PARTY_NOTICES`
- package script excludes `.dev_index`
- package script excludes `test_repos`
- package script does not mention bundling ctags
- docs/release notes do not claim ctags is bundled
- `wi --help` remains current
- all binaries still support `--version`

Docs:
Add or update release docs.

Required docs content:
- how to build a release archive
- what files are included
- how users install from archive manually
- `.dev_index/index.sqlite` is repo-local cache and not included
- Universal Ctags is not bundled and not required after native parser work
- THIRD_PARTY_NOTICES ships with release artifacts
- native installers/signing are a later plan

Suggested doc:
- `docs/RELEASING.md`

Do not over-document CI publishing if it does not exist yet.

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
- release archive packaging script exists
- archive includes all required binaries
- archive includes third-party notices
- archive excludes local caches and test repos
- no Universal Ctags is bundled or referenced as bundled
- packaging docs exist
- current-platform archive can be produced manually
- existing CLI behavior remains stable
- native installers/signing are not added in this plan

Verification:
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --all-targets --all-features -- -D warnings`
- license audit command from PLAN_12, if added
- `cargo run --bin wi -- --help`
- `cargo run --bin wi -- --version`
- `cargo run --bin build_index -- --version`
- `cargo run --bin wi-init -- --version`
- `cargo run --bin wi-stats -- --version`
- run the release packaging script for the current platform
- inspect/list archive contents
- `cargo test --test local_index -- --ignored`
- `cargo test --test real_repos -- --ignored` if `test_repos/` exists

Report:
- changed files
- release script added
- archive path produced
- archive contents summary
- checksum path if generated
- docs updated
- verification commands and results
- whether ignored local test passed
- whether ignored real-repo test ran, skipped, or failed
- remaining packaging caveats
- commit hash

## Implementation tracking

- [x] Confirm PLAN_00 through PLAN_13 are complete and green.
- [x] Add current-platform release archive script with future target structure.
- [x] Include all binaries, `README.md`, `INSTALL.md`, `THIRD_PARTY_NOTICES`, and optional `LICENSE`.
- [x] Exclude repo-local caches, `test_repos/`, build junk, source checkout contents, and benchmark outputs by explicit staging.
- [x] Generate SHA256 checksum sidecars.
- [x] Update release docs and checklist.
- [x] Add packaging tests/checks.
- [x] Run required verification and archive smoke.
- [x] Commit with `Add cross-platform release archives`.
