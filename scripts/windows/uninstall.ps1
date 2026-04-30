param(
    [string]$DestinationDir = (Join-Path $env:LOCALAPPDATA "Programs\thinindex\bin")
)

$ErrorActionPreference = "Stop"
$Binaries = @("wi.exe", "build_index.exe", "wi-init.exe", "wi-stats.exe")

Write-Host "Uninstalling thinindex commands from: $DestinationDir"

foreach ($Binary in $Binaries) {
    $TargetPath = Join-Path $DestinationDir $Binary
    if (Test-Path -Path $TargetPath -PathType Leaf) {
        Remove-Item -Force -Path $TargetPath
        Write-Host "removed: $TargetPath"
    } else {
        Write-Host "not found: $TargetPath"
    }
}

Write-Host ""
Write-Host "This removed installed thinindex commands only."
Write-Host "It does not remove repo-local files such as .dev_index, .thinindexignore, AGENTS.md, or CLAUDE.md."
Write-Host "To remove a repo-local index, run wi-init --remove inside that repository before uninstalling."
