# Roadmap Profesional

Roadmap de producto alineado con **v0.4.0**. Para descargas y estado rapido ver el [README](../README.md).

## Fase 0: Base De Ingenieria — Hecho

- [x] Workspace Rust estable (`aurexalis-shell`, `-installer`, `-importer`, `-blocker`, `-remotefs`, `-core`).
- [x] CI: `rust.yml` (test/clippy/fmt + verify pack), `android-build.yml`, `release.yml`.
- [x] Documentacion de arquitectura, seguridad, build y calidad.
- [x] Identidad visual Aurexalis (banner, paleta, `userChrome.css`).
- [x] Submodulo Floorp en `vendor/floorp` con revision fijada.

## Fase 1: Shell Y UI Aurexalis — Hecho

- [x] Perfil dedicado con `browser/prefs/user.js` y chrome copiado al instalar.
- [x] Tema `userChrome.css` morado / rojo / dorado.
- [x] Modulos `.uc.js` `00`–`06` cargados por `userChrome.js`.
- [x] Barra lateral (AX, GX, RF, BM, DL, IM, BL, PW, ST).
- [x] Sonido reactivo (`aurexalis-03-sound.uc.js`) con prefs `aurexalis.sounds.*`.
- [x] Home Aurexalis (`browser/home/`).
- [x] Panel **ST** + pagina `browser/settings/` interactiva.
- [x] Bloqueador UI: prefs `aurexalis.blocker.*`, boton **BL**, ETP desktop.

## Fase 1b: Multi-Plataforma — v0.3.0

- [x] **Windows:** `Aurexalis-Setup-x86_64.exe`, CLI, runtime zip.
- [x] **Android:** APK GeckoView (`mobile/android/`), ContentBlocking, ajustes embebidos.
- [x] **Linux:** `.deb`, `.rpm`, `.pkg.tar.zst`, tarball (`tools/package-linux.sh`).
- [x] GitHub Release automatico en tag `v*` con los tres targets.

## Fase 2: Importador Local — Parcial

- [x] Deteccion de perfiles Chrome, Brave y Opera (Windows).
- [x] Lectura SQLite/JSON Chromium en crate `aurexalis-importer`.
- [x] Exportacion auditable (`aurexalis import audit`) desde shell, sidebar e instalador.
- [x] Opcion de contrasenas con consentimiento (`--passwords`).
- [x] Escritura directa al perfil Gecko/Floorp (marcadores e historial via `import apply`).
- [ ] Cookies y contrasenas en perfil Gecko.
- [ ] Descifrado Linux: Secret Service / KWallet.
- [ ] Importacion nativa Android (JNI / UniFFI).

## Fase 3: Bloqueador — Parcial

- [x] Crate `aurexalis-blocker` con `adblock-rust` (tests y PoC aislado).
- [x] Integracion UI: Gecko ETP en desktop, ContentBlocking en Android.
- [x] Niveles standard / strict / off desde ajustes y panel **ST**.
- [x] CLI `blocker check` y `blocker sync-lists` (adblock-rust + listas embebidas).
- [ ] Hook al pipeline de requests Gecko antes del render.
- [ ] Listas uBlock/ABP descargables desde URLs.
- [ ] Benchmarks de matching en CI.

## Fase 4: RemoteFS — Diseno + backend

- [x] Crate `aurexalis-remotefs` con cola y backend local testeable.
- [x] Cliente SFTP operativo via CLI (`aurexalis remotefs list|get`).
- [x] Panel **RF** con ayuda SFTP y acceso a descargas.
- [ ] Cliente FTP/FTPS.
- [ ] Explorador integrado estilo gestor de archivos.
- [ ] Credenciales en almacen seguro del SO.

## Fase 5: Gecko / Floorp Core — Parcial

- [x] Submodulo Floorp fijado y documentado (`docs/FLOORP_INTEGRATION.md`).
- [x] Mapa de build system y empaquetado (`docs/BUILD_AND_RELEASE.md`).
- [x] Shell Rust que lanza Floorp/Firefox con perfil Aurexalis.
- [ ] Port de soporte **Chrome Web Store** con branding Aurexalis.
- [ ] Build reproducible del nucleo Gecko propio (sin depender de instalador externo).
- [ ] Canal de actualizaciones propio.

## Proximas versiones (orientativo)

| Version | Objetivo principal |
|---|---|
| **v0.4** | `import apply`, blocker CLI, RemoteFS SFTP — **publicado** |
| **v0.5** | Hook adblock Gecko + listas remotas + explorador RemoteFS |
| **v1.0** | Nucleo empaquetado end-to-end sin Floorp externo obligatorio |
