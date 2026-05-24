# Aplica icono y metadatos Windows a una copia del motor Gecko (Floorp) como Aurexalis.
param(
    [Parameter(Mandatory = $true)][string] $SourceExe,
    [Parameter(Mandatory = $true)][string] $DestExe,
    [Parameter(Mandatory = $true)][string] $IconPath,
    [string] $CacheDir = ""
)

$ErrorActionPreference = "Stop"

function Write-Info([string] $Message) { Write-Host "[INFO] $Message" }
function Write-Warn([string] $Message) { Write-Warning "[WARN] $Message" }

if (-not (Test-Path -LiteralPath $SourceExe)) {
    throw "Motor origen no encontrado: $SourceExe"
}
if (-not (Test-Path -LiteralPath $IconPath)) {
    throw "Icono no encontrado: $IconPath"
}

$destDir = Split-Path -Parent $DestExe
if ($destDir -and -not (Test-Path $destDir)) {
    New-Item -ItemType Directory -Force -Path $destDir | Out-Null
}

Copy-Item -LiteralPath $SourceExe -Destination $DestExe -Force
Write-Info "Copia del motor: $DestExe"

if ([string]::IsNullOrWhiteSpace($CacheDir)) {
    $CacheDir = Join-Path $env:TEMP "aurexalis-brand-cache"
}
New-Item -ItemType Directory -Force -Path $CacheDir | Out-Null

$rceditLocal = Join-Path $CacheDir "rcedit-x64.exe"
$rcedit = $null
if (Test-Path $rceditLocal) {
    $rcedit = $rceditLocal
} elseif (Get-Command rcedit-x64.exe -ErrorAction SilentlyContinue) {
    $rcedit = (Get-Command rcedit-x64.exe).Source
} elseif (Get-Command rcedit.exe -ErrorAction SilentlyContinue) {
    $rcedit = (Get-Command rcedit.exe).Source
}

if (-not $rcedit) {
    $rcedit = $rceditLocal
    $zipUrl = "https://github.com/electron/rcedit/releases/download/v2.0.0/rcedit-x64.exe"
    Write-Info "Descargando rcedit..."
    Invoke-WebRequest -Uri $zipUrl -OutFile $rcedit -UseBasicParsing
}

if (-not (Test-Path $rcedit)) {
    Write-Warn "rcedit no disponible; el .exe conservara icono y nombre Floorp en el Administrador de tareas."
    exit 0
}

$icon = (Resolve-Path $DestExe).Path
$args = @(
    $icon,
    "--set-icon", (Resolve-Path $IconPath).Path,
    "--set-version-string", "ProductName", "Aurexalis",
    "--set-version-string", "FileDescription", "Aurexalis Browser",
    "--set-version-string", "OriginalFilename", "aurexalis-browser.exe",
    "--set-version-string", "InternalName", "aurexalis-browser",
    "--set-version-string", "CompanyName", "Aurexalis Project",
    "--set-file-version", "1.0.0.0",
    "--set-product-version", "1.0.0.0"
)

& $rcedit @args
if ($LASTEXITCODE -ne 0) {
    Write-Warn "rcedit termino con codigo $LASTEXITCODE; revisa el binario manualmente."
    exit 0
}

Write-Info "Motor renombrado en metadatos e icono: Aurexalis"
