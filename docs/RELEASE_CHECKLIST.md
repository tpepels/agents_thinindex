# Release Checklist

Run before cutting a thinindex release:

- `cargo fmt --check`
- `cargo test`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo deny check licenses`
- `scripts/check-ci`
- `scripts/check-build-performance`
- `cargo test --test local_index -- --ignored`
- `cargo test --test real_repos -- --ignored` when `test_repos/` exists
- `scripts/check-release`
- `cargo run --bin wi -- --help`
- `cargo run --bin wi -- --version`
- `cargo run --bin build_index -- --version`
- `cargo run --bin wi-init -- --version`
- `cargo run --bin wi-stats -- --version`
- `cargo run --bin wi-scorecard -- --version`
- confirm `THIRD_PARTY_NOTICES` matches the audited dependency set and is included with release artifacts
- confirm generated archive `SBOM.md` names the version, target, shipped binaries, checksum sidecar, and notice file
- confirm `scripts/check-package-contents <archive>` validates `SBOM.md` fields and rejects unexpected release payload entries
- confirm `docs/NATIVE_DISTRIBUTION_PLAN.md` still distinguishes archive RC readiness from signed/native production distribution readiness
- confirm `docs/SECURITY_PRIVACY.md` matches current index, report, and release artifact behavior
- confirm the documented parser support matrix matches the bundled Tree-sitter grammar dependencies
- confirm `docs/QUALITY_SYSTEM_AUDIT.md` still matches parser, support, quality, ctags, license, deterministic parser/quality fixtures, and release behavior
- confirm `docs/TECHNICAL_FINAL_AUDIT.md` still matches dependency, refs, pack, impact, performance, semantic adapter, and agent integration behavior
- `scripts/package-release`
- `scripts/check-package-contents <archive>`
- `scripts/smoke-release-archive <archive>`
- confirm the archive target is marked publishable in `docs/TARGET_PLATFORM_SMOKE.md`
- inspect/list the generated archive contents
- verify the generated `.sha256` sidecar with `sha256sum -c` or `shasum -a 256 -c`
- treat any archive without a generated and verified `.sha256` sidecar as not release-ready
- unpack smoke with packaged `wi --help`, packaged `wi doctor`, packaged `build_index --stats`, `wi <query>`, `wi refs`, `wi pack`, `wi impact`, and `wi-scorecard` in a temporary repo
- archive install smoke with `scripts/install-archive-unix` from the extracted archive on Unix-like platforms
- archive uninstall smoke with `scripts/uninstall-archive-unix` from the extracted archive on Unix-like platforms
- install smoke with a temp `BIN_DIR`
- uninstall smoke with the same temp `BIN_DIR`
- do not publish any target marked `not smoked` or `do not publish` in `docs/TARGET_PLATFORM_SMOKE.md`

Manual repo smoke:

- `rm -rf .dev_index`
- `build_index`
- `wi build_index`
- `wi refs build_index`
- `wi pack build_index`
- `wi impact build_index`
- `wi-stats`
- `wi-scorecard`

Packaging note:

- Universal Ctags is optional, external, not bundled, not required, and not used by production indexing.
- Tree-sitter parser and grammar dependencies are bundled and must remain permissively licensed.
- Cross-platform release archives and installers require a passing `cargo deny check licenses` run.
- Proprietary packaging remains blocked by GPL, AGPL, LGPL-only, MPL-only, EPL, CDDL, unknown, custom, or non-commercial dependency terms unless a future plan records an explicit review exception.
- Release archives must include all thinindex binaries, `README.md`, `INSTALL.md`, `SBOM.md`, `docs/CI_INTEGRATION.md`, `docs/GETTING_STARTED.md`, `docs/RELEASING.md`, `docs/INSTALLERS.md`, `docs/NATIVE_DISTRIBUTION_PLAN.md`, `docs/LICENSING.md`, `docs/SCORECARD.md`, `docs/SECURITY_PRIVACY.md`, `docs/TEAM_CI_ROADMAP.md`, `docs/TROUBLESHOOTING.md`, helper install/uninstall scripts, and `THIRD_PARTY_NOTICES`.
- Target-platform status is tracked in `docs/TARGET_PLATFORM_SMOKE.md`; archive naming or cross-compilation alone is not publishable evidence.
- Release archives must not include `.dev_index/`, `.dev_index/quality/`, `test_repos/`, `target/`, `dist/`, source checkout contents, local quality reports, generated local benchmark outputs, signing secret material, or optional external comparator binaries.
- Release archive content checks must reject unexpected payload entries and validate `SBOM.md` against the archive basename, checksum sidecar, shipped binaries, notices, and not-bundled boundaries.
- Native package formats, signing, notarization, Homebrew, winget, and managed update channels are deferred/scaffolded only; use `docs/NATIVE_DISTRIBUTION_PLAN.md` before claiming production native distribution readiness.
- Windows Authenticode signing, macOS Developer ID signing/notarization, and Linux package signing are not implemented by default; `scripts/sign-release-artifact` is the local/CI secret-backed scaffold.
- GitHub Actions CI runs format, test, deterministic parser/quality fixtures, clippy, license audit, command smoke, package smoke, archive content checks, checksum verification, and packaged binary smoke.
- Release archive smoke verifies checksum sidecars and packaged binary startup without requiring secrets.
- The release workflow uploads workflow artifacts only; it does not publish GitHub Releases.
- Smoke-test generated artifacts on each target platform before publishing.
- Untested target artifacts are blocked from publication until their target smoke checklist passes on a compatible platform.
