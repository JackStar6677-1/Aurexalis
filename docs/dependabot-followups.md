# Seguimiento de Dependabot

## Rust production group

El PR de Dependabot `#6` agrupa cambios grandes en dependencias de produccion:

- `aes` 0.8 -> 0.9
- `pbkdf2` 0.12 -> 0.13
- `rusqlite` 0.32 -> 0.39
- `sha1` 0.10 -> 0.11
- `zip` 2 -> 8
- `rfd` 0.15 -> 0.17

Se deja pendiente para una pasada dedicada porque el salto de `aes` introduce
dos familias incompatibles de `cipher` en el grafo (`0.4` y `0.5`). Eso rompe el
descifrado CBC usado por `aurexalis-importer` y requiere migrar la capa crypto
con pruebas especificas, no solo actualizar el lockfile.

Plan sugerido:

1. Separar el update crypto (`aes`, `pbkdf2`, `sha1`) del update de UI/archivo (`rfd`, `zip`) y base de datos (`rusqlite`).
2. Migrar `linux_crypt.rs` a APIs compatibles con `cipher` 0.5 o mantener `aes` en 0.8 hasta que `aes-gcm` y `cbc` puedan alinearse.
3. Agregar pruebas de descifrado para los formatos soportados antes de fusionar el cambio.
