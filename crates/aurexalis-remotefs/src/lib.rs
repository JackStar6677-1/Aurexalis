//! Remote filesystem abstractions for Aurexalis.
//!
//! The module models SFTP/FTP/FTPS connections without mounting remote folders
//! into the operating system.

#![forbid(unsafe_code)]

use std::fmt;
use std::path::{Component, Path, PathBuf};

#[derive(Debug)]
pub enum RemoteFsError {
    UnsupportedProtocol(String),
    PathNotAllowed,
}

impl fmt::Display for RemoteFsError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RemoteFsError::UnsupportedProtocol(value) => {
                write!(formatter, "unsupported remote protocol: {value}")
            }
            RemoteFsError::PathNotAllowed => {
                formatter.write_str("remote path is not allowed for this operation")
            }
        }
    }
}

impl std::error::Error for RemoteFsError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RemoteProtocol {
    Sftp,
    Ftp,
    Ftps,
}

#[derive(Debug, Clone)]
pub struct RemoteConnectionProfile {
    pub name: String,
    pub host: String,
    pub port: u16,
    pub protocol: RemoteProtocol,
    pub username: Option<String>,
}

#[derive(Debug, Clone)]
pub struct RemoteEntry {
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
    pub size: Option<u64>,
}

pub trait RemoteFileSystem {
    fn list(&self, path: &str) -> Result<Vec<RemoteEntry>, RemoteFsError>;
    fn download(&self, remote_path: &str, local_path: &str) -> Result<(), RemoteFsError>;
    fn upload(&self, local_path: &str, remote_path: &str) -> Result<(), RemoteFsError>;
}

pub fn default_port(protocol: RemoteProtocol) -> u16 {
    match protocol {
        RemoteProtocol::Sftp => 22,
        RemoteProtocol::Ftp => 21,
        RemoteProtocol::Ftps => 990,
    }
}

pub fn normalize_remote_path(path: &str) -> Result<String, RemoteFsError> {
    if path.trim().is_empty() {
        return Ok("/".to_owned());
    }

    let mut segments = Vec::new();
    for component in Path::new(path).components() {
        match component {
            Component::RootDir | Component::CurDir => {}
            Component::Normal(value) => {
                let Some(segment) = value.to_str() else {
                    return Err(RemoteFsError::PathNotAllowed);
                };
                if segment.contains('\0') {
                    return Err(RemoteFsError::PathNotAllowed);
                }
                segments.push(segment.to_owned());
            }
            Component::ParentDir | Component::Prefix(_) => {
                return Err(RemoteFsError::PathNotAllowed);
            }
        }
    }

    if segments.is_empty() {
        Ok("/".to_owned())
    } else {
        Ok(format!("/{}", segments.join("/")))
    }
}

impl RemoteConnectionProfile {
    pub fn new(
        name: impl Into<String>,
        host: impl Into<String>,
        protocol: RemoteProtocol,
        username: Option<String>,
    ) -> Self {
        Self {
            name: name.into(),
            host: host.into(),
            port: default_port(protocol),
            protocol,
            username,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_default_ports() {
        assert_eq!(default_port(RemoteProtocol::Sftp), 22);
        assert_eq!(default_port(RemoteProtocol::Ftp), 21);
        assert_eq!(default_port(RemoteProtocol::Ftps), 990);
    }

    #[test]
    fn normalizes_remote_paths() {
        assert_eq!(normalize_remote_path("").expect("empty path"), "/");
        assert_eq!(
            normalize_remote_path("/var/www/site").expect("absolute path"),
            "/var/www/site"
        );
        assert_eq!(
            normalize_remote_path("plugins/Slimefun").expect("relative path"),
            "/plugins/Slimefun"
        );
    }

    #[test]
    fn rejects_path_traversal() {
        let error = normalize_remote_path("../secret").expect_err("traversal should fail");
        assert!(matches!(error, RemoteFsError::PathNotAllowed));
    }

    #[test]
    fn builds_connection_with_default_port() {
        let profile = RemoteConnectionProfile::new(
            "Drakes",
            "example.test",
            RemoteProtocol::Sftp,
            Some("jack".to_owned()),
        );

        assert_eq!(profile.port, 22);
        assert_eq!(profile.name, "Drakes");
    }
}
