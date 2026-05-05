# PLAN_53_SCORECARD_INSTALL_DOC_ALIGNMENT.md

Use superpowers:subagent-driven-development.

Do not implement this until PLAN_52 is complete and
`prompts/recovery/RECOVERY_STATUS.md` identifies the scorecard install/docs
alignment as the next focused blocker before a release candidate.

Goal:
Align `wi-scorecard` docs, help, install scripts, archive packaging, and release
smoke behavior so user-facing instructions only mention commands that installed
users actually receive.

Problem:
`README.md` and `docs/GETTING_STARTED.md` present `wi-scorecard` as a normal
installed command, but the local install and archive install paths currently
ship only `wi`, `build_index`, `wi-init`, and `wi-stats`. This makes quickstart
instructions unreliable for installed/archive users even though source-built
`cargo run --bin wi-scorecard` works.

Decision to make:
Choose exactly one product stance and implement it consistently:

1. Install and package `wi-scorecard` with the other user-facing binaries; or
2. Keep `wi-scorecard` source/developer-only and update docs/help/installers so
   installed/archive users are not told to run it as an installed command.

Scope:
- Audit `wi-scorecard` references in README, docs, install scripts, archive
  packaging scripts, release smoke tests, package content checks, and installer
  docs.
- Align source, PATH, archive, and docs behavior with the chosen stance.
- Preserve schema/version consistency for every installed binary.
- Keep `wi-scorecard` local-only. Do not add telemetry, hosted reporting, cloud
  behavior, payments, licensing enforcement, or release publishing.

Do not add:
- package-manager execution;
- network access;
- LSP/compiler dependencies;
- Universal Ctags as production parser;
- JSONL canonical storage;
- `WI.md`;
- native package/signing/publishing behavior beyond the existing archive/local
  release flow.

Phases:
- [ ] Confirm PLAN_52 and recovery status identify this as the next focused
      blocker before RC.
- [ ] Inventory every `wi-scorecard` reference in docs, help, installers,
      package scripts, tests, and release checks.
- [ ] Choose the product stance: installed command or source/developer-only.
- [ ] Implement the smallest consistent change across docs/scripts/tests.
- [ ] Verify PATH and source binaries still agree on version/schema.
- [ ] Run release/archive smoke checks affected by the stance.
- [ ] Run full verification.
- [ ] Commit.

Acceptance:
- Installed/archive users are not instructed to run a missing command.
- If `wi-scorecard` is installed, all install/archive/package scripts include it
  and smoke-test `--version` with index schema output.
- If `wi-scorecard` remains source/developer-only, docs show `cargo run --bin
  wi-scorecard -- ...` or remove it from installed-command workflows.
- `wi --help`, README, getting-started docs, installer docs, release docs, and
  package content checks do not contradict each other.
- PATH/source schema agreement remains intact.
- No unrelated roadmap or release-publishing work is implemented.

Verification:
- `which wi`
- `which build_index`
- `wi --version`
- `build_index --version`
- `cargo run --bin wi -- --version`
- `cargo run --bin build_index -- --version`
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo run --bin build_index`
- `cargo run --bin wi -- doctor`
- `scripts/check-release` if package/release scripts or docs changed
- `scripts/smoke-release-archive <archive>` if a release archive is generated
  as part of the chosen implementation

Commit:
Align scorecard install docs
