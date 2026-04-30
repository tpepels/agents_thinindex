# Installers And Signing

This document describes the current native installer and signing status. It does not add payments, license enforcement, telemetry, cloud behavior, auto-updates, repository publishing, or native package-manager distribution.

## Current Status

Implemented:

- release archives from `scripts/package-release`
- Unix-like archive install helper: `scripts/install-archive-unix`
- Unix-like archive uninstall helper: `scripts/uninstall-archive-unix`
- Windows archive install helper: `scripts/windows/install.ps1`
- Windows archive uninstall helper: `scripts/windows/uninstall.ps1`
- SHA256 checksum sidecars for release archives
- compact archive SBOM file: `SBOM.md`
- signing/notarization scaffold: `scripts/sign-release-artifact`

Not implemented:

- Windows MSI, MSIX, WiX, Inno Setup, Store packages, or completed Authenticode signing
- macOS `.pkg`, `.dmg`, completed Developer ID signing, notarization, or stapling
- Linux `.deb`, `.rpm`, AppImage, package repositories, or completed package signing
- GitHub Release publishing, native installer signing, or update channels

All installer helpers install only the thinindex commands:

- `wi`
- `build_index`
- `wi-init`
- `wi-stats`

They do not run `wi-init`, create `.dev_index`, mutate user repositories, delete repo-local `.dev_index`, or remove project files. Uninstall helpers remove only installed thinindex command files from the selected bin directory.

`THIRD_PARTY_NOTICES` ships with release artifacts and must stay with distributed archives or installers.

`SBOM.md` ships with release archives. It identifies the thinindex package
version, target triple, shipped binaries, checksum sidecar, and the
`THIRD_PARTY_NOTICES` file that contains dependency and parser grammar notices.

## Platform Matrix

| Platform | Implemented artifact | Native package status | Signing status |
| --- | --- | --- | --- |
| Windows | `.zip` archive with PowerShell helpers | MSI/MSIX/WiX/Inno Setup are documented scaffolds only | Authenticode scaffold only; unsigned by default |
| macOS | `.tar.gz` archive with Unix helpers | `.pkg`/`.dmg` are documented scaffolds only | Developer ID/notarization scaffold only; unsigned by default |
| Linux | `.tar.gz` archive with Unix helpers | `.deb`/`.rpm`/AppImage are documented scaffolds only | GPG/package signing scaffold only; unsigned by default |

## Windows

Current install path:

1. Download or build the Windows `.zip` archive.
2. Verify the `.zip.sha256` checksum.
3. Extract the archive.
4. Run PowerShell from the extracted archive root:

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\windows\install.ps1
```

By default the script installs to:

```text
%LOCALAPPDATA%\Programs\thinindex\bin
```

Pass `-DestinationDir` to choose another bin directory. Add that directory to PATH manually if needed.

Uninstall:

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\windows\uninstall.ps1
```

Windows signing status:

- Authenticode signing is not implemented.
- No signing certificates, private keys, or secrets are committed.
- Unsigned binaries may trigger Microsoft Defender SmartScreen warnings.
- Authenticode signing is required before polished public Windows distribution.
- `scripts/sign-release-artifact --platform windows --artifact <path>` is a scaffold for future executable, MSI, MSIX, or catalog signing.
- The scaffold requires `signtool` plus `THININDEX_WINDOWS_CERT_PATH`, `THININDEX_WINDOWS_CERT_PASSWORD`, and `THININDEX_WINDOWS_TIMESTAMP_URL` from the local environment or secure CI secrets.

Future native package work should add tested MSI/MSIX generation before claiming those formats as implemented.

## macOS

Current install path:

1. Download or build the macOS `.tar.gz` archive.
2. Verify the `.tar.gz.sha256` checksum.
3. Extract the archive.
4. Run from the extracted archive root:

```bash
scripts/install-archive-unix
```

By default the script installs to:

```text
$HOME/.local/bin
```

Pass `BIN_DIR=/path/to/bin` or `PREFIX=/path` to choose another install location.

Uninstall:

```bash
scripts/uninstall-archive-unix
```

macOS signing status:

- Developer ID signing is not implemented.
- Notarization and stapling are not implemented.
- No signing certificates, private keys, app-specific passwords, or notarization secrets are committed.
- Unsigned binaries may be quarantined by Gatekeeper after download.
- Developer ID signing and notarization are required before polished public macOS distribution.
- `scripts/sign-release-artifact --platform macos --artifact <path>` is a scaffold for future package/app signing and notarization submission.
- The scaffold requires `codesign`, `xcrun notarytool`, `THININDEX_APPLE_DEVELOPER_ID`, `THININDEX_APPLE_TEAM_ID`, and `THININDEX_APPLE_NOTARY_PROFILE` from the local environment or secure CI secrets.

Future native package work should add tested `.pkg` or `.dmg` generation before claiming those formats as implemented.

## Linux

Current install path:

1. Download or build the Linux `.tar.gz` archive.
2. Verify the `.tar.gz.sha256` checksum.
3. Extract the archive.
4. Run from the extracted archive root:

```bash
scripts/install-archive-unix
```

By default the script installs to:

```text
$HOME/.local/bin
```

Pass `BIN_DIR=/path/to/bin` or `PREFIX=/path` to choose another install location.

Uninstall:

```bash
scripts/uninstall-archive-unix
```

Linux packaging status:

- `.deb`, `.rpm`, AppImage, repository metadata, and package manager publishing are not implemented.
- Linux package signing is not implemented.
- No signing keys or secrets are committed.
- SHA256 checksums are implemented for release archives.
- `scripts/sign-release-artifact --platform linux --artifact <path>` is a scaffold for future detached GPG signatures.
- The scaffold requires `gpg` and `THININDEX_LINUX_GPG_KEY_ID` from the local environment or secure CI secrets.

Package manager formats and signing are future work. Do not claim `.deb`, `.rpm`, AppImage, repository metadata, or package-manager signing support until those artifacts are generated and content-checked in CI.

## Checksum Verification

Release archives include SHA256 sidecars.

Linux:

```bash
sha256sum -c thinindex-<version>-<target>.tar.gz.sha256
```

macOS:

```bash
shasum -a 256 -c thinindex-<version>-<target>.tar.gz.sha256
```

Windows PowerShell:

```powershell
Get-FileHash .\thinindex-<version>-<target>.zip -Algorithm SHA256
```

Compare the PowerShell hash output with the `.zip.sha256` contents.
