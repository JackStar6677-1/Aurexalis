# Integracion Floorp

Aurexalis usa Floorp como base Gecko auditada mediante submodulo Git en
`vendor/floorp`.

## Estado Actual

| Pieza | Estado | Nota |
|---|---|---|
| Submodulo Floorp | Integrado | `vendor/floorp` apunta a `Floorp-Projects/Floorp` |
| Revision inicial | Fijada | `f9f9e347588173c5edae2d14e76a13fdbd1284d4` |
| Build system | Estudiado | `feles-build` sobre Deno |
| Chrome Web Store | Localizado | `browser-features/chrome/common/addons` |
| Empaquetado | Mapeado | workflows de Floorp + fases `before-mach` / `after-mach` |

## Como Inicializar El Submodulo

```powershell
git submodule update --init --depth 1 vendor/floorp
```

Para actualizarlo de forma controlada:

```powershell
git -C vendor/floorp fetch --depth 1 origin main
git -C vendor/floorp checkout origin/main
git add vendor/floorp
```

Cada avance de revision debe quedar en un commit propio para poder auditar que
parches cambiaron.

## Build System De Floorp

Floorp no se construye como un paquete Rust o Node aislado. Su orquestador es
`feles-build`, expuesto como tarea Deno en `vendor/floorp/deno.json`.

Comandos clave detectados:

```powershell
deno task feles-build stage
deno task feles-build build --phase before-mach
deno task feles-build build --phase after-mach
```

Flujo interno observado en `vendor/floorp/tools/feles-build.ts`:

| Fase | Acciones principales |
|---|---|
| `dev` | aplica parches, prefs, symlinks, compila assets dev, inyecta XHTML y lanza navegador |
| `stage` | aplica parches, prefs, symlinks, compila assets de produccion, inyecta manifiestos y arranca navegador |
| `build --phase before-mach` | prepara symlinks y compila assets de produccion antes de `mach` |
| `build --phase after-mach` | inyecta XHTML posterior al build de Gecko |

La integracion profesional de Aurexalis debe envolver estas fases, no
duplicarlas, hasta tener un fork completo del arbol Gecko/Floorp.

## Chrome Web Store En Floorp

El soporte visible de Chrome Web Store esta concentrado en:

- `vendor/floorp/browser-features/chrome/common/addons/index.ts`
- `vendor/floorp/browser-features/chrome/common/addons/observer.ts`
- `vendor/floorp/browser-features/chrome/common/addons/notification-customizer.ts`
- `vendor/floorp/browser-features/chrome/common/addons/types.ts`

La logica principal:

1. `Addons` crea estado compartido para `pendingChromeWebStoreInstall`.
2. `createCWSObserver` registra observers de Gecko:
   - `floorp-chrome-web-store-install-started`
   - `webextension-permission-prompt`
3. El observer guarda la metadata de la extension iniciada desde CWS.
4. `overrideInstallConfirmation` envuelve `gXPInstallObserver.showInstallConfirmation`.
5. `NotificationCustomizer` reescribe el prompt de permisos para mostrar mensaje,
   badge y advertencia propios de Chrome Web Store.

Esto no es un traductor universal de APIs Chromium. Es una capa de integracion
UI/instalacion sobre WebExtensions de Gecko. Para Aurexalis hay que portarla con
branding propio y pruebas contra extensiones reales antes de prometer
compatibilidad total.

## Empaquetado

Floorp usa workflows dedicados para empaquetado y publicacion. El patron util
para Aurexalis es:

1. Compilar assets del navegador antes de `mach`.
2. Construir Gecko/Floorp con configuracion reproducible.
3. Inyectar XHTML/manifiestos despues del build.
4. Empaquetar por sistema operativo.
5. Firmar solo cuando exista canal propio y certificado.

Hasta que Aurexalis tenga canal propio, los builds deben considerarse
artefactos locales o de CI sin firma oficial.

## Riesgos Tecnicos

- Floorp se mueve rapido; el submodulo debe actualizarse con revision fijada.
- Los hooks de instalacion de extensiones dependen de internals de Firefox.
- La capa CWS cubre instalacion/prompts, no garantiza todas las APIs exclusivas
  de Chrome.
- Build y empaquetado real requieren toolchains grandes de Gecko y Deno.
- No se deben modificar secretos, certificados ni workflows de firma de Floorp.

## Siguiente Trabajo

- Crear adaptador `aurexalis-extensions` con branding propio sobre el modelo de
  Floorp.
- Hacer smoke test manual de instalacion CWS en perfil aislado.
- Ver plan detallado en `docs/CWS_PORT.md` (v0.5: prefs + `aurexalis-07-cws-brand.uc.js`).
- Definir wrapper de build `tools/aurexalis-build.ps1` que invoque Floorp sin
  tocar credenciales ni publicar builds.
- Separar parches Aurexalis en una carpeta propia antes de tocar archivos del
  submodulo.
