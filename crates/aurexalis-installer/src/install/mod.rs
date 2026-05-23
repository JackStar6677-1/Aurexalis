//! Orquestacion de la instalacion completa de Aurexalis.

mod floorp;
mod github;
mod profile;
mod windows;

use serde::Serialize;
use std::fs::{self, File};
use std::io::{copy, BufReader};
use std::path::Path;
use zip::ZipArchive;

pub use github::{aurexalis_runtime_url, download_file, floorp_installer_url};

const RUNTIME_ZIP: &str = "aurexalis-runtime.zip";
const FLOORP_INSTALLER: &str = "floorp-installer.exe";

#[derive(Debug, Clone, Serialize)]
pub struct InstallConfig {
    pub version: String,
    pub install_root: String,
    pub browser: String,
    pub profile: String,
}

/// Ejecuta todos los pasos de instalacion; reporta progreso global 0..1.
pub fn run_full_install(
    install_root: &Path,
    version: &str,
    download_floorp: bool,
    progress: &dyn Fn(f32, &str),
) -> Result<InstallConfig, String> {
    let client = reqwest::blocking::Client::builder()
        .user_agent("Aurexalis-Installer")
        .build()
        .map_err(|e| e.to_string())?;

    fs::create_dir_all(install_root).map_err(|e| e.to_string())?;
    let temp = install_root.join(".install-temp");
    fs::create_dir_all(&temp).map_err(|e| e.to_string())?;

    progress(0.02, "Preparando carpetas...");

    let runtime_zip = temp.join(RUNTIME_ZIP);
    let runtime_url = github::aurexalis_runtime_url(version);
    progress(0.05, "Descargando runtime Aurexalis...");
    github::download_file(
        &client,
        &runtime_url,
        &runtime_zip,
        &|ratio, msg| progress(0.05 + ratio * 0.35, msg),
    )?;

    progress(0.42, "Extrayendo componentes...");
    extract_zip(&runtime_zip, install_root)?;
    let _ = fs::remove_file(&runtime_zip);

    let profile_dir = install_root.join("profiles").join("default");
    fs::create_dir_all(&profile_dir).map_err(|e| e.to_string())?;
    profile::apply_browser_pack(install_root, &profile_dir)?;

    let engine_dir = install_root.join("Engine");
    let browser = if download_floorp {
        progress(0.48, "Descargando motor Floorp...");
        let floorp_url = github::floorp_installer_url(&client)?;
        let floorp_path = temp.join(FLOORP_INSTALLER);
        github::download_file(
            &client,
            &floorp_url,
            &floorp_path,
            &|ratio, msg| progress(0.48 + ratio * 0.28, msg),
        )?;

        progress(0.78, "Instalando motor Gecko (Floorp)...");
        floorp::run_floorp_installer(&floorp_path, &engine_dir)?;
        let _ = fs::remove_file(&floorp_path);

        floorp::resolve_floorp_binary(&engine_dir)
            .ok_or("Floorp se instalo pero no se encontro floorp.exe".to_string())?
    } else {
        floorp::resolve_floorp_binary(&engine_dir).ok_or(
            "No se encontro Floorp. Activa la descarga del motor o instala Floorp manualmente.",
        )?
    };

    progress(0.88, "Guardando configuracion...");
    let config = InstallConfig {
        version: version.to_string(),
        install_root: install_root.to_string_lossy().into_owned(),
        browser: browser.to_string_lossy().into_owned(),
        profile: profile_dir.to_string_lossy().into_owned(),
    };
    write_config(install_root, &config)?;

    progress(0.92, "Creando acceso directo...");
    let launcher = install_root.join("aurexalis.exe");
    windows::create_desktop_shortcut(
        "Aurexalis",
        &launcher,
        install_root,
        Some("--launch-installed"),
    )?;

    let _ = fs::remove_dir_all(&temp);
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
