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

Not implemented:

- Windows MSI, MSIX, WiX, Inno Setup, Store packages, or Authenticode signing
- macOS `.pkg`, `.dmg`, Developer ID signing, notarization, or stapling
- Linux `.deb`, `.rpm`, AppImage, package repositories, or package signing
- CI publishing, native installer signing, or update channels

All installer helpers install only the thinindex commands:

- `wi`
- `build_index`
- `wi-init`
- `wi-stats`

They do not run `wi-init`, create `.dev_index`, mutate user repositories, delete repo-local `.dev_index`, or remove project files. Uninstall helpers remove only installed thinindex command files from the selected bin directory.

`THIRD_PARTY_NOTICES` ships with release artifacts and must stay with distributed archives or installers. Universal Ctags is not bundled and not required.

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

Future signing scaffolding should use environment variables or secure CI secret storage for certificate material, never committed files.

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

Future signing scaffolding should use environment variables or secure CI secret storage for Apple certificate, team, and notarization credentials.

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

Package manager formats and signing are future work.

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
