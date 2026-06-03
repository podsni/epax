# epax uninstall script — Windows (PowerShell)
# Usage: iwr -useb https://raw.githubusercontent.com/podsni/epax/main/scripts/uninstall.ps1 | iex

[CmdletBinding()]
param(
    [switch]$Purge,
    [switch]$Force
)

$ErrorActionPreference = "Stop"
$BinName = "epax.exe"
$InstallDir = "$env:LOCALAPPDATA\epax"
$BinPath = "$InstallDir\$BinName"

if (-not (Test-Path $BinPath)) {
    # try PATH
    $found = Get-Command epax -ErrorAction SilentlyContinue
    if ($found) { $BinPath = $found.Source } else {
        Write-Host "epax is not installed."
        exit 0
    }
}

if (-not $Force) {
    $ans = Read-Host "Uninstall epax at $BinPath? [y/N]"
    if ($ans -notmatch '^[yY]') { Write-Host "Aborted."; exit 0 }
}

Remove-Item -Force $BinPath
Write-Host "Removed $BinPath"

# remove from PATH
$UserPath = [Environment]::GetEnvironmentVariable("PATH", "User")
$Clean = ($UserPath -split ";") | Where-Object { $_ -ne $InstallDir } | Join-String -Separator ";"
[Environment]::SetEnvironmentVariable("PATH", $Clean, "User")

if ($Purge) {
    foreach ($cfg in @("$env:APPDATA\epax", "$env:LOCALAPPDATA\epax")) {
        if (Test-Path $cfg) { Remove-Item -Recurse -Force $cfg; Write-Host "Removed $cfg" }
    }
}

Write-Host "epax uninstalled."
