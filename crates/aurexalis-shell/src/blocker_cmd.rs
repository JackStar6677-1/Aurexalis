//! Comandos `aurexalis blocker` — comprobacion adblock-rust y sincronizacion de listas.

use aurexalis_blocker::{BlockerEngine, PROFILE_FILTER_REL_PATH};
use aurexalis_core::{NetworkRequest, ResourceKind};
use std::fs;
use std::path::PathBuf;

const DEFAULT_LISTS: &[&str] = &[
    "||doubleclick.net^",
    "||googlesyndication.com^",
    "||googleadservices.com^",
    "||adservice.google.com^",
    "||facebook.com/tr^",
    "||analytics.google.com^",
    "/ads/*",
    "||taboola.com^",
    "||outbrain.com^",
];

/// Comprueba si una URL seria bloqueada por las listas del perfil.
pub fn check_url(url: &str, source: Option<&str>, kind: ResourceKind) -> Result<(), String> {
    let engine = load_engine(default_lists_path())?;
    let request = NetworkRequest::parse(url, source, kind).map_err(|e| e.to_string())?;
    let decision = engine.check(&request).map_err(|e| e.to_string())?;
    println!("[BLOCKER] {url} → {decision:?}");
    Ok(())
}

/// Escribe listas por defecto en el perfil para uso futuro del hook Gecko.
pub fn sync_lists(profile_dir: &std::path::Path) -> Result<(), String> {
    let dest = profile_dir.join(PROFILE_FILTER_REL_PATH);
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let body = DEFAULT_LISTS.join("\n");
    fs::write(&dest, body).map_err(|e| e.to_string())?;
    println!("[SUCCESS] Listas en {}", dest.display());
    Ok(())
}

fn load_engine(lists_path: PathBuf) -> Result<BlockerEngine, String> {
    let rules = if lists_path.is_file() {
        vec![fs::read_to_string(&lists_path).map_err(|e| e.to_string())?]
    } else {
        vec![DEFAULT_LISTS.join("\n")]
    };
    BlockerEngine::from_filter_lists(&rules).map_err(|e| e.to_string())
}

fn default_lists_path() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|exe| {
            exe.parent()
                .map(|p| p.join("profiles").join("default").join(PROFILE_FILTER_REL_PATH))
        })
        .unwrap_or_else(|| PathBuf::from(PROFILE_FILTER_REL_PATH))
}

pub fn parse_resource_kind(value: &str) -> Result<ResourceKind, String> {
    match value.to_lowercase().as_str() {
        "document" => Ok(ResourceKind::Document),
        "script" => Ok(ResourceKind::Script),
        "stylesheet" | "css" => Ok(ResourceKind::Stylesheet),
        "image" => Ok(ResourceKind::Image),
        "media" => Ok(ResourceKind::Media),
        "font" => Ok(ResourceKind::Font),
        "xhr" | "fetch" => Ok(ResourceKind::Xhr),
        _ => Ok(ResourceKind::Other),
    }
}
