# Parches CWS Aurexalis (stub)

Coloca aqui parches contra `vendor/floorp` cuando el submodulo este inicializado.

## Objetivos del fork (futuro)

1. Renombrar strings i18n `addToFloorp` → **Add to Aurexalis** / **Anadir a Aurexalis**.
2. Opcional: duplicar topic `floorp-chrome-web-store-install-started` como `aurexalis-chrome-web-store-install-started` manteniendo alias en transicion.
3. Alinear `MOZ_APP_DISPLAYNAME` y manifiestos con `docs/REBRAND.md`.

## Aplicacion (planificado)

```powershell
# Ejemplo futuro — no implementado aun
.\tools\aurexalis-build.ps1 -Phase before-mach
```

Hasta entonces, el rebranding de prompts vive en `browser/chrome/aurexalis-07-cws-brand.uc.js` (capa perfil).

## Referencia upstream

Revision documentada en commits del repo padre al actualizar `vendor/floorp`.

Archivos fuente Floorp:

- `browser-features/chrome/common/addons/`
- `browser-features/modules/modules/chrome-web-store/`
- `browser-features/modules/actors/NRChromeWebStoreChild.sys.mts`
- `browser-features/modules/actors/NRChromeWebStoreParent.sys.mts`
