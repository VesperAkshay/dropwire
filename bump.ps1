# ============================================================
#  DropWire Version Bump Script
#  Usage: .\bump.ps1   (run from F:\Blow)
# ============================================================

$CLI_TOML = "$PSScriptRoot\dropwire\Cargo.toml"
$TUI_TOML = "$PSScriptRoot\dropwire-tui\Cargo.toml"

function Get-TomlVersion($tomlPath) {
    $line = Get-Content $tomlPath | Where-Object { $_ -match '^version\s*=' } | Select-Object -First 1
    if ($line -match '"([^"]+)"') { return $Matches[1] }
    return $null
}

function Set-TomlVersion($tomlPath, $oldVer, $newVer) {
    $content = Get-Content $tomlPath -Raw
    $updated  = $content -replace ('version = "' + [regex]::Escape($oldVer) + '"'), ('version = "' + $newVer + '"')
    Set-Content $tomlPath $updated -NoNewline
}

function Get-NextVersion($ver, $type) {
    $p = $ver -split '\.'
    $major = [int]$p[0]; $minor = [int]$p[1]; $patch = [int]$p[2]
    switch ($type) {
        "major" { $major++; $minor = 0; $patch = 0 }
        "minor" { $minor++; $patch = 0 }
        "patch" { $patch++ }
    }
    return "$major.$minor.$patch"
}

function Show-Header {
    Clear-Host
    Write-Host ""
    Write-Host "  ██████╗ ██████╗  ██████╗ ██████╗ ██╗    ██╗██╗██████╗ ███████╗" -ForegroundColor Yellow
    Write-Host "  ██╔══██╗██╔══██╗██╔═══██╗██╔══██╗██║    ██║██║██╔══██╗██╔════╝" -ForegroundColor Yellow
    Write-Host "  ██║  ██║██████╔╝██║   ██║██████╔╝██║ █╗ ██║██║██████╔╝█████╗  " -ForegroundColor Yellow
    Write-Host "  ██║  ██║██╔══██╗██║   ██║██╔═══╝ ██║███╗██║██║██╔══██╗██╔══╝  " -ForegroundColor Yellow
    Write-Host "  ██████╔╝██║  ██║╚██████╔╝██║     ╚███╔███╔╝██║██║  ██║███████╗" -ForegroundColor Yellow
    Write-Host "  ╚═════╝ ╚═╝  ╚═╝ ╚═════╝ ╚═╝      ╚══╝╚══╝ ╚═╝╚═╝  ╚═╝╚══════╝" -ForegroundColor Yellow
    Write-Host ""
    Write-Host "  Version Bump Tool" -ForegroundColor Cyan
    Write-Host "  ─────────────────────────────────────────────" -ForegroundColor DarkGray
    Write-Host ""
}

function Ask-BumpType($label, $currentVer) {
    Write-Host "  [$label]  Current: " -NoNewline -ForegroundColor DarkGray
    Write-Host "v$currentVer" -ForegroundColor Green
    Write-Host ""
    Write-Host "    [1]  patch   v$currentVer  ->  v$(Get-NextVersion $currentVer 'patch')" -ForegroundColor White
    Write-Host "    [2]  minor   v$currentVer  ->  v$(Get-NextVersion $currentVer 'minor')" -ForegroundColor White
    Write-Host "    [3]  major   v$currentVer  ->  v$(Get-NextVersion $currentVer 'major')" -ForegroundColor White
    Write-Host "    [s]  skip" -ForegroundColor DarkGray
    Write-Host ""
    $choice = (Read-Host "  Choice").Trim().ToLower()
    switch ($choice) {
        "1" { return "patch" }
        "2" { return "minor" }
        "3" { return "major" }
        default { return "skip" }
    }
}

# ─── Main ──────────────────────────────────────────────────────────────────────

Show-Header

$cliVer = Get-TomlVersion $CLI_TOML
$tuiVer = Get-TomlVersion $TUI_TOML

Write-Host "  What do you want to bump?" -ForegroundColor Cyan
Write-Host ""
Write-Host "    [1]  CLI only      (dropwire v$cliVer)" -ForegroundColor White
Write-Host "    [2]  TUI only      (dropwire-tui v$tuiVer)" -ForegroundColor White
Write-Host "    [3]  Both" -ForegroundColor White
Write-Host "    [q]  Quit" -ForegroundColor DarkGray
Write-Host ""
$target = (Read-Host "  Choice").Trim().ToLower()

$bumpCli = $false
$bumpTui = $false
switch ($target) {
    "1" { $bumpCli = $true }
    "2" { $bumpTui = $true }
    "3" { $bumpCli = $true; $bumpTui = $true }
    "q" { Write-Host "`n  Aborted." -ForegroundColor DarkGray; exit 0 }
    default { Write-Host "`n  Invalid choice." -ForegroundColor Red; exit 1 }
}

Write-Host ""
Write-Host "  ─────────────────────────────────────────────" -ForegroundColor DarkGray

$tags      = @()
$commitMsg = "release:"

# CLI
if ($bumpCli) {
    Write-Host ""
    $bt = Ask-BumpType "CLI" $cliVer
    if ($bt -ne "skip") {
        $newVer = Get-NextVersion $cliVer $bt
        Set-TomlVersion $CLI_TOML $cliVer $newVer
        Write-Host "  OK  CLI: v$cliVer -> v$newVer" -ForegroundColor Green
        $tags      += "cli-v$newVer"
        $commitMsg += " cli-v$newVer"
    } else {
        Write-Host "  --  CLI skipped" -ForegroundColor DarkGray
    }
}

# TUI
if ($bumpTui) {
    Write-Host ""
    $bt = Ask-BumpType "TUI" $tuiVer
    if ($bt -ne "skip") {
        $newVer = Get-NextVersion $tuiVer $bt
        Set-TomlVersion $TUI_TOML $tuiVer $newVer
        Write-Host "  OK  TUI: v$tuiVer -> v$newVer" -ForegroundColor Green
        $tags      += "tui-v$newVer"
        $commitMsg += " tui-v$newVer"
    } else {
        Write-Host "  --  TUI skipped" -ForegroundColor DarkGray
    }
}

if ($tags.Count -eq 0) {
    Write-Host ""
    Write-Host "  Nothing bumped. Exiting." -ForegroundColor DarkGray
    exit 0
}

# ─── Git ───────────────────────────────────────────────────────────────────────
Write-Host ""
Write-Host "  ─────────────────────────────────────────────" -ForegroundColor DarkGray
Write-Host "  Staging..." -ForegroundColor Cyan
git -C $PSScriptRoot add .

Write-Host "  Committing: $commitMsg" -ForegroundColor Cyan
git -C $PSScriptRoot commit -m $commitMsg

Write-Host "  Pushing code -> main..." -ForegroundColor Cyan
git -C $PSScriptRoot push origin main

foreach ($tag in $tags) {
    Write-Host "  Tagging $tag..." -ForegroundColor Cyan
    git -C $PSScriptRoot tag $tag
    git -C $PSScriptRoot push origin $tag
    Write-Host "  OK  Tag pushed: $tag" -ForegroundColor Green
}

Write-Host ""
Write-Host "  ─────────────────────────────────────────────" -ForegroundColor DarkGray
Write-Host "  Done!  GitHub Actions will now build the release binaries." -ForegroundColor Yellow
Write-Host ""
