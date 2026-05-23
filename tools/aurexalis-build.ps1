[CmdletBinding()]
param(
    [ValidateSet("check", "build", "run")]
    [string] $Mode = "check",
    [string] $BrowserBinary = $env:AUREXALIS_BROWSER
)

$ErrorActionPreference = "Stop"

function Write-Info {
    param([string] $Message)
    Write-Host "[INFO] $Message"
}

function Write-Success {
    param([string] $Message)
    Write-Host "[SUCCESS] $Message"
}

try {
    $repoRoot = Resolve-Path (Join-Path $PSScriptRoot "..")
    $cargo = Join-Path $env:USERPROFILE ".rustup\toolchains\stable-x86_64-pc-windows-msvc\bin\cargo.exe"
    if (-not (Test-Path $cargo)) {
        $cargo = "cargo"
    }

    Write-Info "Repo: $repoRoot"
    Write-Info "Modo: $Mode"

    Push-Location $repoRoot
    try {
        if ($Mode -eq "check") {
            & $cargo check --workspace --all-features
            Write-Success "Workspace validado"
            return
        }

        & $cargo build --workspace --all-features
        Write-Success "Binarios Rust compilados"

        if ($Mode -eq "run") {
            if ([string]::IsNullOrWhiteSpace($BrowserBinary)) {
                throw "Define AUREXALIS_BROWSER o pasa -BrowserBinary con Firefox/Floorp."
            }
            & (Join-Path $repoRoot "target\debug\aurexalis.exe") launch $BrowserBinary
        }
    } finally {
        Pop-Location
    }
} catch {
    Write-Error "[ERROR] $($_.Exception.Message)"
    exit 1
}
