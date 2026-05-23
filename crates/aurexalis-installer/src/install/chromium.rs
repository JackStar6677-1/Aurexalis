//! Exportacion local de datos Chromium tras la instalacion.

use aurexalis_importer::{export_audit_snapshot, find_first_chromium_profile, AuditExportOptions};
use std::path::Path;

/// Exporta marcadores, historial y cookies (sin contrasenas) al perfil Aurexalis.
pub fn export_staging_snapshot(profile_dir: &Path) -> Result<String, String> {
    let candidate = find_first_chromium_profile()
        .ok_or("no se encontro un perfil local de Chrome, Brave u Opera")?;

    let destination = profile_dir.join("import").join("chromium-audit.json");
    export_audit_snapshot(&candidate, &destination, AuditExportOptions::default())
        .map_err(|error| error.to_string())?;

    Ok(format!(
        "{:?} / {} → {}",
        candidate.browser,
        candidate.profile_name,
        destination.display()
    ))
}
