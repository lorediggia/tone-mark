use crate::data::library::TagColor;
use crate::ui::theme::col;
use eframe::egui::{self, Color32, FontId, Pos2, RichText, Rounding, Stroke, Vec2};
use std::f32::consts::PI;

pub fn card<R>(ui: &mut egui::Ui, body: impl FnOnce(&mut egui::Ui) -> R) -> R {
    egui::Frame::none()
        .fill(col::CARD)
        .stroke(Stroke::new(1.0, col::STROKE))
        .rounding(Rounding::same(8.0))
        .inner_margin(egui::Margin::same(14.0))
        .show(ui, body)
        .inner
}

pub fn draw_arc(p: &egui::Painter, c: Pos2, r: f32, start: f32, span: f32, w: f32, color: Color32) {
    let segs = 48;
    let mut pts = Vec::with_capacity(segs + 1);
    for i in 0..=segs {
        let t = i as f32 / segs as f32;
        let a = start + span * t;
        pts.push(Pos2::new(c.x + a.cos() * r, c.y + a.sin() * r));
    }
    if pts.len() >= 2 {
        p.add(egui::Shape::line(pts, Stroke::new(w, color)));
    }
}

pub fn knob(
    ui: &mut egui::Ui,
    value: &mut u8,
    range: std::ops::RangeInclusive<u8>,
    label: &str,
) -> egui::Response {
    let desired = Vec2::new(64.0, 88.0);
    let (rect, mut resp) = ui.allocate_exact_size(desired, egui::Sense::click_and_drag());

    if resp.dragged() {
        let dy = resp.drag_delta().y;
        let step = if ui.input(|i| i.modifiers.shift) {
            0.15
        } else {
            0.6
        };
        let lo = *range.start() as f32;
        let hi = *range.end() as f32;
        let new = (*value as f32 - dy * step).clamp(lo, hi);
        if (new.round() as u8) != *value {
            *value = new.round() as u8;
            resp.mark_changed();
        }
    }
    if resp.double_clicked() {
        let lo = *range.start();
        let hi = *range.end();
        *value = lo + (hi - lo) / 2;
        resp.mark_changed();
    }

    let painter = ui.painter();
    let center = Pos2::new(rect.center().x, rect.top() + 30.0);
    let radius = 24.0;
    let lo = *range.start() as f32;
    let hi = *range.end() as f32;
    let pct = ((*value as f32 - lo) / (hi - lo)).clamp(0.0, 1.0);

    let start_a = PI * 0.75;
    let span = PI * 1.5;
    draw_arc(painter, center, radius + 5.0, start_a, span, 2.0, col::STROKE);
    draw_arc(
        painter,
        center,
        radius + 5.0,
        start_a,
        span * pct,
        2.5,
        if resp.hovered() {
            col::accent_hi()
        } else {
            col::accent()
        },
    );

    painter.circle_filled(center, radius, col::CARD_HI);
    painter.circle_filled(center - Vec2::new(0.0, 3.0), radius - 2.0, col::CARD);
    painter.circle_stroke(center, radius, Stroke::new(1.0, col::STROKE));

    let ang = start_a + span * pct;
    let p_out = Pos2::new(
        center.x + ang.cos() * (radius - 4.0),
        center.y + ang.sin() * (radius - 4.0),
    );
    let p_in = Pos2::new(
        center.x + ang.cos() * (radius - 12.0),
        center.y + ang.sin() * (radius - 12.0),
    );
    painter.line_segment([p_in, p_out], Stroke::new(2.5, col::TEXT));

    painter.text(
        center,
        egui::Align2::CENTER_CENTER,
        format!("{}", *value),
        FontId::proportional(11.0),
        col::TEXT_DIM,
    );
    painter.text(
        Pos2::new(rect.center().x, rect.bottom() - 6.0),
        egui::Align2::CENTER_CENTER,
        label,
        FontId::proportional(10.5),
        col::TEXT_DIM,
    );
    resp
}

pub fn led(ui: &mut egui::Ui, on: bool) -> egui::Response {
    let size = Vec2::splat(12.0);
    let (rect, resp) = ui.allocate_exact_size(size, egui::Sense::click());
    let painter = ui.painter();
    let c = rect.center();
    if on {
        let a = col::accent();
        for i in 0..4 {
            let alpha = 18 + i as u8 * 6;
            painter.circle_filled(
                c,
                8.0 - i as f32,
                Color32::from_rgba_unmultiplied(a.r(), a.g(), a.b(), alpha),
            );
        }
        painter.circle_filled(c, 4.0, col::accent_hi());
    } else {
        painter.circle_filled(c, 4.0, col::TEXT_FAINT);
    }
    resp
}

pub fn block_header(ui: &mut egui::Ui, title: &str, on: &mut bool) -> bool {
    let mut toggled = false;
    ui.horizontal(|ui| {
        if led(ui, *on).clicked() {
            *on = !*on;
            toggled = true;
        }
        ui.add_space(6.0);
        ui.label(
            RichText::new(title)
                .size(13.0)
                .strong()
                .extra_letter_spacing(1.5)
                .color(col::TEXT),
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let txt = if *on { "ON" } else { "OFF" };
            let col_b = if *on { col::accent() } else { col::TEXT_FAINT };
            if ui
                .add(
                    egui::Button::new(RichText::new(txt).color(col_b).size(11.0))
                        .fill(col::CARD)
                        .stroke(Stroke::new(1.0, col::STROKE))
                        .rounding(Rounding::same(4.0)),
                )
                .clicked()
            {
                *on = !*on;
                toggled = true;
            }
        });
    });
    toggled
}

pub fn type_combo(
    ui: &mut egui::Ui,
    id_source: &str,
    label: &str,
    items: &[&str],
    current: &mut usize,
) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
        ui.label(RichText::new(label).size(10.0).color(col::TEXT_DIM));
        let preview = items.get(*current).copied().unwrap_or("?");
        egui::ComboBox::from_id_source(id_source)
            .selected_text(RichText::new(preview).color(col::TEXT))
            .width(200.0)
            .show_ui(ui, |ui| {
                for (i, name) in items.iter().enumerate() {
                    if ui.selectable_label(*current == i, *name).clicked() {
                        if *current != i {
                            changed = true;
                        }
                        *current = i;
                    }
                }
            });
    });
    changed
}

pub fn status_dot(ui: &mut egui::Ui, color: Color32, with_glow: bool) {
    let size = Vec2::splat(if with_glow { 14.0 } else { 8.0 });
    let (rect, _) = ui.allocate_exact_size(size, egui::Sense::hover());
    let p = ui.painter();
    let c = rect.center();
    if with_glow {
        for i in 0..3 {
            let alpha = 22 + i as u8 * 8;
            p.circle_filled(
                c,
                6.0 - i as f32 * 1.2,
                Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), alpha),
            );
        }
    }
    p.circle_filled(c, 3.5, color);
}

pub fn tag_chip(
    ui: &mut egui::Ui,
    label: &str,
    color: TagColor,
    selected: bool,
    removable: bool,
) -> (egui::Response, bool) {
    let (fg, bg) = color.pair();
    let font = FontId::proportional(10.5);
    let galley = ui.fonts(|f| f.layout_no_wrap(label.to_string(), font.clone(), fg));
    let pad_x = 10.0;
    let extra_x = if removable { 14.0 } else { 0.0 };
    let w = galley.size().x + pad_x * 2.0 + extra_x;
    let h = 22.0;

    let (rect, resp) = ui.allocate_exact_size(Vec2::new(w, h), egui::Sense::click());
    let p = ui.painter();
    let fill = if selected {
        Color32::from_rgba_unmultiplied(fg.r(), fg.g(), fg.b(), 60)
    } else {
        bg
    };
    p.rect_filled(rect, Rounding::same(11.0), fill);
    let stroke_w = if selected || resp.hovered() { 1.0 } else { 0.0 };
    if stroke_w > 0.0 {
        p.rect_stroke(rect, Rounding::same(11.0), Stroke::new(stroke_w, fg));
    }

    let text_pos = Pos2::new(
        rect.left() + pad_x + galley.size().x * 0.5,
        rect.center().y,
    );
    p.text(text_pos, egui::Align2::CENTER_CENTER, label, font, fg);

    let mut x_clicked = false;
    if removable {
        let x_center = Pos2::new(rect.right() - pad_x, rect.center().y);
        let half = 3.5;
        p.line_segment(
            [
                Pos2::new(x_center.x - half, x_center.y - half),
                Pos2::new(x_center.x + half, x_center.y + half),
            ],
            Stroke::new(1.2, fg),
        );
        p.line_segment(
            [
                Pos2::new(x_center.x - half, x_center.y + half),
                Pos2::new(x_center.x + half, x_center.y - half),
            ],
            Stroke::new(1.2, fg),
        );
        if resp.clicked() {
            if let Some(pos) = resp.interact_pointer_pos() {
                if (pos.x - x_center.x).abs() < 8.0 {
                    x_clicked = true;
                }
            }
        }
    }
    (resp, x_clicked)
}

pub fn small_label(ui: &mut egui::Ui, text: &str, height: f32, color: Color32) {
    let font = FontId::proportional(10.0);
    let galley = ui.fonts(|f| f.layout_no_wrap(text.to_string(), font.clone(), color));
    let w = galley.size().x + 4.0;
    let (rect, _) = ui.allocate_exact_size(Vec2::new(w, height), egui::Sense::hover());
    ui.painter().text(
        Pos2::new(rect.left() + 2.0, rect.center().y),
        egui::Align2::LEFT_CENTER,
        text,
        font,
        color,
    );
}

pub fn color_swatch(ui: &mut egui::Ui, color: TagColor, selected: bool) -> egui::Response {
    let size = Vec2::splat(26.0);
    let (rect, resp) = ui.allocate_exact_size(size, egui::Sense::click());
    let (fg, _) = color.pair();
    let p = ui.painter();
    let c = rect.center();
    if selected {
        p.circle_stroke(c, 11.5, Stroke::new(2.0, col::TEXT));
    } else if resp.hovered() {
        p.circle_stroke(c, 11.5, Stroke::new(1.0, col::TEXT_DIM));
    }
    p.circle_filled(c, 8.5, fg);
    resp
}

pub fn close_button(ui: &mut egui::Ui) -> egui::Response {
    let size = Vec2::splat(26.0);
    let (rect, resp) = ui.allocate_exact_size(size, egui::Sense::click());
    let p = ui.painter();
    let bg = if resp.hovered() {
        col::CARD_HI
    } else {
        Color32::TRANSPARENT
    };
    let fg = if resp.hovered() {
        col::TEXT
    } else {
        col::TEXT_DIM
    };
    p.rect_filled(rect, Rounding::same(6.0), bg);
    let c = rect.center();
    let half = 5.0;
    p.line_segment(
        [
            Pos2::new(c.x - half, c.y - half),
            Pos2::new(c.x + half, c.y + half),
        ],
        Stroke::new(1.5, fg),
    );
    p.line_segment(
        [
            Pos2::new(c.x - half, c.y + half),
            Pos2::new(c.x + half, c.y - half),
        ],
        Stroke::new(1.5, fg),
    );
    resp
}
