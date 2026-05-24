[CmdletBinding()]
param(
    [string] $RepoRoot = ""
)

$ErrorActionPreference = "Stop"

if ([string]::IsNullOrWhiteSpace($RepoRoot)) {
    $RepoRoot = Split-Path -Parent $PSScriptRoot
    if ([string]::IsNullOrWhiteSpace($RepoRoot)) {
        $RepoRoot = (Get-Location).Path
    }
}
$RepoRoot = (Resolve-Path $RepoRoot).Path

function Assert-File([string] $Path, [string] $Label) {
    if (-not (Test-Path -LiteralPath $Path)) {
        throw "Falta $Label : $Path"
    }
}

function Assert-Contains([string] $Path, [string] $Needle, [string] $Label) {
    $text = Get-Content -Raw -LiteralPath $Path
    if ($text -notmatch [regex]::Escape($Needle)) {
        throw "En $Label no aparece: $Needle"
    }
}

Write-Host "[INFO] Verificando pack Aurexalis en $RepoRoot"

$required = @(
    "browser/chrome/userChrome.js",
    "browser/chrome/userChrome.css",
    "browser/chrome/aurexalis-00-core.uc.js",
    "browser/chrome/aurexalis-01-brand.uc.js",
    "browser/chrome/aurexalis-02-blocker.uc.js",
    "browser/chrome/aurexalis-03-sound.uc.js",
    "browser/chrome/aurexalis-04-settings-panel.uc.js",
    "browser/chrome/aurexalis-05-sidebar.uc.js",
    "browser/chrome/aurexalis-06-settings-inject.uc.js",
    "browser/home/index.html",
    "browser/settings/index.html",
    "browser/prefs/user.js"
)

foreach ($rel in $required) {
    Assert-File (Join-Path $RepoRoot $rel) $rel
}

Assert-Contains (Join-Path $RepoRoot "browser/prefs/user.js") "aurexalis.sounds.enabled" "user.js"
Assert-Contains (Join-Path $RepoRoot "browser/prefs/user.js") "aurexalis.blocker.enabled" "user.js"
Assert-Contains (Join-Path $RepoRoot "browser/prefs/user.js") "disableFloorpStart" "user.js"
Assert-Contains (Join-Path $RepoRoot "browser/chrome/userChrome.js") "aurexalis-02-blocker.uc.js" "userChrome.js"

$ucGlob = Join-Path $RepoRoot "browser/chrome/aurexalis-*.uc.js"
$ucOrder = Get-ChildItem -Path $ucGlob | Sort-Object Name
$expectedOrder = @(
    "aurexalis-00-core.uc.js",
    "aurexalis-01-brand.uc.js",
    "aurexalis-02-blocker.uc.js",
    "aurexalis-03-sound.uc.js",
    "aurexalis-04-settings-panel.uc.js",
    "aurexalis-05-sidebar.uc.js",
    "aurexalis-06-settings-inject.uc.js"
)
$actual = $ucOrder | ForEach-Object { $_.Name }
if (($actual -join ",") -ne ($expectedOrder -join ",")) {
    throw "Orden uc.js invalido: $($actual -join ', ')"
}

Write-Host "[SUCCESS] Pack Aurexalis integrado y verificado."
exit 0
