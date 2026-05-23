#![forbid(unsafe_code)]

use aurexalis_importer::{default_profile_roots, discover_profiles, ChromiumBrowser};
use std::env;
use std::path::PathBuf;
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
        Some("launch") => launch(args.next().map(PathBuf::from)),
        Some("profiles") => list_profiles(),
        Some("floorp") => print_floorp_hint(),
        Some("help") | None => {
            print_help();
            Ok(())
        }
        Some(other) => Err(format!("comando desconocido: {other}")),
    }
}

fn launch(binary: Option<PathBuf>) -> Result<(), String> {
    let browser = binary
        .or_else(|| env::var_os("AUREXALIS_BROWSER").map(PathBuf::from))
        .ok_or("define AUREXALIS_BROWSER o pasa la ruta al binario Firefox/Floorp")?;

    let profile = env::current_dir()
        .map_err(|error| error.to_string())?
        .join("profiles")
        .join("aurexalis-dev");
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
    println!("  aurexalis profiles");
    println!("  aurexalis floorp");
}
