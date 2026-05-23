//! Configuracion escrita por el instalador (`config.json` junto al ejecutable).

use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct InstallConfig {
    pub browser: PathBuf,
    pub profile: PathBuf,
}

/// Carga `config.json` desde el directorio de instalacion.
pub fn load(install_root: &Path) -> Result<InstallConfig, String> {
    let path = install_root.join("config.json");
    let raw = fs::read_to_string(&path)
        .map_err(|e| format!("no se pudo leer {}: {e}", path.display()))?;

    let value: serde_json::Value =
        serde_json::from_str(&raw).map_err(|e| format!("config.json invalido: {e}"))?;

    let browser = parse_path(&value, "browser")?;
    let profile = parse_path(&value, "profile")?;

    Ok(InstallConfig { browser, profile })
}

fn parse_path(value: &serde_json::Value, key: &str) -> Result<PathBuf, String> {
    value
        .get(key)
        .and_then(|v| v.as_str())
        .map(PathBuf::from)
        .ok_or_else(|| format!("falta campo `{key}` en config.json"))
}
