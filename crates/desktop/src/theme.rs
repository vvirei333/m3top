//! Material 3 Expressive — dark theme. Colors are real M3 dark baseline
//! tonal roles (not an invented palette): a surface-container ladder for
//! elevation instead of borders, primary/secondary containers for
//! accents, M3 shape scale (large/extra-large/full) for cards, chips
//! and indicators.

#![allow(dead_code)]
use eframe::egui::{self, Color32, CornerRadius, Stroke};

/// Font family name for genuinely bold text. egui's `RichText::strong()`
/// only changes the COLOR, not stroke weight — metric numbers need an
/// actual different font weight, otherwise "bolder" just doesn't happen.
pub const BOLD_FONT: &str = "NotoSansBold";

/// Installs Noto Sans Bold (a local file, Apache-2.0, not downloaded
/// from the network) as a separate `BOLD_FONT` font family. Called once
/// on app startup.
pub fn install_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    fonts.font_data.insert(
        BOLD_FONT.to_owned(),
        std::sync::Arc::new(egui::FontData::from_static(include_bytes!("../assets/NotoSans-Bold.ttf"))),
    );
    fonts.families.insert(egui::FontFamily::Name(BOLD_FONT.into()), vec![BOLD_FONT.to_owned()]);
    ctx.set_fonts(fonts);
}

pub fn bold_font(size: f32) -> egui::FontId {
    egui::FontId::new(size, egui::FontFamily::Name(BOLD_FONT.into()))
}

// --- Surface roles ---
// Window background is a deep dark purple (like the frame in the
// reference), cards are near-black boxes on top of it: that's exactly
// what the Expressive reference looks like (black tiles on a purple
// backdrop, not a uniform gray UI).
pub const SURFACE_DIM: Color32 = Color32::from_rgb(0x1c, 0x10, 0x26);
pub const SURFACE_CONTAINER_LOWEST: Color32 = Color32::from_rgb(0x08, 0x06, 0x0a);
pub const SURFACE_CONTAINER_LOW: Color32 = Color32::from_rgb(0x0d, 0x0a, 0x10);
pub const SURFACE_CONTAINER: Color32 = Color32::from_rgb(0x0d, 0x0a, 0x10);
pub const SURFACE_CONTAINER_HIGH: Color32 = Color32::from_rgb(0x4a, 0x40, 0x5c);
pub const SURFACE_CONTAINER_HIGHEST: Color32 = Color32::from_rgb(0x5a, 0x4f, 0x6e);
pub const OUTLINE_VARIANT: Color32 = Color32::from_rgb(0x49, 0x45, 0x4f);

// --- Content on surface ---
pub const ON_SURFACE: Color32 = Color32::from_rgb(0xe6, 0xe0, 0xe9);
pub const ON_SURFACE_VARIANT: Color32 = Color32::from_rgb(0xca, 0xc4, 0xd0);

// --- Accent roles ---
pub const PRIMARY: Color32 = Color32::from_rgb(0xd0, 0xbc, 0xff);
pub const ON_PRIMARY: Color32 = Color32::from_rgb(0x38, 0x1e, 0x72);
pub const PRIMARY_CONTAINER: Color32 = Color32::from_rgb(0x4f, 0x37, 0x8b);
pub const ON_PRIMARY_CONTAINER: Color32 = Color32::from_rgb(0xea, 0xdd, 0xff);
pub const SECONDARY_CONTAINER: Color32 = Color32::from_rgb(0x4a, 0x44, 0x58);
pub const ON_SECONDARY_CONTAINER: Color32 = Color32::from_rgb(0xe8, 0xde, 0xf8);

// --- Semantic levels (good/warn/bad) use M3 tertiary/error roles ---
pub const GOOD: Color32 = Color32::from_rgb(0x9c, 0xd6, 0x7a); // tertiary-ish green
pub const WARN: Color32 = Color32::from_rgb(0xf0, 0xc0, 0x5a);
pub const BAD: Color32 = Color32::from_rgb(0xf2, 0xb8, 0xb5); // error
pub const ON_BAD: Color32 = Color32::from_rgb(0x60, 0x14, 0x10);

// --- Shape scale (M3): extraLarge for big card containers,
// large for nested elements, full for pills (chips, search, indicators) ---
pub const SHAPE_EXTRA_LARGE: CornerRadius = CornerRadius::same(28);
pub const SHAPE_LARGE: CornerRadius = CornerRadius::same(16);
pub const SHAPE_FULL: CornerRadius = CornerRadius::same(255);

pub fn level_color(percent: f32, warn_at: f32, bad_at: f32) -> Color32 {
    if percent >= bad_at {
        BAD
    } else if percent >= warn_at {
        WARN
    } else {
        GOOD
    }
}

pub fn apply(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();
    style.visuals = egui::Visuals::dark();
    style.visuals.window_fill = SURFACE_DIM;
    style.visuals.panel_fill = SURFACE_DIM;
    style.visuals.widgets.noninteractive.bg_fill = SURFACE_CONTAINER;
    style.visuals.widgets.inactive.bg_fill = SURFACE_CONTAINER_HIGH;
    style.visuals.widgets.inactive.weak_bg_fill = SURFACE_CONTAINER_HIGH;
    style.visuals.widgets.hovered.bg_fill = SURFACE_CONTAINER_HIGHEST;
    style.visuals.widgets.active.bg_fill = SECONDARY_CONTAINER;
    style.visuals.selection.bg_fill = PRIMARY.linear_multiply(0.35);
    style.visuals.override_text_color = Some(ON_SURFACE);
    // egui draws a ProgressBar's empty track in `extreme_bg_color`. Left at
    // its dark default it's invisible against our near-black cards, so an
    // empty/near-zero bar looked like a stray dot (egui always paints a
    // minimum-width filled stub). Make the track visibly lighter than the
    // card so an empty bar reads as "empty bar", not "missing bar".
    style.visuals.extreme_bg_color = SURFACE_CONTAINER_HIGH;
    // Remove the harsh 1px widget borders — M3 distinguishes surfaces
    // by tonal elevation, not by outline.
    style.visuals.widgets.noninteractive.bg_stroke = Stroke::NONE;
    style.visuals.widgets.inactive.bg_stroke = Stroke::NONE;
    style.visuals.widgets.hovered.bg_stroke = Stroke::new(1.0_f32, PRIMARY.linear_multiply(0.5));
    style.visuals.widgets.active.bg_stroke = Stroke::NONE;
    for w in [
        &mut style.visuals.widgets.noninteractive,
        &mut style.visuals.widgets.inactive,
        &mut style.visuals.widgets.hovered,
        &mut style.visuals.widgets.active,
    ] {
        w.corner_radius = SHAPE_FULL;
    }
    // Tight layout: minimal air between elements, cards sit
    // "shoulder to shoulder" without large empty gaps.
    style.spacing.item_spacing = egui::vec2(8.0, 8.0);
    style.spacing.button_padding = egui::vec2(12.0, 6.0);
    ctx.set_style(style);
}

/// Material 3 Expressive card: a large surface with tonal elevation
/// (surface-container), no outline, generous inner padding.
pub fn card(ui: &mut egui::Ui, add_contents: impl FnOnce(&mut egui::Ui)) {
    card_squish(ui, 0.0, add_contents);
}

/// Same thing, but with a "squish" 0..1 (see [`crate::anim::Anim::width`]):
/// on collision with the edge of its column/window the card doesn't
/// overlap its neighbor, it squashes slightly instead — padding and
/// corner radius shrink a bit, as if the card really did bump into something.
pub fn card_squish(ui: &mut egui::Ui, squish: f32, add_contents: impl FnOnce(&mut egui::Ui)) {
    let squish = squish.clamp(0.0, 1.0);
    let shrink = 1.0 - squish * 0.4;
    let margin = egui::Margin::symmetric((14.0 * shrink) as i8, (12.0 * shrink) as i8);
    let radius = (16.0 * (1.0 - squish * 0.5)) as u8;
    egui::Frame::new()
        .fill(SURFACE_CONTAINER)
        .corner_radius(CornerRadius::same(radius))
        .inner_margin(margin)
        .show(ui, add_contents);
}

/// Card title in M3 "overline"/label style: small, tracked out,
/// muted, uppercase.
pub fn card_title(ui: &mut egui::Ui, text: &str) {
    ui.label(
        egui::RichText::new(text.to_uppercase())
            .color(ON_SURFACE_VARIANT)
            .size(12.0)
            .extra_letter_spacing(1.2),
    );
}

/// M3 filter chip: a pill, filled (primary-container) when selected,
/// outlined/tonal otherwise. Returns true on click.
pub fn chip(ui: &mut egui::Ui, label: &str, selected: bool) -> bool {
    sort_chip(ui, label, selected, None)
}

/// Small up/down triangle drawn with painter primitives (not the unicode
/// "▼"/"▲" glyphs — like "●"/"🔍" they're missing from egui's default
/// font and render as tofu squares).
fn draw_sort_arrow(ui: &mut egui::Ui, color: egui::Color32, ascending: bool) {
    let size = 8.0;
    let (rect, _) = ui.allocate_exact_size(egui::vec2(size, size), egui::Sense::hover());
    let c = rect.center();
    let (top, bottom) = if ascending { (-1.0, 1.0) } else { (1.0, -1.0) };
    let points = vec![
        c + egui::vec2(0.0, top * size * 0.35),
        c + egui::vec2(-size * 0.35, bottom * size * 0.35),
        c + egui::vec2(size * 0.35, bottom * size * 0.35),
    ];
    ui.painter().add(egui::Shape::convex_polygon(points, color, egui::Stroke::NONE));
}

/// Same chip, with an optional sort-direction arrow next to the label —
/// used for sortable table headers. `arrow = Some(descending)`.
pub fn sort_chip(ui: &mut egui::Ui, label: &str, selected: bool, arrow: Option<bool>) -> bool {
    // Pills in Expressive are always colorful (not "dark gray by default"):
    // selected is a bright primary, unselected is a muted tonal color.
    let (fill, text_color) =
        if selected { (PRIMARY, ON_PRIMARY) } else { (SURFACE_CONTAINER_HIGH, ON_SURFACE) };
    let resp = egui::Frame::new()
        .fill(fill)
        .corner_radius(SHAPE_FULL)
        .inner_margin(egui::Margin::symmetric(14, 8))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(label).color(text_color).size(13.0).strong());
                if let Some(desc) = arrow {
                    draw_sort_arrow(ui, text_color, !desc);
                }
            });
        })
        .response;
    let resp = resp.interact(egui::Sense::click());
    if resp.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    resp.clicked()
}

/// Search icon drawn with painter primitives (a circle + a handle),
/// not the unicode glyph "🔍" — it's not in egui's default font and
/// renders as an empty tofu square.
fn draw_search_glyph(ui: &mut egui::Ui, size: f32) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(size, size), egui::Sense::hover());
    let painter = ui.painter();
    let r = size * 0.32;
    let center = rect.center() - egui::vec2(size * 0.12, size * 0.12);
    let stroke = Stroke::new(1.6_f32, ON_SURFACE_VARIANT);
    painter.circle_stroke(center, r, stroke);
    let dir = egui::vec2(1.0, 1.0).normalized();
    painter.line_segment([center + dir * r, center + dir * (r + size * 0.32)], stroke);
}

/// M3 search bar: a fully rounded pill text field with a magnifier icon.
pub fn search_bar(ui: &mut egui::Ui, text: &mut String, hint: &str, width: f32) {
    egui::Frame::new()
        .fill(SURFACE_CONTAINER_HIGH)
        .corner_radius(SHAPE_FULL)
        .inner_margin(egui::Margin::symmetric(16, 6))
        .show(ui, |ui| {
            ui.set_width(width);
            ui.horizontal(|ui| {
                draw_search_glyph(ui, 14.0);
                ui.add(
                    egui::TextEdit::singleline(text)
                        .hint_text(hint)
                        .frame(false)
                        .desired_width(width - 48.0),
                );
            });
        });
}
