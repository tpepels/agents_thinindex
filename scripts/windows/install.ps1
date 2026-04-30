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

    & $TargetPath --version
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
