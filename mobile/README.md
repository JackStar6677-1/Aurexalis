# Aurexalis Mobile (Android)

Navegador Aurexalis para Android basado en **GeckoView** (motor Gecko/Mozilla).

## Que incluye

- Home y ajustes Aurexalis embebidos (`browser/home`, `browser/settings`)
- Barra de URL, atras/adelante, recarga, inicio y ajustes
- Enlaces http/https y busqueda DuckDuckGo
- Tema morado / rojo / dorado

## Build local (automatico)

```powershell
.\tools\bootstrap-android.ps1
.\tools\setup-android-sdk.ps1   # solo la primera vez; descarga SDK sin Android Studio
$env:ANDROID_HOME = "$env:LOCALAPPDATA\Android\Sdk"
cd mobile\android
.\gradlew.bat assembleRelease
```

APK: `mobile/android/app/build/outputs/apk/release/app-release.apk`

## CI / Release

- Push a `main` → workflow `Android Build` genera artefacto APK
- Tag `v*` → release publica `Aurexalis-android-{version}-universal.apk` junto a los binarios Windows

## Notas

- La importacion Chromium nativa en movil llega en una fase posterior (JNI/UniFFI).
- El keystore de CI se genera en cada build de release; para Play Store usar secretos propios.
