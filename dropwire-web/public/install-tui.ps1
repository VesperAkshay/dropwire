$ErrorActionPreference = "Stop"

Write-Host "Downloading DropWire TUI (dropwirex)..." -ForegroundColor Cyan

$Target = "x86_64-pc-windows-msvc"
$ReleasesApi = "https://api.github.com/repos/VesperAkshay/dropwire/releases"

# Find latest TUI release
$Releases = Invoke-RestMethod -Uri $ReleasesApi
$LatestRelease = $Releases | Where-Object { $_.tag_name -match "^v" -or $_.tag_name -match "^tui-v" } | Select-Object -First 1

if (-not $LatestRelease) {
    Write-Error "Could not find a valid release."
    exit 1
}

$LatestTag = $LatestRelease.tag_name
Write-Host "Found latest version: $LatestTag" -ForegroundColor Green

$AssetName = "dropwire-tui-$Target.zip"
$DownloadUrl = "https://github.com/VesperAkshay/dropwire/releases/download/$LatestTag/$AssetName"
$TempZip = "$env:TEMP\$AssetName"
$InstallDir = "$env:LOCALAPPDATA\DropWire\bin"

Write-Host "Downloading from: $DownloadUrl"
Invoke-WebRequest -Uri $DownloadUrl -OutFile $TempZip

Write-Host "Extracting..."
if (-not (Test-Path $InstallDir)) {
    New-Item -ItemType Directory -Path $InstallDir | Out-Null
}

Expand-Archive -Path $TempZip -DestinationPath $InstallDir -Force

# Add to PATH if not already there
$UserPath = [Environment]::GetEnvironmentVariable("PATH", "User")
if ($UserPath -notmatch [regex]::Escape($InstallDir)) {
    Write-Host "Adding $InstallDir to user PATH..."
    [Environment]::SetEnvironmentVariable("PATH", "$UserPath;$InstallDir", "User")
    $env:PATH = "$env:PATH;$InstallDir"
}

# Cleanup
Remove-Item $TempZip

Write-Host ""
Write-Host "==================================" -ForegroundColor Green
Write-Host "DropWire TUI installed successfully!" -ForegroundColor Green
Write-Host "Run 'dropwirex' to launch the interactive UI." -ForegroundColor Green
Write-Host "==================================" -ForegroundColor Green
