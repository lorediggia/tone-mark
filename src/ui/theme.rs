use eframe::egui;
use eframe::egui::{Color32, Rounding, Stroke, Vec2};
use std::sync::atomic::{AtomicU32, Ordering};

pub mod col {
    use super::{AtomicU32, Color32, Ordering};

    pub const BG: Color32 = Color32::from_rgb(14, 14, 16);
    pub const PANEL: Color32 = Color32::from_rgb(20, 20, 23);
    pub const CARD: Color32 = Color32::from_rgb(28, 28, 32);
    pub const CARD_HI: Color32 = Color32::from_rgb(36, 36, 41);
    pub const STROKE: Color32 = Color32::from_rgb(50, 50, 56);
    pub const TEXT: Color32 = Color32::from_rgb(225, 225, 230);
    pub const TEXT_DIM: Color32 = Color32::from_rgb(135, 135, 145);
    pub const TEXT_FAINT: Color32 = Color32::from_rgb(85, 85, 92);
    pub const OK: Color32 = Color32::from_rgb(70, 200, 130);
    pub const WARN: Color32 = Color32::from_rgb(230, 180, 60);
    pub const ERR: Color32 = Color32::from_rgb(220, 80, 80);

    static ACCENT_RGB: AtomicU32 = AtomicU32::new(0x00E87A26);
    static ACCENT_HI_RGB: AtomicU32 = AtomicU32::new(0x00FF963C);

    fn unpack(v: u32) -> Color32 {
        Color32::from_rgb((v >> 16) as u8, (v >> 8) as u8, v as u8)
    }
    fn pack(c: Color32) -> u32 {
        ((c.r() as u32) << 16) | ((c.g() as u32) << 8) | (c.b() as u32)
    }

    pub fn accent() -> Color32 {
        unpack(ACCENT_RGB.load(Ordering::Relaxed))
    }
    pub fn accent_hi() -> Color32 {
        unpack(ACCENT_HI_RGB.load(Ordering::Relaxed))
    }
    pub fn set_accent(c: Color32, c_hi: Color32) {
        ACCENT_RGB.store(pack(c), Ordering::Relaxed);
        ACCENT_HI_RGB.store(pack(c_hi), Ordering::Relaxed);
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum PaletteId {
    Amber,
    Cyan,
    Crimson,
    Lime,
    Violet,
    Pink,
}

impl PaletteId {
    pub const ALL: [PaletteId; 6] = [
        Self::Amber,
        Self::Cyan,
        Self::Crimson,
        Self::Lime,
        Self::Violet,
        Self::Pink,
    ];

    pub fn id(self) -> &'static str {
        match self {
            Self::Amber => "amber",
            Self::Cyan => "cyan",
            Self::Crimson => "crimson",
            Self::Lime => "lime",
            Self::Violet => "violet",
            Self::Pink => "pink",
        }
    }

    pub fn from_id(s: &str) -> PaletteId {
        match s {
            "cyan" => Self::Cyan,
            "crimson" => Self::Crimson,
            "lime" => Self::Lime,
            "violet" => Self::Violet,
            "pink" => Self::Pink,
            _ => Self::Amber,
        }
    }

    pub fn pair(self) -> (Color32, Color32) {
        match self {
            Self::Amber => (
                Color32::from_rgb(232, 122, 38),
                Color32::from_rgb(255, 150, 60),
            ),
            Self::Cyan => (
                Color32::from_rgb(56, 200, 224),
                Color32::from_rgb(110, 225, 245),
            ),
            Self::Crimson => (
                Color32::from_rgb(225, 70, 85),
                Color32::from_rgb(250, 105, 120),
            ),
            Self::Lime => (
                Color32::from_rgb(150, 210, 90),
                Color32::from_rgb(180, 235, 120),
            ),
            Self::Violet => (
                Color32::from_rgb(180, 110, 240),
                Color32::from_rgb(210, 150, 255),
            ),
            Self::Pink => (
                Color32::from_rgb(240, 110, 180),
                Color32::from_rgb(255, 150, 205),
            ),
        }
    }

    pub fn apply(self) {
        let (a, ahi) = self.pair();
        col::set_accent(a, ahi);
    }
}

pub fn apply_theme(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::dark();
    visuals.window_fill = col::BG;
    visuals.panel_fill = col::BG;
    visuals.extreme_bg_color = col::PANEL;
    visuals.faint_bg_color = col::CARD;

    let r = Rounding::same(6.0);
    visuals.widgets.noninteractive.rounding = r;
    visuals.widgets.inactive.rounding = r;
    visuals.widgets.hovered.rounding = r;
    visuals.widgets.active.rounding = r;
    visuals.widgets.open.rounding = r;

    visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, col::STROKE);
    visuals.widgets.inactive.bg_fill = col::CARD;
    visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, col::STROKE);
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, col::TEXT_DIM);
    visuals.widgets.hovered.bg_fill = col::CARD_HI;
    visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, col::accent());
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, col::TEXT);
    visuals.widgets.active.bg_fill = col::CARD_HI;
    visuals.widgets.active.bg_stroke = Stroke::new(1.2, col::accent());
    visuals.widgets.active.fg_stroke = Stroke::new(1.0, col::TEXT);

    visuals.selection.bg_fill = col::accent().linear_multiply(0.35);
    visuals.selection.stroke = Stroke::new(1.0, col::accent());
    visuals.override_text_color = Some(col::TEXT);

    ctx.set_visuals(visuals);

    let mut style: egui::Style = (*ctx.style()).clone();
    style.spacing.item_spacing = Vec2::new(8.0, 8.0);
    style.spacing.button_padding = Vec2::new(12.0, 6.0);
    style.spacing.slider_width = 200.0;
    ctx.set_style(style);
}
