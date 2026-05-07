# thinindex 0.1.4 RC Handoff

## Decision

RC decision: `ready_with_caveats`.

Readiness commit: `5467e4b Assess release candidate readiness`.

## Artifact

Local Linux archive:

```text
dist/thinindex-0.1.4-x86_64-unknown-linux-gnu.tar.gz
```

SHA256:

```text
a9f6c65f1ded053541a3cddcb10ee82228bb4bc2f2d77525538a1d486f1073af
```

Checksum sidecar:

```text
dist/thinindex-0.1.4-x86_64-unknown-linux-gnu.tar.gz.sha256
```

## Included Binaries

- `wi`
- `build_index`
- `wi-init`
- `wi-stats`
- `wi-scorecard`

Each binary reports `0.1.4 (index schema 12)`.

## Install

```bash
tar -xzf dist/thinindex-0.1.4-x86_64-unknown-linux-gnu.tar.gz -C /tmp
cd /tmp/thinindex-0.1.4-x86_64-unknown-linux-gnu
scripts/install-archive-unix
```

The installer copies thinindex commands only. It does not run `wi-init`, create
`.dev_index`, or mutate user repositories.

## Smoke Tests

Run after install:

```bash
wi --version
build_index --version
wi-init --version
wi-stats --version
wi-scorecard --version
wi doctor
build_index --stats
wi build_index
wi refs build_index
wi pack build_index
wi impact build_index
wi-scorecard
```

Run against the archive before handoff:

```bash
sha256sum dist/thinindex-0.1.4-x86_64-unknown-linux-gnu.tar.gz
scripts/smoke-release-archive dist/thinindex-0.1.4-x86_64-unknown-linux-gnu.tar.gz
scripts/check-release
```

## Known Caveats

- RC status is `ready_with_caveats`, not a fully published release.
- This handoff covers the local Linux `x86_64-unknown-linux-gnu` archive.
- Other target archives need their own target-platform smoke before publishing.
- Target-platform status is tracked in `docs/TARGET_PLATFORM_SMOKE.md`; targets
  marked `not smoked` or `do not publish` are blocked from publication.
- Optional local `test_repos/` evidence is checkout-local; third-party repos are
  intentionally not committed.
- File-reference extraction remains best-effort and local; it does not claim
  package-manager, compiler, LSP, framework alias, package export-map, or
  network semantics.

## Intentionally Not Included

- `.dev_index/` or repo-local indexes
- `test_repos/`
- local quality reports or comparator triage reports
- generated benchmark/local state
- source checkout contents
- build output such as `target/` or `dist/`
- signing keys, certificates, provisioning profiles, or other secrets
- optional external comparator binaries
- native installers, signing, notarization, package-manager publishing, hosted
  services, telemetry, payment behavior, or license enforcement
- `WI.md`
- JSONL as canonical storage

## Before Non-Linux Publishing

For each non-Linux target archive:

1. Build the target archive with `scripts/package-release --target <target>`.
2. Verify the generated checksum sidecar.
3. Run `scripts/check-package-contents <archive>`.
4. Run `scripts/smoke-release-archive <archive>` on a compatible target.
5. Smoke the archive installer on the target platform.
6. Confirm all packaged binaries report `0.1.4 (index schema 12)`.
7. Confirm `wi doctor`, `build_index --stats`, `wi <query>`, `wi refs`, `wi
   pack`, `wi impact`, and `wi-scorecard` work in a temporary repository.
8. Update `docs/TARGET_PLATFORM_SMOKE.md` with concrete target evidence before
   treating the artifact as publishable.

## Rollback

From the extracted archive root:

```bash
scripts/uninstall-archive-unix
```

This removes installed thinindex commands from the selected bin directory only.
It does not remove repo-local files such as `.dev_index/`, `.thinindexignore`,
`AGENTS.md`, or `CLAUDE.md`.

## Next Recommended Action

Tag/package the 0.1.4 RC from a clean worktree after rerunning the release
checklist on the target platform being published.
