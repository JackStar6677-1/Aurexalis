# Reaplica marca Aurexalis en una instalacion existente (motor + perfil).
param(
    [Parameter(Mandatory = $true)][string] $InstallRoot
)

$ErrorActionPreference = "Stop"
$InstallRoot = (Resolve-Path $InstallRoot).Path
$engineDir = Join-Path $InstallRoot "Engine"
$icon = Join-Path $InstallRoot "aurexalis.ico"
$profile = Join-Path $InstallRoot "profiles\default"
$scriptBrand = Join-Path $PSScriptRoot "brand-engine.ps1"

if (-not (Test-Path $icon)) {
    $srcIco = Join-Path (Split-Path $PSScriptRoot -Parent) "assets\branding\aurexalis.ico"
    if (Test-Path $srcIco) { Copy-Item $srcIco $icon -Force }
    else { throw "Falta aurexalis.ico en $InstallRoot" }
}

$floorp = @(
    (Join-Path $engineDir "floorp.exe"),
    (Join-Path $engineDir "Ablaze Floorp\floorp.exe")
) | Where-Object { Test-Path $_ } | Select-Object -First 1

if (-not $floorp) {
    throw "No se encontro floorp.exe bajo $engineDir"
}

$dest = Join-Path $engineDir "aurexalis-browser.exe"
& $scriptBrand -SourceExe $floorp -DestExe $dest -IconPath $icon
Write-Host "[SUCCESS] Motor marcado: $dest"

if (Test-Path (Join-Path $PSScriptRoot "..\browser")) {
    $browserSrc = Join-Path (Split-Path $PSScriptRoot -Parent) "browser"
    Copy-Item (Join-Path $browserSrc "chrome") (Join-Path $profile "chrome") -Recurse -Force
    Copy-Item (Join-Path $browserSrc "prefs\user.js") (Join-Path $profile "user.js") -Force
    Write-Host "[SUCCESS] Chrome y user.js actualizados en el perfil"
}

$config = Join-Path $InstallRoot "config.json"
if (Test-Path $config) {
    $json = Get-Content $config -Raw | ConvertFrom-Json
    $json.browser = $dest
    $json | ConvertTo-Json -Depth 6 | Set-Content $config -Encoding UTF8
    Write-Host "[SUCCESS] config.json apunta a aurexalis-browser.exe"
}

Write-Host "[INFO] Cierra todas las ventanas del navegador y abre Aurexalis desde el acceso directo."
