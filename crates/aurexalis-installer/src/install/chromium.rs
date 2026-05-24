//! Exportacion local de datos Chromium tras la instalacion.

use aurexalis_importer::{export_audit_snapshot, find_first_chromium_profile, AuditExportOptions};
use std::path::Path;

/// Exporta marcadores, historial y cookies; contrasenas solo si se solicita.
pub fn export_staging_snapshot(
    profile_dir: &Path,
    include_passwords: bool,
) -> Result<String, String> {
    let candidate = find_first_chromium_profile()
        .ok_or("no se encontro un perfil local de Chrome, Brave u Opera")?;

    let destination = profile_dir.join("import").join("chromium-audit.json");
    export_audit_snapshot(
        &candidate,
        &destination,
        AuditExportOptions { include_passwords },
    )
    .map_err(|error| error.to_string())?;

    let kind = if include_passwords {
        "datos + contrasenas"
    } else {
        "datos (sin contrasenas)"
    };

    Ok(format!(
        "{kind}: {:?} / {} → {}",
        candidate.browser,
        candidate.profile_name,
        destination.display()
    ))
}

#[cfg(test)]
mod tests {
    use super::export_staging_snapshot;
    use std::path::PathBuf;

    #[test]
    fn export_staging_snapshot_requires_chromium_profile() {
        let profile = PathBuf::from("profiles/default-test-missing-chromium");
        let result = export_staging_snapshot(&profile, false);
        assert!(result.is_err());
    }
}
