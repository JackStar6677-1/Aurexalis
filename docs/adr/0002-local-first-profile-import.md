# ADR 0002: Importacion Local De Perfiles

## Estado

Aceptada.

## Contexto

El navegador necesita migrar cookies, historial, marcadores, favicons, preferencias y contrasenas desde Chrome, Brave y Opera. Estos datos son sensibles y pueden comprometer cuentas si se tratan mal.

## Decision

El importador sera local-first:

- No inicia sesion en cuentas externas.
- No envia datos a red.
- Requiere accion explicita del usuario.
- Usa APIs locales del sistema para descifrar cuando sea posible.
- Escribe a un staging auditable antes de modificar el perfil Aurexalis.

## Consecuencias

- La migracion es mas segura y trazable.
- Algunas claves no podran importarse si el sistema no permite descifrado local.
- La implementacion debe separar inventario, lectura, descifrado y escritura.

