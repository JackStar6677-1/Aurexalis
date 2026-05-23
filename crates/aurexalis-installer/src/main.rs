#![forbid(unsafe_code)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod install;
mod theme;

use eframe::egui::{self, CentralPanel, ProgressBar, RichText, ScrollArea, Vec2};
use std::path::PathBuf;
use std::sync::mpsc::Receiver;
use std::thread;

const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

enum Screen {
    Welcome,
    Installing,
    Done,
    Error,
}

enum WorkerMsg {
    Progress(f32, String),
    Done(install::InstallConfig),
    Failed(String),
}

struct InstallerApp {
    screen: Screen,
    install_path: String,
    download_floorp: bool,
    progress: f32,
    status: String,
    error: String,
    result: Option<install::InstallConfig>,
    worker_rx: Option<Receiver<WorkerMsg>>,
}

impl Default for InstallerApp {
    fn default() -> Self {
        let default_path = dirs::data_local_dir()
            .map(|p| p.join("Aurexalis"))
            .unwrap_or_else(|| PathBuf::from(r"C:\Aurexalis"));

        Self {
            screen: Screen::Welcome,
            install_path: default_path.to_string_lossy().into_owned(),
            download_floorp: true,
            progress: 0.0,
            status: String::new(),
            error: String::new(),
            result: None,
            worker_rx: None,
        }
    }
}

impl eframe::App for InstallerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.poll_worker(ctx);

        CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(12.0);
                self.draw_header(ui);
                ui.add_space(8.0);
            });

            ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.set_max_width(460.0);
                    ui.vertical_centered(|ui| match self.screen {
                        Screen::Welcome => self.draw_welcome(ui),
                        Screen::Installing => self.draw_installing(ui),
                        Screen::Done => self.draw_done(ui),
                        Screen::Error => self.draw_error(ui),
                    });
                });
        });
    }
}

impl InstallerApp {
    fn poll_worker(&mut self, ctx: &egui::Context) {
        let mut messages = Vec::new();
        if let Some(rx) = &self.worker_rx {
            while let Ok(msg) = rx.try_recv() {
                messages.push(msg);
            }
        }

        for msg in messages {
            match msg {
                WorkerMsg::Progress(value, status) => {
                    self.progress = value;
                    self.status = status;
                }
                WorkerMsg::Done(config) => {
                    self.result = Some(config);
                    self.screen = Screen::Done;
                    self.worker_rx = None;
                }
                WorkerMsg::Failed(error) => {
                    self.error = error;
                    self.screen = Screen::Error;
                    self.worker_rx = None;
                }
            }
        }

        if self.worker_rx.is_some() {
            ctx.request_repaint();
        }
    }

    fn draw_header(&self, ui: &mut egui::Ui) {
        ui.label(
            RichText::new("AUREXALIS")
                .size(28.0)
                .strong()
                .color(theme::GOLD),
        );
        ui.label(
            RichText::new("Instalador del navegador · morado · rojo · dorado")
                .size(13.0)
                .color(theme::MUTED),
        );
    }

    fn draw_welcome(&mut self, ui: &mut egui::Ui) {
        egui::Frame::new()
            .fill(theme::SURFACE)
            .corner_radius(12)
            .inner_margin(18)
            .show(ui, |ui| {
                ui.label(
                    RichText::new("Bienvenido")
                        .size(20.0)
                        .color(theme::TEXT)
                        .strong(),
                );
                ui.add_space(6.0);
                ui.label(
                    RichText::new(
                        "Este asistente descarga el runtime Aurexalis desde GitHub, \
instala el motor Floorp (Gecko) y deja el navegador listo con tema y perfil propios.",
                    )
                    .color(theme::MUTED),
                );
                ui.add_space(12.0);
                ui.label(RichText::new("Carpeta de instalacion").color(theme::GOLD));
                ui.text_edit_singleline(&mut self.install_path);
                ui.add_space(6.0);
                ui.checkbox(
                    &mut self.download_floorp,
                    "Descargar e instalar Floorp (recomendado)",
                );
                ui.label(
                    RichText::new(
                        "Requiere conexion a Internet. Floorp se obtiene del release oficial de Floorp-Projects.",
                    )
                    .small()
                    .color(theme::MUTED),
                );
            });

        ui.add_space(16.0);
        ui.vertical_centered(|ui| {
            if theme::primary_button(ui, "  Instalar Aurexalis  ").clicked() {
                self.start_install();
            }
            ui.add_space(6.0);
            ui.label(
                RichText::new(format!("Versión del instalador v{APP_VERSION}"))
                    .small()
                    .color(theme::MUTED),
            );
        });
    }

    fn draw_installing(&self, ui: &mut egui::Ui) {
        egui::Frame::new()
            .fill(theme::SURFACE)
            .corner_radius(12)
            .inner_margin(18)
            .show(ui, |ui| {
                ui.label(
                    RichText::new("Instalando...")
                        .size(20.0)
                        .color(theme::RED)
                        .strong(),
                );
                ui.add_space(10.0);
                ui.add(
                    ProgressBar::new(self.progress)
                        .fill(theme::PURPLE)
                        .animate(true),
                );
                ui.label(RichText::new(&self.status).color(theme::TEXT));
                ui.add_space(8.0);
                ui.label(
                    RichText::new("No cierres esta ventana hasta que termine.")
                        .small()
                        .color(theme::MUTED),
                );
            });
    }

    fn draw_done(&mut self, ui: &mut egui::Ui) {
        egui::Frame::new()
            .fill(theme::SURFACE)
            .corner_radius(12)
            .inner_margin(18)
            .show(ui, |ui| {
                ui.label(
                    RichText::new("Listo")
                        .size(22.0)
                        .color(theme::GOLD)
                        .strong(),
                );
                if let Some(config) = &self.result {
                    ui.add_space(8.0);
                    ui.label(
                        RichText::new(format!("Instalado en:\n{}", config.install_root))
                            .color(theme::TEXT),
                    );
                    ui.label(
                        RichText::new(format!("Motor: {}", config.browser))
                            .small()
                            .color(theme::MUTED),
                    );
                }
                ui.add_space(8.0);
                ui.label(
                    RichText::new(
                        "Se creo un acceso directo en el escritorio. \
Puedes abrir Aurexalis desde ahi.",
                    )
                    .color(theme::MUTED),
                );
            });

        ui.add_space(14.0);
        ui.horizontal(|ui| {
            if theme::primary_button(ui, "Abrir Aurexalis").clicked() {
                if let Some(config) = &self.result {
                    let exe = PathBuf::from(&config.install_root).join("aurexalis.exe");
                    let _ = std::process::Command::new(exe)
                        .arg("--launch-installed")
                        .current_dir(&config.install_root)
                        .spawn();
                }
            }
            if ui
                .button(RichText::new("Cerrar").color(theme::TEXT))
                .clicked()
            {
                ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
            }
        });
    }

    fn draw_error(&mut self, ui: &mut egui::Ui) {
        egui::Frame::new()
            .fill(theme::SURFACE)
            .corner_radius(12)
            .inner_margin(18)
            .show(ui, |ui| {
                ui.label(
                    RichText::new("Error de instalacion")
                        .size(20.0)
                        .color(theme::RED)
                        .strong(),
                );
                ui.add_space(8.0);
                ui.label(RichText::new(&self.error).color(theme::TEXT));
            });

        ui.add_space(12.0);
        if theme::primary_button(ui, "Volver").clicked() {
            self.screen = Screen::Welcome;
            self.error.clear();
            self.progress = 0.0;
        }
    }

    fn start_install(&mut self) {
        let install_root = PathBuf::from(self.install_path.trim());
        if install_root.as_os_str().is_empty() {
            self.error = "Indica una carpeta de instalacion valida.".to_string();
            self.screen = Screen::Error;
            return;
        }

        let (tx, rx) = mpsc::channel();
        self.worker_rx = Some(rx);
        self.screen = Screen::Installing;
        self.progress = 0.0;
        self.status = "Iniciando...".to_string();
        self.result = None;

        let download_floorp = self.download_floorp;
        let version = APP_VERSION.to_string();

        thread::spawn(move || {
            let progress = |value: f32, status: &str| {
                let _ = tx.send(WorkerMsg::Progress(value, status.to_string()));
            };

            match install::run_full_install(&install_root, &version, download_floorp, &progress) {
                Ok(config) => {
                    let _ = tx.send(WorkerMsg::Done(config));
                }
                Err(error) => {
                    let _ = tx.send(WorkerMsg::Failed(error));
                }
            }
        });
    }
}

fn main() -> eframe::Result<()> {
    let native = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size(Vec2::new(520.0, 620.0))
            .with_min_inner_size(Vec2::new(480.0, 560.0))
            .with_title(format!("Aurexalis Setup v{APP_VERSION}")),
        ..Default::default()
    };

    eframe::run_native(
        "Aurexalis Setup",
        native,
        Box::new(|cc| {
            theme::apply(&cc.egui_ctx);
            Ok(Box::new(InstallerApp::default()))
        }),
    )
}
