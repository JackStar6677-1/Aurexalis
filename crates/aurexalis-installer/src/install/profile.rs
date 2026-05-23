//! Aplica chrome y prefs Aurexalis al perfil Firefox/Floorp.

use std::fs;
use std::path::Path;

/// Copia `browser/chrome` y `browser/prefs` del runtime al perfil destino.
pub fn apply_browser_pack(install_root: &Path, profile_dir: &Path) -> Result<(), String> {
    let chrome_src = install_root.join("browser").join("chrome");
    let prefs_src = install_root.join("browser").join("prefs").join("user.js");
    let chrome_dst = profile_dir.join("chrome");

    if chrome_src.is_dir() {
        copy_dir_recursive(&chrome_src, &chrome_dst)?;
    }

    if prefs_src.is_file() {
        fs::copy(&prefs_src, profile_dir.join("user.js"))
            .map_err(|e| format!("copiar user.js: {e}"))?;
    }

    Ok(())
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<(), String> {
    fs::create_dir_all(dst).map_err(|e| e.to_string())?;
    for entry in fs::read_dir(src).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let from = entry.path();
        let to = dst.join(entry.file_name());
        if from.is_dir() {
            copy_dir_recursive(&from, &to)?;
        } else {
            fs::copy(&from, &to).map_err(|e| format!("copiar {}: {e}", from.display()))?;
        }
    }
    Ok(())
}
