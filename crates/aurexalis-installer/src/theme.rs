//! Paleta visual Aurexalis para el instalador egui.

use egui::{Color32, CornerRadius, Stroke, Visuals};

pub const BG: Color32 = Color32::from_rgb(8, 5, 15);
pub const SURFACE: Color32 = Color32::from_rgb(18, 10, 30);
pub const SURFACE_2: Color32 = Color32::from_rgb(30, 16, 45);
pub const PURPLE: Color32 = Color32::from_rgb(111, 56, 255);
pub const RED: Color32 = Color32::from_rgb(255, 31, 85);
pub const GOLD: Color32 = Color32::from_rgb(255, 209, 102);
pub const TEXT: Color32 = Color32::from_rgb(247, 242, 255);
pub const MUTED: Color32 = Color32::from_rgb(184, 169, 204);

/// Aplica el tema oscuro morado/rojo/dorado al contexto egui.
pub fn apply(ctx: &egui::Context) {
    let mut visuals = Visuals::dark();
    visuals.window_fill = BG;
    visuals.panel_fill = SURFACE;
    visuals.extreme_bg_color = BG;
    visuals.faint_bg_color = SURFACE_2;
    visuals.widgets.noninteractive.bg_fill = SURFACE_2;
    visuals.widgets.inactive.bg_fill = SURFACE_2;
    visuals.widgets.hovered.bg_fill = Color32::from_rgb(45, 24, 68);
    visuals.widgets.active.bg_fill = PURPLE;
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, MUTED);
    visuals.widgets.active.fg_stroke = Stroke::new(1.5, TEXT);
    visuals.selection.bg_fill = Color32::from_rgba_premultiplied(255, 31, 85, 80);
    visuals.hyperlink_color = GOLD;
    visuals.warn_fg_color = GOLD;
    visuals.error_fg_color = RED;
    visuals.window_corner_radius = CornerRadius::same(12);
    visuals.menu_corner_radius = CornerRadius::same(8);
    ctx.set_visuals(visuals);

    let mut style = (*ctx.global_style()).clone();
    style.spacing.button_padding = egui::vec2(14.0, 8.0);
    style.spacing.item_spacing = egui::vec2(10.0, 8.0);
    ctx.set_global_style(style);
}

/// Boton principal con acento rojo/morado.
pub fn primary_button(ui: &mut egui::Ui, label: &str) -> egui::Response {
    ui.add(
        egui::Button::new(egui::RichText::new(label).color(TEXT).size(16.0).strong())
            .fill(RED)
            .stroke(Stroke::new(1.0, GOLD))
            .corner_radius(CornerRadius::same(10)),
    )
}
