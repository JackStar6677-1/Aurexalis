[CmdletBinding()]
param(
    [string] $RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path,
    [string] $CargoTarget = "x86_64-pc-windows-msvc",
    [string] $OutDir = ""
)

$ErrorActionPreference = "Stop"

function Write-Info([string] $Message) { Write-Host "[INFO] $Message" }
function Write-Success([string] $Message) { Write-Host "[SUCCESS] $Message" }

try {
    if ([string]::IsNullOrWhiteSpace($OutDir)) {
        $OutDir = Join-Path $RepoRoot "dist\runtime"
    }

    $cargo = Join-Path $env:USERPROFILE ".rustup\toolchains\stable-x86_64-pc-windows-msvc\bin\cargo.exe"
    if (-not (Test-Path $cargo)) { $cargo = "cargo" }

    $exe = Join-Path $RepoRoot "target\$CargoTarget\release\aurexalis.exe"
    Write-Info "Compilando aurexalis-shell..."
    Push-Location $RepoRoot
    & $cargo build --release -p aurexalis-shell --target $CargoTarget
    Pop-Location

    if (-not (Test-Path $exe)) {
        throw "No se encontro $exe"
    }

    $stage = Join-Path $env:TEMP "aurexalis-runtime-stage"
    if (Test-Path $stage) { Remove-Item $stage -Recurse -Force }
    New-Item -ItemType Directory -Force -Path $stage | Out-Null

    Copy-Item $exe (Join-Path $stage "aurexalis.exe")
    Copy-Item (Join-Path $RepoRoot "browser") (Join-Path $stage "browser") -Recurse
    $ico = Join-Path $RepoRoot "assets\branding\aurexalis.ico"
    if (-not (Test-Path $ico)) {
        python (Join-Path $RepoRoot "tools\gen_installer_icon.py")
    }
    if (Test-Path $ico) {
        Copy-Item $ico (Join-Path $stage "aurexalis.ico")
    }
    New-Item -ItemType Directory -Force -Path (Join-Path $stage "profiles\default") | Out-Null

    @"
Aurexalis runtime pack.
Incluye shell, chrome UI y prefs. El motor Gecko/Floorp se instala aparte.
"@ | Set-Content -Path (Join-Path $stage "README.txt") -Encoding UTF8

    New-Item -ItemType Directory -Force -Path $OutDir | Out-Null
    $zipPath = Join-Path $OutDir "aurexalis-runtime-windows-x86_64.zip"
    if (Test-Path $zipPath) { Remove-Item $zipPath -Force }
    Compress-Archive -Path (Join-Path $stage "*") -DestinationPath $zipPath -Force
    Remove-Item $stage -Recurse -Force

    Write-Success "Runtime empaquetado: $zipPath"
} catch {
    Write-Error "[ERROR] $($_.Exception.Message)"
    exit 1
}
