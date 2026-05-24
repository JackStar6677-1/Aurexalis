# UI Aurexalis

La UI usa una mezcla visual de morado oscuro, rojo neon y dorado. El objetivo es conservar la energia de un navegador gaming sin copiar assets propietarios.

## Archivos

| Archivo | Rol |
|---|---|
| `browser/chrome/userChrome.css` | Tema base para tabs, urlbar, toolbar y sidebar |
| `browser/chrome/userChrome.js` | Loader ordenado de modulos `.uc.js` |
| `browser/chrome/aurexalis-00-core.uc.js` | Preferencias, launcher y utilidades |
| `browser/chrome/aurexalis-01-brand.uc.js` | Identidad visual Aurexalis |
| `browser/chrome/aurexalis-02-blocker.uc.js` | Bloqueador Gecko ETP |
| `browser/chrome/aurexalis-03-sound.uc.js` | Sonido reactivo (click, hover, teclado) |
| `browser/chrome/aurexalis-04-settings-panel.uc.js` | Panel **ST** en sidebar |
| `browser/chrome/aurexalis-05-sidebar.uc.js` | Barra lateral vertical tipo GX |
| `browser/chrome/aurexalis-06-settings-inject.uc.js` | Puente prefs en pagina de ajustes |
| `browser/settings/` | Pagina HTML de ajustes interactiva |
| `browser/chrome/sounds/README.md` | Slots para sonidos locales OGG |
| `browser/prefs/user.js` | Preferencias iniciales del perfil |

## Barra Lateral

| Boton | Funcion |
|---|---|
| `AX` | Home Aurexalis |
| `GX` | GX Corner |
| `RF` | Archivos remotos (panel + backend Rust) |
| `BM` | Marcadores |
| `DL` | Descargas |
| `IM` | Importador Chromium local |
| `BL` | Bloqueador on/off rapido |
| `PW` | Contrasenas |
| `ST` | Ajustes Aurexalis (panel completo) |

`RF` e `IM` muestran paneles con acciones hacia RemoteFS e importacion local. El importador ejecuta el shell Rust (`aurexalis import audit`) desde el panel o desde la pagina de ajustes.

## Montaje En Firefox/Floorp

`userChrome.css` funciona con la preferencia:

```js
user_pref("toolkit.legacyUserProfileCustomizations.stylesheets", true);
```

Los archivos `.uc.js` se cargan via `userChrome.js` en el perfil. No se copian sonidos propietarios; los audios locales deben colocarse en `browser/chrome/sounds/`.

Verificacion estructural del pack:

```powershell
.\tools\verify-browser-pack.ps1
```
