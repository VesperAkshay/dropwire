$ErrorActionPreference = "Stop"

Write-Host "Downloading DropWire CLI..." -ForegroundColor Cyan

$Target = "x86_64-pc-windows-msvc"
$ReleasesApi = "https://api.github.com/repos/VesperAkshay/dropwire/releases"

# Find latest CLI tag
$Releases = Invoke-RestMethod -Uri $ReleasesApi
$LatestCliRelease = $Releases | Where-Object { $_.tag_name -match "^cli-v" } | Select-Object -First 1

if (-not $LatestCliRelease) {
    Write-Error "Could not find a valid CLI release."
    exit 1
}

$LatestTag = $LatestCliRelease.tag_name
Write-Host "Found latest version: $LatestTag" -ForegroundColor Green

$AssetName = "dropwire-cli-$Target.zip"
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
Write-Host "DropWire CLI installed successfully!" -ForegroundColor Green
Write-Host "Run 'dropwire --help' to get started." -ForegroundColor Green
Write-Host "==================================" -ForegroundColor Green
