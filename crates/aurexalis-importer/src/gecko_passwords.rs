//! Staging local de contrasenas para importacion manual en Gecko.
//!
//! Firefox cifra `logins.json` con NSS (`key4.db`). Escribir credenciales en claro
//! en esos archivos romperia el almacen. Aurexalis genera CSV + manifest solo con
//! consentimiento explicito (`--passwords-consent`).

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::{ImporterError, LoginRecord, SecretValue};

/// Opciones de staging de contrasenas (solo disco local).
#[derive(Debug, Clone, Copy)]
pub struct PasswordStagingOptions {
    /// El usuario acepto escribir credenciales en texto claro bajo `profile/import/`.
    pub user_consent: bool,
}

/// Resultado del staging de contrasenas.
#[derive(Debug, Clone, Default)]
pub struct PasswordStagingReport {
    pub logins_staged: usize,
    pub logins_skipped: usize,
    pub staging_dir: Option<PathBuf>,
    pub csv_path: Option<PathBuf>,
    pub manifest_path: Option<PathBuf>,
}

/// Escribe `passwords-import.csv` y `passwords-import.manifest.json` en el perfil.
pub fn stage_passwords_for_manual_import(
    profile_dir: &Path,
    logins: &[LoginRecord],
    options: PasswordStagingOptions,
) -> Result<PasswordStagingReport, ImporterError> {
    if !options.user_consent {
        return Err(ImporterError::Io(std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            "importacion de contrasenas requiere --passwords-consent (solo local)",
        )));
    }

    let staging_dir = profile_dir.join("import");
    fs::create_dir_all(&staging_dir)?;

    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let csv_path = staging_dir.join(format!("passwords-import-{stamp}.csv"));
    let manifest_path = staging_dir.join(format!("passwords-import-{stamp}.manifest.json"));

    let mut lines = vec!["url,username,password".to_owned()];
    let mut staged = 0usize;
    let mut skipped = 0usize;

    for login in logins {
        let Some(password) = login_plain_password(login) else {
            skipped += 1;
            continue;
        };
        if login.origin_url.is_empty() || login.username_value.is_empty() {
            skipped += 1;
            continue;
        }
        lines.push(format!(
            "{},{},{}",
            csv_escape(&login.origin_url),
            csv_escape(&login.username_value),
            csv_escape(&password),
        ));
        staged += 1;
    }

    fs::write(&csv_path, lines.join("\n"))?;
    let manifest = serde_json::json!({
        "version": 1,
        "source": "aurexalis-importer",
        "staged_at_unix": stamp,
        "logins_staged": staged,
        "logins_skipped": skipped,
        "csv": csv_path.file_name().and_then(|n| n.to_str()),
        "instructions_es": "En about:logins → Importar desde archivo, o usa el asistente de Firefox. No subas este CSV a red.",
        "nss_direct_write": "deferred — requiere integracion key4.db/NSS en fase posterior",
    });
    fs::write(&manifest_path, serde_json::to_string_pretty(&manifest)?)?;

    Ok(PasswordStagingReport {
        logins_staged: staged,
        logins_skipped: skipped,
        staging_dir: Some(staging_dir),
        csv_path: Some(csv_path),
        manifest_path: Some(manifest_path),
    })
}

fn login_plain_password(login: &LoginRecord) -> Option<String> {
    match &login.password_value {
        SecretValue::PlainText(value) | SecretValue::Decrypted(value) => {
            if value.is_empty() {
                None
            } else {
                Some(value.clone())
            }
        }
        SecretValue::Encrypted(_) => None,
    }
}

fn csv_escape(value: &str) -> String {
    if value.contains(',') || value.contains('"') || value.contains('\n') {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn requires_consent() {
        let dir = std::env::temp_dir().join("aurexalis-password-consent");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("dir");

        let err = stage_passwords_for_manual_import(
            &dir,
            &[],
            PasswordStagingOptions {
                user_consent: false,
            },
        )
        .expect_err("no consent");

        assert!(matches!(err, ImporterError::Io(_)));
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn stages_csv_with_consent() {
        let dir = std::env::temp_dir().join("aurexalis-password-stage");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("dir");

        let report = stage_passwords_for_manual_import(
            &dir,
            &[LoginRecord {
                origin_url: "https://example.test".to_owned(),
                action_url: String::new(),
                username_element: String::new(),
                username_value: "jack".to_owned(),
                password_element: String::new(),
                password_value: SecretValue::Decrypted("secret".to_owned()),
                date_created: 0,
            }],
            PasswordStagingOptions { user_consent: true },
        )
        .expect("stage");

        assert_eq!(report.logins_staged, 1);
        assert!(report.csv_path.as_ref().is_some_and(|p| p.is_file()));

        let _ = fs::remove_dir_all(dir);
    }
}
