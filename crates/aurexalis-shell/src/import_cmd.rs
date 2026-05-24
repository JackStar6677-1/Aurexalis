//! Comandos `aurexalis import` para inventario, exportacion y aplicacion al perfil Gecko.

use aurexalis_importer::{
    apply_snapshot_to_profile, default_profile_roots, discover_profiles, export_audit_snapshot,
    find_first_chromium_profile, load_audit_snapshot, ApplySurface, AuditExportOptions,
    ChromiumBrowser,
};
use std::env;
use std::path::{Path, PathBuf};

/// Lista perfiles Chromium detectados en el sistema.
pub fn list_profiles() -> Result<(), String> {
    let mut found = false;
    for browser in [
        ChromiumBrowser::Brave,
        ChromiumBrowser::Chrome,
        ChromiumBrowser::Opera,
    ] {
        for root in default_profile_roots(browser) {
            match discover_profiles(browser, &root) {
                Ok(profiles) => {
                    for profile in profiles {
                        found = true;
                        println!(
                            "[IMPORT] {:?} {} {}",
                            profile.browser,
                            profile.profile_name,
                            profile.root.display()
                        );
                        for surface in profile.existing_surfaces() {
                            println!("  - {surface:?}: {}", profile.path_for(surface).display());
                        }
                    }
                }
                Err(_) => {
                    println!("[MISS] {:?} {}", browser, root.display());
                }
            }
        }
    }

    if found {
        Ok(())
    } else {
        Err("no se encontraron perfiles Chromium importables".to_string())
    }
}

/// Exporta snapshot auditable al path indicado o al perfil instalado.
pub fn export_audit(output: Option<PathBuf>, include_passwords: bool) -> Result<(), String> {
    let candidate = find_first_chromium_profile()
        .ok_or("no se encontro perfil Chrome, Brave u Opera en este equipo")?;

    let destination = output.unwrap_or_else(default_audit_path);
    if let Some(parent) = destination.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    let snapshot = export_audit_snapshot(
        &candidate,
        &destination,
        AuditExportOptions { include_passwords },
    )
    .map_err(|e| e.to_string())?;

    println!(
        "[SUCCESS] {:?} / {} → {}",
        candidate.browser,
        candidate.profile_name,
        destination.display()
    );
    println!(
        "[INFO] cookies={} logins={} history={} bookmarks={}",
        snapshot.cookies.len(),
        snapshot.logins.len(),
        snapshot.history.len(),
        snapshot.bookmarks.len()
    );
    Ok(())
}

/// Aplica JSON auditable al perfil Gecko (marcadores + historial). Navegador cerrado.
pub fn apply_audit(
    audit_path: Option<PathBuf>,
    profile_dir: Option<PathBuf>,
    surfaces: &[ApplySurface],
) -> Result<(), String> {
    let audit = audit_path.unwrap_or_else(default_audit_path);
    let profile = profile_dir.unwrap_or_else(default_profile_path);

    if !profile.is_dir() {
        return Err(format!(
            "perfil Gecko inexistente: {} (usa --profile o instala Aurexalis)",
            profile.display()
        ));
    }

    let snapshot = load_audit_snapshot(&audit).map_err(|e| e.to_string())?;
    let report =
        apply_snapshot_to_profile(&profile, &snapshot, surfaces).map_err(|e| e.to_string())?;

    println!("[SUCCESS] Importacion aplicada a {}", profile.display());
    println!(
        "[INFO] bookmarks={} history={}",
        report.bookmarks_added, report.history_added
    );
    if let Some(backup) = report.backup_dir {
        println!("[INFO] backup places.sqlite en {}", backup.display());
    }
    Ok(())
}

fn default_audit_path() -> PathBuf {
    install_root()
        .map(|root| {
            root.join("profiles")
                .join("default")
                .join("import")
                .join("chromium-audit.json")
        })
        .unwrap_or_else(|| PathBuf::from("chromium-audit.json"))
}

fn default_profile_path() -> PathBuf {
    install_root()
        .map(|root| root.join("profiles").join("default"))
        .unwrap_or_else(|| {
            env::current_dir()
                .unwrap_or_else(|_| PathBuf::from("."))
                .join("profiles")
                .join("default")
        })
}

fn install_root() -> Option<PathBuf> {
    env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(Path::to_path_buf))
}
