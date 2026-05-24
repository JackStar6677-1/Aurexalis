//! Copia el motor Gecko y aplica icono/nombre Windows visibles como Aurexalis.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const BRAND_SCRIPT: &str = include_str!("../../../../tools/brand-engine.ps1");
const BRANDED_EXE: &str = "aurexalis-browser.exe";

/// Nombre del ejecutable con metadatos Aurexalis (icono y descripcion en Windows).
pub fn branded_binary_name() -> &'static str {
    BRANDED_EXE
}

/// Ruta preferida del motor: copia marcada si existe, si no el binario Floorp original.
pub fn resolve_engine_binary(engine_dir: &Path, floorp_source: &Path) -> PathBuf {
    let branded = engine_dir.join(branded_binary_name());
    if branded.is_file() {
        return branded;
    }
    floorp_source.to_path_buf()
}

/// Genera `aurexalis-browser.exe` a partir de `floorp.exe` con icono Aurexalis.
pub fn brand_engine_binary(
    floorp_source: &Path,
    engine_dir: &Path,
    icon_path: &Path,
) -> Result<PathBuf, String> {
    let dest = engine_dir.join(branded_binary_name());
    if dest.is_file() {
        return Ok(dest);
    }

    fs::create_dir_all(engine_dir).map_err(|e| e.to_string())?;

    let script_path = std::env::temp_dir().join("aurexalis-brand-engine.ps1");
    fs::write(&script_path, BRAND_SCRIPT).map_err(|e| format!("escribir script de marca: {e}"))?;

    let cache_dir = engine_dir.join(".brand-cache");
    let status = Command::new("powershell.exe")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-ExecutionPolicy",
            "Bypass",
            "-File",
            script_path.to_string_lossy().as_ref(),
            "-SourceExe",
            &floorp_source.to_string_lossy(),
            "-DestExe",
            &dest.to_string_lossy(),
            "-IconPath",
            &icon_path.to_string_lossy(),
            "-CacheDir",
            &cache_dir.to_string_lossy(),
        ])
        .status()
        .map_err(|e| format!("brand-engine.ps1: {e}"))?;

    if status.success() && dest.is_file() {
        return Ok(dest);
    }

    fs::copy(floorp_source, &dest)
        .map_err(|e| format!("copiar motor a {}: {e}", dest.display()))?;
    Ok(dest)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn branded_binary_name_is_stable() {
        assert_eq!(branded_binary_name(), "aurexalis-browser.exe");
    }
}
