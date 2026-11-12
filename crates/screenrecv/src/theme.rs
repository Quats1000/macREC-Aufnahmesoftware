//! Design-System von niklaslegler.at fuer egui.
//!
//! Grundsaetze:
//! * ausschliesslich Sans-Serif (DM Sans),
//! * nur Farben aus der Palette von niklaslegler.at,
//! * keine Kisten und Rahmen um Text - nur Weissraum, Haarlinien und
//!   typografische Hierarchie (wie bei Apple),
//! * nichts steht direkt am Fensterrand.

use std::sync::Arc;

use egui::{Color32, CornerRadius, FontData, FontDefinitions, FontFamily, FontId, Stroke, TextStyle};

// ---------------------------------------------------------------- Farbpalette

pub const WHITE: Color32 = Color32::from_rgb(0xff, 0xff, 0xff);
pub const OFF_WHITE: Color32 = Color32::from_rgb(0xf8, 0xf7, 0xff);
pub const PURPLE: Color32 = Color32::from_rgb(0x53, 0x4a, 0xb7);
pub const PURPLE_DARK: Color32 = Color32::from_rgb(0x3f, 0x37, 0x92);
pub const PURPLE_LIGHT: Color32 = Color32::from_rgb(0xee, 0xed, 0xfe);
pub const DARK: Color32 = Color32::from_rgb(0x0f, 0x0e, 0x17);
pub const MUTED: Color32 = Color32::from_rgb(0x6b, 0x6b, 0x80);
pub const LINE: Color32 = Color32::from_rgb(0xe8, 0xe6, 0xf2);

// ------------------------------------------------------------------ Schrift

const FONT_REGULAR: &str = "dm-sans-regular";
const FONT_MEDIUM: &str = "dm-sans-medium";
const FONT_BOLD: &str = "dm-sans-bold";

/// Familien fuer die drei Schnitte von DM Sans.
pub fn sans() -> FontFamily {
    FontFamily::Name(Arc::from("sans"))
}
pub fn sans_medium() -> FontFamily {
    FontFamily::Name(Arc::from("sans-medium"))
}
pub fn sans_bold() -> FontFamily {
    FontFamily::Name(Arc::from("sans-bold"))
}

/// Registriert die Schriften und setzt das globale egui-Styling.
pub fn install(ctx: &egui::Context) {
    let mut fonts = FontDefinitions::default();

    fonts.font_data.insert(
        FONT_REGULAR.to_owned(),
        Arc::new(FontData::from_static(crate::assets::DM_SANS_REGULAR)),
    );
    fonts.font_data.insert(
        FONT_MEDIUM.to_owned(),
        Arc::new(FontData::from_static(crate::assets::DM_SANS_MEDIUM)),
    );
    fonts.font_data.insert(
        FONT_BOLD.to_owned(),
        Arc::new(FontData::from_static(crate::assets::DM_SANS_BOLD)),
    );

    // Fallback-Zeichenschriften: fehlen Zeichen in DM Sans, uebernimmt die
    // Systemschrift (San Francisco / Segoe UI) bzw. die egui-Standardschrift.
    const FALLBACKS: [&str; 2] = ["Ubuntu-Light", "Hack"];

    let mut sans_family = vec![FONT_REGULAR.to_owned()];
    sans_family.extend(FALLBACKS.iter().map(|name| (*name).to_owned()));
    fonts.families.entry(sans()).or_default().extend(sans_family);

    let mut medium_family = vec![FONT_MEDIUM.to_owned()];
    medium_family.extend(FALLBACKS.iter().map(|name| (*name).to_owned()));
    fonts
        .families
        .entry(sans_medium())
        .or_default()
        .extend(medium_family);

    let mut bold_family = vec![FONT_BOLD.to_owned()];
    bold_family.extend(FALLBACKS.iter().map(|name| (*name).to_owned()));
    fonts.families.entry(sans_bold()).or_default().extend(bold_family);
    fonts
        .families
        .entry(FontFamily::Proportional)
        .or_default()
        .push(FONT_REGULAR.to_owned());
    fonts
        .families
        .entry(FontFamily::Monospace)
        .or_default()
        .push(FONT_REGULAR.to_owned());

    ctx.set_fonts(fonts);
    ctx.set_theme(egui::Theme::Light);

    let mut style = (*ctx.global_style()).clone();
    let visuals = &mut style.visuals;

    visuals.panel_fill = WHITE;
    visuals.window_fill = WHITE;
    visuals.hyperlink_color = PURPLE;
    visuals.faint_bg_color = OFF_WHITE;
    visuals.extreme_bg_color = OFF_WHITE;
    visuals.text_edit_bg_color = None;
    visuals.selection.bg_fill = PURPLE_LIGHT;
    visuals.selection.stroke = Stroke::new(1.0, PURPLE);
    visuals.window_stroke = Stroke::new(1.0, LINE);
    // Echte Ecken sind abgerundet - im egui-Viewport bleiben dabei aber
    // schwarze Ecken stehen. Daher kantige Fenster.
    visuals.window_corner_radius = CornerRadius::same(0);
    visuals.window_shadow = egui::Shadow::NONE;
    visuals.menu_corner_radius = CornerRadius::same(12);
    visuals.error_fg_color = Color32::from_rgb(0xc0, 0x2b, 0x2b);
    visuals.warn_fg_color = Color32::from_rgb(0xb5, 0x6b, 0x00);
    visuals.button_frame = false;

    visuals.widgets.noninteractive.bg_fill = WHITE;
    visuals.widgets.noninteractive.weak_bg_fill = WHITE;
    visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, LINE);
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, DARK);
    visuals.widgets.noninteractive.corner_radius = CornerRadius::same(10);

    visuals.widgets.inactive.bg_fill = WHITE;
    visuals.widgets.inactive.weak_bg_fill = WHITE;
    visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, LINE);
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, DARK);
    visuals.widgets.inactive.corner_radius = CornerRadius::same(10);

    visuals.widgets.hovered.bg_fill = OFF_WHITE;
    visuals.widgets.hovered.weak_bg_fill = OFF_WHITE;
    visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, LINE);
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, PURPLE);
    visuals.widgets.hovered.corner_radius = CornerRadius::same(10);

    visuals.widgets.active.bg_fill = PURPLE_LIGHT;
    visuals.widgets.active.weak_bg_fill = PURPLE_LIGHT;
    visuals.widgets.active.bg_stroke = Stroke::new(1.0, PURPLE);
    visuals.widgets.active.fg_stroke = Stroke::new(1.0, DARK);
    visuals.widgets.active.corner_radius = CornerRadius::same(10);

    visuals.widgets.open.bg_fill = WHITE;
    visuals.widgets.open.weak_bg_fill = WHITE;
    visuals.widgets.open.bg_stroke = Stroke::new(1.0, PURPLE);
    visuals.widgets.open.fg_stroke = Stroke::new(1.0, DARK);
    visuals.widgets.open.corner_radius = CornerRadius::same(10);

    style.spacing.item_spacing = egui::vec2(12.0, 12.0);
    style.spacing.button_padding = egui::vec2(22.0, 10.0);
    style.spacing.window_margin = egui::Margin::same(24);
    style.spacing.menu_margin = egui::Margin::same(8);
    style.spacing.interact_size = egui::vec2(44.0, 44.0);
    style.spacing.text_edit_width = f32::INFINITY;

    style
        .text_styles
        .insert(TextStyle::Heading, FontId::new(28.0, sans_bold()));
    style
        .text_styles
        .insert(TextStyle::Body, FontId::new(15.0, sans()));
    style
        .text_styles
        .insert(TextStyle::Button, FontId::new(15.0, sans_medium()));
    style
        .text_styles
        .insert(TextStyle::Small, FontId::new(13.0, sans()));
    style
        .text_styles
        .insert(TextStyle::Monospace, FontId::new(13.0, sans()));

    ctx.set_global_style(style);
}

// --------------------------------------------------------------- Bausteine

/// Grosser Titel (Apple: Large Title).
pub fn title(ui: &mut egui::Ui, text: &str) {
    ui.add(
        egui::Label::new(
            egui::RichText::new(text)
                .font(FontId::new(28.0, sans_bold()))
                .color(DARK)
                .line_height(Some(34.0)),
        )
        .wrap(),
    );
}

/// Fliesstext in Grau.
pub fn subtitle(ui: &mut egui::Ui, text: &str) {
    ui.add(
        egui::Label::new(
            egui::RichText::new(text)
                .font(FontId::new(15.0, sans()))
                .color(MUTED)
                .line_height(Some(22.0)),
        )
        .wrap(),
    );
}

/// Kleine gesperrte Grossbuchstaben-Beschriftung (`.label` auf der Website).
pub fn label(ui: &mut egui::Ui, text: &str) {
    ui.label(
        egui::RichText::new(text.to_uppercase())
            .font(FontId::new(11.0, sans_medium()))
            .color(MUTED)
            .extra_letter_spacing(1.2),
    );
}

/// Textfeld im Stil von Apple: fuelliger, weicher Hintergrund, kein Rahmen.
pub fn field(ui: &mut egui::Ui, text: &mut String, hint: &str) {
    egui::TextEdit::singleline(text)
        .hint_text(hint)
        .desired_width(f32::INFINITY)
        .font(FontId::new(15.0, sans()))
        .text_color(DARK)
        .align(egui::Align2::LEFT_CENTER)
        .frame(
            egui::Frame::new()
                .fill(OFF_WHITE)
                .corner_radius(CornerRadius::same(10))
                .inner_margin(egui::Margin::symmetric(16, 12)),
        )
        .show(ui);
}

/// Unveraenderliches Feld (z. B. das Datum).
pub fn read_only_field(ui: &mut egui::Ui, value: &str) {
    egui::Frame::new()
        .fill(OFF_WHITE)
        .corner_radius(CornerRadius::same(10))
        .inner_margin(egui::Margin::symmetric(14, 11))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.label(
                egui::RichText::new(value)
                    .font(FontId::new(15.0, sans()))
                    .color(MUTED),
            );
        });
}

/// Hauptaktion: farbige Kapsel.
pub fn capsule(ui: &mut egui::Ui, text: &str) -> egui::Response {
    let height = 50.0;
    let width = ui.available_width().min(320.0);
    let (rect, response) = ui.allocate_exact_size(egui::vec2(width, height), egui::Sense::click());

    if ui.is_rect_visible(rect) {
        let hovered = response.hovered();
        let fill = if hovered { PURPLE_DARK } else { PURPLE };
        let painter = ui.painter();
        painter.rect_filled(rect, height / 2.0, fill);
        painter.text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            text,
            FontId::new(16.0, sans_medium()),
            WHITE,
        );
    }

    response
}

/// Nebenaktion: dezente Kapsel mit weissem Hintergrund.
pub fn soft_capsule(ui: &mut egui::Ui, text: &str) -> egui::Response {
    let height = 44.0;
    let width = ui.available_width().min(320.0);
    let (rect, response) = ui.allocate_exact_size(egui::vec2(width, height), egui::Sense::click());

    if ui.is_rect_visible(rect) {
        let hovered = response.hovered();
        let fill = if hovered { PURPLE_LIGHT } else { OFF_WHITE };
        let painter = ui.painter();
        painter.rect_filled(rect, height / 2.0, fill);
        painter.text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            text,
            FontId::new(15.0, sans_medium()),
            if hovered { PURPLE } else { DARK },
        );
    }

    response
}

/// Textaktion ohne Hintergrund (Apple: „Cancel“).
pub fn text_button(ui: &mut egui::Ui, text: &str) -> egui::Response {
    let galley = ui.painter().layout_no_wrap(
        text.to_owned(),
        FontId::new(15.0, sans_medium()),
        MUTED,
    );
    let size = galley.size();
    let (rect, response) =
        ui.allocate_exact_size(egui::vec2(size.x + 12.0, size.y + 8.0), egui::Sense::click());
    if ui.is_rect_visible(rect) {
        let color = if response.hovered() {
            PURPLE
        } else {
            MUTED
        };
        ui.painter().galley(rect.min, galley, color);
    }
    response
}

/// Feine Trennlinie mit grosszuegigem Abstand.
pub fn hairline(ui: &mut egui::Ui) {
    ui.add_space(18.0);
    let (rect, _) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 1.0), egui::Sense::hover());
    ui.painter().rect_filled(rect, 0.0, LINE);
    ui.add_space(18.0);
}
