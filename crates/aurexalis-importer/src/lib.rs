use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ImporterError {
    #[error("profile root was not found")]
    ProfileRootMissing,

    #[error("browser profile is unsupported: {0}")]
    UnsupportedBrowser(String),
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

