# Release Checklist

Run before cutting a thinindex release:

- `cargo fmt --check`
- `cargo test`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test local_index -- --ignored`
- `cargo test --test real_repos -- --ignored` when `test_repos/` exists
- `cargo run --bin wi -- --help`
- `cargo run --bin wi -- --version`
- `cargo run --bin build_index -- --version`
- `cargo run --bin wi-init -- --version`
- `cargo run --bin wi-stats -- --version`
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

- Cross-platform release archives and installers require dependency license audit coverage.
- Smoke-test generated artifacts on each target platform before publishing.
