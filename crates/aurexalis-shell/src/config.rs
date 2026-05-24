//! Configuracion escrita por el instalador (`config.json` junto al ejecutable).

use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct InstallConfig {
    pub browser: PathBuf,
    pub profile: PathBuf,
    pub launcher: Option<PathBuf>,
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
    let launcher = value
        .get("launcher")
        .and_then(|v| v.as_str())
        .map(PathBuf::from);

    Ok(InstallConfig {
        browser,
        profile,
        launcher,
    })
}

fn parse_path(value: &serde_json::Value, key: &str) -> Result<PathBuf, String> {
    value
        .get(key)
        .and_then(|v| v.as_str())
        .map(PathBuf::from)
        .ok_or_else(|| format!("falta campo `{key}` en config.json"))
}

#[cfg(test)]
mod tests {
    use super::load;
    use std::fs;
    use std::path::PathBuf;

    #[test]
    fn load_accepts_optional_launcher() {
        let dir = std::env::temp_dir().join("aurexalis-config-test");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        fs::write(
            dir.join("config.json"),
            r#"{
  "browser": "C:/Aurexalis/Engine/floorp.exe",
  "profile": "C:/Aurexalis/profiles/default",
  "launcher": "C:/Aurexalis/aurexalis.exe"
}"#,
        )
        .unwrap();

        let cfg = load(&dir).expect("config");
        assert_eq!(cfg.browser, PathBuf::from("C:/Aurexalis/Engine/floorp.exe"));
        assert_eq!(
            cfg.launcher.as_deref(),
            Some(PathBuf::from("C:/Aurexalis/aurexalis.exe").as_path())
        );

        let _ = fs::remove_dir_all(dir);
    }
}
