# Seguridad Y Datos Sensibles

Aurexalis puede tocar datos delicados: cookies, historial, contrasenas guardadas y conexiones remotas. Por eso el diseno base es local-first y con consentimiento.

## Nunca Versionar

- `.env`
- perfiles de navegador
- `Cookies`
- `Login Data`
- `History`
- `Local State`
- claves privadas
- dumps de SQLite
- archivos exportados de sesiones remotas

## Importacion De Perfiles

El importador no debe iniciar sesion en cuentas ni enviar datos fuera del equipo. Su trabajo es detectar perfiles locales, copiar bases SQLite a staging seguro, descifrar solo usando APIs locales del sistema y migrar al perfil de Aurexalis.

## Conexiones Remotas

El modulo RemoteFS debe pedir confirmacion antes de borrar, reemplazar o subir archivos. Las credenciales deben guardarse con el mecanismo seguro del sistema operativo o pedirse por sesion.

