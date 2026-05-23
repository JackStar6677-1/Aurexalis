use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RemoteFsError {
    #[error("unsupported remote protocol: {0}")]
    UnsupportedProtocol(String),

    #[error("remote path is not allowed for this operation")]
    PathNotAllowed,
}

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

