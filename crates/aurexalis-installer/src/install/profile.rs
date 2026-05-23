//! Aplica chrome y prefs Aurexalis al perfil Firefox/Floorp.

use std::fs;
use std::path::{Path, PathBuf};

/// Copia `browser/chrome`, `browser/home` y prefs al perfil destino.
pub fn apply_browser_pack(install_root: &Path, profile_dir: &Path) -> Result<(), String> {
    let chrome_src = install_root.join("browser").join("chrome");
    let prefs_src = install_root.join("browser").join("prefs").join("user.js");
    let home_src = install_root.join("browser").join("home");
    let chrome_dst = profile_dir.join("chrome");

    if chrome_src.is_dir() {
        copy_dir_recursive(&chrome_src, &chrome_dst)?;
    }

    if home_src.is_dir() {
        copy_dir_recursive(&home_src, &profile_dir.join("home"))?;
    }

    if prefs_src.is_file() {
        fs::copy(&prefs_src, profile_dir.join("user.js"))
            .map_err(|e| format!("copiar user.js: {e}"))?;
    }

    if home_src.is_dir() {
        write_home_prefs(profile_dir)?;
    }

    Ok(())
}

/// Registra la pagina local como inicio y nueva pestana.
fn write_home_prefs(profile_dir: &Path) -> Result<(), String> {
    let home_index = profile_dir.join("home").join("index.html");
    if !home_index.is_file() {
        return Ok(());
    }

    let uri = file_uri(&home_index)?;
    let prefs_path = profile_dir.join("user.js");
    let mut contents = if prefs_path.is_file() {
        fs::read_to_string(&prefs_path).map_err(|e| e.to_string())?
    } else {
        String::new()
    };

    if contents.contains("Aurexalis custom home") {
        return Ok(());
    }

    contents.push_str("\n// Aurexalis custom home (generated at install)\n");
    contents.push_str("user_pref(\"browser.newtabpage.enabled\", false);\n");
    contents.push_str(&format!("user_pref(\"browser.newtab.url\", \"{uri}\");\n"));
    contents.push_str(&format!("user_pref(\"browser.startup.homepage\", \"{uri}\");\n"));
    contents.push_str("user_pref(\"browser.startup.page\", 1);\n");
    contents.push_str("user_pref(\"browser.startup.firstrunSkipsHomepage\", false);\n");
    contents.push_str("user_pref(\"browser.aboutHomeSnippets.updateUrl\", \"\");\n");
    contents.push_str("user_pref(\"browser.newtabpage.activity-stream.feeds.section.topstories\", false);\n");

    fs::write(&prefs_path, contents).map_err(|e| format!("escribir user.js: {e}"))
}

/// Convierte una ruta local en URI `file:///` compatible con Gecko.
fn file_uri(path: &Path) -> Result<String, String> {
    let abs = fs::canonicalize(path).map_err(|e| format!("canonicalizar {}: {e}", path.display()))?;
    let mut normalized = abs.to_string_lossy().replace('\\', "/");
    if let Some(stripped) = normalized.strip_prefix("//?/") {
        normalized = stripped.to_string();
    }
    Ok(format!("file:///{normalized}"))
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

#[cfg(test)]
mod tests {
    use super::file_uri;
    use std::fs;

    #[test]
    fn file_uri_uses_forward_slashes() {
        let dir = std::env::temp_dir().join("aurexalis-home-test");
        fs::create_dir_all(&dir).unwrap();
        let file = dir.join("index.html");
        fs::write(&file, "<html></html>").unwrap();

        let uri = file_uri(&file).expect("uri");
        assert!(uri.starts_with("file:///"));
        assert!(!uri.contains('\\'));

        let _ = fs::remove_dir_all(dir);
    }
}
