//! Cliente FTP/FTPS minimo para Aurexalis RemoteFS.

use std::cell::RefCell;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::str::FromStr;

use suppaftp::list::File;
use suppaftp::native_tls::TlsConnector;
use suppaftp::{FtpError, FtpStream, NativeTlsConnector, NativeTlsFtpStream};

use crate::{
    default_port, normalize_remote_path, RemoteConnectionProfile, RemoteEntry, RemoteFileSystem,
    RemoteFsError, RemoteProtocol,
};

fn ftp_error(error: FtpError) -> RemoteFsError {
    RemoteFsError::Io(std::io::Error::other(error.to_string()))
}

/// Stream FTP plano o FTPS envuelto para mutabilidad interior.
enum FtpStreamKind {
    Plain(FtpStream),
    Secure(NativeTlsFtpStream),
}

impl FtpStreamKind {
    fn login(&mut self, user: &str, password: &str) -> Result<(), RemoteFsError> {
        match self {
            Self::Plain(stream) => stream.login(user, password).map_err(ftp_error),
            Self::Secure(stream) => stream.login(user, password).map_err(ftp_error),
        }
    }

    fn list(&mut self, pathname: Option<&str>) -> Result<Vec<String>, RemoteFsError> {
        match self {
            Self::Plain(stream) => stream.list(pathname).map_err(ftp_error),
            Self::Secure(stream) => stream.list(pathname).map_err(ftp_error),
        }
    }

    fn retr_to_file(&mut self, remote_path: &str, local_path: &str) -> Result<(), RemoteFsError> {
        if let Some(parent) = Path::new(local_path).parent() {
            std::fs::create_dir_all(parent)?;
        }
        let mut local = std::fs::File::create(local_path)?;
        let copy = |remote: &mut dyn Read| -> Result<(), FtpError> {
            std::io::copy(remote, &mut local)
                .map(|_| ())
                .map_err(FtpError::ConnectionError)
        };
        match self {
            Self::Plain(stream) => stream.retr(remote_path, copy).map_err(ftp_error)?,
            Self::Secure(stream) => stream.retr(remote_path, copy).map_err(ftp_error)?,
        }
        Ok(())
    }

    fn put_file(&mut self, remote_path: &str, local_path: &str) -> Result<(), RemoteFsError> {
        let mut local = std::fs::File::open(local_path)?;
        match self {
            Self::Plain(stream) => {
                stream.put_file(remote_path, &mut local).map_err(ftp_error)?;
            }
            Self::Secure(stream) => {
                stream.put_file(remote_path, &mut local).map_err(ftp_error)?;
            }
        }
        Ok(())
    }
}

/// Cliente FTP o FTPS conectado a un servidor remoto.
pub struct FtpFileSystem {
    stream: RefCell<FtpStreamKind>,
}

impl FtpFileSystem {
    /// Abre sesion FTP/FTPS con usuario y contrasena.
    pub fn connect(
        profile: &RemoteConnectionProfile,
        password: &str,
    ) -> Result<Self, RemoteFsError> {
        match profile.protocol {
            RemoteProtocol::Ftp => Self::connect_plain(profile, password),
            RemoteProtocol::Ftps => Self::connect_ftps(profile, password),
            RemoteProtocol::Sftp => Err(RemoteFsError::UnsupportedProtocol(
                "SFTP no usa FtpFileSystem".to_owned(),
            )),
        }
    }

    fn connect_plain(
        profile: &RemoteConnectionProfile,
        password: &str,
    ) -> Result<Self, RemoteFsError> {
        let addr = format!("{}:{}", profile.host, profile.port);
        let mut stream = FtpStream::connect(&addr).map_err(ftp_error)?;
        let user = profile.username.as_deref().unwrap_or("anonymous");
        stream.login(user, password).map_err(ftp_error)?;
        Ok(Self {
            stream: RefCell::new(FtpStreamKind::Plain(stream)),
        })
    }

    fn connect_ftps(
        profile: &RemoteConnectionProfile,
        password: &str,
    ) -> Result<Self, RemoteFsError> {
        let addr = format!("{}:{}", profile.host, profile.port);
        let user = profile.username.as_deref().unwrap_or("anonymous");
        let tls = NativeTlsConnector::from(
            TlsConnector::new()
                .map_err(|e| RemoteFsError::Io(std::io::Error::other(e.to_string())))?,
        );

        let stream = if profile.port == default_port(RemoteProtocol::Ftps) {
            NativeTlsFtpStream::connect_secure_implicit(&addr, tls.clone()).map_err(ftp_error)?
        } else {
            let plain = NativeTlsFtpStream::connect(&addr).map_err(ftp_error)?;
            plain
                .into_secure(tls, &profile.host)
                .map_err(ftp_error)?
        };

        let mut kind = FtpStreamKind::Secure(stream);
        kind.login(user, password)?;
        Ok(Self {
            stream: RefCell::new(kind),
        })
    }

    fn parse_list_line(line: &str, parent: &str) -> Option<RemoteEntry> {
        let file = File::from_str(line.trim()).ok()?;
        let name = file.name().trim().to_owned();
        if name.is_empty() || name == "." || name == ".." {
            return None;
        }
        let is_dir = file.is_directory();
        let remote_path = if parent == "/" {
            format!("/{name}")
        } else {
            format!("{parent}/{name}")
        };
        Some(RemoteEntry {
            name,
            path: PathBuf::from(remote_path),
            is_dir,
            size: if is_dir {
                None
            } else {
                Some(file.size() as u64)
            },
        })
    }
}

impl RemoteFileSystem for FtpFileSystem {
    fn list(&self, path: &str) -> Result<Vec<RemoteEntry>, RemoteFsError> {
        let normalized = normalize_remote_path(path)?;
        let listing = self
            .stream
            .borrow_mut()
            .list(Some(normalized.as_str()))?;
        let mut entries = Vec::new();
        for line in listing {
            if let Some(entry) = Self::parse_list_line(&line, &normalized) {
                entries.push(entry);
            }
        }
        entries.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(entries)
    }

    fn download(&self, remote_path: &str, local_path: &str) -> Result<(), RemoteFsError> {
        let normalized = normalize_remote_path(remote_path)?;
        self.stream
            .borrow_mut()
            .retr_to_file(&normalized, local_path)
    }

    fn upload(&self, local_path: &str, remote_path: &str) -> Result<(), RemoteFsError> {
        let normalized = normalize_remote_path(remote_path)?;
        self.stream
            .borrow_mut()
            .put_file(&normalized, local_path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_posix_list_line() {
        let entry = FtpFileSystem::parse_list_line(
            "-rw-rw-r-- 1 0 1 8192 Nov 5 2018 readme.txt",
            "/pub",
        )
        .expect("parse line");
        assert_eq!(entry.name, "readme.txt");
        assert!(!entry.is_dir);
        assert_eq!(entry.path.to_string_lossy(), "/pub/readme.txt");
    }
}
