# Roadmap Profesional

Este roadmap reemplaza la idea de MVP suelto por fases de producto.

## Fase 0: Base De Ingenieria

- Workspace Rust estable.
- CI con check, clippy, fmt y tests.
- Documentacion de arquitectura, seguridad y calidad.
- UI chrome experimental aislada.

## Fase 1: Shell Aurexalis

- Perfil dedicado Firefox/Floorp.
- Tema `userChrome.css` mantenible.
- Barra lateral funcional con paneles internos.
- Sonido reactivo con assets locales.
- Preferencias reproducibles.

## Fase 2: Importador Local

- Deteccion robusta de Chrome, Brave y Opera.
- Lectura de Bookmarks, Preferences, Secure Preferences y Local State.
- Lectura de Cookies, Login Data, History y Favicons con SQLite.
- Descifrado Windows DPAPI.
- Descifrado AES-GCM Chromium con llave local.
- Descifrado Linux Secret Service/KWallet cuando este disponible.
- Exportacion hacia formato intermedio auditable.

## Fase 3: Bloqueador Nativo

- Integracion real de `adblock-rust` en crate aislado.
- Benchmarks de matching.
- Listas configurables.
- Politica de bloqueo antes del render.
- Adaptador Gecko/Floorp.

## Fase 4: RemoteFS

- Cliente SFTP.
- Cliente FTP/FTPS.
- Explorador de archivos integrado.
- Cola de transferencias.
- Confirmaciones para operaciones destructivas.
- Credenciales en almacen seguro.

## Fase 5: Gecko/Floorp Core

- Submodulo Floorp fijado en `vendor/floorp`.
- Analisis de parches Floorp documentado.
- Mapa inicial de soporte Chrome Web Store.
- Mapa inicial de build system y empaquetado.
- Port Aurexalis de soporte Chrome Web Store.
- Empaquetado reproducible.
- Canal de builds propio.
