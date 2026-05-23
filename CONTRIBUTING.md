# Contribuir A Aurexalis

Aurexalis se trabaja como proyecto de navegador, no como demo. Cada cambio debe dejar una pieza verificable, documentada y compatible con la arquitectura modular.

## Flujo

1. Crear una rama corta y descriptiva.
2. Mantener el cambio acotado a un modulo o decision.
3. Actualizar tests y documentacion cuando cambie comportamiento.
4. Ejecutar la suite local posible.
5. Abrir PR con resumen, riesgos y pruebas.

## Commits

Usar Conventional Commits en espanol:

- `feat: agrega detector de perfiles Brave`
- `fix: corrige normalizacion de rutas remotas`
- `docs: documenta politica de seguridad`
- `test: cubre reglas de excepcion del bloqueador`

## Calidad Minima

Antes de fusionar:

```powershell
cargo fmt --all -- --check
cargo check --workspace --all-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
```

En Windows MSVC, `cargo test` requiere Visual Studio Build Tools con C++ linker.

## Reglas De Codigo

- `unsafe` esta prohibido salvo ADR explicita y revision dedicada.
- No se versionan perfiles, cookies, claves, tokens, dumps ni credenciales.
- Los modulos Rust deben exponer APIs testeables sin depender de Gecko.
- Los archivos `.uc.js` deben mantenerse pequenos y tolerantes a APIs ausentes.
- Cada integracion con Floorp/Gecko debe tener una nota de arquitectura antes de entrar al core.

