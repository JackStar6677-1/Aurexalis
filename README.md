<p align="center">
  <img src="./assets/aurexalis-banner.svg" width="100%" alt="Aurexalis browser banner" />
</p>

<div align="center">

# Aurexalis

**Gecko/Floorp core · Brave-grade blocking · Opera GX-inspired UX · purple/red/gold identity**

<img src="https://readme-typing-svg.demolab.com?font=Fira+Code&weight=700&size=22&pause=1200&color=FF1F55&center=true&vCenter=true&width=940&height=86&lines=Aurexalis%3A+navegacion+modular+y+afilada.;Gecko+por+compatibilidad.+Rust+por+rendimiento.;Morado+profundo%2C+rojo+neon+y+dorado+reactivo." alt="Typing SVG" />

[![Status](https://img.shields.io/badge/status-v0.3.0%20multi--plataforma-FFD166?style=for-the-badge)](#estado)
[![Engine](https://img.shields.io/badge/engine-Gecko%20%2F%20Floorp-7C3AED?style=for-the-badge&logo=firefoxbrowser&logoColor=white)](#arquitectura)
[![Rust](https://img.shields.io/badge/rust-core%20modules-CE412B?style=for-the-badge&logo=rust&logoColor=white)](#stack)
[![Privacy](https://img.shields.io/badge/privacy-local%20first-101018?style=for-the-badge)](#principios)
[![Rust](https://github.com/JackStar6677-1/Aurexalis/actions/workflows/rust.yml/badge.svg)](https://github.com/JackStar6677-1/Aurexalis/actions/workflows/rust.yml)
[![Android](https://github.com/JackStar6677-1/Aurexalis/actions/workflows/android-build.yml/badge.svg)](https://github.com/JackStar6677-1/Aurexalis/actions/workflows/android-build.yml)
[![Release](https://img.shields.io/github/v/release/JackStar6677-1/Aurexalis?include_prereleases&label=release&style=for-the-badge)](https://github.com/JackStar6677-1/Aurexalis/releases)

</div>

## Vision

**Aurexalis** es un proyecto personal para construir un navegador propio mediante kitbashing serio: tomar componentes open-source maduros, integrarlos con criterio y evitar cargar con la pesadez de una base Chromium completa.

La base es **Floorp/Firefox sobre Gecko** en escritorio y **GeckoView** en Android, con interfaz gaming, bloqueo integrado, WebExtensions y migracion local desde Chromium.

No busca ser un fork cosmetico. La idea es una plataforma personal, optimizada y modular, publicada en **Windows, Android y Linux** desde v0.3.0.

## Estado

> Fase actual: **v0.3.0 — shell ejecutable, UI Aurexalis integrada, bloqueador, ajustes y releases multi-plataforma**.

Disponible hoy:

- **Windows:** instalador GUI, CLI portable y runtime zip.
- **Android:** APK GeckoView con home, ajustes y bloqueador ContentBlocking.
- **Linux:** `.deb` (Ubuntu/Debian), `.rpm` (Fedora/RHEL), `.pkg.tar.zst` (Arch) y tarball portable.

El chrome del navegador (`browser/chrome/*.uc.js`) carga sidebar, sonidos, bloqueador, panel **ST** y pagina de ajustes interactiva. Floorp sigue como submodulo auditable en `vendor/floorp` para build Gecko y capa Chrome Web Store.

## Descargas

Cada tag `v*` publica un [GitHub Release](https://github.com/JackStar6677-1/Aurexalis/releases) con:

| Plataforma | Artefactos |
|---|---|
| **Windows** | `Aurexalis-Setup-x86_64.exe`, `aurexalis-windows-x86_64.exe`, `aurexalis-runtime-windows-x86_64.zip` |
| **Android** | `Aurexalis-android-{version}-gecko.apk` |
| **Linux** | `aurexalis_{version}_amd64.deb`, `aurexalis-{version}-1.x86_64.rpm`, `aurexalis-{version}-x86_64.pkg.tar.zst`, `aurexalis-runtime-linux-x86_64.tar.gz` |

En Linux necesitas **Firefox o Floorp** instalado como motor Gecko; el paquete Aurexalis aporta shell, tema, chrome y prefs.

**Ultima release:** [v0.3.0](https://github.com/JackStar6677-1/Aurexalis/releases/tag/v0.3.0) (pre-release).

Detalle de build y empaquetado: [docs/BUILD_AND_RELEASE.md](./docs/BUILD_AND_RELEASE.md).

## Principios

- **Gecko primero:** compatibilidad moderna sin convertir Aurexalis en otro Chromium.
- **Rust donde duela:** modulos de alto rendimiento para red, migracion, parsing y filtros.
- **Privacidad local:** datos de perfiles, cookies y claves se procesan localmente con consentimiento explicito.
- **UI reactiva:** estetica Aurexalis en morado profundo, rojo neon y dorado brillante, con animaciones, barra lateral y sonido local.
- **Modularidad real:** cada pieza debe poder probarse aislada antes de entrar al navegador.
- **Rendimiento visible:** bloquear antes de renderizar, cachear donde corresponda y evitar trabajo inutil.

## Nombre

**Aurexalis** mezcla la raiz aurea/dorada de `AureonVault` con el cierre astronomico de `Coronalis` y `AuroralisStar`. La intencion es que suene a una pieza del mismo universo de repositorios, pero con identidad propia para un navegador.

## Arquitectura

```mermaid
flowchart TB
  User["Usuario"] --> UI["Aurexalis UI Layer"]
  UI --> Theme["userChrome.css + modulos .uc.js"]
  UI --> Sound["Sonido reactivo<br/>03-sound.uc.js"]
  UI --> Settings["Ajustes ST + browser/settings"]

  UI --> Desktop["Gecko / Floorp — escritorio"]
  UI --> Mobile["GeckoView — Android APK"]

  Desktop --> BlockETP["Bloqueador v0.3<br/>Gecko ETP + prefs aurexalis.blocker.*"]
  Mobile --> BlockCB["ContentBlocking<br/>GeckoRuntime Android"]

  BlockETP --> Web["HTTP / HTTPS"]
  BlockCB --> Web

  Desktop --> Ext["WebExtensions"]
  Ext --> FloorpCWS["Floorp CWS — pendiente port Aurexalis"]

  Desktop --> Profile["Perfil Aurexalis"]
  Profile --> Importer["aurexalis-importer<br/>export audit local"]
  Importer --> Chrome["Chrome / Brave / Opera"]

  UI --> RemoteFiles["RemoteFS — cola Rust"]
  RemoteFiles --> SFTP["SFTP / FTP / FTPS"]

  BlockETP -.-> BlockRust["adblock-rust crate<br/>hook de red — proximo"]
```

## Modulos

| Modulo / crate | Objetivo | Estado v0.3 |
|---|---|---|
| `browser/chrome/*.uc.js` | UI morado/rojo/dorado, sidebar, sonidos, bloqueador, panel ST | **Integrado** |
| `browser/settings/` | Pagina de ajustes interactiva (desktop + Android) | **Integrado** |
| `aurexalis-shell` | Launcher CLI, perfiles, import audit | **Release** |
| `aurexalis-installer` | Instalador GUI Windows | **Release** |
| `mobile/android/` | APK GeckoView | **Release** |
| `aurexalis-blocker` | Crate `adblock-rust`; hoy UI aplica Gecko ETP / ContentBlocking | **Parcial** |
| `aurexalis-importer` | Lectura Chromium + export JSON auditable (+ contrasenas opcional) | **Parcial** |
| `aurexalis-remotefs` | Cola SFTP/FTP/FTPS | **Backend Rust** |
| Floorp CWS | Chrome Web Store sobre Gecko | **Pendiente** |

## Stack

<div align="center">

<img src="https://skillicons.dev/icons?i=rust,cpp,js,html,css,firefox,sqlite,linux,windows,android,git,github&perline=11" alt="Stack" />

<br />

![Gecko](https://img.shields.io/badge/Gecko-rendering%20core-FF7139?style=flat-square&logo=firefoxbrowser&logoColor=white)
![Floorp](https://img.shields.io/badge/Floorp-fork%20base-7C3AED?style=flat-square)
![adblock-rust](https://img.shields.io/badge/adblock--rust-network%20blocking-FFD166?style=flat-square)
![SQLite](https://img.shields.io/badge/SQLite-profile%20data-003B57?style=flat-square&logo=sqlite)
![DPAPI](https://img.shields.io/badge/Windows-DPAPI-0078D4?style=flat-square&logo=windows)
![Secret Service](https://img.shields.io/badge/Linux-Secret%20Service-FCC624?style=flat-square&logo=linux&logoColor=111111)
![GeckoView](https://img.shields.io/badge/GeckoView-Android%20APK-FF7139?style=flat-square&logo=firefoxbrowser&logoColor=white)
![Release](https://img.shields.io/badge/release-v0.3.0-FFD166?style=flat-square)

</div>

## Flujo De Red

**Hoy (v0.3):** el bloqueador usa las APIs nativas de Gecko — **ETP** en escritorio (`aurexalis-02-blocker.uc.js`) y **ContentBlocking** en Android (`AurexalisPrefs` + `GeckoRuntime`). Las prefs `aurexalis.blocker.*` se editan desde **ST** o la pagina de ajustes.

**Proximo:** enganchar el crate `aurexalis-blocker` (`adblock-rust`) al pipeline de requests antes del render.

```mermaid
sequenceDiagram
  participant Page as Pagina
  participant Gecko as Gecko / GeckoView
  participant Prefs as prefs aurexalis.blocker.*
  participant ETP as ETP / ContentBlocking
  participant Rust as adblock-rust (proximo)
  participant Net as Red

  Page->>Gecko: solicita recurso
  Gecko->>Prefs: nivel standard / strict / off
  Prefs->>ETP: politica activa
  alt v0.3 actual
    ETP-->>Gecko: bloquear rastreadores / ads conocidos
  else hook futuro
    Gecko->>Rust: match(url, tipo)
    Rust-->>Gecko: block / allow
  end
  Gecko->>Net: request permitido
  Net-->>Page: respuesta
```

## Migracion De Perfil

El importador es **local-first**: no inicia sesion en cuentas ni envia datos fuera del equipo.

**Implementado hoy:**

- Deteccion de perfiles Chrome, Brave y Opera en Windows.
- Exportacion auditable a JSON en el perfil (`aurexalis import audit`).
- Opcion de incluir contrasenas con consentimiento explicito (`--passwords`).
- Acceso desde panel **IM** / **ST** del sidebar y pagina `browser/settings/`.

**Pendiente:** escritura directa al perfil Gecko, descifrado Linux (Secret Service/KWallet) e importacion nativa Android.

## Archivos Remotos

Aurexalis tambien tendra un modulo de navegador de archivos remoto: conexiones SFTP, FTP y FTPS dentro del propio navegador, pensado como una alternativa integrada a montar unidades tipo RaiDrive.

El modulo se documenta en [docs/REMOTE_FS.md](./docs/REMOTE_FS.md) y queda separado para implementarlo despues sin mezclarlo con el motor web.

## Base Floorp

Floorp esta integrado como submodulo Git en `vendor/floorp` para mantener una
referencia auditable al nucleo Gecko elegido.

```powershell
git submodule update --init --depth 1 vendor/floorp
.\tools\floorp-status.ps1
```

El analisis de parches, build system, empaquetado y soporte Chrome Web Store se
mantiene en [docs/FLOORP_INTEGRATION.md](./docs/FLOORP_INTEGRATION.md).

## UI Aurexalis

Paleta inicial:

| Token | Color | Uso |
|---|---:|---|
| `--ax-bg` | `#08050F` | fondo raiz |
| `--ax-surface` | `#120A1E` | barras, sidebar, paneles |
| `--ax-surface-2` | `#1E102D` | tabs y controles |
| `--ax-purple` | `#6F38FF` | profundidad y energia secundaria |
| `--ax-red` | `#FF1F55` | acento principal tipo GX |
| `--ax-gold` | `#FFD166` | foco, premium y acciones destacadas |
| `--ax-text` | `#F7F2FF` | texto principal |

### Barra lateral

Accesos del dock vertical (`browser/chrome/aurexalis-05-sidebar.uc.js`):

| Boton | Funcion |
|---|---|
| **AX** | Home Aurexalis |
| **GX** | GX Corner |
| **RF** | Archivos remotos (panel + backend Rust) |
| **BM** | Marcadores |
| **DL** | Descargas |
| **IM** | Importador Chromium local |
| **BL** | Bloqueador on/off rapido |
| **PW** | Contrasenas (`about:logins`) |
| **ST** | Panel de ajustes integrado |

### Modulos chrome (orden de carga)

`userChrome.js` carga en serie:

1. `aurexalis-00-core.uc.js` — prefs y launcher
2. `aurexalis-01-brand.uc.js` — identidad visual
3. `aurexalis-02-blocker.uc.js` — bloqueador Gecko ETP
4. `aurexalis-03-sound.uc.js` — sonidos reactivos
5. `aurexalis-04-settings-panel.uc.js` — panel **ST**
6. `aurexalis-05-sidebar.uc.js` — sidebar
7. `aurexalis-06-settings-inject.uc.js` — puente prefs en pagina de ajustes

### Ajustes y bloqueador

Preferencias bajo el prefijo `aurexalis.*` (editables desde **ST** o desde `browser/settings/`):

- **Sonidos:** master, click, hover, teclado, ambiente, panel
- **UI:** animaciones on/off
- **Bloqueador:** activo, nivel (`standard` / `strict` / `off`), filtros cosmeticos
- **Importacion:** exportacion local Chromium (con opcion de contrasenas bajo consentimiento)

En desktop la pagina `browser/settings/index.html` recibe `AurexalisPrefsBridge` al abrirse en una pestaña. En Android los mismos controles hablan con la app via `aurexalis://pref/set`.

Mas detalle en [docs/UI.md](./docs/UI.md).

## Roadmap

Resumen por fases (detalle en [docs/ROADMAP.md](./docs/ROADMAP.md)):

| Fase | Alcance | Estado |
|---|---|---|
| **0 — Ingenieria** | Workspace Rust, CI, docs, identidad | Hecho |
| **1 — Shell y UI** | `userChrome`, sidebar, sonidos, home, ajustes ST + HTML | Hecho |
| **1b — Multi-plataforma** | Windows installer, APK Android, Linux deb/rpm/arch | **v0.3.0** |
| **2 — Importador** | Export audit Chromium (+ contrasenas opcional) | Parcial |
| **3 — Bloqueador** | ETP desktop + ContentBlocking Android; hook `adblock-rust` | Parcial |
| **4 — RemoteFS** | SFTP/FTP/FTPS integrado en navegador | Diseno + cola Rust |
| **5 — Gecko/Floorp** | Submodulo, build map, port Chrome Web Store | Parcial |

```mermaid
gantt
  title Roadmap Aurexalis (actualizado v0.3.0)
  dateFormat  YYYY-MM-DD
  axisFormat  %d/%m

  section Hecho
  Base repo + CI + Floorp submodule     :done, r1, 2026-05-23, 2d
  UI chrome sidebar sonidos home          :done, u1, 2026-05-23, 3d
  Panel ST + settings + bloqueador UI     :done, u2, 2026-05-24, 1d
  Release Windows Android Linux           :done, p1, 2026-05-24, 1d

  section En curso
  Importador escritura perfil Gecko       :active, m1, 2026-05-25, 7d
  Hook adblock-rust en pipeline red       :active, b1, 2026-05-28, 10d

  section Proximo
  RemoteFS explorador UI                  :         f1, 2026-06-10, 8d
  Port Chrome Web Store Floorp            :         e1, 2026-06-12, 10d
  Importador Android nativo               :         m2, 2026-06-15, 7d
```

## Entregables

### v0.3.0 (publicado)

- [x] Release Windows: instalador, CLI y runtime zip.
- [x] Release Android: APK GeckoView con home, ajustes y bloqueador.
- [x] Release Linux: `.deb`, `.rpm`, `.pkg.tar.zst` y tarball portable.
- [x] Modulos chrome numerados `00`–`06` con loader `userChrome.js`.
- [x] Bloqueador: Gecko ETP (desktop) + ContentBlocking (Android).
- [x] Ajustes unificados: panel **ST**, `browser/settings/`, prefs `aurexalis.*`.
- [x] Boton **BL** en sidebar (toggle rapido del bloqueador).

### Base previa

- [x] Workspace Rust modular, tests y CI (`rust.yml`, `android-build.yml`, `release.yml`).
- [x] `userChrome.css`, sidebar GX, sonido reactivo, home Aurexalis.
- [x] `aurexalis-importer` + export audit Chromium (con opcion contrasenas).
- [x] Crate `aurexalis-blocker` con `adblock-rust` (PoC fuera del navegador).
- [x] Shell `aurexalis-shell` e instalador GUI Windows.
- [x] Submodulo Floorp auditable en `vendor/floorp`.

### Pendiente

- [ ] Portar capa Chrome Web Store de Floorp con branding Aurexalis.
- [ ] Hook de red Gecko con `adblock-rust` en el pipeline de requests.
- [ ] Importacion Chromium: escritura directa al perfil + Linux + Android nativo.
- [ ] RemoteFS: explorador integrado en UI (hoy solo backend/cola Rust).

## Shell Ejecutable

El binario arrancable vive en `aurexalis-shell`:

```powershell
.\tools\aurexalis-build.ps1 -Mode build
.\target\debug\aurexalis.exe profiles
.\target\debug\aurexalis.exe launch "C:\Ruta\A\floorp.exe"
```

**Windows:** descarga en [GitHub Releases](https://github.com/JackStar6677-1/Aurexalis/releases) — `Aurexalis-Setup-x86_64.exe` (recomendado), `aurexalis-windows-x86_64.exe` (CLI) o runtime zip.

**Linux:**

```bash
./tools/package-linux.sh          # genera .deb, .rpm, .pkg.tar.zst y tarball
sudo dpkg -i dist/aurexalis_*_amd64.deb   # Ubuntu/Debian
# o rpm -i / pacman -U segun tu distro
aurexalis --launch-installed
```

**Android:** ver [mobile/README.md](./mobile/README.md). Sincroniza assets web antes del build:

```powershell
.\tools\sync-mobile-assets.ps1
.\tools\bootstrap-android.ps1
```

Verificacion del pack de chrome:

```powershell
.\tools\verify-browser-pack.ps1
```

La documentacion de build y empaquetado esta en
[docs/BUILD_AND_RELEASE.md](./docs/BUILD_AND_RELEASE.md).

## Pruebas

La suite esta documentada en [docs/TESTING.md](./docs/TESTING.md).

| Workflow | Que valida |
|---|---|
| [`rust.yml`](./.github/workflows/rust.yml) | `cargo test`, clippy, `verify-browser-pack.ps1` |
| [`android-build.yml`](./.github/workflows/android-build.yml) | APK release en push a `main` |
| [`release.yml`](./.github/workflows/release.yml) | Windows + Android + Linux en tag `v*` |

## Profesionalizacion

- [docs/QUALITY.md](./docs/QUALITY.md): gates de calidad y reglas de ingenieria.
- [docs/ROADMAP.md](./docs/ROADMAP.md): fases de producto.
- [CONTRIBUTING.md](./CONTRIBUTING.md): flujo de cambios.
- [SECURITY.md](./SECURITY.md): politica de datos sensibles.
- [docs/adr](./docs/adr): decisiones arquitectonicas.

## Licencia Y Uso

Proyecto personal open-source (MIT). v0.3.0 publica binarios en GitHub Releases como **pre-release**. Assets propietarios de terceros (sonidos comerciales, temas cerrados) no se incluyen; usa tus propios OGG en `browser/chrome/sounds/`.

---

<p align="center">
  <strong>Aurexalis</strong><br />
  Morado profundo. Rojo neon. Dorado reactivo. Control local.
</p>
