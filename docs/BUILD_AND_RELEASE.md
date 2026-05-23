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
git tag v0.2.2
git push origin v0.2.2
```

### Instalador grafico

`Aurexalis-Setup-x86_64.exe` incluye pantallas de bienvenida, licencia MIT,
idioma ES/EN, selector de carpeta, barra de progreso e icono de marca.

1. Descarga `aurexalis-runtime-*.zip` del mismo release en GitHub.
2. Descarga e instala **Floorp** en `{instalacion}\Engine`.
3. Aplica chrome/prefs al perfil `profiles\default`.
4. Escribe `config.json`, `LICENSE` y `uninstall.ps1`.
5. Crea accesos directos en **escritorio** y **menu Inicio**.

Ruta por defecto: `%LOCALAPPDATA%\Aurexalis`. Ver [INSTALLER.md](./INSTALLER.md).

### CLI portable (desarrollo)

```powershell
.\aurexalis-windows-x86_64.exe profiles
$env:AUREXALIS_BROWSER="C:\Ruta\A\floorp.exe"
.\aurexalis-windows-x86_64.exe launch
```

> Pre-release: el navegador completo empaquetado (Gecko propio) llegara en fases
> posteriores; hoy el instalador orquesta Floorp oficial + capa Aurexalis.
