//! Utilidades Windows: accesos directos y comprobaciones basicas.

use std::path::Path;
use std::process::Command;

/// Crea un acceso directo en el escritorio del usuario actual.
pub fn create_desktop_shortcut(
    name: &str,
    target: &Path,
    working_dir: &Path,
    arguments: Option<&str>,
) -> Result<(), String> {
    let desktop = dirs::desktop_dir().ok_or("no se pudo resolver el escritorio")?;
    let shortcut = desktop.join(format!("{name}.lnk"));

    let target = target.to_string_lossy().replace('\'', "''");
    let working_dir = working_dir.to_string_lossy().replace('\'', "''");
    let shortcut = shortcut.to_string_lossy().replace('\'', "''");
    let args = arguments.unwrap_or("").replace('\'', "''");

    let script = format!(
        r#"
$WshShell = New-Object -ComObject WScript.Shell
$Shortcut = $WshShell.CreateShortcut('{shortcut}')
$Shortcut.TargetPath = '{target}'
$Shortcut.WorkingDirectory = '{working_dir}'
$Shortcut.Arguments = '{args}'
$Shortcut.Description = 'Aurexalis Browser'
$Shortcut.Save()
"#
    );

    let status = Command::new("powershell.exe")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            &script,
        ])
        .status()
        .map_err(|e| format!("acceso directo: {e}"))?;

    if status.success() {
        Ok(())
    } else {
        Err(format!(
            "PowerShell no pudo crear el acceso directo (codigo {:?})",
            status.code()
        ))
    }
}
