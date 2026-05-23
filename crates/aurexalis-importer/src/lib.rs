//! Local browser profile discovery for Aurexalis.
//!
//! This crate only maps and inventories local profile artifacts. Decryption and
//! migration must remain explicit, local and user-controlled.

#![forbid(unsafe_code)]

use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub enum ImporterError {
    ProfileRootMissing,
    UnsupportedBrowser(String),
    ReadDir(std::io::Error),
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
        }
    }
}

impl std::error::Error for ImporterError {}

impl From<std::io::Error> for ImporterError {
    fn from(error: std::io::Error) -> Self {
        Self::ReadDir(error)
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
    for entry in fs::read_dir(user_data_root)? {
        let entry = entry?;
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
}
