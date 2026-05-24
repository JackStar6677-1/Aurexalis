//! Comandos `aurexalis remotefs` — SFTP/FTP/FTPS list/download.

use aurexalis_remotefs::{
    connect_remote, default_port, parse_protocol, protocol_env_var, RemoteConnectionProfile,
    RemoteFileSystem, RemoteProtocol,
};
use std::path::Path;

fn build_profile(
    host: &str,
    port: Option<u16>,
    protocol: RemoteProtocol,
    username: &str,
) -> RemoteConnectionProfile {
    RemoteConnectionProfile::new("cli", host, protocol, Some(username.to_owned()))
        .with_port(port.unwrap_or_else(|| default_port(protocol)))
}

fn open_remote(
    host: &str,
    port: Option<u16>,
    protocol: RemoteProtocol,
    username: &str,
    password: &str,
) -> Result<Box<dyn RemoteFileSystem>, String> {
    let profile = build_profile(host, port, protocol, username);
    connect_remote(&profile, password).map_err(|e| e.to_string())
}

/// Lista un directorio remoto via SFTP, FTP o FTPS.
pub fn list_remote(
    protocol: RemoteProtocol,
    host: &str,
    port: Option<u16>,
    username: &str,
    password: &str,
    remote_path: &str,
) -> Result<(), String> {
    let fs = open_remote(host, port, protocol, username, password)?;
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

/// Descarga un archivo remoto via SFTP, FTP o FTPS.
pub fn get_remote(
    protocol: RemoteProtocol,
    host: &str,
    port: Option<u16>,
    username: &str,
    password: &str,
    remote_path: &str,
    local_path: &str,
) -> Result<(), String> {
    let fs = open_remote(host, port, protocol, username, password)?;
    fs.download(remote_path, local_path)
        .map_err(|e| e.to_string())?;
    println!("[SUCCESS] {remote_path} → {local_path}");
    Ok(())
}

pub fn print_help() {
    println!("Aurexalis remotefs");
    println!("  remotefs list --host H --user U --path /remote/dir [--protocol sftp|ftp|ftps]");
    println!("  remotefs get  --host H --user U --remote /path --local C:/dest/file [--protocol ...]");
    println!("  Password: --password P o env AUREXALIS_SFTP_PASS | AUREXALIS_FTP_PASS | AUREXALIS_FTPS_PASS");
}

pub fn password_from_env_or_flag(
    protocol: RemoteProtocol,
    flag: Option<&str>,
) -> Result<String, String> {
    if let Some(value) = flag {
        return Ok(value.to_owned());
    }
  let env_name = protocol_env_var(protocol);
    std::env::var(env_name).map_err(|_| {
        format!("define --password o {env_name} para {:?}", protocol)
    })
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

pub fn parse_protocol_flag(value: Option<&str>) -> Result<RemoteProtocol, String> {
    match value {
        None => Ok(RemoteProtocol::Sftp),
        Some(raw) => parse_protocol(raw),
    }
}
