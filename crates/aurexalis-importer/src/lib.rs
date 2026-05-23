use std::fs;
use std::fmt;
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
    pub cookies_db: PathBuf,
    pub login_db: PathBuf,
    pub history_db: PathBuf,
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

    Ok(ProfileCandidate {
        browser,
        profile_name: "Default".to_owned(),
        cookies_db: profile_root.join("Network/Cookies"),
        login_db: profile_root.join("Login Data"),
        history_db: profile_root.join("History"),
        root: profile_root,
    })
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
    ProfileCandidate {
        browser,
        profile_name,
        cookies_db: profile_root.join("Network/Cookies"),
        login_db: profile_root.join("Login Data"),
        history_db: profile_root.join("History"),
        root: profile_root,
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
        assert!(profiles[0].cookies_db.ends_with("Network/Cookies"));
        assert!(profiles[0].login_db.ends_with("Login Data"));

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
}
