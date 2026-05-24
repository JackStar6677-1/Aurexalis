[CmdletBinding()]
param(
    [string] $RepoRoot = ""
)

$ErrorActionPreference = "Stop"

if ([string]::IsNullOrWhiteSpace($RepoRoot)) {
    $RepoRoot = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
}
$RepoRoot = (Resolve-Path $RepoRoot).Path
$androidRoot = Join-Path $RepoRoot "mobile\android"

function Write-Info([string] $Message) { Write-Host "[INFO] $Message" }

Write-Info "Bootstrap Android Aurexalis"

& (Join-Path $RepoRoot "tools\sync-mobile-assets.ps1") -RepoRoot $RepoRoot

# version.properties desde aurexalis-shell
$cargoToml = Join-Path $RepoRoot "crates\aurexalis-shell\Cargo.toml"
$version = "0.2.3"
if (Test-Path $cargoToml) {
    $match = Select-String -Path $cargoToml -Pattern '^version\s*=\s*"(.+)"' | Select-Object -First 1
    if ($match) { $version = $match.Matches[0].Groups[1].Value }
}
$parts = $version.Split(".")
$major = [int]$parts[0]
$minor = if ($parts.Length -gt 1) { [int]$parts[1] } else { 0 }
$patch = if ($parts.Length -gt 2) { [int]$parts[2] } else { 0 }
$code = ($major * 10000) + ($minor * 100) + $patch
$versionFile = Join-Path $androidRoot "version.properties"
@(
    "versionName=$version"
    "versionCode=$code"
) | Set-Content -Path $versionFile -Encoding ASCII

Write-Info "versionName=$version versionCode=$code"

# Keystore de sideload (debug interno del proyecto, no produccion Play Store)
$keystoreDir = Join-Path $androidRoot "keystore"
$keystore = Join-Path $keystoreDir "aurexalis-release.jks"
New-Item -ItemType Directory -Force -Path $keystoreDir | Out-Null
if (-not (Test-Path $keystore)) {
    $keytool = Get-Command keytool -ErrorAction SilentlyContinue
    if (-not $keytool) {
        $javaHome = $env:JAVA_HOME
        if ($javaHome) { $keytool = Join-Path $javaHome "bin\keytool.exe" }
    }
    if ($keytool -and (Test-Path $keytool)) {
        Write-Info "Generando keystore de release local..."
        & $keytool -genkeypair -v `
            -keystore $keystore `
            -storepass aurexalis `
            -alias aurexalis `
            -keypass aurexalis `
            -keyalg RSA `
            -keysize 2048 `
            -validity 10000 `
            -dname "CN=Aurexalis Mobile, OU=Mobile, O=Aurexalis, L=Local, S=Local, C=ES"
    } else {
        Write-Warning "keytool no encontrado; CI generara el keystore."
    }
}

# Gradle wrapper
$gradleVer = "8.9"
$gradleZip = Join-Path $env:TEMP "gradle-$gradleVer-bin.zip"
$gradleHome = Join-Path $env:TEMP "gradle-$gradleVer"
if (-not (Test-Path "$gradleHome\bin\gradle.bat")) {
    if (-not (Test-Path $gradleZip)) {
        Invoke-WebRequest -Uri "https://services.gradle.org/distributions/gradle-$gradleVer-bin.zip" -OutFile $gradleZip
    }
    Expand-Archive -Path $gradleZip -DestinationPath (Split-Path $gradleHome) -Force
}
Push-Location $androidRoot
& "$gradleHome\bin\gradle.bat" wrapper --gradle-version $gradleVer
Pop-Location

Write-Host "[SUCCESS] Bootstrap Android completado."
