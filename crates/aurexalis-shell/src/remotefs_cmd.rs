//! Comandos `aurexalis remotefs` — SFTP list/download.

use aurexalis_remotefs::{
    default_port, RemoteConnectionProfile, RemoteFileSystem, RemoteProtocol, SftpFileSystem,
};
use std::path::Path;

/// Lista un directorio remoto via SFTP.
pub fn list_remote(
    host: &str,
    port: Option<u16>,
    username: &str,
    password: &str,
    remote_path: &str,
) -> Result<(), String> {
    let profile = RemoteConnectionProfile::new(
        "cli",
        host,
        RemoteProtocol::Sftp,
        Some(username.to_owned()),
    )
    .with_port(port.unwrap_or_else(|| default_port(RemoteProtocol::Sftp)));
    let fs = SftpFileSystem::connect(&profile, password).map_err(|e| e.to_string())?;
    for entry in fs.list(remote_path).map_err(|e| e.to_string())? {
        let kind = if entry.is_dir { "DIR" } else { "FILE" };
        let size = entry
            .size
            .map(|s| s.to_string())
            .unwrap_or_else(|| "-".to_owned());
        println!("[{kind}] {} ({size} bytes)", entry.name);
    }
    Ok(())
}

/// Descarga un archivo remoto via SFTP.
pub fn get_remote(
    host: &str,
    port: Option<u16>,
    username: &str,
    password: &str,
    remote_path: &str,
    local_path: &str,
) -> Result<(), String> {
    let profile = RemoteConnectionProfile::new(
        "cli",
        host,
        RemoteProtocol::Sftp,
        Some(username.to_owned()),
    )
    .with_port(port.unwrap_or_else(|| default_port(RemoteProtocol::Sftp)));
    let fs = SftpFileSystem::connect(&profile, password).map_err(|e| e.to_string())?;
    fs.download(remote_path, local_path)
        .map_err(|e| e.to_string())?;
    println!("[SUCCESS] {remote_path} → {local_path}");
    Ok(())
}

pub fn print_help() {
    println!("Aurexalis remotefs");
    println!("  remotefs list --host H --user U --path /remote/dir");
    println!("  remotefs get  --host H --user U --remote /path --local C:/dest/file");
    println!("  Password: --password P o env AUREXALIS_SFTP_PASS");
}

pub fn password_from_env_or_flag(flag: Option<&str>) -> Result<String, String> {
    if let Some(value) = flag {
        return Ok(value.to_owned());
    }
    std::env::var("AUREXALIS_SFTP_PASS")
        .map_err(|_| "define --password o AUREXALIS_SFTP_PASS para SFTP".to_string())
}

pub fn parse_port(value: Option<&str>) -> Result<Option<u16>, String> {
    match value {
        None => Ok(None),
        Some(raw) => raw
            .parse::<u16>()
            .map(Some)
            .map_err(|_| format!("puerto invalido: {raw}")),
    }
}

pub fn require(value: Option<&str>, name: &str) -> Result<String, String> {
    value
        .map(str::to_owned)
        .ok_or_else(|| format!("falta --{name}"))
}

pub fn local_download_dir() -> std::path::PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.join("downloads")))
        .unwrap_or_else(|| Path::new("downloads").to_path_buf())
}
