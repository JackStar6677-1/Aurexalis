# Aurexalis Mobile (Android)

Navegador Aurexalis para Android basado en **GeckoView** (motor Gecko/Mozilla).

## Que incluye

- Home y ajustes Aurexalis embebidos (`browser/home`, `browser/settings`)
- Barra de URL, atras/adelante, recarga, inicio y ajustes
- Enlaces http/https y busqueda DuckDuckGo
- Tema morado / rojo / dorado
- **Bloqueador** via `ContentBlocking` (prefs `aurexalis.blocker.*`, nivel estandar/estricto/apagado)
- Pagina de ajustes interactiva (sonidos, UI, bloqueador) sincronizada con la app

## Build local

```powershell
.\tools\sync-mobile-assets.ps1   # copia home/settings al APK
.\tools\bootstrap-android.ps1
.\tools\setup-android-sdk.ps1    # solo la primera vez; descarga SDK sin Android Studio
$env:ANDROID_HOME = "$env:LOCALAPPDATA\Android\Sdk"
cd mobile\android
.\gradlew.bat assembleRelease
```

APK: `mobile/android/app/build/outputs/apk/release/app-release.apk`

En Linux/macOS:

```bash
./tools/sync-mobile-assets.sh
cd mobile/android && ./gradlew assembleRelease
```

## CI / Release

- Push a `main` → workflow **Android Build** genera artefacto APK
- Tag `v*` → release publica `Aurexalis-android-{version}-gecko.apk` junto a Windows y Linux

## Notas

- GeckoView carga assets embebidos con `resource://android/assets/...` (no `file:///android_asset/`, que es solo WebView).
- La importacion Chromium nativa en movil llega en una fase posterior (JNI/UniFFI).
- Al cambiar prefs del bloqueador, la app recrea el `GeckoRuntime` para aplicar ContentBlocking.
- El keystore de CI se genera en cada build de release; para Play Store usar secretos propios.
