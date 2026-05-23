//! Instalacion silenciosa del motor Floorp (Gecko).

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// Ejecuta el instalador oficial de Floorp en modo silencioso para el usuario actual.
pub fn run_floorp_installer(installer: &Path, engine_dir: &Path) -> Result<(), String> {
    std::fs::create_dir_all(engine_dir).map_err(|e| e.to_string())?;

    let engine = engine_dir
        .to_string_lossy()
        .replace('\'', "''")
        .replace('"', "");

    let installer_path = installer
        .to_string_lossy()
        .replace('\'', "''")
        .replace('"', "");

    let script = format!(
        r#"
$ErrorActionPreference = 'Stop'
$args = @('/S', '/CURRENTUSER', '/InstallDirectoryPath={engine}')
$p = Start-Process -FilePath '{installer_path}' -ArgumentList $args -Wait -PassThru
exit $p.ExitCode
"#
    );

    let status = Command::new("powershell.exe")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            &script,
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .status()
        .map_err(|e| format!("no se pudo lanzar el instalador Floorp: {e}"))?;

    if status.success() {
        Ok(())
    } else {
        Err(format!(
            "el instalador Floorp termino con codigo {:?}",
            status.code()
        ))
    }
}

/// Busca `floorp.exe` dentro del directorio del motor o rutas habituales.
pub fn resolve_floorp_binary(engine_dir: &Path) -> Option<PathBuf> {
    let candidates = [
        engine_dir.join("floorp.exe"),
        engine_dir.join("Ablaze Floorp").join("floorp.exe"),
    ];

    for path in candidates {
        if path.is_file() {
            return Some(path);
        }
    }

    if let Some(local) = dirs::data_local_dir() {
        let user_path = local.join("Ablaze Floorp").join("floorp.exe");
        if user_path.is_file() {
            return Some(user_path);
        }
    }

    if let Ok(pf) = std::env::var("ProgramFiles") {
        let path = PathBuf::from(pf).join("Ablaze Floorp").join("floorp.exe");
        if path.is_file() {
            return Some(path);
        }
    }

    None
}
