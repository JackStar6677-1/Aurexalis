# Testing

La base Rust de Aurexalis se valida con `cargo test` sobre todo el workspace.

## Comandos

```powershell
cargo check --workspace --all-features
cargo test --workspace --all-features
```

Si `rustfmt` esta instalado, tambien puedes ejecutar `cargo fmt --all -- --check`.

En Windows con toolchain `msvc`, `cargo test` requiere `link.exe` de Visual Studio Build Tools. Si no esta instalado, `cargo check` sigue validando compilacion de crates sin enlazar binarios de prueba.

## Cobertura Inicial

| Crate | Pruebas actuales |
|---|---|
| `aurexalis-core` | parsing de requests, hosts, third-party, errores de URL |
| `aurexalis-blocker` | reglas de dominio, excepciones, opciones por recurso |
| `aurexalis-importer` | descubrimiento de perfiles Chromium y Opera |
| `aurexalis-remotefs` | puertos por protocolo, normalizacion de rutas, bloqueo de traversal |

## CI

GitHub Actions ejecuta las pruebas completas en:

- `ubuntu-latest`
- `windows-latest`

Esto protege el proyecto aunque la maquina local no tenga Rust instalado todavia.
