$ErrorActionPreference = "Stop"

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "    Uninstalling DropWire Suite         " -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan

$InstallDir = "$env:LOCALAPPDATA\DropWire\bin"
if (Test-Path $InstallDir) {
    Remove-Item -Path $InstallDir -Recurse -Force
    Write-Host "✓ Removed installation directory: $InstallDir" -ForegroundColor Green
} else {
    Write-Host "DropWire directory not found at $InstallDir" -ForegroundColor Yellow
}

$UserPath = [Environment]::GetEnvironmentVariable("PATH", "User")
$PathArray = $UserPath -split ";"
$NewPathArray = $PathArray | Where-Object { $_ -ne $InstallDir -and $_ -ne "$InstallDir\" }
$NewPath = $NewPathArray -join ";"

if ($UserPath -ne $NewPath) {
    [Environment]::SetEnvironmentVariable("PATH", $NewPath, "User")
    Write-Host "✓ Removed DropWire from user PATH." -ForegroundColor Green
} else {
    Write-Host "DropWire not found in user PATH." -ForegroundColor Yellow
}

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "DropWire Suite has been fully uninstalled." -ForegroundColor Green
Write-Host "========================================" -ForegroundColor Cyan
