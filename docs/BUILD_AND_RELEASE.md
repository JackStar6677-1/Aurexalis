# Build Y Distribucion

Este documento separa lo que ya es ejecutable de lo que requiere toolchain
completa Gecko/Floorp.

## Shell Rust

El binario arrancable inicial es `aurexalis`, dentro del crate
`aurexalis-shell`.

```powershell
.\tools\aurexalis-build.ps1 -Mode build
.\target\debug\aurexalis.exe profiles
.\target\debug\aurexalis.exe launch "C:\Ruta\A\floorp.exe"
```

Tambien se puede definir:

```powershell
$env:AUREXALIS_BROWSER="C:\Ruta\A\floorp.exe"
.\target\debug\aurexalis.exe launch
```

## Floorp

Floorp vive como submodulo en `vendor/floorp`. Su build real usa Deno y `mach`:

```powershell
git submodule update --init --depth 1 vendor/floorp
.\tools\floorp-status.ps1
```

Fases de Floorp que Aurexalis debe envolver:

```powershell
deno task feles-build build --phase before-mach
deno task feles-build build --phase after-mach
```

## Empaquetado

El pipeline profesional queda dividido en cuatro artefactos:

1. `aurexalis-shell`: CLI y servicios Rust.
2. Perfil Aurexalis: prefs, chrome CSS/JS y branding.
3. Floorp/Gecko: binario base compilado.
4. Paquete final: instalador Windows o bundle Linux.

Los builds del navegador completo no deben firmarse hasta tener certificados
propios. El **shell Rust** (`aurexalis.exe`) si se publica en GitHub Releases.

## GitHub Releases (Windows)

Cada tag `v*` dispara [`.github/workflows/release.yml`](../.github/workflows/release.yml)
y publica tres artefactos:

| Archivo | Uso |
|---|---|
| `Aurexalis-Setup-x86_64.exe` | **Instalador con GUI** (recomendado) |
| `aurexalis-runtime-windows-x86_64.zip` | Runtime (shell + chrome + prefs) |
| `aurexalis-windows-x86_64.exe` | CLI portable para desarrolladores |

```powershell
git tag v0.2.0
git push origin v0.2.0
```

### Instalador grafico

`Aurexalis-Setup-x86_64.exe` muestra un asistente con la identidad visual
(morado / rojo / dorado) y:

1. Descarga `aurexalis-runtime-*.zip` del mismo release en GitHub.
2. Descarga e instala **Floorp** (motor Gecko) en `{instalacion}\Engine`.
3. Aplica `userChrome.css`, scripts UC y `user.js` al perfil `profiles\default`.
4. Escribe `config.json` y crea un acceso directo **Aurexalis** en el escritorio.

Ruta por defecto: `%LOCALAPPDATA%\Aurexalis`.

### CLI portable (desarrollo)

```powershell
.\aurexalis-windows-x86_64.exe profiles
$env:AUREXALIS_BROWSER="C:\Ruta\A\floorp.exe"
.\aurexalis-windows-x86_64.exe launch
```

> Pre-release: el navegador completo empaquetado (Gecko propio) llegara en fases
> posteriores; hoy el instalador orquesta Floorp oficial + capa Aurexalis.
