# Target Platform Smoke

Release rule: do not publish a target artifact until that exact archive has
passed smoke checks on a compatible target platform.

This document tracks archive smoke status only. It does not claim native
package formats, signing, notarization, package-manager publishing, update
channels, hosted services, telemetry, payment behavior, or license enforcement.

## Status Matrix

| Target | Archive | Smoke platform | Status | Publish rule | Evidence |
| --- | --- | --- | --- | --- | --- |
| `x86_64-unknown-linux-gnu` | `.tar.gz` | Linux x86_64 | verified for the local 0.1.4 RC archive | publish only after rerunning the checklist below for the archive being handed off | `scripts/check-release`; `scripts/smoke-release-archive dist/thinindex-0.1.4-x86_64-unknown-linux-gnu.tar.gz` when the archive exists |
| `aarch64-unknown-linux-gnu` | `.tar.gz` | Linux ARM64 | not smoked | do not publish | no target-platform smoke recorded |
| `x86_64-apple-darwin` | `.tar.gz` | macOS Intel | not smoked | do not publish | no target-platform smoke recorded |
| `aarch64-apple-darwin` | `.tar.gz` | macOS Apple Silicon | not smoked | do not publish | no target-platform smoke recorded |
| `x86_64-pc-windows-msvc` | `.zip` | Windows x64 | not smoked | do not publish | no target-platform smoke recorded |

## Required Target Smoke Checklist

Run this checklist on the compatible target platform for each archive:

- build the artifact with `scripts/package-release --target <target>`;
- verify the generated checksum sidecar;
- run `scripts/check-package-contents <archive>`;
- run `scripts/smoke-release-archive <archive>` on the target platform;
- smoke the archive installer on the target platform;
- confirm all packaged binaries report the same version and index schema;
- run packaged `wi doctor`;
- run packaged `build_index --stats`;
- run packaged `wi <query>`;
- run packaged `wi refs <query>`;
- run packaged `wi pack <query>`;
- run packaged `wi impact <query>`;
- run packaged `wi-scorecard`.

The query should be a symbol created inside a temporary smoke repository, such
as `thinindex_release_smoke_symbol`. Do not reuse source-checkout results as
target-platform evidence.

## Current Linux RC Evidence

The Linux `x86_64-unknown-linux-gnu` RC path is the only target with local
archive smoke evidence in this checkout. Before handoff or publication, rerun:

```bash
scripts/check-release
scripts/smoke-release-archive dist/thinindex-0.1.4-x86_64-unknown-linux-gnu.tar.gz
```

Record the resulting checksum in `docs/RC_0.1.4_HANDOFF.md` only for the exact
archive currently being handed off.

## Untested Targets

Untested target archives are blocked from publication. Archive naming or
cross-compilation support is not enough: a target becomes publishable only after
the target smoke checklist passes on that platform and the status matrix is
updated with concrete evidence.
