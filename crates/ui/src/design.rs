use eframe::egui;

pub const BG_PRIMARY: egui::Color32 = egui::Color32::from_rgb(10, 10, 15);
pub const BG_SECONDARY: egui::Color32 = egui::Color32::from_rgb(18, 18, 26);
pub const BG_TERTIARY: egui::Color32 = egui::Color32::from_rgb(26, 26, 37);
pub const BG_ELEVATED: egui::Color32 = egui::Color32::from_rgb(32, 32, 48);

pub const SURFACE_HOVER: egui::Color32 = egui::Color32::from_rgb(40, 40, 58);
pub const SURFACE_ACTIVE: egui::Color32 = egui::Color32::from_rgb(48, 48, 70);

pub const BORDER: egui::Color32 = egui::Color32::from_rgb(42, 42, 58);
pub const BORDER_HOVER: egui::Color32 = egui::Color32::from_rgb(58, 58, 82);
pub const BORDER_ACCENT: egui::Color32 = egui::Color32::from_rgb(59, 130, 246);

pub const ACCENT: egui::Color32 = egui::Color32::from_rgb(59, 130, 246);
pub const ACCENT_DIM: egui::Color32 = egui::Color32::from_rgba_premultiplied(59, 130, 246, 77);
pub const ACCENT_GLOW: egui::Color32 = egui::Color32::from_rgb(96, 165, 250);
pub const ACCENT_SECONDARY: egui::Color32 = egui::Color32::from_rgb(139, 92, 246);
pub const ACCENT_SECONDARY_DIM: egui::Color32 =
    egui::Color32::from_rgba_premultiplied(139, 92, 246, 77);

pub const SUCCESS: egui::Color32 = egui::Color32::from_rgb(34, 197, 94);
pub const WARNING: egui::Color32 = egui::Color32::from_rgb(245, 158, 11);
pub const ERROR: egui::Color32 = egui::Color32::from_rgb(239, 68, 68);

pub const TEXT_PRIMARY: egui::Color32 = egui::Color32::from_rgb(232, 232, 237);
pub const TEXT_SECONDARY: egui::Color32 = egui::Color32::from_rgb(139, 139, 158);
pub const TEXT_MUTED: egui::Color32 = egui::Color32::from_rgb(90, 90, 110);
pub const TEXT_ON_ACCENT: egui::Color32 = egui::Color32::WHITE;

pub const CHART_BLUE: egui::Color32 = egui::Color32::from_rgb(59, 130, 246);
pub const CHART_PURPLE: egui::Color32 = egui::Color32::from_rgb(139, 92, 246);
pub const CHART_TEAL: egui::Color32 = egui::Color32::from_rgb(20, 184, 166);
pub const CHART_ORANGE: egui::Color32 = egui::Color32::from_rgb(249, 115, 22);
pub const CHART_PINK: egui::Color32 = egui::Color32::from_rgb(236, 72, 153);

pub const GAP_XS: f32 = 4.0;
pub const GAP_SM: f32 = 8.0;
pub const GAP_MD: f32 = 12.0;
pub const GAP_LG: f32 = 16.0;
pub const GAP_XL: f32 = 24.0;
pub const GAP_XXL: f32 = 32.0;

pub const RADIUS_SM: f32 = 6.0;
pub const RADIUS_MD: f32 = 8.0;
pub const RADIUS_LG: f32 = 12.0;
pub const RADIUS_XL: f32 = 16.0;
pub const RADIUS_FULL: f32 = 999.0;

pub const TEXT_XS: f32 = 10.0;
pub const TEXT_SM: f32 = 11.0;
pub const TEXT_BASE: f32 = 13.0;
pub const TEXT_LG: f32 = 15.0;
pub const TEXT_XL: f32 = 18.0;
pub const TEXT_2XL: f32 = 22.0;
pub const TEXT_3XL: f32 = 28.0;

pub const NAV_WIDTH: f32 = 80.0;
pub const SIDEBAR_WIDTH: f32 = 240.0;
pub const HEADER_HEIGHT: f32 = 56.0;
pub const CARD_PADDING: f32 = 20.0;

pub fn apply_theme(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::dark();

    visuals.window_fill = BG_SECONDARY;
    visuals.panel_fill = BG_SECONDARY;
    visuals.extreme_bg_color = BG_PRIMARY;
    visuals.faint_bg_color = BG_TERTIARY;

    visuals.widgets.noninteractive.weak_bg_fill = BG_TERTIARY;
    visuals.widgets.noninteractive.bg_fill = BG_ELEVATED;
    visuals.widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0, BORDER);
    visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, TEXT_PRIMARY);
    visuals.widgets.noninteractive.corner_radius = RADIUS_MD.into();

    visuals.widgets.inactive.weak_bg_fill = BG_TERTIARY;
    visuals.widgets.inactive.bg_fill = BG_ELEVATED;
    visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, BORDER);
    visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, TEXT_SECONDARY);
    visuals.widgets.inactive.corner_radius = RADIUS_MD.into();

    visuals.widgets.hovered.weak_bg_fill = SURFACE_HOVER;
    visuals.widgets.hovered.bg_fill = SURFACE_HOVER;
    visuals.widgets.hovered.bg_stroke = egui::Stroke::new(1.0, BORDER_HOVER);
    visuals.widgets.hovered.fg_stroke = egui::Stroke::new(1.0, TEXT_PRIMARY);
    visuals.widgets.hovered.corner_radius = RADIUS_MD.into();

    visuals.widgets.active.weak_bg_fill = SURFACE_ACTIVE;
    visuals.widgets.active.bg_fill = SURFACE_ACTIVE;
    visuals.widgets.active.bg_stroke = egui::Stroke::new(1.0, BORDER_ACCENT);
    visuals.widgets.active.fg_stroke = egui::Stroke::new(1.0, TEXT_PRIMARY);
    visuals.widgets.active.corner_radius = RADIUS_MD.into();

    visuals.widgets.open.weak_bg_fill = BG_ELEVATED;
    visuals.widgets.open.bg_fill = BG_ELEVATED;
    visuals.widgets.open.bg_stroke = egui::Stroke::new(1.0, BORDER_ACCENT);
    visuals.widgets.open.fg_stroke = egui::Stroke::new(1.0, TEXT_PRIMARY);
    visuals.widgets.open.corner_radius = RADIUS_MD.into();

    visuals.selection.bg_fill = ACCENT_DIM;
    visuals.selection.stroke = egui::Stroke::new(1.0, ACCENT);

    visuals.override_text_color = Some(TEXT_PRIMARY);
    visuals.text_cursor.stroke = egui::Stroke::new(1.0, ACCENT);

    visuals.hyperlink_color = ACCENT_GLOW;

    visuals.window_corner_radius = RADIUS_LG.into();
    visuals.window_stroke = egui::Stroke::new(1.0, BORDER);
    visuals.window_shadow = egui::epaint::Shadow::NONE;

    visuals.popup_shadow = egui::epaint::Shadow::NONE;

    visuals.error_fg_color = ERROR;
    visuals.warn_fg_color = WARNING;

    ctx.set_visuals(visuals);

    let mut style = (*ctx.style()).clone();
    style.spacing.item_spacing = egui::vec2(GAP_SM, GAP_SM);
    style.spacing.window_margin = egui::Margin::same(GAP_LG as i8);
    style.spacing.button_padding = egui::vec2(GAP_MD, GAP_SM);
    style.spacing.indent = GAP_LG;
    style.spacing.interact_size = egui::vec2(40.0, 20.0);
    style.spacing.combo_width = 120.0;
    style.spacing.slider_width = 100.0;

    ctx.set_style(style);
}

pub fn card_frame() -> egui::Frame {
    egui::Frame::new()
        .corner_radius(RADIUS_MD)
        .fill(BG_TERTIARY)
        .stroke(egui::Stroke::new(1.0, BORDER))
        .inner_margin(CARD_PADDING)
}

pub fn inner_card_frame() -> egui::Frame {
    egui::Frame::new()
        .corner_radius(RADIUS_SM)
        .fill(BG_ELEVATED)
        .stroke(egui::Stroke::new(1.0, BORDER))
        .inner_margin(GAP_MD)
}

pub fn header_text(text: impl Into<String>) -> egui::RichText {
    egui::RichText::new(text)
        .size(TEXT_2XL)
        .color(TEXT_PRIMARY)
        .strong()
}

pub fn section_title(text: impl Into<String>) -> egui::RichText {
    egui::RichText::new(text)
        .size(TEXT_LG)
        .color(TEXT_PRIMARY)
        .strong()
}

pub fn body_text(text: impl Into<String>) -> egui::RichText {
    egui::RichText::new(text).size(TEXT_BASE).color(TEXT_PRIMARY)
}

pub fn muted_text(text: impl Into<String>) -> egui::RichText {
    egui::RichText::new(text).size(TEXT_SM).color(TEXT_SECONDARY)
}

pub fn label_text(text: impl Into<String>) -> egui::RichText {
    egui::RichText::new(text).size(TEXT_XS).color(TEXT_MUTED)
}

pub fn stat_value(text: impl Into<String>) -> egui::RichText {
    egui::RichText::new(text)
        .size(TEXT_3XL)
        .color(TEXT_PRIMARY)
        .strong()
}

pub fn separator(ui: &mut egui::Ui) {
    let space = ui.available_width();
    let y = ui.cursor().min.y + GAP_SM;
    let line = egui::Shape::line_segment(
        [
            egui::pos2(ui.cursor().min.x, y),
            egui::pos2(ui.cursor().min.x + space, y),
        ],
        egui::Stroke::new(1.0, BORDER),
    );
    ui.painter().add(line);
    ui.add_space(GAP_SM * 2.0);
}
