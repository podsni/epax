# epax install script — Windows (PowerShell)
# Usage: iwr -useb https://raw.githubusercontent.com/podsni/epax/main/scripts/install.ps1 | iex

[CmdletBinding()]
param(
    [string]$Version = "",
    [string]$InstallDir = "$env:LOCALAPPDATA\epax"
)

$ErrorActionPreference = "Stop"
$Repo = "podsni/epax"
$BinName = "epax.exe"
$OsTag = "x86_64-pc-windows-msvc"

# resolve version
if (-not $Version) {
    Write-Host "Fetching latest version..."
    $rel = Invoke-RestMethod "https://api.github.com/repos/$Repo/releases/latest"
    $Version = $rel.tag_name
}

Write-Host "Installing epax $Version"

$Archive = "epax-${OsTag}.zip"
$Url = "https://github.com/$Repo/releases/download/$Version/$Archive"
$Tmp = [System.IO.Path]::GetTempPath() + [System.IO.Path]::GetRandomFileName()
New-Item -ItemType Directory -Path $Tmp | Out-Null

try {
    Write-Host "Downloading $Url"
    $ZipPath = "$Tmp\$Archive"
    Invoke-WebRequest -Uri $Url -OutFile $ZipPath -UseBasicParsing
    Expand-Archive -Path $ZipPath -DestinationPath $Tmp -Force

    if (-not (Test-Path $InstallDir)) {
        New-Item -ItemType Directory -Path $InstallDir | Out-Null
    }

    Copy-Item "$Tmp\epax-${OsTag}\$BinName" "$InstallDir\$BinName" -Force

    # add to user PATH if not already there
    $UserPath = [Environment]::GetEnvironmentVariable("PATH", "User")
    if ($UserPath -notlike "*$InstallDir*") {
        [Environment]::SetEnvironmentVariable("PATH", "$UserPath;$InstallDir", "User")
        Write-Host "Added $InstallDir to your PATH (restart shell to take effect)"
    }

    Write-Host ""
    Write-Host "epax $Version installed to $InstallDir\$BinName"
    Write-Host "Run: epax --help"
}
finally {
    Remove-Item -Recurse -Force $Tmp -ErrorAction SilentlyContinue
}
