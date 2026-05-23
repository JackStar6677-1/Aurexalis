#![forbid(unsafe_code)]

mod config;
mod import_cmd;

use aurexalis_importer::{default_profile_roots, discover_profiles, ChromiumBrowser};
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
        Some("import") => run_import(&mut args),
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

fn run_import(args: &mut std::env::Args) -> Result<(), String> {
    match args.next().as_deref() {
        Some("list") => import_cmd::list_profiles(),
        Some("audit") => {
            let include_passwords = args.any(|arg| arg == "--passwords");
            import_cmd::export_audit(None, include_passwords)
        }
        Some("help") | None => {
            print_import_help();
            Ok(())
        }
        Some(other) => Err(format!("subcomando import desconocido: {other}")),
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
    println!("  aurexalis import list|audit [--passwords]");
    println!("  aurexalis floorp");
}

fn print_import_help() {
    println!("Aurexalis import");
    println!("  import list              inventario Chromium local");
    println!("  import audit [--passwords] exporta JSON auditable");
}
