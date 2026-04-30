# PLAN_08_INSTALL_AND_RELEASE_HARDENING.md

Use superpowers:subagent-driven-development.

Do not implement this until PLAN_00 through PLAN_07 are complete and green.

Goal:
Harden thinindex installation, uninstall, version reporting, and release readiness now that SQLite, refs, context commands, impact, benchmarks, and real-repo benchmark manifests exist.

This pass does not add new search semantics, reference extraction rules, ML prediction, or context commands.

Phase tracking:
- [x] Harden install/uninstall messaging and script assertions.
- [x] Add version/help smoke tests for all binaries.
- [x] Update operator docs for install, uninstall, SQLite cache behavior, and current commands.
- [x] Add lightweight release checklist.
- [x] Run required formatting, tests, lint, version/help commands, and ignored tests.
- [x] Commit with `Harden install and release readiness`.

Product rule:
Installation and removal must be boring, repeatable, and test-visible.

Scope:
Audit and harden:

- `install.sh`
- `uninstall.sh`
- CLI binary install behavior
- `wi --version`
- `build_index --version`
- `wi-init --version`
- `wi-stats --version`
- installed command availability
- SQLite dependency behavior
- repo-local `.dev_index` behavior after install/uninstall
- shell/PATH instructions if present

Required behavior:
- Install should install all expected binaries:
  - `wi`
  - `build_index`
  - `wi-init`
  - `wi-stats`
- Uninstall should remove all installed thinindex binaries it installed.
- Install/uninstall should be idempotent.
- Install should not delete repo-local `.dev_index`.
- Uninstall should not delete repo-local `.dev_index`.
- `wi-init --remove` remains the command that removes repo-local `.dev_index`.
- Version commands should work for all binaries.
- SQLite storage must not require unexpected runtime setup beyond the binary/dependency choice from PLAN_00.
- If SQLite is bundled, docs/tests should not imply users must install system SQLite.
- If SQLite is not bundled, docs/tests must clearly state the runtime requirement.

Version reporting:
Ensure all binaries support `--version` through Clap or equivalent.

Expected examples:

- `wi --version`
- `build_index --version`
- `wi-init --version`
- `wi-stats --version`

Exact output may include package version. Tests should check command success and meaningful version text, not fragile exact formatting unless already standardized.

Install script tests:
Update or add tests for:

- install script references all binaries
- uninstall script references all binaries
- install script includes SQLite-safe binary build/install flow
- uninstall script does not remove `.dev_index`
- install script does not remove `.dev_index`

If tests execute scripts, isolate with a temp install directory. If tests only inspect scripts, keep assertions precise enough to catch missing binaries.

Smoke tests:
Add normal tests if practical:

- installed binary list includes `wi`, `build_index`, `wi-init`, `wi-stats`
- `--version` succeeds for each binary via `assert_cmd`
- `wi --help` succeeds and includes current subcommands:
  - `refs`
  - `pack`
  - `impact`
  - `bench` if implemented

Do not require actual user PATH mutation in tests.

Docs:
Update README or install docs if present.

Required docs topics:

- install
- uninstall
- initialize a repo with `wi-init`
- build index with `build_index`
- search with `wi`
- use `wi --help`
- remove repo-local index with `wi-init --remove`
- note that `.dev_index/index.sqlite` is a disposable local cache
- note that old JSONL `.dev_index` caches are automatically rebuilt by `build_index`
- no `WI.md` dependency

Do not over-document internals. Keep operator docs task-oriented.

AGENTS/CLAUDE:
- Keep AGENTS.md and existing CLAUDE.md generation in sync with the canonical Repository search block.
- Do not change the block unless install/release hardening changes normal agent workflow.
- Do not reintroduce `WI.md`.

Release readiness:
Add or update a lightweight release checklist doc if one exists or if useful.

Suggested checklist:

- `cargo fmt --check`
- `cargo test`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test local_index -- --ignored`
- `cargo test --test real_repos -- --ignored` when `test_repos/` exists
- install script smoke
- uninstall script smoke
- `wi --help`
- all binary `--version` commands
- manual smoke:
  - `rm -rf .dev_index`
  - `build_index`
  - `wi build_index`
  - `wi refs build_index`
  - `wi pack build_index`
  - `wi impact build_index`
  - `wi-stats`

Normal tests:
Keep normal `cargo test` independent of:

- local `.dev_index`
- `test_repos/`
- user PATH
- network access
- globally installed binaries

Ignored/manual tests:
Existing ignored local/real-repo tests should still pass.

Packaging constraint:
- Do not bundle Universal Ctags.
- Do not ship release archives/installers that require bundled ctags.
- Universal Ctags is optional quality-comparator tooling only; install/release hardening must not describe it as a production parser or bundled dependency.
- Cross-platform release artifacts must use the Tree-sitter production parser stack from PLAN_11A through PLAN_11C and must not bundle ctags.

Acceptance:
- install/uninstall scripts mention and handle all binaries.
- all binaries support `--version`.
- `wi --help` remains current and tidy.
- docs no longer mention `WI.md` as an instruction file.
- docs explain `.dev_index/index.sqlite` as disposable local cache.
- install/uninstall behavior is idempotent or documented/test-visible.
- existing search, refs, pack, impact, bench, stats, and wi-init behavior remains stable.
- no JSONL storage is reintroduced.
- no new product features are added.

Verification:
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo run --bin wi -- --help`
- `cargo run --bin wi -- --version`
- `cargo run --bin build_index -- --version`
- `cargo run --bin wi-init -- --version`
- `cargo run --bin wi-stats -- --version`
- `cargo test --test local_index -- --ignored`
- `cargo test --test real_repos -- --ignored` if `test_repos/` exists

Report:
- changed files
- install/uninstall behavior changes
- docs updated
- version command outputs
- verification commands and results
- whether ignored local test passed
- whether ignored real-repo test ran, skipped, or failed
- commit hash
