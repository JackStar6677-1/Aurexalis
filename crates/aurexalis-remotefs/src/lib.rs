//! Remote filesystem abstractions for Aurexalis.
//!
//! The module models SFTP/FTP/FTPS connections without mounting remote folders
//! into the operating system.

#![forbid(unsafe_code)]

mod sftp;

pub use sftp::SftpFileSystem;

use std::fmt;
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Debug)]
pub enum RemoteFsError {
    UnsupportedProtocol(String),
    PathNotAllowed,
    Io(std::io::Error),
    TransferNotFound(u64),
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
            RemoteFsError::Io(error) => write!(formatter, "io error: {error}"),
            RemoteFsError::TransferNotFound(id) => write!(formatter, "transfer not found: {id}"),
        }
    }
}

impl std::error::Error for RemoteFsError {}

impl From<std::io::Error> for RemoteFsError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CredentialRef {
    Anonymous,
    SystemKeyring { service: String, account: String },
    Prompt,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransferDirection {
    Download,
    Upload,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransferStatus {
    Queued,
    Running,
    Completed,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransferJob {
    pub id: u64,
    pub direction: TransferDirection,
    pub remote_path: String,
    pub local_path: PathBuf,
    pub status: TransferStatus,
    pub error: Option<String>,
}

#[derive(Debug, Default)]
pub struct TransferQueue {
    next_id: AtomicU64,
    jobs: Vec<TransferJob>,
}

pub trait RemoteFileSystem {
    fn list(&self, path: &str) -> Result<Vec<RemoteEntry>, RemoteFsError>;
    fn download(&self, remote_path: &str, local_path: &str) -> Result<(), RemoteFsError>;
    fn upload(&self, local_path: &str, remote_path: &str) -> Result<(), RemoteFsError>;
}

#[derive(Debug, Clone)]
pub struct LocalMirrorFileSystem {
    root: PathBuf,
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

    pub fn with_port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }
}

impl TransferQueue {
    pub fn enqueue_download(
        &mut self,
        remote_path: &str,
        local_path: impl Into<PathBuf>,
    ) -> Result<u64, RemoteFsError> {
        let remote_path = normalize_remote_path(remote_path)?;
        Ok(self.push(TransferDirection::Download, remote_path, local_path.into()))
    }

    pub fn enqueue_upload(
        &mut self,
        local_path: impl Into<PathBuf>,
        remote_path: &str,
    ) -> Result<u64, RemoteFsError> {
        let remote_path = normalize_remote_path(remote_path)?;
        Ok(self.push(TransferDirection::Upload, remote_path, local_path.into()))
    }

    pub fn mark_running(&mut self, id: u64) -> Result<(), RemoteFsError> {
        self.update_status(id, TransferStatus::Running, None)
    }

    pub fn mark_completed(&mut self, id: u64) -> Result<(), RemoteFsError> {
        self.update_status(id, TransferStatus::Completed, None)
    }

    pub fn mark_failed(&mut self, id: u64, error: impl Into<String>) -> Result<(), RemoteFsError> {
        self.update_status(id, TransferStatus::Failed, Some(error.into()))
    }

    pub fn jobs(&self) -> &[TransferJob] {
        &self.jobs
    }

    fn push(
        &mut self,
        direction: TransferDirection,
        remote_path: String,
        local_path: PathBuf,
    ) -> u64 {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed) + 1;
        self.jobs.push(TransferJob {
            id,
            direction,
            remote_path,
            local_path,
            status: TransferStatus::Queued,
            error: None,
        });
        id
    }

    fn update_status(
        &mut self,
        id: u64,
        status: TransferStatus,
        error: Option<String>,
    ) -> Result<(), RemoteFsError> {
        let Some(job) = self.jobs.iter_mut().find(|job| job.id == id) else {
            return Err(RemoteFsError::TransferNotFound(id));
        };
        job.status = status;
        job.error = error;
        Ok(())
    }
}

impl LocalMirrorFileSystem {
    pub fn new(root: impl Into<PathBuf>) -> Result<Self, RemoteFsError> {
        let root = root.into();
        fs::create_dir_all(&root)?;
        Ok(Self { root })
    }

    fn resolve(&self, remote_path: &str) -> Result<PathBuf, RemoteFsError> {
        let normalized = normalize_remote_path(remote_path)?;
        Ok(self.root.join(normalized.trim_start_matches('/')))
    }
}

impl RemoteFileSystem for LocalMirrorFileSystem {
    fn list(&self, path: &str) -> Result<Vec<RemoteEntry>, RemoteFsError> {
        let root = self.resolve(path)?;
        if !root.exists() {
            return Ok(Vec::new());
        }

        let mut entries = Vec::new();
        for entry in fs::read_dir(root)? {
            let entry = entry?;
            let metadata = entry.metadata()?;
            entries.push(RemoteEntry {
                name: entry.file_name().to_string_lossy().into_owned(),
                path: entry.path(),
                is_dir: metadata.is_dir(),
                size: metadata.is_file().then_some(metadata.len()),
            });
        }
        entries.sort_by(|left, right| left.name.cmp(&right.name));
        Ok(entries)
    }

    fn download(&self, remote_path: &str, local_path: &str) -> Result<(), RemoteFsError> {
        let source = self.resolve(remote_path)?;
        if let Some(parent) = Path::new(local_path).parent() {
            fs::create_dir_all(parent)?;
        }
        fs::copy(source, local_path)?;
        Ok(())
    }

    fn upload(&self, local_path: &str, remote_path: &str) -> Result<(), RemoteFsError> {
        let target = self.resolve(remote_path)?;
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::copy(local_path, target)?;
        Ok(())
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

    #[test]
    fn queues_transfers_with_status_updates() {
        let mut queue = TransferQueue::default();
        let id = queue
            .enqueue_download("/var/www/index.html", "C:/tmp/index.html")
            .expect("queue download");

        queue.mark_running(id).expect("running");
        queue.mark_completed(id).expect("completed");

        assert_eq!(queue.jobs()[0].status, TransferStatus::Completed);
    }

    #[test]
    fn local_mirror_upload_download_and_list() {
        let root = std::env::temp_dir().join("aurexalis-remotefs-local-mirror");
        if root.exists() {
            fs::remove_dir_all(&root).expect("cleanup old root");
        }
        let local = root.join("local.txt");
        fs::create_dir_all(&root).expect("create root");
        fs::write(&local, "hello").expect("write local");

        let remote = LocalMirrorFileSystem::new(root.join("remote")).expect("mirror");
        remote
            .upload(local.to_str().expect("utf8"), "/www/index.txt")
            .expect("upload");
        let entries = remote.list("/www").expect("list");
        let downloaded = root.join("downloaded.txt");
        remote
            .download("/www/index.txt", downloaded.to_str().expect("utf8"))
            .expect("download");

        assert_eq!(entries[0].name, "index.txt");
        assert_eq!(fs::read_to_string(downloaded).expect("read"), "hello");

        fs::remove_dir_all(root).expect("cleanup");
    }
}
