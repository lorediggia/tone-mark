use std::path::PathBuf;
use eframe::egui::{self, Color32, RichText, Rounding, Stroke, Vec2};

use crate::app::ToneApp;
use crate::data::library::{Tag, TagColor};
use crate::midi::katana::{
    ad,
    AMP_TYPES, AMP_TYPE_VALUES, BOOSTER_TYPES, BOOSTER_TYPE_VALUES,
    MOD_TYPES, MOD_TYPE_VALUES, FX_TYPES, FX_TYPE_VALUES,
    DELAY_TYPES, DELAY_TYPE_VALUES, REVERB_TYPES, REVERB_TYPE_VALUES,
};
use crate::ui::theme::{col, PaletteId};
use crate::ui::widgets::*;

pub trait AppViews {
    fn ui_library(&mut self, ui: &mut egui::Ui);
    fn ui_editor(&mut self, ui: &mut egui::Ui);
    fn ui_settings(&mut self, ui: &mut egui::Ui);

    fn ui_amp_section(&mut self, ui: &mut egui::Ui);
    fn ui_booster_section(&mut self, ui: &mut egui::Ui);
    fn ui_mod_section(&mut self, ui: &mut egui::Ui);
    fn ui_fx_section(&mut self, ui: &mut egui::Ui);
    fn ui_delay_section(&mut self, ui: &mut egui::Ui);
    fn ui_reverb_section(&mut self, ui: &mut egui::Ui);
    fn ui_ns_section(&mut self, ui: &mut egui::Ui);
    fn ui_signal_chain(&mut self, ui: &mut egui::Ui);
    fn ui_tag_manager(&mut self, ui: &mut egui::Ui);
}

const PANEL_MIN_H: f32 = 168.0;

fn type_dependent_fill(ui: &mut egui::Ui, name: &str, on: bool) {
    ui.add_space(20.0);
}

impl AppViews for ToneApp {
    fn ui_library(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if ui.button("Choose Folder...").clicked() {
                if let Some(folder) = rfd::FileDialog::new().pick_folder() {
                    self.library_path = Some(folder);
                    self.refresh_library();
                    self.save_config();
                }
            }
            if ui.button("Refresh").clicked() {
                self.refresh_library();
            }
            if let Some(p) = &self.library_path {
                ui.add_space(10.0);
                ui.label(
                    RichText::new(p.display().to_string())
                        .color(col::TEXT_DIM)
                        .size(11.0),
                );
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let manage_label = if self.show_tag_manager {
                    "HIDE TAGS"
                } else {
                    "MANAGE TAGS"
                };
                if ui
                    .add(
                        egui::Button::new(RichText::new(manage_label).size(10.5).color(col::TEXT_DIM))
                            .fill(col::CARD)
                            .stroke(Stroke::new(1.0, col::STROKE))
                            .rounding(Rounding::same(5.0)),
                    )
                    .clicked()
                {
                    self.show_tag_manager = !self.show_tag_manager;
                }
            });
        });
        ui.add_space(10.0);

        if self.show_tag_manager {
            self.ui_tag_manager(ui);
            ui.add_space(10.0);
        }

        ui.horizontal(|ui| {
            small_label(ui, "SEARCH", 24.0, col::TEXT_DIM);
            ui.add_space(4.0);
            ui.add(
                egui::TextEdit::singleline(&mut self.library_search)
                    .hint_text("name...")
                    .desired_width(260.0),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(
                    RichText::new(format!("{} patches", self.patches.len()))
                        .color(col::TEXT_DIM)
                        .size(11.0),
                );
            });
        });
        ui.add_space(8.0);

        if !self.library_meta.tags.is_empty() {
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing = Vec2::new(6.0, 6.0);
                small_label(ui, "FILTER", 22.0, col::TEXT_DIM);
                ui.add_space(2.0);

                let mut toggle: Option<String> = None;
                for tag in &self.library_meta.tags {
                    let selected = self.active_tag_filters.contains(&tag.name);
                    let (resp, _) = tag_chip(ui, &tag.name, tag.color, selected, false);
                    if resp.clicked() {
                        toggle = Some(tag.name.clone());
                    }
                }
                if let Some(n) = toggle {
                    if let Some(pos) = self.active_tag_filters.iter().position(|x| x == &n) {
                        self.active_tag_filters.remove(pos);
                    } else {
                        self.active_tag_filters.push(n);
                    }
                }
                if !self.active_tag_filters.is_empty() {
                    ui.add_space(6.0);
                    if ui
                        .add(
                            egui::Button::new(RichText::new("CLEAR").size(10.0).color(col::TEXT_FAINT))
                                .fill(col::PANEL)
                                .stroke(Stroke::new(1.0, col::STROKE))
                                .rounding(Rounding::same(11.0))
                                .min_size(Vec2::new(0.0, 22.0)),
                        )
                        .clicked()
                    {
                        self.active_tag_filters.clear();
                    }
                }
            });
            ui.add_space(10.0);
        }

        let q = self.library_search.to_lowercase();
        let active_filters = self.active_tag_filters.clone();
        let library_path = self.library_path.clone();
        let assignments_ref = &self.library_meta.assignments;

        let filtered: Vec<PathBuf> = self
            .patches
            .iter()
            .filter(|p| {
                let name_ok = if q.is_empty() {
                    true
                } else {
                    p.file_stem()
                        .and_then(|s| s.to_str())
                        .map(|s| s.to_lowercase().contains(&q))
                        .unwrap_or(false)
                };
                if !name_ok {
                    return false;
                }
                if active_filters.is_empty() {
                    return true;
                }
                let rel = library_path
                    .as_ref()
                    .and_then(|b| p.strip_prefix(b).ok())
                    .map(|r| r.to_string_lossy().replace('\\', "/"))
                    .unwrap_or_default();
                match assignments_ref.get(&rel) {
                    Some(patch_tags) => active_filters.iter().all(|t| patch_tags.contains(t)),
                    None => false,
                }
            })
            .cloned()
            .collect();

        let mut clicked_patch: Option<PathBuf> = None;
        let mut apply_request = false;
        let mut tag_popup_request: Option<String> = None;
        let mut tag_toggle: Option<(String, String)> = None;
        let mut tag_remove: Option<(String, String)> = None;
        let connected = self.midi_conn.is_some();

        egui::ScrollArea::vertical()
            .id_source("library_cards")
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                if filtered.is_empty() {
                    ui.add_space(40.0);
                    ui.vertical_centered(|ui| {
                        let msg = if self.patches.is_empty() {
                            "No patches found. Choose a folder containing .tsl files."
                        } else {
                            "No patches match the current filter."
                        };
                        ui.label(RichText::new(msg).color(col::TEXT_FAINT).size(12.0));
                    });
                    return;
                }

                for patch_path in &filtered {
                    let rel = self.rel_path(patch_path).unwrap_or_default();
                    let is_sel = self.selected_patch.as_ref() == Some(patch_path);
                    let filename = patch_path
                        .file_name()
                        .map(|s| s.to_string_lossy().to_string())
                        .unwrap_or_default();
                    let display_name = patch_path
                        .file_stem()
                        .map(|s| s.to_string_lossy().to_string())
                        .unwrap_or_else(|| filename.clone());
                    let patch_tags = self.library_meta.tags_for(&rel);

                    let frame_resp = egui::Frame::none()
                        .fill(if is_sel { col::CARD_HI } else { col::CARD })
                        .stroke(Stroke::new(
                            if is_sel { 1.5 } else { 1.0 },
                            if is_sel { col::accent() } else { col::STROKE },
                        ))
                        .rounding(Rounding::same(8.0))
                        .inner_margin(egui::Margin::symmetric(16.0, 12.0))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.vertical(|ui| {
                                    ui.label(
                                        RichText::new(&display_name)
                                            .size(15.0)
                                            .strong()
                                            .color(col::TEXT),
                                    );
                                    ui.add_space(2.0);
                                    ui.label(
                                        RichText::new(&filename)
                                            .size(10.0)
                                            .color(col::TEXT_FAINT),
                                    );
                                });
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        ui.spacing_mut().item_spacing.x = 6.0;
                                        ui.push_id(&rel, |ui| {
                                            if ui
                                                .add(
                                                    egui::Button::new(
                                                        RichText::new("+")
                                                            .size(13.0)
                                                            .color(col::TEXT_DIM),
                                                    )
                                                    .fill(col::PANEL)
                                                    .stroke(Stroke::new(1.0, col::STROKE))
                                                    .rounding(Rounding::same(11.0))
                                                    .min_size(Vec2::new(22.0, 22.0)),
                                                )
                                                .clicked()
                                            {
                                                tag_popup_request = Some(rel.clone());
                                            }
                                        });
                                        for tag in patch_tags.iter().rev() {
                                            let (resp, x_clicked) =
                                                tag_chip(ui, &tag.name, tag.color, false, is_sel);
                                            if x_clicked {
                                                tag_remove =
                                                    Some((rel.clone(), tag.name.clone()));
                                            } else if resp.clicked() {
                                                tag_toggle =
                                                    Some(("__filter__".into(), tag.name.clone()));
                                            }
                                        }
                                    },
                                );
                            });

                            if is_sel {
                                ui.add_space(10.0);
                                ui.separator();
                                ui.add_space(8.0);
                                ui.horizontal(|ui| {
                                    let btn_text = RichText::new(if connected {
                                        "APPLY TO AMP"
                                    } else {
                                        "CONNECT AMP FIRST"
                                    })
                                    .size(11.5)
                                    .color(if connected {
                                        col::accent()
                                    } else {
                                        col::TEXT_FAINT
                                    });
                                    let btn = egui::Button::new(btn_text)
                                        .fill(col::PANEL)
                                        .stroke(Stroke::new(
                                            1.0,
                                            if connected { col::accent() } else { col::STROKE },
                                        ))
                                        .rounding(Rounding::same(5.0))
                                        .min_size(Vec2::new(150.0, 30.0));
                                    if ui.add_enabled(connected, btn).clicked() {
                                        apply_request = true;
                                    }
                                    ui.label(
                                        RichText::new(format!("TARGET  {}", self.target.short()))
                                            .color(col::TEXT_FAINT)
                                            .size(11.0)
                                            .extra_letter_spacing(1.5),
                                    );
                                    if let Some(info) = &self.selected_patch_info {
                                        ui.with_layout(
                                            egui::Layout::right_to_left(egui::Align::Center),
                                            |ui| {
                                                if !info.memo.is_empty() {
                                                    ui.label(
                                                        RichText::new(&info.memo)
                                                            .color(col::TEXT_DIM)
                                                            .size(10.5)
                                                            .italics(),
                                                    );
                                                }
                                            },
                                        );
                                    }
                                });
                            }
                        });

                    let card_resp = frame_resp.response.interact(egui::Sense::click());
                    if card_resp.clicked() {
                        clicked_patch = Some(patch_path.clone());
                    }
                    if card_resp.double_clicked() {
                        clicked_patch = Some(patch_path.clone());
                        apply_request = true;
                    }

                    ui.add_space(8.0);
                }
            });

        if let Some(rel) = self.tag_popup_for.clone() {
            let pname = self
                .patches
                .iter()
                .find(|p| self.rel_path(p).as_deref() == Some(rel.as_str()))
                .and_then(|p| p.file_stem().map(|s| s.to_string_lossy().to_string()))
                .unwrap_or_else(|| rel.clone());

            let mut close = false;
            egui::Window::new("Tag assignment")
                .title_bar(false)
                .id(egui::Id::new("tag_assign_popup"))
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .frame(
                    egui::Frame::none()
                        .fill(col::PANEL)
                        .stroke(Stroke::new(1.0, col::STROKE))
                        .rounding(Rounding::same(12.0))
                        .inner_margin(egui::Margin::same(20.0))
                        .shadow(egui::epaint::Shadow {
                            offset: egui::Vec2::ZERO,
                            blur: 18.0,
                            spread: 2.0,
                            color: Color32::from_rgba_unmultiplied(0, 0, 0, 120),
                        }),
                )
                .show(ui.ctx(), |ui| {
                    ui.set_min_width(360.0);
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            ui.label(
                                RichText::new("TAGS")
                                    .size(9.5)
                                    .extra_letter_spacing(2.5)
                                    .color(col::accent()),
                            );
                            ui.add_space(2.0);
                            ui.label(
                                RichText::new(&pname)
                                    .size(15.0)
                                    .strong()
                                    .color(col::TEXT),
                            );
                        });
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if close_button(ui).clicked() {
                                close = true;
                            }
                        });
                    });
                    ui.add_space(16.0);

                    let assigned: Vec<String> = self
                        .library_meta
                        .assignments
                        .get(&rel)
                        .cloned()
                        .unwrap_or_default();

                    ui.horizontal_wrapped(|ui| {
                        ui.spacing_mut().item_spacing = Vec2::new(6.0, 6.0);
                        for tag in &self.library_meta.tags {
                            let on = assigned.contains(&tag.name);
                            let (resp, _) = tag_chip(ui, &tag.name, tag.color, on, false);
                            if resp.clicked() {
                                tag_toggle = Some((rel.clone(), tag.name.clone()));
                            }
                        }
                    });
                });
            if close {
                self.tag_popup_for = None;
            }
        }

        if let Some(p) = clicked_patch {
            let changed = self.selected_patch.as_ref() != Some(&p);
            self.selected_patch = Some(p.clone());
            if changed {
                self.selected_patch_info = Self::parse_patch_info(&p);
            }
        }
        if let Some(req_rel) = tag_popup_request {
            self.tag_popup_for = Some(req_rel);
        }
        if let Some((rel, tag_name)) = tag_toggle {
            if rel == "__filter__" {
                if let Some(pos) = self.active_tag_filters.iter().position(|x| x == &tag_name) {
                    self.active_tag_filters.remove(pos);
                } else {
                    self.active_tag_filters.push(tag_name);
                }
            } else {
                self.library_meta.toggle_assignment(&rel, &tag_name);
                self.save_library_meta();
            }
        }
        if let Some((rel, tag_name)) = tag_remove {
            self.library_meta.toggle_assignment(&rel, &tag_name);
            self.save_library_meta();
        }
        if apply_request {
            self.apply_patch();
        }
    }

    fn ui_editor(&mut self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical().auto_shrink([false; 2]).show(ui, |ui| {
            self.ui_signal_chain(ui);
            ui.add_space(16.0);
            self.ui_amp_section(ui);
            ui.add_space(12.0);
            ui.columns(2, |cols| {
                self.ui_booster_section(&mut cols[0]);
                self.ui_mod_section(&mut cols[1]);
            });
            ui.add_space(12.0);
            ui.columns(2, |cols| {
                self.ui_fx_section(&mut cols[0]);
                self.ui_delay_section(&mut cols[1]);
            });
            ui.add_space(12.0);
            ui.columns(2, |cols| {
                self.ui_reverb_section(&mut cols[0]);
                self.ui_ns_section(&mut cols[1]);
            });
            ui.add_space(16.0);

            card(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("A/B COMPARE")
                            .size(11.0)
                            .extra_letter_spacing(2.0)
                            .color(col::accent()),
                    );
                    ui.add_space(14.0);
                    let has_a = self.snap_a.is_some();
                    let has_b = self.snap_b.is_some();
                    let snap_btn = |ui: &mut egui::Ui, txt: &str, filled: bool| -> egui::Response {
                        let rt = RichText::new(txt)
                            .size(11.0)
                            .color(if filled { col::accent() } else { col::TEXT_DIM });
                        ui.add(
                            egui::Button::new(rt)
                                .fill(col::CARD)
                                .stroke(Stroke::new(
                                    1.0,
                                    if filled { col::accent() } else { col::STROKE },
                                ))
                                .rounding(Rounding::same(5.0))
                                .min_size(Vec2::new(70.0, 28.0)),
                        )
                    };
                    if snap_btn(ui, "SAVE A", has_a).clicked() {
                        self.snap_a = Some(self.make_snapshot());
                        self.status_msg = "Snapshot A saved.".into();
                    }
                    if ui
                        .add_enabled(
                            has_a,
                            egui::Button::new(RichText::new("RECALL A").size(11.0).color(col::TEXT))
                                .fill(col::CARD)
                                .stroke(Stroke::new(1.0, col::STROKE))
                                .rounding(Rounding::same(5.0))
                                .min_size(Vec2::new(82.0, 28.0)),
                        )
                        .clicked()
                    {
                        if let Some(s) = self.snap_a {
                            self.load_snapshot(s);
                            self.status_msg = "Recalled A.".into();
                        }
                    }
                    ui.add_space(10.0);
                    if snap_btn(ui, "SAVE B", has_b).clicked() {
                        self.snap_b = Some(self.make_snapshot());
                        self.status_msg = "Snapshot B saved.".into();
                    }
                    if ui
                        .add_enabled(
                            has_b,
                            egui::Button::new(RichText::new("RECALL B").size(11.0).color(col::TEXT))
                                .fill(col::CARD)
                                .stroke(Stroke::new(1.0, col::STROKE))
                                .rounding(Rounding::same(5.0))
                                .min_size(Vec2::new(82.0, 28.0)),
                        )
                        .clicked()
                    {
                        if let Some(s) = self.snap_b {
                            self.load_snapshot(s);
                            self.status_msg = "Recalled B.".into();
                        }
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui
                            .add(
                                egui::Button::new(
                                    RichText::new("RESET")
                                        .size(11.0)
                                        .extra_letter_spacing(1.5)
                                        .color(col::TEXT_DIM),
                                )
                                .fill(col::CARD)
                                .stroke(Stroke::new(1.0, col::STROKE))
                                .rounding(Rounding::same(5.0))
                                .min_size(Vec2::new(82.0, 28.0)),
                            )
                            .clicked()
                        {
                            self.reset_defaults();
                        }
                        ui.add_space(6.0);
                        if ui
                            .add(
                                egui::Button::new(
                                    RichText::new("EXPORT TSL")
                                        .size(11.0)
                                        .extra_letter_spacing(1.5)
                                        .color(col::accent()),
                                )
                                .fill(col::CARD)
                                .stroke(Stroke::new(1.0, col::accent()))
                                .rounding(Rounding::same(5.0))
                                .min_size(Vec2::new(110.0, 28.0)),
                            )
                            .clicked()
                        {
                            if let Some(path) = rfd::FileDialog::new()
                                .set_file_name("tone_mk2_dump.tsl")
                                .save_file()
                            {
                                match self.save_state_as_tsl(&path) {
                                    Ok(_) => self.status_msg = "State saved.".into(),
                                    Err(e) => self.status_msg = format!("Save error: {e}"),
                                }
                            }
                        }
                    });
                });
            });
        });
    }

    fn ui_tag_manager(&mut self, ui: &mut egui::Ui) {
        card(ui, |ui| {
            ui.label(
                RichText::new("TAGS")
                    .size(11.0)
                    .extra_letter_spacing(2.5)
                    .color(col::accent()),
            );
            ui.add_space(10.0);

            let mut remove_tag: Option<String> = None;
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing = Vec2::new(6.0, 6.0);
                if self.library_meta.tags.is_empty() {
                    ui.label(
                        RichText::new("(no tags yet)")
                            .color(col::TEXT_FAINT)
                            .size(11.0),
                    );
                }
                for tag in self.library_meta.tags.clone().iter() {
                    let (resp, x_clicked) = tag_chip(ui, &tag.name, tag.color, false, true);
                    if x_clicked {
                        remove_tag = Some(tag.name.clone());
                    } else if resp.clicked() {
                        if let Some(pos) =
                            self.active_tag_filters.iter().position(|x| x == &tag.name)
                        {
                            self.active_tag_filters.remove(pos);
                        } else {
                            self.active_tag_filters.push(tag.name.clone());
                        }
                    }
                }
            });
            if let Some(name) = remove_tag {
                self.library_meta.tags.retain(|t| t.name != name);
                for v in self.library_meta.assignments.values_mut() {
                    v.retain(|n| n != &name);
                }
                self.library_meta.assignments.retain(|_, v| !v.is_empty());
                self.active_tag_filters.retain(|n| n != &name);
                self.save_library_meta();
            }

            ui.add_space(12.0);
            ui.separator();
            ui.add_space(10.0);

            ui.horizontal(|ui| {
                small_label(ui, "NEW", 28.0, col::TEXT_DIM);
                ui.add_space(4.0);
                ui.add(
                    egui::TextEdit::singleline(&mut self.new_tag_name)
                        .hint_text("tag name...")
                        .desired_width(160.0),
                );
                ui.add_space(10.0);

                for c in TagColor::ALL {
                    let selected = c == self.new_tag_color;
                    if color_swatch(ui, c, selected).clicked() {
                        self.new_tag_color = c;
                    }
                }

                ui.add_space(12.0);
                let can_add = !self.new_tag_name.trim().is_empty()
                    && !self
                        .library_meta
                        .tags
                        .iter()
                        .any(|t| t.name.eq_ignore_ascii_case(self.new_tag_name.trim()));
                let add_btn = egui::Button::new(
                    RichText::new("ADD TAG")
                        .size(11.0)
                        .strong()
                        .extra_letter_spacing(1.5)
                        .color(if can_add { Color32::WHITE } else { col::TEXT_FAINT }),
                )
                .fill(if can_add { col::accent() } else { col::CARD })
                .stroke(Stroke::new(0.0, Color32::TRANSPARENT))
                .rounding(Rounding::same(14.0))
                .min_size(Vec2::new(100.0, 28.0));
                if ui.add_enabled(can_add, add_btn).clicked() {
                    self.library_meta.tags.push(Tag {
                        name: self.new_tag_name.trim().to_string(),
                        color: self.new_tag_color,
                    });
                    self.new_tag_name.clear();
                    self.save_library_meta();
                }
            });
        });
    }

    fn ui_settings(&mut self, ui: &mut egui::Ui) {
        card(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("PALETTE")
                        .size(11.0)
                        .extra_letter_spacing(2.5)
                        .color(col::accent()),
                );
                ui.add_space(18.0);

                let mut chosen: Option<PaletteId> = None;
                for p in PaletteId::ALL {
                    let selected = p == self.palette;
                    let (acc, _) = p.pair();
                    let size = Vec2::splat(28.0);
                    let (rect, resp) = ui.allocate_exact_size(size, egui::Sense::click());
                    let painter = ui.painter();
                    let c = rect.center();
                    if selected {
                        painter.circle_stroke(c, 12.5, Stroke::new(2.0, col::TEXT));
                    } else if resp.hovered() {
                        painter.circle_stroke(c, 12.5, Stroke::new(1.0, col::TEXT_DIM));
                    }
                    painter.circle_filled(c, 9.5, acc);
                    if resp.clicked() {
                        chosen = Some(p);
                    }
                }
                if let Some(p) = chosen {
                    if p != self.palette {
                        self.palette = p;
                        p.apply();
                        self.pending_theme_refresh = true;
                        self.save_config();
                    }
                }
            });
        });
    }

    fn ui_amp_section(&mut self, ui: &mut egui::Ui) {
        card(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("AMP")
                        .size(14.0)
                        .strong()
                        .extra_letter_spacing(2.5)
                        .color(col::accent()),
                );
                ui.add_space(12.0);
                let preview = AMP_TYPES.get(self.amp.type_idx).copied().unwrap_or("?");
                egui::ComboBox::from_id_source("amp_type")
                    .selected_text(RichText::new(preview).color(col::TEXT))
                    .width(200.0)
                    .show_ui(ui, |ui| {
                        let mut chosen: Option<usize> = None;
                        for (i, n) in AMP_TYPES.iter().enumerate() {
                            if ui.selectable_label(self.amp.type_idx == i, *n).clicked() {
                                chosen = Some(i);
                            }
                        }
                        if let Some(i) = chosen {
                            if self.amp.type_idx != i {
                                self.amp.type_idx = i;
                                self.send_param(
                                    ad::AMP_TYPE,
                                    AMP_TYPE_VALUES.get(i).copied().unwrap_or(i as u8),
                                );
                            }
                        }
                    });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let mut b = self.amp.bright;
                    if ui
                        .checkbox(
                            &mut b,
                            RichText::new("BRIGHT").color(col::TEXT_DIM).size(11.0),
                        )
                        .changed()
                    {
                        self.amp.bright = b;
                        self.send_param(ad::AMP_BRIGHT, b as u8);
                    }
                });
            });
            ui.add_space(10.0);
            ui.separator();
            ui.add_space(14.0);
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 14.0;
                if knob(ui, &mut self.amp.gain, 0..=120, "GAIN").changed() {
                    let v = self.amp.gain;
                    self.send_param(ad::AMP_GAIN, v);
                }
                if knob(ui, &mut self.amp.bass, 0..=100, "BASS").changed() {
                    let v = self.amp.bass;
                    self.send_param(ad::AMP_BASS, v);
                }
                if knob(ui, &mut self.amp.mid, 0..=100, "MIDDLE").changed() {
                    let v = self.amp.mid;
                    self.send_param(ad::AMP_MID, v);
                }
                if knob(ui, &mut self.amp.treble, 0..=100, "TREBLE").changed() {
                    let v = self.amp.treble;
                    self.send_param(ad::AMP_TREB, v);
                }
                if knob(ui, &mut self.amp.pres, 0..=100, "PRESENCE").changed() {
                    let v = self.amp.pres;
                    self.send_param(ad::AMP_PRES, v);
                }
                if knob(ui, &mut self.amp.vol, 0..=120, "VOLUME").changed() {
                    let v = self.amp.vol;
                    self.send_param(ad::AMP_VOL, v);
                }
                ui.add_space(10.0);
                if knob(ui, &mut self.amp.sag, 0..=100, "SAG").changed() {
                    let v = self.amp.sag;
                    self.send_param(ad::AMP_SAG, v);
                }
                if knob(ui, &mut self.amp.res, 0..=100, "RESON.").changed() {
                    let v = self.amp.res;
                    self.send_param(ad::AMP_RES, v);
                }
            });
        });
    }

    fn ui_booster_section(&mut self, ui: &mut egui::Ui) {
        card(ui, |ui| {
            ui.set_min_height(PANEL_MIN_H);
            let mut on = self.booster.on;
            if block_header(ui, "BOOSTER", &mut on) {
                self.booster.on = on;
                self.send_param(ad::BST_ON, on as u8);
            }
            ui.add_space(8.0);
            let mut idx = self.booster.type_idx;
            if type_combo(ui, "booster_type", "TYPE", BOOSTER_TYPES, &mut idx) {
                self.booster.type_idx = idx;
                let raw = BOOSTER_TYPE_VALUES.get(idx).copied().unwrap_or(idx as u8);
                self.send_param(ad::BST_TYPE, raw);
            }
            ui.add_space(10.0);
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 14.0;
                if knob(ui, &mut self.booster.p1, 0..=120, "DRIVE").changed() {
                    let v = self.booster.p1;
                    self.send_param(ad::BST_DRIVE, v);
                }
                if knob(ui, &mut self.booster.p2, 0..=100, "BOTTOM").changed() {
                    let v = self.booster.p2;
                    self.send_param(ad::BST_BOTTOM, v);
                }
                if knob(ui, &mut self.booster.p3, 0..=100, "TONE").changed() {
                    let v = self.booster.p3;
                    self.send_param(ad::BST_TONE, v);
                }
                if knob(ui, &mut self.booster.p4, 0..=100, "LEVEL").changed() {
                    let v = self.booster.p4;
                    self.send_param(ad::BST_LEVEL, v);
                }
            });
        });
    }

    fn ui_mod_section(&mut self, ui: &mut egui::Ui) {
        card(ui, |ui| {
            ui.set_min_height(PANEL_MIN_H);
            let mut on = self.mod_fx.on;
            if block_header(ui, "MOD", &mut on) {
                self.mod_fx.on = on;
                self.send_param(ad::MOD_ON, on as u8);
            }
            ui.add_space(8.0);
            let mut idx = self.mod_fx.type_idx;
            if type_combo(ui, "mod_type", "TYPE", MOD_TYPES, &mut idx) {
                self.mod_fx.type_idx = idx;
                let raw = MOD_TYPE_VALUES.get(idx).copied().unwrap_or(0);
                self.send_param(ad::MOD_TYPE, raw);
            }
            let name = MOD_TYPES.get(idx).copied().unwrap_or("—");
            type_dependent_fill(ui, name, self.mod_fx.on);
        });
    }

    fn ui_fx_section(&mut self, ui: &mut egui::Ui) {
        card(ui, |ui| {
            ui.set_min_height(PANEL_MIN_H);
            let mut on = self.fx.on;
            if block_header(ui, "FX", &mut on) {
                self.fx.on = on;
                self.send_param(ad::FX_ON, on as u8);
            }
            ui.add_space(8.0);
            let mut idx = self.fx.type_idx;
            if type_combo(ui, "fx_type", "TYPE", FX_TYPES, &mut idx) {
                self.fx.type_idx = idx;
                let raw = FX_TYPE_VALUES.get(idx).copied().unwrap_or(0);
                self.send_param(ad::FX_TYPE, raw);
            }
            let name = FX_TYPES.get(idx).copied().unwrap_or("—");
            type_dependent_fill(ui, name, self.fx.on);
        });
    }

    fn ui_delay_section(&mut self, ui: &mut egui::Ui) {
        card(ui, |ui| {
            ui.set_min_height(PANEL_MIN_H);
            let mut on = self.delay.on;
            if block_header(ui, "DELAY", &mut on) {
                self.delay.on = on;
                self.send_param(ad::DLY1_ON, on as u8);
            }
            ui.add_space(8.0);
            let mut idx = self.delay.type_idx;
            if type_combo(ui, "delay_type", "TYPE", DELAY_TYPES, &mut idx) {
                self.delay.type_idx = idx;
                let raw = DELAY_TYPE_VALUES.get(idx).copied().unwrap_or(0);
                self.send_param(ad::DLY1_TYPE, raw);
            }
            ui.add_space(10.0);
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 14.0;
                if knob(ui, &mut self.delay.time, 0..=100, "TIME").changed() {
                    let v = self.delay.time;
                    self.send_param(ad::DLY1_TIME, v);
                }
                if knob(ui, &mut self.delay.feedback, 0..=100, "FEEDBACK").changed() {
                    let v = self.delay.feedback;
                    self.send_param(ad::DLY1_FB, v);
                }
                if knob(ui, &mut self.delay.level, 0..=100, "LEVEL").changed() {
                    let v = self.delay.level;
                    self.send_param(ad::DLY1_LVL, v);
                }
            });
        });
    }

    fn ui_reverb_section(&mut self, ui: &mut egui::Ui) {
        card(ui, |ui| {
            ui.set_min_height(PANEL_MIN_H);
            let mut on = self.reverb.on;
            if block_header(ui, "REVERB", &mut on) {
                self.reverb.on = on;
                self.send_param(ad::REV_ON, on as u8);
            }
            ui.add_space(8.0);
            let mut idx = self.reverb.type_idx;
            if type_combo(ui, "reverb_type", "TYPE", REVERB_TYPES, &mut idx) {
                self.reverb.type_idx = idx;
                let raw = REVERB_TYPE_VALUES.get(idx).copied().unwrap_or(0);
                self.send_param(ad::REV_TYPE, raw);
            }
            ui.add_space(10.0);
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 14.0;
                if knob(ui, &mut self.reverb.time, 0..=100, "TIME").changed() {
                    let v = self.reverb.time;
                    self.send_param(ad::REV_TIME, v);
                }
                if knob(ui, &mut self.reverb.pre, 0..=100, "PRE").changed() {
                    let v = self.reverb.pre;
                    self.send_param(ad::REV_PRE, v);
                }
                if knob(ui, &mut self.reverb.density, 0..=100, "DENSITY").changed() {
                    let v = self.reverb.density;
                    self.send_param(ad::REV_DENS, v);
                }
                if knob(ui, &mut self.reverb.level, 0..=100, "LEVEL").changed() {
                    let v = self.reverb.level;
                    self.send_param(ad::REV_LVL, v);
                }
            });
        });
    }

    fn ui_ns_section(&mut self, ui: &mut egui::Ui) {
        card(ui, |ui| {
            ui.set_min_height(PANEL_MIN_H);
            let mut on = self.ns.on;
            if block_header(ui, "NOISE SUPPRESSOR", &mut on) {
                self.ns.on = on;
                self.send_param(ad::NS_ON, on as u8);
            }
            ui.add_space(10.0);
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 14.0;
                if knob(ui, &mut self.ns.threshold, 0..=100, "THRESH").changed() {
                    let v = self.ns.threshold;
                    self.send_param(ad::NS_THR, v);
                }
                if knob(ui, &mut self.ns.release, 0..=100, "RELEASE").changed() {
                    let v = self.ns.release;
                    self.send_param(ad::NS_REL, v);
                }
            });
        });
    }

    fn ui_signal_chain(&mut self, ui: &mut egui::Ui) {
        let h = 56.0;
        let (rect, _) = ui.allocate_exact_size(
            Vec2::new(ui.available_width(), h),
            egui::Sense::hover(),
        );
        let p = ui.painter();
        p.rect_filled(rect, Rounding::same(10.0), col::CARD);
        p.rect_stroke(rect, Rounding::same(10.0), Stroke::new(1.0, col::STROKE));

        let blocks: [(&str, bool, bool); 7] = [
            ("BST", self.booster.on, false),
            ("MOD", self.mod_fx.on, false),
            ("AMP", true, true),
            ("FX", self.fx.on, false),
            ("DLY", self.delay.on, false),
            ("REV", self.reverb.on, false),
            ("NS", self.ns.on, false),
        ];

        let n = blocks.len() as f32;
        let bw = ((rect.width() - 40.0) / n).min(120.0);
        let start_x = rect.left() + (rect.width() - bw * n) / 2.0;
        let cy = rect.center().y;

        for (i, (name, on, hero)) in blocks.iter().enumerate() {
            let x = start_x + i as f32 * bw;
            let block_h = if *hero { 36.0 } else { 30.0 };
            let bx = egui::Rect::from_center_size(
                egui::pos2(x + bw * 0.5, cy),
                Vec2::new(bw - 14.0, block_h),
            );
            let (fill, stroke_c, txt_c) = match (*on, *hero) {
                (true, true) => (col::CARD_HI, col::accent_hi(), col::TEXT),
                (true, false) => (col::CARD_HI, col::accent(), col::TEXT),
                (false, _) => (col::CARD, col::STROKE, col::TEXT_FAINT),
            };
            p.rect_filled(bx, Rounding::same(6.0), fill);
            p.rect_stroke(
                bx,
                Rounding::same(6.0),
                Stroke::new(if *hero { 1.5 } else { 1.0 }, stroke_c),
            );
            let font_size = if *hero { 13.0 } else { 11.5 };
            p.text(
                bx.center(),
                egui::Align2::CENTER_CENTER,
                *name,
                egui::FontId::proportional(font_size),
                txt_c,
            );

            if i < blocks.len() - 1 {
                let x0 = bx.right();
                let x1 = start_x + (i as f32 + 1.0) * bw + bw * 0.5 - (bw - 14.0) * 0.5;
                p.line_segment(
                    [egui::pos2(x0 + 2.0, cy), egui::pos2(x1 - 2.0, cy)],
                    Stroke::new(1.2, col::STROKE),
                );
            }
        }
    }
}
