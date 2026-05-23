# Calidad De Ingenieria

Aurexalis se desarrolla bajo criterios de proyecto profesional.

## Gates

| Gate | Objetivo |
|---|---|
| `cargo fmt` | formato consistente |
| `cargo check` | compilacion rapida del workspace |
| `cargo clippy -D warnings` | detectar deuda temprana |
| `cargo test` | validar comportamiento |
| revision de secretos | evitar fuga de datos sensibles |

## Politica Rust

- `unsafe_code = forbid`.
- Crates pequenos, con responsabilidad unica.
- APIs testeables fuera del navegador.
- Dependencias externas justificadas por necesidad real.
- Errores explicitos con `Result`, no panics en codigo de produccion.

## Politica UI

- `userChrome.css` define tokens de diseno antes de reglas concretas.
- `.uc.js` debe fallar de forma silenciosa y loguear claro.
- No se incluyen assets propietarios.
- La barra lateral es un shell de integracion, no el backend de los modulos.

## Politica De Datos

- El importador solo inventaria y migra datos locales con accion del usuario.
- RemoteFS no monta unidades ni cachea arboles completos por defecto.
- Los staging temporales deben poder limpiarse de forma deterministica.

