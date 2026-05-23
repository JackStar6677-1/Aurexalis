<p align="center">
  <img src="./assets/aurexalis-banner.svg" width="100%" alt="Aurexalis browser banner" />
</p>

<div align="center">

# Aurexalis

**Gecko/Floorp core · Brave-grade blocking · Opera GX-inspired UX · gold reactive identity**

<img src="https://readme-typing-svg.demolab.com?font=Fira+Code&weight=700&size=22&pause=1200&color=FFD166&center=true&vCenter=true&width=940&height=86&lines=Aurexalis%3A+navegacion+modular+y+afilada.;Gecko+por+compatibilidad.+Rust+por+rendimiento.;Morado+profundo%2C+dorado+reactivo%2C+control+local." alt="Typing SVG" />

[![Status](https://img.shields.io/badge/status-arquitectura%20inicial-FFD166?style=for-the-badge)](#estado)
[![Engine](https://img.shields.io/badge/engine-Gecko%20%2F%20Floorp-7C3AED?style=for-the-badge&logo=firefoxbrowser&logoColor=white)](#arquitectura)
[![Rust](https://img.shields.io/badge/rust-core%20modules-CE412B?style=for-the-badge&logo=rust&logoColor=white)](#stack)
[![Privacy](https://img.shields.io/badge/privacy-local%20first-101018?style=for-the-badge)](#principios)

</div>

## Vision

**Aurexalis** es un proyecto personal para construir un navegador propio mediante kitbashing serio: tomar componentes open-source maduros, integrarlos con criterio y evitar cargar con la pesadez de una base Chromium completa.

La base prevista es **Floorp/Firefox sobre Gecko**, con una interfaz personalizada tipo gaming, bloqueo nativo de red, compatibilidad fuerte con WebExtensions y herramientas locales de migracion desde navegadores Chromium.

No busca ser un fork cosmetico. La idea es una plataforma personal, optimizada y modular.

## Estado

> Fase actual: **arquitectura, identidad visual y plan tecnico**.

Este repositorio comienza como base publica del proyecto: documentacion, roadmap, identidad visual, diagramas y decisiones tecnicas. El codigo duro se integrara por modulos para mantener control, pruebas y rollback.

## Principios

- **Gecko primero:** compatibilidad moderna sin convertir Aurexalis en otro Chromium.
- **Rust donde duela:** modulos de alto rendimiento para red, migracion, parsing y filtros.
- **Privacidad local:** datos de perfiles, cookies y claves se procesan localmente con consentimiento explicito.
- **UI reactiva:** estetica Aurexalis en morado profundo y dorado brillante, con animaciones y sonido local.
- **Modularidad real:** cada pieza debe poder probarse aislada antes de entrar al navegador.
- **Rendimiento visible:** bloquear antes de renderizar, cachear donde corresponda y evitar trabajo inutil.

## Nombre

**Aurexalis** mezcla la raiz aurea/dorada de `AureonVault` con el cierre astronomico de `Coronalis` y `AuroralisStar`. La intencion es que suene a una pieza del mismo universo de repositorios, pero con identidad propia para un navegador.

## Arquitectura

```mermaid
flowchart TB
  User["Usuario"] --> UI["Aurexalis UI Layer"]
  UI --> Theme["Theme Engine<br/>userChrome.css + UI modules"]
  UI --> Sound["Reactive Sound Engine<br/>AudioContext + local assets"]

  UI --> Gecko["Gecko / Floorp Core"]

  Gecko --> Net["Network Policy Layer"]
  Net --> Adblock["adblock-rust<br/>filter lists + matching"]
  Net --> Web["HTTP / HTTPS Requests"]

  Gecko --> Ext["WebExtensions Layer"]
  Ext --> FloorpCWS["Floorp Chrome Web Store support"]
  Ext --> FirefoxAPI["Firefox browser.* / chrome.* APIs"]

  Gecko --> Profile["Aurexalis Profile"]
  Profile --> Importer["Rust Profile Importer"]
  Importer --> Chrome["Chrome"]
  Importer --> Brave["Brave"]
  Importer --> Opera["Opera"]
```

## Modulos

| Modulo | Objetivo | Base tecnica |
|---|---|---|
| `aurexalis-ui` | Interfaz morado/dorado, sidebar, tabs, animaciones y estilo propio | Firefox chrome UI, CSS, JS |
| `aurexalis-sound` | Sonidos reactivos de click, hover, tipeo y acciones de UI | JavaScript, AudioContext, assets locales |
| `aurexalis-blocker` | Bloqueo nativo antes del renderizado | Rust, `adblock-rust`, filtros uBlock/ABP |
| `aurexalis-importer` | Migracion local de cookies, historial, bookmarks, claves y contrasenas | Rust, SQLite, DPAPI, Secret Service/KWallet |
| `aurexalis-extensions` | Compatibilidad con Chrome Web Store sobre Gecko | Floorp, WebExtensions, manifests |
| `aurexalis-profile` | Perfil local endurecido, preferencias y defaults | Firefox prefs, policies, profile templates |

## Stack

<div align="center">

<img src="https://skillicons.dev/icons?i=rust,cpp,js,html,css,firefox,sqlite,linux,windows,git,github&perline=11" alt="Stack" />

<br />

![Gecko](https://img.shields.io/badge/Gecko-rendering%20core-FF7139?style=flat-square&logo=firefoxbrowser&logoColor=white)
![Floorp](https://img.shields.io/badge/Floorp-fork%20base-7C3AED?style=flat-square)
![adblock-rust](https://img.shields.io/badge/adblock--rust-network%20blocking-FFD166?style=flat-square)
![SQLite](https://img.shields.io/badge/SQLite-profile%20data-003B57?style=flat-square&logo=sqlite)
![DPAPI](https://img.shields.io/badge/Windows-DPAPI-0078D4?style=flat-square&logo=windows)
![Secret Service](https://img.shields.io/badge/Linux-Secret%20Service-FCC624?style=flat-square&logo=linux&logoColor=111111)

</div>

## Flujo De Red

```mermaid
sequenceDiagram
  participant Page as Page / Document
  participant Gecko as Gecko Request Pipeline
  participant Policy as Aurexalis Network Policy
  participant Blocker as adblock-rust Engine
  participant Net as Network

  Page->>Gecko: solicita recurso
  Gecko->>Policy: consulta politica de request
  Policy->>Blocker: match(url, source, type)
  alt bloqueado
    Blocker-->>Policy: block
    Policy-->>Gecko: cancelar antes de render
  else permitido
    Blocker-->>Policy: allow / redirect / modify
    Policy->>Net: continuar request
    Net-->>Gecko: respuesta
    Gecko-->>Page: render
  end
```

## Migracion De Perfil

El importador sera una herramienta local y explicita. No inicia sesion en cuentas por ti ni envia datos fuera del equipo.

```mermaid
flowchart LR
  Detect["Detectar perfiles"] --> Copy["Copiar SQLite a staging seguro"]
  Copy --> Read["Leer Cookies / History / Login Data"]
  Read --> Decrypt["Descifrar con API local del sistema"]
  Decrypt --> Transform["Normalizar formato Aurexalis"]
  Transform --> Write["Escribir en perfil Aurexalis"]

  Decrypt --> Win["Windows<br/>DPAPI + Local State"]
  Decrypt --> Linux["Linux<br/>Secret Service / KWallet"]
```

Alcance previsto:

- Cookies de Chrome, Brave y Opera.
- Historial y bookmarks.
- Claves y contrasenas guardadas cuando el sistema permita descifrado local.
- Importacion controlada hacia el perfil Aurexalis.

## UI Aurexalis

Paleta inicial:

| Token | Color | Uso |
|---|---:|---|
| `--js-bg` | `#090512` | fondo raiz |
| `--js-surface` | `#140A24` | barras, sidebar, paneles |
| `--js-surface-2` | `#211034` | tabs y controles |
| `--js-purple` | `#7C3AED` | energia principal |
| `--js-gold` | `#FFD166` | acento reactivo |
| `--js-gold-hot` | `#FFE29A` | hover, glow, foco |
| `--js-text` | `#F7F2FF` | texto principal |

## Roadmap

```mermaid
gantt
  title Roadmap inicial Aurexalis
  dateFormat  YYYY-MM-DD
  axisFormat  %d/%m

  section Base
  Repo, README e identidad       :done,    r1, 2026-05-23, 1d
  Analisis Floorp/Firefox        :active,  r2, 2026-05-24, 5d

  section UI
  userChrome.css Aurexalis       :         u1, 2026-05-25, 3d
  Motor de sonido reactivo       :         u2, after u1, 3d

  section Rust
  Importador Brave cookies       :         m1, 2026-05-27, 4d
  Importador claves/login data   :         m2, after m1, 5d
  Integracion adblock-rust PoC   :         b1, 2026-06-03, 7d

  section Gecko
  Capa CWS de Floorp             :         e1, 2026-06-08, 7d
  Hook de red Gecko              :         e2, after b1, 8d
```

## Primeros Entregables

- [x] Crear repo base.
- [x] Definir identidad visual Aurexalis.
- [x] Documentar arquitectura modular.
- [ ] Crear `userChrome.css` inicial.
- [ ] Crear `aurexalis-sound` PoC.
- [ ] Crear `aurexalis-importer` Rust para cookies Brave.
- [ ] Probar `adblock-rust` fuera del navegador.
- [ ] Estudiar parches de Floorp para Chrome Web Store.

## Licencia Y Uso

Proyecto personal en etapa temprana. La base publica documenta arquitectura e identidad. Assets propietarios de terceros, como sonidos comerciales o temas cerrados, no se incluyen en este repositorio.

---

<p align="center">
  <strong>Aurexalis</strong><br />
  Morado profundo. Dorado reactivo. Control local.
</p>
