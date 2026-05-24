//! Orquestacion de la instalacion completa de Aurexalis.

mod chromium;
mod engine_brand;
mod floorp;
mod github;
mod profile;
pub mod windows;

use serde::Serialize;
use std::fs::{self, File};
use std::io::{copy, BufReader};
use std::path::Path;
use zip::ZipArchive;

const RUNTIME_ZIP: &str = "aurexalis-runtime.zip";
const FLOORP_INSTALLER: &str = "floorp-installer.exe";

#[derive(Debug, Clone, Serialize)]
pub struct InstallConfig {
    pub version: String,
    pub install_root: String,
    pub launcher: String,
    pub browser: String,
    pub profile: String,
    pub chromium_audit: Option<String>,
}

/// Ejecuta todos los pasos de instalacion; reporta progreso global 0..1.
pub fn run_full_install(
    install_root: &Path,
    version: &str,
    download_floorp: bool,
    import_chromium: bool,
    import_passwords: bool,
    progress: &dyn Fn(f32, &str),
) -> Result<InstallConfig, String> {
    let client = reqwest::blocking::Client::builder()
        .user_agent("Aurexalis-Installer")
        .build()
        .map_err(|e| e.to_string())?;

    fs::create_dir_all(install_root).map_err(|e| e.to_string())?;
    windows::ensure_disk_space(install_root)?;
    let temp = install_root.join(".install-temp");
    fs::create_dir_all(&temp).map_err(|e| e.to_string())?;

    progress(0.02, "Preparando carpetas...");

    let runtime_zip = temp.join(RUNTIME_ZIP);
    let runtime_url = github::aurexalis_runtime_url(version);
    progress(0.05, "Descargando runtime Aurexalis...");
    github::download_file(&client, &runtime_url, &runtime_zip, &|ratio, msg| {
        progress(0.05 + ratio * 0.35, msg)
    })?;

    progress(0.42, "Extrayendo componentes...");
    extract_zip(&runtime_zip, install_root)?;
    let _ = fs::remove_file(&runtime_zip);

    let profile_dir = install_root.join("profiles").join("default");
    fs::create_dir_all(&profile_dir).map_err(|e| e.to_string())?;
    profile::apply_browser_pack(install_root, &profile_dir)?;

    let engine_dir = install_root.join("Engine");
    let browser = if download_floorp {
        progress(0.48, "Descargando nucleo Gecko...");
        let floorp_url = github::floorp_installer_url(&client)?;
        let floorp_path = temp.join(FLOORP_INSTALLER);
        github::download_file(&client, &floorp_url, &floorp_path, &|ratio, msg| {
            progress(0.48 + ratio * 0.28, msg)
        })?;

        progress(0.78, "Instalando nucleo Gecko (Aurexalis)...");
        floorp::run_floorp_installer(&floorp_path, &engine_dir)?;
        let _ = fs::remove_file(&floorp_path);

        floorp::resolve_floorp_binary(&engine_dir)
            .ok_or("El motor Gecko se instalo pero no se encontro floorp.exe".to_string())?
    } else {
        floorp::resolve_floorp_binary(&engine_dir).ok_or(
            "No se encontro el motor Gecko. Activa la descarga del motor o instalalo manualmente.",
        )?
    };

    progress(0.82, "Aplicando marca Aurexalis al motor...");
    copy_branding_icon(install_root)?;
    let icon = install_root.join("aurexalis.ico");
    let branded = engine_brand::brand_engine_binary(&browser, &engine_dir, &icon)?;
    let browser = engine_brand::resolve_engine_binary(&engine_dir, &branded);

    progress(0.86, "Guardando configuracion...");
    let mut chromium_audit = None;
    if import_chromium {
        progress(0.88, "Exportando snapshot Chromium local...");
        chromium_audit = Some(
            chromium::export_staging_snapshot(&profile_dir, import_passwords)
                .unwrap_or_else(|error| format!("AVISO: {error}")),
        );
    }

    let launcher = install_root.join("aurexalis.exe");
    let config = InstallConfig {
        version: version.to_string(),
        install_root: install_root.to_string_lossy().into_owned(),
        launcher: launcher.to_string_lossy().into_owned(),
        browser: browser.to_string_lossy().into_owned(),
        profile: profile_dir.to_string_lossy().into_owned(),
        chromium_audit,
    };
    write_config(install_root, &config)?;

    progress(0.90, "Copiando licencia...");
    copy_license(install_root)?;

    progress(0.92, "Creando accesos directos y registro Windows...");
    let launcher = install_root.join("aurexalis.exe");
    let icon = install_root.join("aurexalis.ico");
    let launch_args = Some("--launch-installed");
    windows::write_uninstaller(install_root, version, Some(&icon))?;
    windows::create_desktop_shortcut(
        "Aurexalis",
        &launcher,
        install_root,
        launch_args,
        Some(&icon),
    )?;
    windows::create_start_menu_shortcut(
        "Aurexalis",
        &launcher,
        install_root,
        launch_args,
        Some(&icon),
    )?;

    let _ = fs::remove_dir_all(&temp);
    profile::refresh_runtime_prefs(install_root, &profile_dir)?;
    progress(1.0, "Instalacion completa");
    Ok(config)
}

fn extract_zip(archive_path: &Path, destination: &Path) -> Result<(), String> {
    let file = File::open(archive_path).map_err(|e| e.to_string())?;
    let reader = BufReader::new(file);
    let mut archive = ZipArchive::new(reader).map_err(|e| format!("zip invalido: {e}"))?;

    for index in 0..archive.len() {
        let mut entry = archive.by_index(index).map_err(|e| e.to_string())?;
        let outpath = destination.join(
            entry
                .enclosed_name()
                .ok_or_else(|| format!("entrada zip insegura: {}", entry.name()))?,
        );

        if entry.name().ends_with('/') {
            fs::create_dir_all(&outpath).map_err(|e| e.to_string())?;
            continue;
        }

        if let Some(parent) = outpath.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }

        let mut outfile = File::create(&outpath).map_err(|e| e.to_string())?;
        copy(&mut entry, &mut outfile).map_err(|e| e.to_string())?;
    }

    Ok(())
}

fn write_config(install_root: &Path, config: &InstallConfig) -> Result<(), String> {
    let json = serde_json::to_string_pretty(config).map_err(|e| e.to_string())?;
    fs::write(install_root.join("config.json"), json).map_err(|e| e.to_string())
}

fn copy_branding_icon(install_root: &Path) -> Result<(), String> {
    let dest = install_root.join("aurexalis.ico");
    if dest.is_file() {
        return Ok(());
    }

    const ICON: &[u8] = include_bytes!("../../../../assets/branding/aurexalis.ico");
    fs::write(&dest, ICON).map_err(|e| format!("copiar icono: {e}"))
}

fn copy_license(install_root: &Path) -> Result<(), String> {
    let dest = install_root.join("LICENSE");
    if dest.is_file() {
        return Ok(());
    }

    const LICENSE: &[u8] = include_bytes!("../../../../LICENSE");
    fs::write(&dest, LICENSE).map_err(|e| format!("copiar LICENSE: {e}"))
}
