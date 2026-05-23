[CmdletBinding()]
param()

$ErrorActionPreference = "Stop"

function Write-Info {
    param([string] $Message)
    Write-Host "[INFO] $Message"
}

function Write-Success {
    param([string] $Message)
    Write-Host "[SUCCESS] $Message"
}

function Write-Warn {
    param([string] $Message)
    Write-Host "[WARN] $Message"
}

try {
    $repoRoot = Resolve-Path (Join-Path $PSScriptRoot "..")
    $floorpPath = Join-Path $repoRoot "vendor\floorp"

    Write-Info "Repo Aurexalis: $repoRoot"

    if (-not (Test-Path $floorpPath)) {
        throw "No existe vendor\floorp. Ejecuta: git submodule update --init --depth 1 vendor/floorp"
    }

    $gitDir = Join-Path $floorpPath ".git"
    if (-not (Test-Path $gitDir)) {
        throw "vendor\floorp existe, pero no parece un submodulo Git inicializado."
    }

    $commit = (& git -C $floorpPath rev-parse HEAD).Trim()
    $branch = (& git -C $floorpPath rev-parse --abbrev-ref HEAD).Trim()
    $remote = (& git -C $floorpPath config --get remote.origin.url).Trim()

    Write-Success "Floorp revision: $commit"
    Write-Info "Floorp branch: $branch"
    Write-Info "Floorp remote: $remote"

    $requiredFiles = @(
        "deno.json",
        "tools\feles-build.ts",
        "browser-features\chrome\common\addons\index.ts",
        "browser-features\chrome\common\addons\observer.ts",
        "browser-features\chrome\common\addons\notification-customizer.ts",
        "browser-features\chrome\common\addons\types.ts"
    )

    foreach ($relative in $requiredFiles) {
        $target = Join-Path $floorpPath $relative
        if (Test-Path $target) {
            Write-Success "OK $relative"
        } else {
            Write-Warn "Falta $relative"
        }
    }
} catch {
    Write-Error "[ERROR] $($_.Exception.Message)"
    exit 1
}
