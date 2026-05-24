//! Local browser profile discovery for Aurexalis.
//!
//! This crate only maps and inventories local profile artifacts. Decryption and
//! migration must remain explicit, local and user-controlled.

#![forbid(unsafe_code)]

mod gecko_cookies;
mod gecko_passwords;
mod gecko_write;

#[cfg(target_os = "linux")]
mod linux_crypt;

pub use gecko_write::{
    apply_snapshot_to_profile, apply_snapshot_to_profile_with_options, load_audit_snapshot,
    ApplyOptions, ApplyReport, ApplySurface,
};

use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Nonce};
use base64::Engine;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug)]
pub enum ImporterError {
    ProfileRootMissing,
    UnsupportedBrowser(String),
    ReadDir(std::io::Error),
    Io(std::io::Error),
    Sqlite(rusqlite::Error),
    Json(serde_json::Error),
    Crypto(String),
    UnsupportedDecryption(&'static str),
}

impl fmt::Display for ImporterError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ImporterError::ProfileRootMissing => formatter.write_str("profile root was not found"),
            ImporterError::UnsupportedBrowser(value) => {
                write!(formatter, "browser profile is unsupported: {value}")
            }
            ImporterError::ReadDir(error) => {
                write!(formatter, "failed to read profile directory: {error}")
            }
            ImporterError::Io(error) => write!(formatter, "io error: {error}"),
            ImporterError::Sqlite(error) => write!(formatter, "sqlite error: {error}"),
            ImporterError::Json(error) => write!(formatter, "json error: {error}"),
            ImporterError::Crypto(error) => write!(formatter, "crypto error: {error}"),
            ImporterError::UnsupportedDecryption(value) => {
                write!(formatter, "unsupported decryption: {value}")
            }
        }
    }
}

impl std::error::Error for ImporterError {}

impl From<std::io::Error> for ImporterError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<rusqlite::Error> for ImporterError {
    fn from(error: rusqlite::Error) -> Self {
        Self::Sqlite(error)
    }
}

impl From<serde_json::Error> for ImporterError {
    fn from(error: serde_json::Error) -> Self {
        Self::Json(error)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChromiumBrowser {
    Brave,
    Chrome,
    Opera,
}

#[derive(Debug, Clone)]
pub struct ProfileCandidate {
    pub browser: ChromiumBrowser,
    pub profile_name: String,
    pub root: PathBuf,
    pub artifacts: ProfileArtifacts,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProfileArtifacts {
    pub cookies_db: PathBuf,
    pub login_db: PathBuf,
    pub history_db: PathBuf,
    pub bookmarks_json: PathBuf,
    pub favicons_db: PathBuf,
    pub preferences_json: PathBuf,
    pub secure_preferences_json: PathBuf,
    pub local_state_json: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CookieRecord {
    pub host_key: String,
    pub name: String,
    pub path: String,
    pub expires_utc: i64,
    pub is_secure: bool,
    pub is_httponly: bool,
    pub same_site: i64,
    pub value: SecretValue,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LoginRecord {
    pub origin_url: String,
    pub action_url: String,
    pub username_element: String,
    pub username_value: String,
    pub password_element: String,
    pub password_value: SecretValue,
    pub date_created: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub url: String,
    pub title: String,
    pub visit_count: i64,
    pub typed_count: i64,
    pub last_visit_time: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FaviconEntry {
    pub page_url: String,
    pub icon_url: String,
    pub width: i64,
    pub height: i64,
    pub image_data: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BookmarkEntry {
    pub name: String,
    pub url: String,
    pub path: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JsonArtifact {
    pub path: PathBuf,
    pub value: Value,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SecretValue {
    PlainText(String),
    Encrypted(Vec<u8>),
    Decrypted(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecryptionContext {
    pub platform_key: Option<Vec<u8>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProfileSnapshot {
    pub cookies: Vec<CookieRecord>,
    pub logins: Vec<LoginRecord>,
    pub history: Vec<HistoryEntry>,
    pub favicons: Vec<FaviconEntry>,
    pub bookmarks: Vec<BookmarkEntry>,
    pub preferences: Option<JsonArtifact>,
    pub secure_preferences: Option<JsonArtifact>,
    pub local_state: Option<JsonArtifact>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImportSurface {
    Cookies,
    Passwords,
    History,
    Bookmarks,
    Favicons,
    Preferences,
}

impl ProfileCandidate {
    pub fn path_for(&self, surface: ImportSurface) -> &Path {
        match surface {
            ImportSurface::Cookies => &self.artifacts.cookies_db,
            ImportSurface::Passwords => &self.artifacts.login_db,
            ImportSurface::History => &self.artifacts.history_db,
            ImportSurface::Bookmarks => &self.artifacts.bookmarks_json,
            ImportSurface::Favicons => &self.artifacts.favicons_db,
            ImportSurface::Preferences => &self.artifacts.preferences_json,
        }
    }

    pub fn existing_surfaces(&self) -> Vec<ImportSurface> {
        [
            ImportSurface::Cookies,
            ImportSurface::Passwords,
            ImportSurface::History,
            ImportSurface::Bookmarks,
            ImportSurface::Favicons,
            ImportSurface::Preferences,
        ]
        .into_iter()
        .filter(|surface| self.path_for(*surface).exists())
        .collect()
    }
}

pub fn default_profile_roots(browser: ChromiumBrowser) -> Vec<PathBuf> {
    let mut roots = Vec::new();

    if cfg!(target_os = "windows") {
        if let Some(local_app_data) = std::env::var_os("LOCALAPPDATA") {
            let base = PathBuf::from(local_app_data);
            match browser {
                ChromiumBrowser::Brave => {
                    roots.push(base.join("BraveSoftware/Brave-Browser/User Data"));
                }
                ChromiumBrowser::Chrome => {
                    roots.push(base.join("Google/Chrome/User Data"));
                }
                ChromiumBrowser::Opera => {
                    roots.push(base.join("Programs/Opera"));
                    roots.push(base.join("Opera Software/Opera Stable"));
                }
            }
        }
    }

    if cfg!(target_os = "linux") {
        if let Some(home) = std::env::var_os("HOME") {
            let base = PathBuf::from(home).join(".config");
            match browser {
                ChromiumBrowser::Brave => {
                    roots.push(base.join("BraveSoftware/Brave-Browser"));
                }
                ChromiumBrowser::Chrome => {
                    roots.push(base.join("google-chrome"));
                }
                ChromiumBrowser::Opera => {
                    roots.push(base.join("opera"));
                }
            }
        }
    }

    roots
}

pub fn build_default_profile_candidate(
    browser: ChromiumBrowser,
    root: PathBuf,
) -> Result<ProfileCandidate, ImporterError> {
    if !root.exists() {
        return Err(ImporterError::ProfileRootMissing);
    }

    let profile_root = match browser {
        ChromiumBrowser::Opera => root.clone(),
        ChromiumBrowser::Brave | ChromiumBrowser::Chrome => root.join("Default"),
    };

    Ok(build_candidate(browser, "Default".to_owned(), profile_root))
}

pub fn discover_profiles(
    browser: ChromiumBrowser,
    user_data_root: &Path,
) -> Result<Vec<ProfileCandidate>, ImporterError> {
    if !user_data_root.exists() {
        return Err(ImporterError::ProfileRootMissing);
    }

    if browser == ChromiumBrowser::Opera {
        return Ok(vec![build_candidate(
            browser,
            "Default".to_owned(),
            user_data_root.to_path_buf(),
        )]);
    }

    let mut profiles = Vec::new();
    for entry in fs::read_dir(user_data_root).map_err(ImporterError::ReadDir)? {
        let entry = entry.map_err(ImporterError::ReadDir)?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let Some(name) = path.file_name().and_then(|value| value.to_str()) else {
            continue;
        };

        if name == "Default" || name.starts_with("Profile ") {
            profiles.push(build_candidate(browser, name.to_owned(), path));
        }
    }

    profiles.sort_by(|left, right| left.profile_name.cmp(&right.profile_name));
    Ok(profiles)
}

pub fn read_profile_snapshot(
    candidate: &ProfileCandidate,
    decryption: Option<&DecryptionContext>,
) -> Result<ProfileSnapshot, ImporterError> {
    Ok(ProfileSnapshot {
        cookies: read_cookies(&candidate.artifacts.cookies_db, decryption)?,
        logins: read_logins(&candidate.artifacts.login_db, decryption)?,
        history: read_history(&candidate.artifacts.history_db)?,
        favicons: read_favicons(&candidate.artifacts.favicons_db)?,
        bookmarks: read_bookmarks(&candidate.artifacts.bookmarks_json)?,
        preferences: read_json_artifact(&candidate.artifacts.preferences_json)?,
        secure_preferences: read_json_artifact(&candidate.artifacts.secure_preferences_json)?,
        local_state: read_json_artifact(&candidate.artifacts.local_state_json)?,
    })
}

pub fn read_cookies(
    path: &Path,
    decryption: Option<&DecryptionContext>,
) -> Result<Vec<CookieRecord>, ImporterError> {
    if !path.exists() {
        return Ok(Vec::new());
    }

    let connection = Connection::open_with_flags(path, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)?;
    let mut statement = connection.prepare(
        "SELECT host_key, name, value, encrypted_value, path, expires_utc, is_secure, \
         is_httponly, samesite FROM cookies ORDER BY host_key, name",
    )?;
    let rows = statement.query_map([], |row| {
        let plain_value: String = row.get(2)?;
        let encrypted_value: Vec<u8> = row.get(3)?;
        Ok(CookieRecord {
            host_key: row.get(0)?,
            name: row.get(1)?,
            value: secret_from_columns(plain_value, encrypted_value, decryption),
            path: row.get(4)?,
            expires_utc: row.get(5)?,
            is_secure: row.get::<_, i64>(6)? != 0,
            is_httponly: row.get::<_, i64>(7)? != 0,
            same_site: row.get(8)?,
        })
    })?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(ImporterError::Sqlite)
}

pub fn read_logins(
    path: &Path,
    decryption: Option<&DecryptionContext>,
) -> Result<Vec<LoginRecord>, ImporterError> {
    if !path.exists() {
        return Ok(Vec::new());
    }

    let connection = Connection::open_with_flags(path, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)?;
    let mut statement = connection.prepare(
        "SELECT origin_url, action_url, username_element, username_value, password_element, \
         password_value, date_created FROM logins ORDER BY origin_url, username_value",
    )?;
    let rows = statement.query_map([], |row| {
        let encrypted_value: Vec<u8> = row.get(5)?;
        Ok(LoginRecord {
            origin_url: row.get(0)?,
            action_url: row.get(1)?,
            username_element: row.get(2)?,
            username_value: row.get(3)?,
            password_element: row.get(4)?,
            password_value: secret_from_columns(String::new(), encrypted_value, decryption),
            date_created: row.get(6)?,
        })
    })?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(ImporterError::Sqlite)
}

pub fn read_history(path: &Path) -> Result<Vec<HistoryEntry>, ImporterError> {
    if !path.exists() {
        return Ok(Vec::new());
    }

    let connection = Connection::open_with_flags(path, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)?;
    let mut statement = connection.prepare(
        "SELECT url, title, visit_count, typed_count, last_visit_time FROM urls \
         ORDER BY last_visit_time DESC",
    )?;
    let rows = statement.query_map([], |row| {
        Ok(HistoryEntry {
            url: row.get(0)?,
            title: row.get(1)?,
            visit_count: row.get(2)?,
            typed_count: row.get(3)?,
            last_visit_time: row.get(4)?,
        })
    })?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(ImporterError::Sqlite)
}

pub fn read_favicons(path: &Path) -> Result<Vec<FaviconEntry>, ImporterError> {
    if !path.exists() {
        return Ok(Vec::new());
    }

    let connection = Connection::open_with_flags(path, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)?;
    let mut statement = connection.prepare(
        "SELECT page_url.url, favicons.url, favicon_bitmaps.width, favicon_bitmaps.height, \
         favicon_bitmaps.image_data \
         FROM icon_mapping \
         JOIN page_url ON page_url.id = icon_mapping.page_url_id \
         JOIN favicons ON favicons.id = icon_mapping.icon_id \
         JOIN favicon_bitmaps ON favicon_bitmaps.icon_id = favicons.id \
         ORDER BY page_url.url, favicons.url",
    )?;
    let rows = statement.query_map([], |row| {
        Ok(FaviconEntry {
            page_url: row.get(0)?,
            icon_url: row.get(1)?,
            width: row.get(2)?,
            height: row.get(3)?,
            image_data: row.get(4)?,
        })
    })?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(ImporterError::Sqlite)
}

pub fn read_bookmarks(path: &Path) -> Result<Vec<BookmarkEntry>, ImporterError> {
    if !path.exists() {
        return Ok(Vec::new());
    }

    let value: Value = serde_json::from_slice(&fs::read(path)?)?;
    let mut bookmarks = Vec::new();
    if let Some(roots) = value.get("roots").and_then(Value::as_object) {
        for (name, node) in roots {
            collect_bookmarks(node, &mut vec![name.clone()], &mut bookmarks);
        }
    }
    Ok(bookmarks)
}

pub fn read_json_artifact(path: &Path) -> Result<Option<JsonArtifact>, ImporterError> {
    if !path.exists() {
        return Ok(None);
    }

    Ok(Some(JsonArtifact {
        path: path.to_path_buf(),
        value: serde_json::from_slice(&fs::read(path)?)?,
    }))
}

pub fn decryption_context_from_local_state(
    local_state: &Path,
) -> Result<Option<DecryptionContext>, ImporterError> {
    if !local_state.exists() {
        return Ok(None);
    }

    let value: Value = serde_json::from_slice(&fs::read(local_state)?)?;
    let Some(encoded_key) = value
        .pointer("/os_crypt/encrypted_key")
        .and_then(Value::as_str)
    else {
        return Ok(None);
    };

    let mut key = base64::engine::general_purpose::STANDARD
        .decode(encoded_key)
        .map_err(|error| ImporterError::Crypto(error.to_string()))?;

    #[cfg(windows)]
    if key.starts_with(b"DPAPI") {
        key.drain(..5);
        key = decrypt_platform_key(&key)?;
    }

    #[cfg(target_os = "linux")]
    {
        key = decrypt_platform_key(&key)?;
    }

    #[cfg(all(not(windows), not(target_os = "linux")))]
    if !key.is_empty() && key.len() != 32 {
        return Err(ImporterError::UnsupportedDecryption(
            "descifrado de clave Chromium no implementado en esta plataforma",
        ));
    }

    Ok(Some(DecryptionContext {
        platform_key: Some(key),
    }))
}

fn secret_from_columns(
    plain_value: String,
    encrypted_value: Vec<u8>,
    decryption: Option<&DecryptionContext>,
) -> SecretValue {
    if !plain_value.is_empty() {
        return SecretValue::PlainText(plain_value);
    }

    if encrypted_value.is_empty() {
        return SecretValue::PlainText(String::new());
    }

    if let Some(context) = decryption {
        if let Ok(value) = decrypt_chromium_secret(&encrypted_value, context) {
            return SecretValue::Decrypted(value);
        }
    }

    SecretValue::Encrypted(encrypted_value)
}

pub fn decrypt_chromium_secret(
    encrypted_value: &[u8],
    context: &DecryptionContext,
) -> Result<String, ImporterError> {
    if encrypted_value.starts_with(b"v10") || encrypted_value.starts_with(b"v11") {
        let Some(key) = &context.platform_key else {
            return Err(ImporterError::UnsupportedDecryption(
                "missing Chromium AES key",
            ));
        };
        if encrypted_value.len() < 3 + 12 + 16 {
            return Err(ImporterError::Crypto(
                "Chromium secret is too short".to_owned(),
            ));
        }

        let cipher = Aes256Gcm::new_from_slice(key)
            .map_err(|error| ImporterError::Crypto(error.to_string()))?;
        let nonce = Nonce::from_slice(&encrypted_value[3..15]);
        let plaintext = cipher
            .decrypt(nonce, &encrypted_value[15..])
            .map_err(|error| ImporterError::Crypto(format!("{error:?}")))?;
        return String::from_utf8(plaintext)
            .map_err(|error| ImporterError::Crypto(error.to_string()));
    }

    #[cfg(windows)]
    {
        let plaintext =
            windows_dpapi::decrypt_data(encrypted_value, windows_dpapi::Scope::User, None)
                .map_err(|error| ImporterError::Crypto(error.to_string()))?;
        String::from_utf8(plaintext).map_err(|error| ImporterError::Crypto(error.to_string()))
    }

    #[cfg(not(windows))]
    {
        let _ = context;
        Err(ImporterError::UnsupportedDecryption(
            "Linux Secret Service/KWallet adapter pending",
        ))
    }
}

#[cfg(windows)]
fn decrypt_platform_key(encrypted_key: &[u8]) -> Result<Vec<u8>, ImporterError> {
    windows_dpapi::decrypt_data(encrypted_key, windows_dpapi::Scope::User, None)
        .map_err(|error| ImporterError::Crypto(error.to_string()))
}

#[cfg(target_os = "linux")]
fn decrypt_platform_key(encrypted_key: &[u8]) -> Result<Vec<u8>, ImporterError> {
    linux_crypt::decrypt_linux_encrypted_key(encrypted_key)
}

#[cfg(all(not(windows), not(target_os = "linux")))]
fn decrypt_platform_key(_encrypted_key: &[u8]) -> Result<Vec<u8>, ImporterError> {
    Err(ImporterError::UnsupportedDecryption(
        "Secret Service/KWallet/Keychain adapter pending",
    ))
}

fn collect_bookmarks(node: &Value, path: &mut Vec<String>, output: &mut Vec<BookmarkEntry>) {
    if node.get("type").and_then(Value::as_str) == Some("url") {
        if let (Some(name), Some(url)) = (
            node.get("name").and_then(Value::as_str),
            node.get("url").and_then(Value::as_str),
        ) {
            output.push(BookmarkEntry {
                name: name.to_owned(),
                url: url.to_owned(),
                path: path.clone(),
            });
        }
        return;
    }

    let folder_name = node.get("name").and_then(Value::as_str);
    if let Some(name) = folder_name {
        path.push(name.to_owned());
    }
    if let Some(children) = node.get("children").and_then(Value::as_array) {
        for child in children {
            collect_bookmarks(child, path, output);
        }
    }
    if folder_name.is_some() {
        path.pop();
    }
}

fn build_candidate(
    browser: ChromiumBrowser,
    profile_name: String,
    profile_root: PathBuf,
) -> ProfileCandidate {
    let user_data_root = match browser {
        ChromiumBrowser::Opera => profile_root.clone(),
        ChromiumBrowser::Brave | ChromiumBrowser::Chrome => profile_root
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| profile_root.clone()),
    };

    ProfileCandidate {
        browser,
        profile_name,
        artifacts: build_artifacts(&profile_root, browser != ChromiumBrowser::Opera)
            .with_local_state(user_data_root.join("Local State")),
        root: profile_root,
    }
}

fn build_artifacts(profile_root: &Path, has_parent_local_state: bool) -> ProfileArtifacts {
    let local_state = if has_parent_local_state {
        profile_root
            .parent()
            .map(|parent| parent.join("Local State"))
            .unwrap_or_else(|| profile_root.join("Local State"))
    } else {
        profile_root.join("Local State")
    };

    ProfileArtifacts {
        cookies_db: profile_root.join("Network/Cookies"),
        login_db: profile_root.join("Login Data"),
        history_db: profile_root.join("History"),
        bookmarks_json: profile_root.join("Bookmarks"),
        favicons_db: profile_root.join("Favicons"),
        preferences_json: profile_root.join("Preferences"),
        secure_preferences_json: profile_root.join("Secure Preferences"),
        local_state_json: local_state,
    }
}

impl ProfileArtifacts {
    fn with_local_state(mut self, local_state_json: PathBuf) -> Self {
        self.local_state_json = local_state_json;
        self
    }
}

/// Opciones para exportacion local auditada (sin red).
#[derive(Debug, Clone, Copy, Default)]
pub struct AuditExportOptions {
    /// Incluir contrasenas descifradas en el JSON (solo uso local explicito).
    pub include_passwords: bool,
}

/// Localiza el primer perfil Chromium disponible (Chrome, luego Brave, luego Opera).
pub fn find_first_chromium_profile() -> Option<ProfileCandidate> {
    for browser in [
        ChromiumBrowser::Chrome,
        ChromiumBrowser::Brave,
        ChromiumBrowser::Opera,
    ] {
        for root in default_profile_roots(browser) {
            if !root.exists() {
                continue;
            }
            if let Ok(profiles) = discover_profiles(browser, &root) {
                if let Some(candidate) = profiles.into_iter().next() {
                    return Some(candidate);
                }
            }
        }
    }
    None
}

/// Exporta un snapshot auditable a JSON (staging previo a migracion al perfil Gecko).
pub fn export_audit_snapshot(
    candidate: &ProfileCandidate,
    destination: &Path,
    options: AuditExportOptions,
) -> Result<ProfileSnapshot, ImporterError> {
    let decryption = decryption_context_from_local_state(&candidate.artifacts.local_state_json)?;
    let mut snapshot = read_profile_snapshot(candidate, decryption.as_ref())?;
    if !options.include_passwords {
        snapshot.logins.clear();
    }

    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent)?;
    }

    let json = serde_json::to_string_pretty(&snapshot)?;
    fs::write(destination, json)?;
    Ok(snapshot)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_root() -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should work")
            .as_nanos();
        std::env::temp_dir().join(format!("aurexalis-importer-test-{unique}"))
    }

    #[test]
    fn discovers_chromium_profiles() {
        let root = temp_root();
        fs::create_dir_all(root.join("Default/Network")).expect("create Default profile");
        fs::create_dir_all(root.join("Profile 1/Network")).expect("create Profile 1");
        fs::create_dir_all(root.join("Crashpad")).expect("create ignored dir");

        let profiles =
            discover_profiles(ChromiumBrowser::Brave, &root).expect("profiles should discover");

        assert_eq!(profiles.len(), 2);
        assert_eq!(profiles[0].profile_name, "Default");
        assert!(profiles[0]
            .artifacts
            .cookies_db
            .ends_with("Network/Cookies"));
        assert!(profiles[0].artifacts.login_db.ends_with("Login Data"));
        assert!(profiles[0].artifacts.bookmarks_json.ends_with("Bookmarks"));
        assert!(profiles[0].artifacts.favicons_db.ends_with("Favicons"));
        assert!(profiles[0]
            .artifacts
            .preferences_json
            .ends_with("Preferences"));
        assert!(profiles[0]
            .artifacts
            .local_state_json
            .ends_with("Local State"));

        fs::remove_dir_all(root).expect("cleanup");
    }

    #[test]
    fn builds_opera_single_profile_from_root() {
        let root = temp_root();
        fs::create_dir_all(root.join("Network")).expect("create opera profile");

        let profiles =
            discover_profiles(ChromiumBrowser::Opera, &root).expect("opera profile should build");

        assert_eq!(profiles.len(), 1);
        assert_eq!(profiles[0].root, root);

        fs::remove_dir_all(profiles[0].root.clone()).expect("cleanup");
    }

    #[test]
    fn missing_root_is_error() {
        let error = discover_profiles(ChromiumBrowser::Chrome, &temp_root())
            .expect_err("missing root should fail");

        assert!(matches!(error, ImporterError::ProfileRootMissing));
    }

    #[test]
    fn reports_existing_import_surfaces() {
        let root = temp_root();
        let profile_root = root.join("Default");
        fs::create_dir_all(profile_root.join("Network")).expect("create profile directories");
        fs::write(profile_root.join("Bookmarks"), "{}").expect("write bookmarks");
        fs::write(profile_root.join("History"), "").expect("write history");

        let candidate =
            build_candidate(ChromiumBrowser::Chrome, "Default".to_owned(), profile_root);
        let surfaces = candidate.existing_surfaces();

        assert!(surfaces.contains(&ImportSurface::Bookmarks));
        assert!(surfaces.contains(&ImportSurface::History));
        assert!(!surfaces.contains(&ImportSurface::Cookies));

        fs::remove_dir_all(root).expect("cleanup");
    }

    #[test]
    fn maps_surface_paths() {
        let root = temp_root();
        fs::create_dir_all(root.join("Default/Network")).expect("create profile");
        let candidate = build_candidate(
            ChromiumBrowser::Brave,
            "Default".to_owned(),
            root.join("Default"),
        );

        assert!(candidate
            .path_for(ImportSurface::Bookmarks)
            .ends_with("Bookmarks"));
        assert!(candidate
            .path_for(ImportSurface::Preferences)
            .ends_with("Preferences"));
        assert!(candidate
            .artifacts
            .secure_preferences_json
            .ends_with("Secure Preferences"));

        fs::remove_dir_all(root).expect("cleanup");
    }

    #[test]
    fn reads_cookie_sqlite_fixture() {
        let root = temp_root();
        let db = root.join("Cookies");
        fs::create_dir_all(&root).expect("create root");
        let connection = Connection::open(&db).expect("open fixture db");
        connection
            .execute_batch(
                "CREATE TABLE cookies (
                    host_key TEXT,
                    name TEXT,
                    value TEXT,
                    encrypted_value BLOB,
                    path TEXT,
                    expires_utc INTEGER,
                    is_secure INTEGER,
                    is_httponly INTEGER,
                    samesite INTEGER
                );",
            )
            .expect("schema");
        connection
            .execute(
                "INSERT INTO cookies VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                rusqlite::params![
                    ".example.test",
                    "sid",
                    "plain-cookie",
                    Vec::<u8>::new(),
                    "/",
                    99_i64,
                    1_i64,
                    0_i64,
                    -1_i64
                ],
            )
            .expect("insert cookie");
        drop(connection);

        let cookies = read_cookies(&db, None).expect("read cookies");

        assert_eq!(cookies.len(), 1);
        assert_eq!(cookies[0].name, "sid");
        assert_eq!(
            cookies[0].value,
            SecretValue::PlainText("plain-cookie".to_owned())
        );
        assert!(cookies[0].is_secure);

        fs::remove_dir_all(root).expect("cleanup");
    }

    #[test]
    fn reads_login_history_and_favicons() {
        let root = temp_root();
        fs::create_dir_all(&root).expect("create root");
        let login_db = root.join("Login Data");
        let history_db = root.join("History");
        let favicon_db = root.join("Favicons");

        let connection = Connection::open(&login_db).expect("open login db");
        connection
            .execute_batch(
                "CREATE TABLE logins (
                    origin_url TEXT,
                    action_url TEXT,
                    username_element TEXT,
                    username_value TEXT,
                    password_element TEXT,
                    password_value BLOB,
                    date_created INTEGER
                );",
            )
            .expect("login schema");
        connection
            .execute(
                "INSERT INTO logins VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                rusqlite::params![
                    "https://example.test",
                    "https://example.test/login",
                    "user",
                    "jack",
                    "pass",
                    vec![1_u8, 2, 3],
                    7_i64
                ],
            )
            .expect("insert login");
        drop(connection);

        let connection = Connection::open(&history_db).expect("open history db");
        connection
            .execute_batch(
                "CREATE TABLE urls (
                    url TEXT,
                    title TEXT,
                    visit_count INTEGER,
                    typed_count INTEGER,
                    last_visit_time INTEGER
                );",
            )
            .expect("history schema");
        connection
            .execute(
                "INSERT INTO urls VALUES (?1, ?2, ?3, ?4, ?5)",
                rusqlite::params!["https://example.test", "Example", 3_i64, 1_i64, 42_i64],
            )
            .expect("insert history");
        drop(connection);

        let connection = Connection::open(&favicon_db).expect("open favicon db");
        connection
            .execute_batch(
                "CREATE TABLE page_url (id INTEGER PRIMARY KEY, url TEXT);
                 CREATE TABLE favicons (id INTEGER PRIMARY KEY, url TEXT);
                 CREATE TABLE icon_mapping (page_url_id INTEGER, icon_id INTEGER);
                 CREATE TABLE favicon_bitmaps (icon_id INTEGER, width INTEGER, height INTEGER, image_data BLOB);",
            )
            .expect("favicon schema");
        connection
            .execute(
                "INSERT INTO page_url VALUES (1, ?1)",
                ["https://example.test"],
            )
            .expect("page");
        connection
            .execute(
                "INSERT INTO favicons VALUES (1, ?1)",
                ["https://example.test/favicon.ico"],
            )
            .expect("favicon");
        connection
            .execute("INSERT INTO icon_mapping VALUES (1, 1)", [])
            .expect("map");
        connection
            .execute(
                "INSERT INTO favicon_bitmaps VALUES (1, 16, 16, ?1)",
                [vec![137_u8, 80, 78, 71]],
            )
            .expect("bitmap");
        drop(connection);

        assert_eq!(read_logins(&login_db, None).expect("logins").len(), 1);
        assert_eq!(
            read_history(&history_db).expect("history")[0].title,
            "Example"
        );
        assert_eq!(read_favicons(&favicon_db).expect("favicons")[0].width, 16);

        fs::remove_dir_all(root).expect("cleanup");
    }

    #[test]
    fn parses_bookmarks_and_json_artifacts() {
        let root = temp_root();
        fs::create_dir_all(&root).expect("create root");
        let bookmarks = root.join("Bookmarks");
        let preferences = root.join("Preferences");
        fs::write(
            &bookmarks,
            r#"{
              "roots": {
                "bookmark_bar": {
                  "name": "Bookmarks bar",
                  "type": "folder",
                  "children": [
                    {"type": "url", "name": "Aurexalis", "url": "https://example.test"}
                  ]
                }
              }
            }"#,
        )
        .expect("write bookmarks");
        fs::write(
            &preferences,
            r#"{"browser":{"check_default_browser":false}}"#,
        )
        .expect("write preferences");

        let parsed = read_bookmarks(&bookmarks).expect("bookmarks");
        let prefs = read_json_artifact(&preferences)
            .expect("preferences")
            .expect("exists");

        assert_eq!(parsed[0].name, "Aurexalis");
        assert_eq!(parsed[0].path, vec!["bookmark_bar", "Bookmarks bar"]);
        assert_eq!(
            prefs.value.pointer("/browser/check_default_browser"),
            Some(&Value::Bool(false))
        );

        fs::remove_dir_all(root).expect("cleanup");
    }
}
