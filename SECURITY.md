# Seguridad

Aurexalis toca superficies sensibles: perfiles de navegador, cookies, contrasenas locales y conexiones remotas. La politica base es **local-first, consentimiento explicito y cero exfiltracion**.

## Reportar Problemas

Si detectas una vulnerabilidad, no la publiques en issues. Reportala de forma privada al propietario del repositorio.

## Datos Que Nunca Deben Subirse

- Cookies, `Login Data`, `History`, `Local State`.
- `Preferences`, `Secure Preferences` cuando provengan de un perfil real.
- `.env`, tokens, claves privadas y credenciales SFTP/FTP.
- Dumps de SQLite o exportaciones de sesiones.
- Sonidos comerciales o assets propietarios.

## Reglas De Implementacion

- El importador de perfiles debe ser local y explicito.
- RemoteFS debe pedir confirmacion antes de subir, reemplazar o borrar.
- No debe haber indexacion recursiva profunda por defecto.
- Las credenciales remotas deben vivir en el almacen seguro del sistema o pedirse por sesion.
- Cualquier uso futuro de `unsafe` requiere ADR y revision separada.

