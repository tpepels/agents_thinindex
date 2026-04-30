# Release Checklist

Run before cutting a thinindex release:

- `cargo fmt --check`
- `cargo test`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo deny check licenses`
- `cargo test --test local_index -- --ignored`
- `cargo test --test real_repos -- --ignored` when `test_repos/` exists
- `cargo run --bin wi -- --help`
- `cargo run --bin wi -- --version`
- `cargo run --bin build_index -- --version`
- `cargo run --bin wi-init -- --version`
- `cargo run --bin wi-stats -- --version`
- confirm `THIRD_PARTY_NOTICES` matches the audited dependency set and is included with release artifacts
- confirm the documented parser support matrix matches the bundled Tree-sitter grammar dependencies
- `scripts/package-release`
- inspect/list the generated archive contents
- install smoke with a temp `BIN_DIR`
- uninstall smoke with the same temp `BIN_DIR`

Manual repo smoke:

- `rm -rf .dev_index`
- `build_index`
- `wi build_index`
- `wi refs build_index`
- `wi pack build_index`
- `wi impact build_index`
- `wi-stats`

Packaging note:

- Universal Ctags is removed, not bundled, and not used.
- Tree-sitter parser and grammar dependencies are bundled and must remain permissively licensed.
- Cross-platform release archives and installers require a passing `cargo deny check licenses` run.
- Proprietary packaging remains blocked by GPL, AGPL, LGPL-only, MPL-only, EPL, CDDL, unknown, custom, or non-commercial dependency terms unless a future plan records an explicit review exception.
- Release archives must include all thinindex binaries, `README.md`, `INSTALL.md`, `docs/RELEASING.md`, and `THIRD_PARTY_NOTICES`.
- Release archives must not include `.dev_index/`, `test_repos/`, `target/`, `dist/`, source checkout contents, or generated local benchmark outputs.
- Native installers, signing, and notarization are later work.
- Smoke-test generated artifacts on each target platform before publishing.
