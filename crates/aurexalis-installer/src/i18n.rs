//! Cadenas de interfaz en espanol e ingles.

/// Idioma activo del instalador.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lang {
    Es,
    En,
}

impl Lang {
    /// Alterna entre espanol e ingles.
    pub fn toggle(self) -> Self {
        match self {
            Self::Es => Self::En,
            Self::En => Self::Es,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Es => "ES",
            Self::En => "EN",
        }
    }
}

/// Textos localizados de la interfaz.
pub struct Strings {
    pub window_title: &'static str,
    pub tagline: &'static str,
    pub welcome_title: &'static str,
    pub welcome_body: &'static str,
    pub install_folder: &'static str,
    pub browse: &'static str,
    pub download_floorp: &'static str,
    pub floorp_hint: &'static str,
    pub disk_space: &'static str,
    pub next: &'static str,
    pub license_title: &'static str,
    pub license_accept: &'static str,
    pub install_btn: &'static str,
    pub installing_title: &'static str,
    pub installing_hint: &'static str,
    pub done_title: &'static str,
    pub done_shortcut: &'static str,
    pub installed_at: &'static str,
    pub engine: &'static str,
    pub open: &'static str,
    pub close: &'static str,
    pub error_title: &'static str,
    pub back: &'static str,
    pub version: &'static str,
    pub err_empty_path: &'static str,
    pub err_low_disk: &'static str,
    pub err_license: &'static str,
}

/// Devuelve las cadenas para el idioma pedido.
pub fn strings(lang: Lang) -> Strings {
    match lang {
        Lang::Es => Strings {
            window_title: "Aurexalis Setup",
            tagline: "Instalador del navegador · morado · rojo · dorado",
            welcome_title: "Bienvenido",
            welcome_body: "Este asistente descarga el runtime Aurexalis desde GitHub, \
instala el motor Floorp (Gecko) y deja el navegador listo con tema y perfil propios.",
            install_folder: "Carpeta de instalacion",
            browse: "Examinar…",
            download_floorp: "Descargar e instalar Floorp (recomendado)",
            floorp_hint: "Requiere Internet. Floorp se obtiene del release oficial de Floorp-Projects.",
            disk_space: "Espacio libre en disco",
            next: "Siguiente",
            license_title: "Licencia y componentes de terceros",
            license_accept: "Acepto la licencia MIT de Aurexalis y entiendo que Floorp tiene su propia licencia",
            install_btn: "Instalar Aurexalis",
            installing_title: "Instalando…",
            installing_hint: "No cierres esta ventana hasta que termine.",
            done_title: "Listo",
            done_shortcut: "Accesos directos creados en el escritorio y en el menu Inicio.",
            installed_at: "Instalado en",
            engine: "Motor",
            open: "Abrir Aurexalis",
            close: "Cerrar",
            error_title: "Error de instalacion",
            back: "Volver",
            version: "Version del instalador",
            err_empty_path: "Indica una carpeta de instalacion valida.",
            err_low_disk: "Se requieren al menos 500 MB libres en la unidad de destino.",
            err_license: "Debes aceptar la licencia para continuar.",
        },
        Lang::En => Strings {
            window_title: "Aurexalis Setup",
            tagline: "Browser installer · purple · red · gold",
            welcome_title: "Welcome",
            welcome_body: "This wizard downloads the Aurexalis runtime from GitHub, \
installs the Floorp engine (Gecko), and prepares your themed browser profile.",
            install_folder: "Install folder",
            browse: "Browse…",
            download_floorp: "Download and install Floorp (recommended)",
            floorp_hint: "Internet required. Floorp is fetched from the official Floorp-Projects release.",
            disk_space: "Free disk space",
            next: "Next",
            license_title: "License and third-party components",
            license_accept: "I accept the Aurexalis MIT license and understand Floorp has its own license",
            install_btn: "Install Aurexalis",
            installing_title: "Installing…",
            installing_hint: "Do not close this window until setup finishes.",
            done_title: "Done",
            done_shortcut: "Shortcuts were added to the desktop and Start menu.",
            installed_at: "Installed at",
            engine: "Engine",
            open: "Open Aurexalis",
            close: "Close",
            error_title: "Installation error",
            back: "Back",
            version: "Installer version",
            err_empty_path: "Choose a valid install folder.",
            err_low_disk: "At least 500 MB of free disk space is required on the target drive.",
            err_license: "You must accept the license to continue.",
        },
    }
}
