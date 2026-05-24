[CmdletBinding()]
param(
    [string] $RepoRoot = ""
)

$ErrorActionPreference = "Stop"

if ([string]::IsNullOrWhiteSpace($RepoRoot)) {
    $RepoRoot = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
    if (-not (Test-Path (Join-Path $RepoRoot "browser"))) {
        $RepoRoot = Split-Path -Parent $PSScriptRoot
    }
}
$RepoRoot = (Resolve-Path $RepoRoot).Path

function Write-Info([string] $Message) { Write-Host "[INFO] $Message" }
function Write-Success([string] $Message) { Write-Host "[SUCCESS] $Message" }

$assetsRoot = Join-Path $RepoRoot "mobile\android\app\src\main\assets\aurexalis"
$homeDst = Join-Path $assetsRoot "home"
$settingsDst = Join-Path $assetsRoot "settings"

Write-Info "Sincronizando assets web a $assetsRoot"

if (Test-Path $assetsRoot) {
    Remove-Item $assetsRoot -Recurse -Force
}

New-Item -ItemType Directory -Force -Path $homeDst | Out-Null
New-Item -ItemType Directory -Force -Path $settingsDst | Out-Null

Copy-Item (Join-Path $RepoRoot "browser\home\*") $homeDst -Recurse -Force
Copy-Item (Join-Path $RepoRoot "browser\settings\*") $settingsDst -Recurse -Force

Write-Success "Assets moviles sincronizados."
