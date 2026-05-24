# Importacion Chromium en Android (stub v0.5)

## Estado

No implementado en runtime. Este documento fija la arquitectura minima para la fase nativa.

## Objetivo

Importar marcadores, historial, cookies y (opcional) contrasenas desde Chrome/Android WebView hacia el perfil GeckoView de Aurexalis, **sin red**, con consentimiento en UI.

## Capas propuestas

```
┌─────────────────────────────────────┐
│  Kotlin UI (Settings → Import)      │
└──────────────┬──────────────────────┘
               │ JNI / UniFFI
┌──────────────▼──────────────────────┐
│  aurexalis_importer (Rust, no_std off)│
│  - discover Android Chrome paths    │
│  - read + decrypt (Keystore stub)   │
│  - gecko_write apply surfaces       │
└──────────────┬──────────────────────┘
               │
┌──────────────▼──────────────────────┐
│  GeckoView profile dir (app files)  │
└─────────────────────────────────────┘
```

## Placeholder en repo

- `mobile/android/app/src/main/java/.../import/ImportBridge.kt` — stub que devuelve `NOT_IMPLEMENTED`.
- Crate compartido: reutilizar `aurexalis-importer` via **UniFFI** (preferido) o JNI fino sobre `cdylib`.

## Descifrado Android (diferido)

Chromium en Android usa **Android Keystore** + clave en `Local State`. Requiere:

1. Leer `/data/data/com.android.chrome/...` (solo con permisos root o backup adb autorizado).
2. Binder/Keystore desde Rust (crate `android-keystore` o bridge Kotlin).

No over-engineer en v0.5: UI muestra mensaje y enlace a esta doc.

## Verificacion futura

1. Export audit en desktop del mismo Google account (opcional).
2. En dispositivo: Settings → Import → list profiles (stub → lista vacia OK).
3. Cuando exista bridge: apply cookies-only con navegador cerrado.
