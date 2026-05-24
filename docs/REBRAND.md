# Rebrand Aurexalis (capa perfil + motor Windows)

## Que corrige esta capa

| Superficie | Mecanismo | Resultado esperado |
|------------|-----------|-------------------|
| Accesos directos / desinstalador | `aurexalis.ico` + launcher `aurexalis.exe` | Nombre **Aurexalis** en escritorio e Inicio |
| Motor en `Engine/aurexalis-browser.exe` | `brand-engine.ps1` + rcedit (instalador) | Icono morado/dorado y descripcion **Aurexalis** en barra de tareas y Administrador de tareas |
| Texto en ajustes y chrome | `aurexalis-01-brand.uc.js` + CSS | Sustituye *Floorp* / *Ablaze* por **Aurexalis** en UI |
| Prefs de perfil | `browser/prefs/user.js` + bloque generado en install | Desactiva enlaces Floorp, inicio Floorp, bloque Mozilla extra |

## Limitacion: binario Gecko sin recompilar

Si el usuario abre **`floorp.exe`** directamente (sin pasar por el instalador Aurexalis), Windows seguira mostrando marca Floorp.

Para rebrand **completo** a nivel `omni.ja` (cadenas Fluent, `brand.properties`, nombre de proceso interno):

1. Fork/build de `vendor/floorp` con `MOZ_APP_DISPLAYNAME=Aurexalis` y assets propios, **o**
2. Resource hacking manual del `floorp.exe` instalado (sustituido por el flujo del instalador).

## Verificacion rapida

1. Reinstalar o ejecutar instalador con descarga de motor activada.
2. Comprobar que existe `Engine/aurexalis-browser.exe` y que `config.json` apunta a esa ruta.
3. Lanzar solo desde el acceso directo **Aurexalis** (no desde Floorp global).
4. Abrir `about:preferences` — el texto de navegador predeterminado debe decir Aurexalis.
5. Administrador de tareas: proceso **Aurexalis Browser** (si rcedit aplico correctamente).

## Regenerar iconos

```powershell
python tools/gen_installer_icon.py
# Logo maestro (opcional): assets/branding/aurexalis-logo.png
```
