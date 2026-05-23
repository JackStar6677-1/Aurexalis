//! Comandos `aurexalis import` para inventario y exportacion local.

use aurexalis_importer::{
    default_profile_roots, discover_profiles, export_audit_snapshot,
    find_first_chromium_profile, AuditExportOptions, ChromiumBrowser,
};
use std::env;
use std::path::PathBuf;

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

    let destination = output.unwrap_or_else(|| default_audit_path());
    if let Some(parent) = destination.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    let snapshot = export_audit_snapshot(
        &candidate,
        &destination,
        AuditExportOptions {
            include_passwords,
        },
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

fn default_audit_path() -> PathBuf {
    if let Ok(exe) = env::current_exe() {
        if let Some(root) = exe.parent() {
            return root
                .join("profiles")
                .join("default")
                .join("import")
                .join("chromium-audit.json");
        }
    }
    PathBuf::from("chromium-audit.json")
}
