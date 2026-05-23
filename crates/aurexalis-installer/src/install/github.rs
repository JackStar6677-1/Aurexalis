//! Descarga de activos desde GitHub Releases.

use serde::Deserialize;
use std::path::Path;

const AUREXALIS_REPO: &str = "JackStar6677-1/Aurexalis";
const FLOORP_REPO: &str = "Floorp-Projects/Floorp";
const FLOORP_INSTALLER: &str = "floorp-windows-x86_64.installer.exe";

#[derive(Debug, Deserialize)]
struct GhRelease {
    tag_name: String,
    assets: Vec<GhAsset>,
}

#[derive(Debug, Deserialize)]
struct GhAsset {
    name: String,
    browser_download_url: String,
}

/// URL directa del runtime Aurexalis para una version concreta (tag `vX.Y.Z`).
pub fn aurexalis_runtime_url(version: &str) -> String {
    let tag = normalize_tag(version);
    format!(
        "https://github.com/{AUREXALIS_REPO}/releases/download/{tag}/aurexalis-runtime-windows-x86_64.zip"
    )
}

/// Resuelve la URL del instalador completo de Floorp (ultimo release).
pub fn floorp_installer_url(client: &reqwest::blocking::Client) -> Result<String, String> {
    let url = format!("https://api.github.com/repos/{FLOORP_REPO}/releases/latest");
    let release: GhRelease = client
        .get(&url)
        .header("User-Agent", "Aurexalis-Installer")
        .send()
        .map_err(|e| format!("API Floorp: {e}"))?
        .error_for_status()
        .map_err(|e| format!("API Floorp HTTP: {e}"))?
        .json()
        .map_err(|e| format!("JSON Floorp: {e}"))?;

    release
        .assets
        .iter()
        .find(|a| a.name == FLOORP_INSTALLER)
        .map(|a| a.browser_download_url.clone())
        .ok_or_else(|| format!("no se encontro {FLOORP_INSTALLER} en releases de Floorp"))
}

/// Descarga un archivo remoto mostrando progreso via callback `0.0..=1.0`.
pub fn download_file(
    client: &reqwest::blocking::Client,
    url: &str,
    destination: &Path,
    progress: &dyn Fn(f32, &str),
) -> Result<(), String> {
    progress(0.0, "Conectando...");
    let response = client
        .get(url)
        .header("User-Agent", "Aurexalis-Installer")
        .send()
        .map_err(|e| format!("descarga: {e}"))?
        .error_for_status()
        .map_err(|e| format!("HTTP descarga: {e}"))?;

    let total = response.content_length().unwrap_or(0);
    let mut file =
        std::fs::File::create(destination).map_err(|e| format!("crear archivo: {e}"))?;
    let mut reader = response;
    let mut downloaded: u64 = 0;
    let mut buffer = [0_u8; 64 * 1024];

    loop {
        let read = std::io::Read::read(&mut reader, &mut buffer)
            .map_err(|e| format!("leer stream: {e}"))?;
        if read == 0 {
            break;
        }
        std::io::Write::write_all(&mut file, &buffer[..read])
            .map_err(|e| format!("escribir archivo: {e}"))?;
        downloaded += u64::try_from(read).unwrap_or(0);
        if total > 0 {
            let ratio = downloaded as f32 / total as f32;
            progress(ratio.clamp(0.0, 0.98), "Descargando...");
        }
    }

    progress(1.0, "Descarga completa");
    Ok(())
}

fn normalize_tag(version: &str) -> String {
    if version.starts_with('v') {
        version.to_string()
    } else {
        format!("v{version}")
    }
}
