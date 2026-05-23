#![forbid(unsafe_code)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod i18n;
mod install;
mod license_text;
mod theme;

use eframe::egui::{self, CentralPanel, ProgressBar, RichText, ScrollArea, Vec2};
use i18n::{strings, Lang};
use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver};
use std::thread;

const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

enum Screen {
    Welcome,
    License,
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
    lang: Lang,
    screen: Screen,
    install_path: String,
    download_floorp: bool,
    license_accepted: bool,
    free_disk_mb: Option<u64>,
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
        let install_path = default_path.to_string_lossy().into_owned();
        let free_disk_mb = install::windows::free_disk_space_mb(PathBuf::from(&install_path).as_path()).ok();

        Self {
            lang: Lang::Es,
            screen: Screen::Welcome,
            install_path,
            download_floorp: true,
            license_accepted: false,
            free_disk_mb,
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
        let t = strings(self.lang);

        CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .button(RichText::new(self.lang.label()).color(theme::GOLD))
                        .clicked()
                    {
                        self.lang = self.lang.toggle();
                    }
                });
            });

            ui.vertical_centered(|ui| {
                ui.add_space(8.0);
                self.draw_header(ui, &t);
                ui.add_space(6.0);
            });

            ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.set_max_width(480.0);
                    ui.vertical_centered(|ui| match self.screen {
                        Screen::Welcome => self.draw_welcome(ui, &t),
                        Screen::License => self.draw_license(ui, &t),
                        Screen::Installing => self.draw_installing(ui, &t),
                        Screen::Done => self.draw_done(ui, &t),
                        Screen::Error => self.draw_error(ui, &t),
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

    fn refresh_disk_space(&mut self) {
        let path = PathBuf::from(self.install_path.trim());
        self.free_disk_mb = install::windows::free_disk_space_mb(&path).ok();
    }

    fn draw_header(&self, ui: &mut egui::Ui, t: &i18n::Strings) {
        ui.label(
            RichText::new("AUREXALIS")
                .size(28.0)
                .strong()
                .color(theme::GOLD),
        );
        ui.label(RichText::new(t.tagline).size(13.0).color(theme::MUTED));
    }

    fn draw_welcome(&mut self, ui: &mut egui::Ui, t: &i18n::Strings) {
        egui::Frame::new()
            .fill(theme::SURFACE)
            .corner_radius(12)
            .inner_margin(18)
            .show(ui, |ui| {
                ui.label(
                    RichText::new(t.welcome_title)
                        .size(20.0)
                        .color(theme::TEXT)
                        .strong(),
                );
                ui.add_space(6.0);
                ui.label(RichText::new(t.welcome_body).color(theme::MUTED));
                ui.add_space(12.0);
                ui.label(RichText::new(t.install_folder).color(theme::GOLD));
                ui.horizontal(|ui| {
                    ui.text_edit_singleline(&mut self.install_path);
                    if ui.button(t.browse).clicked() {
                        if let Some(folder) = install::windows::browse_install_folder() {
                            self.install_path = folder.to_string_lossy().into_owned();
                            self.refresh_disk_space();
                        }
                    }
                });
                if let Some(mb) = self.free_disk_mb {
                    ui.label(
                        RichText::new(format!("{}: {mb} MB", t.disk_space))
                            .small()
                            .color(if mb >= 500 {
                                theme::MUTED
                            } else {
                                theme::RED
                            }),
                    );
                }
                ui.add_space(6.0);
                ui.checkbox(&mut self.download_floorp, t.download_floorp);
                ui.label(RichText::new(t.floorp_hint).small().color(theme::MUTED));
            });

        ui.add_space(14.0);
        ui.vertical_centered(|ui| {
            if theme::primary_button(ui, t.next).clicked() {
                if self.install_path.trim().is_empty() {
                    self.error = t.err_empty_path.to_string();
                    self.screen = Screen::Error;
                    return;
                }
                if self.free_disk_mb.is_some_and(|mb| mb < 500) {
                    self.error = t.err_low_disk.to_string();
                    self.screen = Screen::Error;
                    return;
                }
                self.screen = Screen::License;
            }
            ui.add_space(6.0);
            ui.label(
                RichText::new(format!("{} v{APP_VERSION}", t.version))
                    .small()
                    .color(theme::MUTED),
            );
        });
    }

    fn draw_license(&mut self, ui: &mut egui::Ui, t: &i18n::Strings) {
        egui::Frame::new()
            .fill(theme::SURFACE)
            .corner_radius(12)
            .inner_margin(18)
            .show(ui, |ui| {
                ui.label(
                    RichText::new(t.license_title)
                        .size(18.0)
                        .color(theme::TEXT)
                        .strong(),
                );
                ui.add_space(8.0);
                egui::ScrollArea::vertical()
                    .max_height(220.0)
                    .show(ui, |ui| {
                        ui.label(
                            RichText::new(license_text::BODY)
                                .monospace()
                                .size(11.0)
                                .color(theme::MUTED),
                        );
                    });
                ui.add_space(8.0);
                ui.checkbox(&mut self.license_accepted, t.license_accept);
            });

        ui.add_space(12.0);
        ui.horizontal(|ui| {
            if ui.button(RichText::new(t.back).color(theme::MUTED)).clicked() {
                self.screen = Screen::Welcome;
            }
            if theme::primary_button(ui, t.install_btn).clicked() {
                if !self.license_accepted {
                    self.error = t.err_license.to_string();
                    self.screen = Screen::Error;
                } else {
                    self.start_install();
                }
            }
        });
    }

    fn draw_installing(&self, ui: &mut egui::Ui, t: &i18n::Strings) {
        egui::Frame::new()
            .fill(theme::SURFACE)
            .corner_radius(12)
            .inner_margin(18)
            .show(ui, |ui| {
                ui.label(
                    RichText::new(t.installing_title)
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
                ui.label(RichText::new(t.installing_hint).small().color(theme::MUTED));
            });
    }

    fn draw_done(&mut self, ui: &mut egui::Ui, t: &i18n::Strings) {
        egui::Frame::new()
            .fill(theme::SURFACE)
            .corner_radius(12)
            .inner_margin(18)
            .show(ui, |ui| {
                ui.label(
                    RichText::new(t.done_title)
                        .size(22.0)
                        .color(theme::GOLD)
                        .strong(),
                );
                if let Some(config) = &self.result {
                    ui.add_space(8.0);
                    ui.label(
                        RichText::new(format!("{}:\n{}", t.installed_at, config.install_root))
                            .color(theme::TEXT),
                    );
                    ui.label(
                        RichText::new(format!("{}: {}", t.engine, config.browser))
                            .small()
                            .color(theme::MUTED),
                    );
                }
                ui.add_space(8.0);
                ui.label(RichText::new(t.done_shortcut).color(theme::MUTED));
            });

        ui.add_space(14.0);
        ui.horizontal(|ui| {
            if theme::primary_button(ui, t.open).clicked() {
                if let Some(config) = &self.result {
                    let root = PathBuf::from(&config.install_root);
                    let exe = root.join("aurexalis.exe");
                    let _ = std::process::Command::new(exe)
                        .arg("--launch-installed")
                        .current_dir(&root)
                        .spawn();
                }
            }
            if ui.button(RichText::new(t.close).color(theme::TEXT)).clicked() {
                ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
            }
        });
    }

    fn draw_error(&mut self, ui: &mut egui::Ui, t: &i18n::Strings) {
        egui::Frame::new()
            .fill(theme::SURFACE)
            .corner_radius(12)
            .inner_margin(18)
            .show(ui, |ui| {
                ui.label(
                    RichText::new(t.error_title)
                        .size(20.0)
                        .color(theme::RED)
                        .strong(),
                );
                ui.add_space(8.0);
                ui.label(RichText::new(&self.error).color(theme::TEXT));
            });

        ui.add_space(12.0);
        if theme::primary_button(ui, t.back).clicked() {
            self.screen = match self.license_accepted {
                true => Screen::License,
                false => Screen::Welcome,
            };
            self.error.clear();
            self.progress = 0.0;
        }
    }

    fn start_install(&mut self) {
        let install_root = PathBuf::from(self.install_path.trim());
        let (tx, rx) = mpsc::channel();
        self.worker_rx = Some(rx);
        self.screen = Screen::Installing;
        self.progress = 0.0;
        self.status = "…".to_string();
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
    let t = strings(Lang::Es);
    let icon = egui::include_image!("../../../assets/branding/aurexalis-icon.png");

    let native = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size(Vec2::new(540.0, 660.0))
            .with_min_inner_size(Vec2::new(500.0, 600.0))
            .with_title(format!("{} v{APP_VERSION}", t.window_title))
            .with_icon(icon.into()),
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
