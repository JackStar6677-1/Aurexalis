//! Aplica chrome, home, ajustes y prefs Aurexalis al perfil Gecko.

use std::fs;
use std::path::Path;

/// Copia el pack del navegador y escribe preferencias Aurexalis en el perfil.
pub fn apply_browser_pack(install_root: &Path, profile_dir: &Path) -> Result<(), String> {
    let browser_root = install_root.join("browser");
    let chrome_src = browser_root.join("chrome");
    let prefs_src = browser_root.join("prefs").join("user.js");
    let home_src = browser_root.join("home");
    let settings_src = browser_root.join("settings");
    let chrome_dst = profile_dir.join("chrome");

    if chrome_src.is_dir() {
        copy_dir_recursive(&chrome_src, &chrome_dst)?;
    }

    if home_src.is_dir() {
        copy_dir_recursive(&home_src, &profile_dir.join("home"))?;
    }

    if settings_src.is_dir() {
        copy_dir_recursive(&settings_src, &profile_dir.join("settings"))?;
    }

    if prefs_src.is_file() {
        fs::copy(&prefs_src, profile_dir.join("user.js"))
            .map_err(|e| format!("copiar user.js: {e}"))?;
    }

    write_aurexalis_prefs(install_root, profile_dir)?;

    Ok(())
}

/// Registra home, ajustes, launcher y defaults de identidad Aurexalis.
fn write_aurexalis_prefs(install_root: &Path, profile_dir: &Path) -> Result<(), String> {
    let prefs_path = profile_dir.join("user.js");
    let mut contents = if prefs_path.is_file() {
        fs::read_to_string(&prefs_path).map_err(|e| e.to_string())?
    } else {
        String::new()
    };

    if !contents.contains("Aurexalis custom home") {
        if let Ok(uri) = file_uri(&profile_dir.join("home").join("index.html")) {
            contents.push_str("\n// Aurexalis custom home (generated at install)\n");
            contents.push_str("user_pref(\"browser.newtabpage.enabled\", false);\n");
            contents.push_str(&format!("user_pref(\"browser.newtab.url\", \"{uri}\");\n"));
            contents.push_str(&format!("user_pref(\"browser.startup.homepage\", \"{uri}\");\n"));
            contents.push_str("user_pref(\"browser.startup.page\", 1);\n");
            contents.push_str("user_pref(\"browser.startup.firstrunSkipsHomepage\", false);\n");
            contents.push_str("user_pref(\"browser.aboutHomeSnippets.updateUrl\", \"\");\n");
            contents.push_str(
                "user_pref(\"browser.newtabpage.activity-stream.feeds.section.topstories\", false);\n",
            );
        }
    }

    if !contents.contains("Aurexalis runtime prefs") {
        contents.push_str("\n// Aurexalis runtime prefs (generated at install)\n");

        let launcher = install_root.join("aurexalis.exe");
        if launcher.is_file() {
            let launcher_uri = path_pref_value(&launcher)?;
            contents.push_str(&format!("user_pref(\"aurexalis.shell.path\", \"{launcher_uri}\");\n"));
        }

        if let Ok(settings_uri) = file_uri(&profile_dir.join("settings").join("index.html")) {
            contents.push_str(&format!(
                "user_pref(\"aurexalis.settings.url\", \"{settings_uri}\");\n"
            ));
        }
    }

    if !contents.contains("Aurexalis identity prefs") {
        contents.push_str("\n// Aurexalis identity prefs (generated at install)\n");
        contents.push_str("user_pref(\"svg.context-properties.content.enabled\", true);\n");
        contents.push_str(
            "user_pref(\"floorp.design.configs\", \"{\\\"uiCustomization\\\":{\\\"disableFloorpStart\\\":true}}\");\n",
        );
    }

    fs::write(&prefs_path, contents).map_err(|e| format!("escribir user.js: {e}"))
}

/// Reaplica prefs de runtime tras importacion o actualizacion del launcher.
pub fn refresh_runtime_prefs(install_root: &Path, profile_dir: &Path) -> Result<(), String> {
    write_aurexalis_prefs(install_root, profile_dir)
}

/// Convierte una ruta local en URI `file:///` compatible con Gecko.
fn file_uri(path: &Path) -> Result<String, String> {
    if !path.is_file() {
        return Err(format!("archivo inexistente: {}", path.display()));
    }
    let abs = fs::canonicalize(path).map_err(|e| format!("canonicalizar {}: {e}", path.display()))?;
    Ok(path_to_file_uri(&abs))
}

/// Escapa una ruta Windows para user_pref (barras y comillas).
fn path_pref_value(path: &Path) -> Result<String, String> {
    let abs = fs::canonicalize(path).map_err(|e| format!("canonicalizar {}: {e}", path.display()))?;
    Ok(abs.to_string_lossy().replace('\\', "\\\\"))
}

fn path_to_file_uri(path: &Path) -> String {
    let mut normalized = path.to_string_lossy().replace('\\', "/");
    if let Some(stripped) = normalized.strip_prefix("//?/") {
        normalized = stripped.to_string();
    }
    format!("file:///{normalized}")
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
    use super::{apply_browser_pack, file_uri, path_pref_value, path_to_file_uri, refresh_runtime_prefs};
    use std::fs;
    use std::path::Path;

    fn write_min_browser_pack(root: &Path) {
        let browser = root.join("browser");
        fs::create_dir_all(browser.join("chrome")).unwrap();
        fs::write(
            browser.join("chrome").join("aurexalis-00-core.uc.js"),
            "// core",
        )
        .unwrap();
        fs::create_dir_all(browser.join("home")).unwrap();
        fs::write(browser.join("home").join("index.html"), "<html></html>").unwrap();
        fs::create_dir_all(browser.join("settings")).unwrap();
        fs::write(browser.join("settings").join("index.html"), "<html></html>").unwrap();
        fs::create_dir_all(browser.join("prefs")).unwrap();
        fs::write(
            browser.join("prefs").join("user.js"),
            "user_pref(\"aurexalis.sounds.enabled\", true);\n",
        )
        .unwrap();
        fs::write(root.join("aurexalis.exe"), b"").unwrap();
    }

    #[test]
    fn apply_browser_pack_copies_chrome_home_settings_and_prefs() {
        let root = std::env::temp_dir().join("aurexalis-pack-test");
        let profile = root.join("profiles").join("default");
        let _ = fs::remove_dir_all(&root);
        write_min_browser_pack(&root);

        apply_browser_pack(&root, &profile).expect("apply pack");

        assert!(profile.join("chrome").join("aurexalis-00-core.uc.js").is_file());
        assert!(profile.join("home").join("index.html").is_file());
        assert!(profile.join("settings").join("index.html").is_file());

        let user_js = fs::read_to_string(profile.join("user.js")).expect("user.js");
        assert!(user_js.contains("aurexalis.shell.path"));
        assert!(user_js.contains("aurexalis.settings.url"));
        assert!(user_js.contains("browser.newtab.url"));
        assert!(user_js.contains("disableFloorpStart"));

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn refresh_runtime_prefs_is_idempotent() {
        let root = std::env::temp_dir().join("aurexalis-refresh-test");
        let profile = root.join("profiles").join("default");
        let _ = fs::remove_dir_all(&root);
        write_min_browser_pack(&root);
        apply_browser_pack(&root, &profile).expect("apply pack");

        let before = fs::read_to_string(profile.join("user.js")).expect("user.js");
        refresh_runtime_prefs(&root, &profile).expect("refresh");
        let after = fs::read_to_string(profile.join("user.js")).expect("user.js");
        assert_eq!(before, after);

        let _ = fs::remove_dir_all(root);
    }

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

    #[test]
    fn path_pref_value_escapes_backslashes() {
        let dir = std::env::temp_dir().join("aurexalis-launcher-test");
        fs::create_dir_all(&dir).unwrap();
        let exe = dir.join("aurexalis.exe");
        fs::write(&exe, b"").unwrap();

        let value = path_pref_value(&exe).expect("path");
        assert!(!value.contains('/'));
        assert!(!value.contains("\\\\\\"));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn path_to_file_uri_normalizes() {
        let uri = path_to_file_uri(Path::new("C:/Aurexalis/home/index.html"));
        assert_eq!(uri, "file:///C:/Aurexalis/home/index.html");
    }
}
