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

## GitHub Releases (shell Windows)

Cada tag `v*` dispara [`.github/workflows/release.yml`](../.github/workflows/release.yml),
que compila `aurexalis-shell` en `windows-latest` y adjunta
`aurexalis-windows-x86_64.exe`.

```powershell
git tag v0.1.0
git push origin v0.1.0
```

Tras el workflow, el binario aparece en
[Releases](https://github.com/JackStar6677-1/Aurexalis/releases).

Uso del artefacto:

```powershell
.\aurexalis-windows-x86_64.exe profiles
$env:AUREXALIS_BROWSER="C:\Ruta\A\floorp.exe"
.\aurexalis-windows-x86_64.exe launch
```

> Pre-release: solo CLI/servicios Rust. Necesitas Firefox o Floorp aparte; no
> incluye el motor Gecko empaquetado.
