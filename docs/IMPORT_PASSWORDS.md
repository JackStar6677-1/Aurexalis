# Importacion de contrasenas (v0.5)

## Por que no escribimos `logins.json` directamente

Firefox/Gecko cifra las credenciales con **NSS** (`key4.db`, `cert9.db`). Insertar entradas en claro en `logins.json` deja el almacen inconsistente y puede invalidar el perfil.

## Flujo Aurexalis (local-first + consentimiento)

1. `aurexalis import audit --passwords` — incluye logins descifrados en el JSON de staging (solo disco local).
2. `aurexalis import apply --passwords-only --passwords-consent` — genera:
   - `profile/import/passwords-import-<timestamp>.csv`
   - `profile/import/passwords-import-<timestamp>.manifest.json`
3. En el navegador: **about:logins → Importar desde archivo** (CSV compatible Firefox).

Sin `--passwords-consent` el comando falla de forma explicita.

## Linux / descifrado Chromium

Chrome/Chromium en Linux guarda la clave en **libsecret** (GNOME Keyring) o, en instalaciones antiguas, fallback **V10** (`peanuts` + PBKDF2). Aurexalis intenta ambos via `keyring` + `linux_crypt`.

**KWallet (KDE)** queda diferido: requiere adaptador D-Bus dedicado (roadmap v0.6).

## Roadmap

| Fase | Objetivo |
|------|----------|
| v0.5 | CSV staging + consentimiento |
| v0.6+ | NSS direct write o integracion con Login Manager API de Gecko |
