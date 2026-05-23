//! Utilidades Windows: accesos directos, espacio en disco y desinstalacion.

use std::path::{Path, PathBuf};
use std::process::Command;

const MINIMUM_FREE_MB: u64 = 500;

/// Espacio libre en megabytes para la unidad que contiene `path`.
pub fn free_disk_space_mb(path: &Path) -> Result<u64, String> {
    let drive = drive_root(path)?;
    let drive = drive.replace('\'', "''");

    let script = format!(
        r#"
$ErrorActionPreference = 'Stop'
$disk = Get-PSDrive -Name '{drive}' -ErrorAction Stop
[uint64]([math]::Floor($disk.Free / 1MB))
"#
    );

    let output = Command::new("powershell.exe")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            &script,
        ])
        .output()
        .map_err(|e| format!("espacio en disco: {e}"))?;

    if !output.status.success() {
        return Err("no se pudo consultar espacio libre en disco".to_string());
    }

    let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
    text.parse::<u64>()
        .map_err(|_| format!("respuesta de disco invalida: {text}"))
}

/// Comprueba que hay espacio suficiente para instalar.
pub fn ensure_disk_space(path: &Path) -> Result<(), String> {
    let free = free_disk_space_mb(path)?;
    if free < MINIMUM_FREE_MB {
        return Err(format!(
            "espacio insuficiente: {free} MB libres (minimo {MINIMUM_FREE_MB} MB)"
        ));
    }
    Ok(())
}

/// Abre un dialogo nativo para elegir carpeta de instalacion.
pub fn browse_install_folder() -> Option<PathBuf> {
    rfd::FileDialog::new()
        .set_title("Aurexalis — carpeta de instalacion")
        .pick_folder()
}

/// Registra Aurexalis en Configuracion > Aplicaciones de Windows.
pub fn register_uninstall_entry(install_root: &Path, version: &str) -> Result<(), String> {
    let root = install_root.to_string_lossy().replace('\'', "''");
    let version = version.replace('\'', "''");
    let uninstall_script = install_root.join("uninstall.ps1");
    let uninstall_script = uninstall_script.to_string_lossy().replace('\'', "''");
    let icon = install_root
        .join("aurexalis.ico")
        .to_string_lossy()
        .replace('\'', "''");

    let script = format!(
        r#"
$ErrorActionPreference = 'Stop'
$key = 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\Aurexalis'
New-Item -Path $key -Force | Out-Null
Set-ItemProperty -Path $key -Name DisplayName -Value 'Aurexalis'
Set-ItemProperty -Path $key -Name DisplayVersion -Value '{version}'
Set-ItemProperty -Path $key -Name Publisher -Value 'Aurexalis Project'
Set-ItemProperty -Path $key -Name InstallLocation -Value '{root}'
Set-ItemProperty -Path $key -Name UninstallString -Value "powershell.exe -NoProfile -ExecutionPolicy Bypass -File `"{uninstall_script}`""
Set-ItemProperty -Path $key -Name DisplayIcon -Value '{icon}'
New-ItemProperty -Path $key -Name NoModify -Value 1 -PropertyType DWord -Force | Out-Null
New-ItemProperty -Path $key -Name NoRepair -Value 1 -PropertyType DWord -Force | Out-Null
"#
    );

    run_powershell(&script, "registro de desinstalacion")
}

/// Crea acceso directo en el escritorio.
pub fn create_desktop_shortcut(
    name: &str,
    target: &Path,
    working_dir: &Path,
    arguments: Option<&str>,
    icon: Option<&Path>,
) -> Result<(), String> {
    let desktop = dirs::desktop_dir().ok_or("no se pudo resolver el escritorio")?;
    create_shortcut(
        &desktop.join(format!("{name}.lnk")),
        target,
        working_dir,
        arguments,
        icon,
    )
}

/// Crea acceso directo en el menu Inicio (Programs).
pub fn create_start_menu_shortcut(
    name: &str,
    target: &Path,
    working_dir: &Path,
    arguments: Option<&str>,
    icon: Option<&Path>,
) -> Result<(), String> {
    let start_menu = dirs::data_dir()
        .ok_or("no se pudo resolver AppData")?
        .join("Microsoft")
        .join("Windows")
        .join("Start Menu")
        .join("Programs")
        .join("Aurexalis");
    std::fs::create_dir_all(&start_menu).map_err(|e| e.to_string())?;
    create_shortcut(
        &start_menu.join(format!("{name}.lnk")),
        target,
        working_dir,
        arguments,
        icon,
    )
}

/// Escribe `uninstall.ps1` y accesos directos de desinstalacion.
pub fn write_uninstaller(
    install_root: &Path,
    version: &str,
    icon: Option<&Path>,
) -> Result<(), String> {
    let script_path = install_root.join("uninstall.ps1");
    let root = install_root.to_string_lossy().replace('\'', "''");
    let script = format!(
        r#"# Aurexalis uninstaller — generado automaticamente
$ErrorActionPreference = 'Stop'
$root = '{root}'
if (-not (Test-Path -LiteralPath $root)) {{
    Write-Host "Aurexalis no esta instalado en $root"
    exit 1
}}
$reply = Read-Host "Se eliminara Aurexalis en $root (no desinstala Floorp global). Continuar? [s/N]"
if ($reply -notin @('s','S','y','Y')) {{ exit 0 }}
Remove-Item -LiteralPath $root -Recurse -Force
$desktop = [Environment]::GetFolderPath('Desktop')
$start = Join-Path $env:APPDATA 'Microsoft\Windows\Start Menu\Programs\Aurexalis'
foreach ($name in @('Aurexalis.lnk','Desinstalar Aurexalis.lnk')) {{
    $p = Join-Path $desktop $name
    if (Test-Path $p) {{ Remove-Item $p -Force }}
    $p2 = Join-Path $start $name
    if (Test-Path $p2) {{ Remove-Item $p2 -Force }}
}}
if (Test-Path $start) {{ Remove-Item $start -Force -Recurse -ErrorAction SilentlyContinue }}
Remove-Item -Path 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\Aurexalis' -Recurse -Force -ErrorAction SilentlyContinue
Write-Host "Aurexalis desinstalado."
"#
    );
    std::fs::write(&script_path, script).map_err(|e| format!("escribir uninstall.ps1: {e}"))?;

    register_uninstall_entry(install_root, version)?;

    let desktop = dirs::desktop_dir().ok_or("escritorio no disponible")?;
    create_shortcut(
        &desktop.join("Desinstalar Aurexalis.lnk"),
        &script_path,
        install_root,
        None,
        icon,
    )?;
    create_start_menu_shortcut(
        "Desinstalar Aurexalis",
        &script_path,
        install_root,
        None,
        icon,
    )?;
    Ok(())
}

fn create_shortcut(
    shortcut: &Path,
    target: &Path,
    working_dir: &Path,
    arguments: Option<&str>,
    icon: Option<&Path>,
) -> Result<(), String> {
    let target = target.to_string_lossy().replace('\'', "''");
    let working_dir = working_dir.to_string_lossy().replace('\'', "''");
    let shortcut = shortcut.to_string_lossy().replace('\'', "''");
    let args = arguments.unwrap_or("").replace('\'', "''");
    let icon_line = icon.map(|path| {
        let icon = path.to_string_lossy().replace('\'', "''");
        format!("$Shortcut.IconLocation = '{icon}'")
    }).unwrap_or_default();

    let script = format!(
        r#"
$ErrorActionPreference = 'Stop'
$WshShell = New-Object -ComObject WScript.Shell
$Shortcut = $WshShell.CreateShortcut('{shortcut}')
$Shortcut.TargetPath = '{target}'
$Shortcut.WorkingDirectory = '{working_dir}'
$Shortcut.Arguments = '{args}'
$Shortcut.Description = 'Aurexalis Browser'
{icon_line}
$Shortcut.Save()
"#
    );

    run_powershell(&script, "acceso directo")
}

fn run_powershell(script: &str, context: &str) -> Result<(), String> {
    let status = Command::new("powershell.exe")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            script,
        ])
        .status()
        .map_err(|e| format!("{context}: {e}"))?;

    if status.success() {
        Ok(())
    } else {
        Err(format!(
            "PowerShell fallo en {context} (codigo {:?})",
            status.code()
        ))
    }
}

fn drive_root(path: &Path) -> Result<String, String> {
    if let Some(prefix) = path.to_str() {
        if prefix.len() >= 2 && prefix.as_bytes()[1] == b':' {
            return Ok(prefix[..1].to_uppercase());
        }
    }
    Ok("C".to_string())
}
