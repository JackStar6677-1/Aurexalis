# ADR 0003: RemoteFS Sin Montaje Del Sistema

## Estado

Aceptada.

## Contexto

Aurexalis tendra navegacion SFTP/FTP/FTPS tipo gestor de archivos. Montar unidades como RaiDrive resuelve acceso global del sistema, pero mete dependencias externas, latencia y riesgos de escritura fuera del control del navegador.

## Decision

RemoteFS vivira dentro de Aurexalis como explorador propio. No montara unidades del sistema operativo por defecto.

## Consecuencias

- El navegador controla permisos, confirmaciones y concurrencia.
- Se evita indexacion accidental de servidores completos.
- Las transferencias pueden tener cola, logs y rollback propios.
- La experiencia no reemplaza todos los usos de una unidad montada, pero es mas segura para operaciones desde el navegador.

