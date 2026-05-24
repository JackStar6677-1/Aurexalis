[CmdletBinding()]
param(
    [switch] $Deep
)

$ErrorActionPreference = "Stop"

function Write-Info([string] $Message) { Write-Host "[INFO] $Message" }
function Write-Success([string] $Message) { Write-Host "[SUCCESS] $Message" }

try {
    $repoRoot = Resolve-Path (Join-Path $PSScriptRoot "..")
    Set-Location $repoRoot

    Write-Info "Inicializando submodulo vendor/floorp en $repoRoot"

    if ($Deep) {
        git submodule update --init vendor/floorp
    } else {
        git submodule update --init --depth 1 vendor/floorp
    }

    if ($LASTEXITCODE -ne 0) {
        throw "git submodule update fallo con codigo $LASTEXITCODE"
    }

    Write-Success "Submodulo listo."
    & (Join-Path $PSScriptRoot "floorp-status.ps1")
} catch {
    Write-Error "[ERROR] $($_.Exception.Message)"
    exit 1
}
