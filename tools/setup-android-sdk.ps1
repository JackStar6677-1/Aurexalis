[CmdletBinding()]
param(
    [string] $SdkRoot = (Join-Path $env:LOCALAPPDATA "Android\Sdk")
)

$ErrorActionPreference = "Stop"

function Write-Info([string] $Message) { Write-Host "[INFO] $Message" }

if ($env:ANDROID_HOME -and (Test-Path $env:ANDROID_HOME)) {
    Write-Info "ANDROID_HOME ya configurado: $($env:ANDROID_HOME)"
    exit 0
}

$cmdlineRoot = Join-Path $env:LOCALAPPDATA "Android\cmdline-tools"
$cmdlineLatest = Join-Path $cmdlineRoot "latest"
$sdkmanager = Join-Path $cmdlineLatest "bin\sdkmanager.bat"

if (-not (Test-Path $sdkmanager)) {
    Write-Info "Descargando Android command-line tools..."
    New-Item -ItemType Directory -Force -Path $cmdlineRoot | Out-Null
    $zip = Join-Path $env:TEMP "commandlinetools-win-latest.zip"
    Invoke-WebRequest -Uri "https://dl.google.com/android/repository/commandlinetools-win-11076708_latest.zip" -OutFile $zip
    $extract = Join-Path $env:TEMP "android-cmdline-extract"
    if (Test-Path $extract) { Remove-Item $extract -Recurse -Force }
    Expand-Archive -Path $zip -DestinationPath $extract -Force
    New-Item -ItemType Directory -Force -Path $cmdlineLatest | Out-Null
    Copy-Item (Join-Path $extract "cmdline-tools\*") $cmdlineLatest -Recurse -Force
}

Write-Info "Aceptando licencias Android SDK..."
$yes | & $sdkmanager --sdk_root=$SdkRoot --licenses | Out-Null

Write-Info "Instalando platform-tools, platforms;android-34, build-tools;34.0.0..."
$yes | & $sdkmanager --sdk_root=$SdkRoot "platform-tools" "platforms;android-35" "build-tools;35.0.0"

Write-Info "SDK listo en $SdkRoot"
Write-Host "[SUCCESS] Android SDK instalado. ANDROID_HOME=$SdkRoot"
