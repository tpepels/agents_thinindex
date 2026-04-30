# CI Integration

thinindex CI integration is currently local-only. There is no hosted backend,
account login, source upload, telemetry, license server, or cloud API.

## Local CI Checks

Use the existing local gates:

```bash
cargo fmt --check
cargo test
cargo clippy --all-targets --all-features -- -D warnings
cargo run --bin build_index
cargo run --bin wi -- doctor
cargo run --bin wi -- bench
```

For release packaging:

```bash
scripts/package-release
scripts/check-package-contents dist/thinindex-<version>-<target>.tar.gz
```

These commands operate on the checkout and local `.dev_index/index.sqlite`.
Normal CI should not require `test_repos/`, ignored tests, optional comparator
tools, or network-fetched third-party repositories.

## Local Artifact Format

Future CI report artifacts should be compact, redacted, and local-only by
default. A report should include:

- tool version;
- command pass/fail status;
- `wi doctor` summary;
- index freshness/schema status;
- parser support counts;
- benchmark summary counts and timings;
- package content check status when packaging runs;
- redaction and source-upload flags.

Reports should not include:

- repository source files;
- `.dev_index/index.sqlite`;
- `.dev_index/quality/` detail dumps;
- `test_repos/`;
- unredacted secrets or sensitive paths;
- raw full command logs with source snippets.

## No-Source-Upload Mode

No-source-upload mode is the default future team/CI posture. CI may generate a
small JSON or Markdown artifact locally, then teams can decide whether to store
it in their own CI artifact system.

Required flags for future machine-readable artifacts:

- `source_upload: false`
- `redacted: true`
- `local_only: true`
- `schema_version`

## Hosted Boundary

No hosted thinindex service exists. A future hosted report viewer must accept
only explicit user-provided report artifacts and must not be required for local
commands.

Before any hosted workflow is implemented, a future plan must specify:

- exactly what leaves the repository;
- whether source upload is possible;
- default no-source-upload behavior;
- retention controls;
- redaction behavior;
- authentication/account boundaries;
- support access policy;
- failure behavior when offline.

## GitHub Actions Example

```yaml
name: thinindex local report

on:
  pull_request:
  push:

jobs:
  thinindex:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy,rustfmt
      - run: cargo fmt --check
      - run: cargo test
      - run: cargo clippy --all-targets --all-features -- -D warnings
      - run: cargo run --bin build_index
      - run: cargo run --bin wi -- doctor
      - run: cargo run --bin wi -- bench
```

This example does not upload source and does not contact a thinindex backend.
