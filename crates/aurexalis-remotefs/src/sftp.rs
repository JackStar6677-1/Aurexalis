//! Cliente SFTP minimo para Aurexalis RemoteFS.

use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::Path;

use ssh2::{Session, Sftp};

use crate::{
    normalize_remote_path, RemoteConnectionProfile, RemoteEntry, RemoteFileSystem, RemoteFsError,
    RemoteProtocol,
};

fn ssh2_error(error: ssh2::Error) -> RemoteFsError {
    RemoteFsError::Io(std::io::Error::other(error.to_string()))
}

/// Cliente SFTP conectado a un servidor remoto.
pub struct SftpFileSystem {
    session: Session,
}

impl SftpFileSystem {
    /// Abre sesion SFTP con usuario y contrasena.
    pub fn connect(
        profile: &RemoteConnectionProfile,
        password: &str,
    ) -> Result<Self, RemoteFsError> {
        if profile.protocol != RemoteProtocol::Sftp {
            return Err(RemoteFsError::UnsupportedProtocol(format!(
                "{:?}",
                profile.protocol
            )));
        }

        let addr = format!("{}:{}", profile.host, profile.port);
        let tcp = TcpStream::connect(&addr).map_err(RemoteFsError::Io)?;
        let mut session = Session::new().map_err(|e| RemoteFsError::Io(std::io::Error::other(e.to_string())))?;
        session.set_tcp_stream(tcp);
        session.handshake().map_err(ssh2_error)?;
        let user = profile.username.as_deref().unwrap_or("root");
        session
            .userauth_password(user, password)
            .map_err(ssh2_error)?;
        if !session.authenticated() {
            return Err(RemoteFsError::Io(std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                "autenticacion SFTP fallida",
            )));
        }
        Ok(Self { session })
    }

    fn sftp(&self) -> Result<Sftp, RemoteFsError> {
        self.session.sftp().map_err(ssh2_error)
    }
}

impl RemoteFileSystem for SftpFileSystem {
    fn list(&self, path: &str) -> Result<Vec<RemoteEntry>, RemoteFsError> {
        let normalized = normalize_remote_path(path)?;
        let sftp = self.sftp()?;
        let dir = sftp.readdir(Path::new(&normalized)).map_err(ssh2_error)?;
        let mut entries = Vec::new();
        for (path_buf, stat) in dir {
            let name = path_buf
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_owned();
            if name.is_empty() || name == "." || name == ".." {
                continue;
            }
            let is_dir = stat.is_dir();
            entries.push(RemoteEntry {
                name,
                path: path_buf,
                is_dir,
                size: if is_dir { None } else { stat.size },
            });
        }
        entries.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(entries)
    }

    fn download(&self, remote_path: &str, local_path: &str) -> Result<(), RemoteFsError> {
        let normalized = normalize_remote_path(remote_path)?;
        let sftp = self.sftp()?;
        let mut remote = sftp.open(Path::new(&normalized)).map_err(ssh2_error)?;
        if let Some(parent) = Path::new(local_path).parent() {
            std::fs::create_dir_all(parent)?;
        }
        let mut local = std::fs::File::create(local_path)?;
        let mut buffer = [0_u8; 8192];
        loop {
            let read = remote.read(&mut buffer).map_err(RemoteFsError::Io)?;
            if read == 0 {
                break;
            }
            local
                .write_all(&buffer[..read])
                .map_err(RemoteFsError::Io)?;
        }
        Ok(())
    }

    fn upload(&self, local_path: &str, remote_path: &str) -> Result<(), RemoteFsError> {
        let normalized = normalize_remote_path(remote_path)?;
        let sftp = self.sftp()?;
        let mut local = std::fs::File::open(local_path)?;
        let mut remote = sftp.create(Path::new(&normalized)).map_err(ssh2_error)?;
        let mut buffer = [0_u8; 8192];
        loop {
            let read = local.read(&mut buffer).map_err(RemoteFsError::Io)?;
            if read == 0 {
                break;
            }
            remote
                .write_all(&buffer[..read])
                .map_err(RemoteFsError::Io)?;
        }
        Ok(())
    }
}
