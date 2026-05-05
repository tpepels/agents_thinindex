param(
    [string]$SourceDir = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path,
    [string]$DestinationDir = (Join-Path $env:LOCALAPPDATA "Programs\thinindex\bin")
)

$ErrorActionPreference = "Stop"
$Binaries = @("wi.exe", "build_index.exe", "wi-init.exe", "wi-stats.exe")
$NoticePath = Join-Path $SourceDir "THIRD_PARTY_NOTICES"

if (!(Test-Path -Path $NoticePath -PathType Leaf)) {
    throw "missing archive notice file: $NoticePath"
}

New-Item -ItemType Directory -Force -Path $DestinationDir | Out-Null

foreach ($Binary in $Binaries) {
    $SourcePath = Join-Path $SourceDir $Binary
    if (!(Test-Path -Path $SourcePath -PathType Leaf)) {
        throw "missing archive binary: $SourcePath"
    }

    Copy-Item -Force -Path $SourcePath -Destination (Join-Path $DestinationDir $Binary)
}

foreach ($Binary in $Binaries) {
    $TargetPath = Join-Path $DestinationDir $Binary
    if (!(Test-Path -Path $TargetPath -PathType Leaf)) {
        throw "install failed: $TargetPath"
    }

    $VersionOutput = & $TargetPath --version
    Write-Host $VersionOutput
    if ($VersionOutput -notmatch "index schema") {
        throw "installed $TargetPath did not report its index schema"
    }
}

$ExpectedSchemaOutput = & (Join-Path $DestinationDir "build_index.exe") --version
if ($ExpectedSchemaOutput -notmatch "index schema ([0-9]+)") {
    throw "installed build_index.exe did not report a parseable index schema"
}
$ExpectedSchema = $Matches[1]

foreach ($Binary in $Binaries) {
    $TargetPath = Join-Path $DestinationDir $Binary
    $VersionOutput = & $TargetPath --version
    if ($VersionOutput -notmatch "index schema $ExpectedSchema") {
        throw "installed $TargetPath did not report expected index schema $ExpectedSchema"
    }
}

foreach ($Binary in $Binaries) {
    $CommandName = [System.IO.Path]::GetFileNameWithoutExtension($Binary)
    $Active = Get-Command $CommandName -ErrorAction SilentlyContinue
    $TargetPath = Join-Path $DestinationDir $Binary
    if ($Active -and $Active.Source -ne $TargetPath) {
        Write-Warning "PATH resolves $CommandName to $($Active.Source), not $TargetPath"
    }
}

Write-Host "installed:"
foreach ($Binary in $Binaries) {
    Write-Host "  $(Join-Path $DestinationDir $Binary)"
}

Write-Host ""
Write-Host "Add this directory to PATH if needed:"
Write-Host "  $DestinationDir"
Write-Host ""
Write-Host "This installer copies thinindex commands only."
Write-Host "It does not run wi-init, create .dev_index, or mutate user repositories."
Write-Host "Keep THIRD_PARTY_NOTICES with distributed release artifacts."
