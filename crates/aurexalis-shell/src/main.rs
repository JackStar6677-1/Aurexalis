#![forbid(unsafe_code)]

mod blocker_cmd;
mod config;
mod import_cmd;
mod remotefs_cmd;

use aurexalis_importer::{default_profile_roots, discover_profiles, ApplySurface, ChromiumBrowser};
use std::env;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("[ERROR] {error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), String> {
    let mut args = env::args().skip(1);
    match args.next().as_deref() {
        Some("launch") => launch(args.next().map(PathBuf::from), None),
        Some("--launch-installed") => launch_installed(),
        Some("profiles") => list_profiles(),
        Some("import") => run_import(args),
        Some("blocker") => run_blocker(args),
        Some("remotefs") => run_remotefs(args),
        Some("floorp") => print_floorp_hint(),
        Some("help") | None => {
            print_help();
            Ok(())
        }
        Some(other) => Err(format!("comando desconocido: {other}")),
    }
}

fn install_root_from_exe() -> Result<PathBuf, String> {
    env::current_exe()
        .map_err(|e| e.to_string())?
        .parent()
        .map(Path::to_path_buf)
        .ok_or_else(|| "no se pudo resolver el directorio del ejecutable".to_string())
}

fn default_profile_dir() -> PathBuf {
    install_root_from_exe()
        .map(|root| root.join("profiles").join("default"))
        .unwrap_or_else(|_| PathBuf::from("profiles/default"))
}

fn launch_installed() -> Result<(), String> {
    let root = install_root_from_exe()?;
    let cfg = config::load(&root)?;
    launch(Some(cfg.browser), Some(cfg.profile))
}

fn launch(binary: Option<PathBuf>, profile: Option<PathBuf>) -> Result<(), String> {
    let install_root = install_root_from_exe().ok();

    let browser = binary
        .or_else(|| env::var_os("AUREXALIS_BROWSER").map(PathBuf::from))
        .or_else(|| {
            install_root
                .as_ref()
                .and_then(|root| config::load(root).ok())
                .map(|cfg| cfg.browser)
        })
        .ok_or("define AUREXALIS_BROWSER, config.json o pasa la ruta al binario Firefox/Floorp")?;

    let profile = profile.unwrap_or_else(|| {
        install_root
            .as_ref()
            .and_then(|root| config::load(root).ok())
            .map(|cfg| cfg.profile)
            .unwrap_or_else(|| {
                env::current_dir()
                    .unwrap_or_else(|_| PathBuf::from("."))
                    .join("profiles")
                    .join("aurexalis-dev")
            })
    });

    std::fs::create_dir_all(&profile).map_err(|error| error.to_string())?;

    let status = Command::new(&browser)
        .arg("--no-remote")
        .arg("--profile")
        .arg(&profile)
        .status()
        .map_err(|error| format!("no se pudo arrancar {}: {error}", browser.display()))?;

    if status.success() {
        Ok(())
    } else {
        Err(format!("el navegador termino con estado {status}"))
    }
}

fn run_import(mut args: impl Iterator<Item = String>) -> Result<(), String> {
    match args.next().as_deref() {
        Some("list") => import_cmd::list_profiles(),
        Some("audit") => {
            let include_passwords = args.any(|arg| arg == "--passwords");
            import_cmd::export_audit(None, include_passwords)
        }
        Some("apply") => {
            let mut audit = None;
            let mut profile = None;
            let mut surfaces = vec![ApplySurface::Bookmarks, ApplySurface::History];
            let mut iter = args.peekable();
            while let Some(arg) = iter.next() {
                match arg.as_str() {
                    "--from" => audit = iter.next().map(PathBuf::from),
                    "--profile" => profile = iter.next().map(PathBuf::from),
                    "--bookmarks-only" => surfaces = vec![ApplySurface::Bookmarks],
                    "--history-only" => surfaces = vec![ApplySurface::History],
                    other => return Err(format!("flag import apply desconocido: {other}")),
                }
            }
            import_cmd::apply_audit(audit, profile, &surfaces)
        }
        Some("help") | None => {
            print_import_help();
            Ok(())
        }
        Some(other) => Err(format!("subcomando import desconocido: {other}")),
    }
}

fn run_blocker(mut args: impl Iterator<Item = String>) -> Result<(), String> {
    match args.next().as_deref() {
        Some("check") => {
            let url = args
                .next()
                .ok_or("uso: blocker check <url> [--source URL] [--type script]")?;
            let mut source = None;
            let mut kind = aurexalis_core::ResourceKind::Script;
            let mut iter = args;
            while let Some(arg) = iter.next() {
                match arg.as_str() {
                    "--source" => source = iter.next().as_deref(),
                    "--type" => {
                        let value = iter
                            .next()
                            .ok_or("falta valor para --type")?;
                        kind = blocker_cmd::parse_resource_kind(&value)?;
                    }
                    other => return Err(format!("flag blocker check desconocido: {other}")),
                }
            }
            blocker_cmd::check_url(&url, source, kind)
        }
        Some("sync-lists") => blocker_cmd::sync_lists(&default_profile_dir()),
        Some("help") | None => {
            println!("Aurexalis blocker");
            println!("  blocker check <url> [--source URL] [--type script]");
            println!("  blocker sync-lists");
            Ok(())
        }
        Some(other) => Err(format!("subcomando blocker desconocido: {other}")),
    }
}

fn run_remotefs(mut args: impl Iterator<Item = String>) -> Result<(), String> {
    match args.next().as_deref() {
        Some("list") => {
            let mut host = None;
            let mut user = None;
            let mut password = None;
            let mut port = None;
            let mut path = Some("/".to_owned());
            let mut iter = args;
            while let Some(arg) = iter.next() {
                match arg.as_str() {
                    "--host" => host = Some(remotefs_cmd::require(iter.next().as_deref(), "host")?),
                    "--user" => user = Some(remotefs_cmd::require(iter.next().as_deref(), "user")?),
                    "--password" => {
                        password = Some(remotefs_cmd::require(iter.next().as_deref(), "password")?)
                    }
                    "--port" => port = remotefs_cmd::parse_port(iter.next().as_deref())?,
                    "--path" => path = Some(remotefs_cmd::require(iter.next().as_deref(), "path")?),
                    other => return Err(format!("flag remotefs list desconocido: {other}")),
                }
            }
            let pass = remotefs_cmd::password_from_env_or_flag(password.as_deref())?;
            remotefs_cmd::list_remote(
                &host.ok_or("falta --host")?,
                port,
                &user.ok_or("falta --user")?,
                &pass,
                path.as_deref().unwrap_or("/"),
            )
        }
        Some("get") => {
            let mut host = None;
            let mut user = None;
            let mut password = None;
            let mut port = None;
            let mut remote = None;
            let mut local = None;
            let mut iter = args;
            while let Some(arg) = iter.next() {
                match arg.as_str() {
                    "--host" => host = Some(remotefs_cmd::require(iter.next().as_deref(), "host")?),
                    "--user" => user = Some(remotefs_cmd::require(iter.next().as_deref(), "user")?),
                    "--password" => {
                        password = Some(remotefs_cmd::require(iter.next().as_deref(), "password")?)
                    }
                    "--port" => port = remotefs_cmd::parse_port(iter.next().as_deref())?,
                    "--remote" => {
                        remote = Some(remotefs_cmd::require(iter.next().as_deref(), "remote")?)
                    }
                    "--local" => local = Some(remotefs_cmd::require(iter.next().as_deref(), "local")?),
                    other => return Err(format!("flag remotefs get desconocido: {other}")),
                }
            }
            let pass = remotefs_cmd::password_from_env_or_flag(password.as_deref())?;
            let local_path = local.unwrap_or_else(|| {
                remotefs_cmd::local_download_dir()
                    .join("remote-download.bin")
                    .to_string_lossy()
                    .into_owned()
            });
            remotefs_cmd::get_remote(
                &host.ok_or("falta --host")?,
                port,
                &user.ok_or("falta --user")?,
                &pass,
                &remote.ok_or("falta --remote")?,
                &local_path,
            )
        }
        Some("help") | None => {
            remotefs_cmd::print_help();
            Ok(())
        }
        Some(other) => Err(format!("subcomando remotefs desconocido: {other}")),
    }
}

fn list_profiles() -> Result<(), String> {
    for browser in [
        ChromiumBrowser::Brave,
        ChromiumBrowser::Chrome,
        ChromiumBrowser::Opera,
    ] {
        for root in default_profile_roots(browser) {
            match discover_profiles(browser, &root) {
                Ok(profiles) => {
                    for profile in profiles {
                        println!(
                            "[PROFILE] {:?} {} {}",
                            profile.browser,
                            profile.profile_name,
                            profile.root.display()
                        );
                    }
                }
                Err(_) => {
                    println!("[MISS] {:?} {}", browser, root.display());
                }
            }
        }
    }
    Ok(())
}

fn print_floorp_hint() -> Result<(), String> {
    println!("[INFO] Floorp vive en vendor/floorp");
    println!("[INFO] Inicializa con: git submodule update --init --depth 1 vendor/floorp");
    println!("[INFO] Verifica con: .\\tools\\floorp-status.ps1");
    Ok(())
}

fn print_help() {
    println!("Aurexalis shell");
    println!("  aurexalis launch [ruta-firefox-floorp]");
    println!("  aurexalis --launch-installed");
    println!("  aurexalis profiles");
    println!("  aurexalis import list|audit|apply");
    println!("  aurexalis blocker check|sync-lists");
    println!("  aurexalis remotefs list|get");
    println!("  aurexalis floorp");
}

fn print_import_help() {
    println!("Aurexalis import");
    println!("  import list                         inventario Chromium local");
    println!("  import audit [--passwords]          exporta JSON auditable");
    println!("  import apply [--from PATH]          escribe marcadores/historial en perfil Gecko");
}
