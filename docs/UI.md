# UI Aurexalis

La UI inicial usa una mezcla visual de morado oscuro, rojo neon y dorado. El objetivo es conservar la energia de un navegador gaming sin copiar assets propietarios.

## Archivos

| Archivo | Rol |
|---|---|
| `browser/chrome/userChrome.css` | Tema base para tabs, urlbar, toolbar y sidebar |
| `browser/chrome/aurexalis-sidebar.uc.js` | Barra lateral vertical estilo GX |
| `browser/chrome/aurexalis-sound.uc.js` | Sonido reactivo para clicks, hover y tipeo |
| `browser/chrome/sounds/README.md` | Slots para sonidos locales |
| `browser/prefs/user.js` | Preferencias iniciales de perfil de prueba |

## Barra Lateral

La barra lateral incluye accesos iniciales:

- `AX`: home.
- `GX`: GX Corner.
- `RF`: Remote Files.
- `BM`: marcadores.
- `DL`: descargas.
- `IM`: importador de perfiles.
- `PW`: contrasenas.
- `ST`: ajustes.

`RF` e `IM` muestran paneles placeholder internos porque sus backends todavia estan en desarrollo. La intencion es que RemoteFS abra el explorador SFTP/FTP y que Importer ejecute el flujo local de migracion.

## Montaje En Firefox/Floorp

`userChrome.css` funciona con la preferencia:

```js
user_pref("toolkit.legacyUserProfileCustomizations.stylesheets", true);
```

Los archivos `.uc.js` requieren un loader de `userChrome.js` o los componentes legacy equivalentes de Floorp. No se copian sonidos propietarios; los audios locales deben colocarse manualmente en `browser/chrome/sounds/`.

