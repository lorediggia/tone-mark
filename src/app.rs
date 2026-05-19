use eframe::egui::{self, Color32, RichText, Rounding, Stroke};
use midir::{MidiOutput, MidiOutputConnection};
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;

use crate::data::config::{load_config, save_config};
use crate::data::library::{find_tsl_files, LibraryMeta, TagColor};
use crate::data::state::*;
use crate::midi::katana::*;
use crate::midi::sysex::build_sysex;
use crate::ui::theme::{apply_theme, col, PaletteId};
use crate::ui::views::AppViews;

pub struct ToneApp {
    pub midi_conn: Option<MidiOutputConnection>,
    pub amp_name: Option<String>,
    pub available_ports: Vec<String>,
    pub status_msg: String,
    pub target: TargetChannel,
    pub current_tab: AppTab,
    pub palette: PaletteId,
    pub pending_theme_refresh: bool,

    pub library_path: Option<PathBuf>,
    pub patches: Vec<PathBuf>,
    pub selected_patch: Option<PathBuf>,
    pub library_search: String,
    pub selected_patch_info: Option<PatchInfo>,
    pub library_meta: LibraryMeta,
    pub active_tag_filters: Vec<String>,
    pub tag_popup_for: Option<String>,
    pub show_tag_manager: bool,
    pub new_tag_name: String,
    pub new_tag_color: TagColor,

    pub amp: AmpState,
    pub booster: FxBlock,
    pub mod_fx: FxBlock,
    pub fx: FxBlock,
    pub delay: DelayState,
    pub reverb: ReverbState,
    pub ns: NsState,
    pub snap_a: Option<Snapshot>,
    pub snap_b: Option<Snapshot>,
}

impl Default for ToneApp {
    fn default() -> Self {
        Self {
            midi_conn: None,
            amp_name: None,
            available_ports: vec![],
            status_msg: "Ready.".into(),
            target: TargetChannel::Panel,
            current_tab: AppTab::Library,
            palette: PaletteId::Amber,
            pending_theme_refresh: false,
            library_path: None,
            patches: vec![],
            selected_patch: None,
            library_search: String::new(),
            selected_patch_info: None,
            library_meta: LibraryMeta::default(),
            active_tag_filters: vec![],
            tag_popup_for: None,
            show_tag_manager: false,
            new_tag_name: String::new(),
            new_tag_color: TagColor::Sky,
            amp: AmpState::default(),
            booster: FxBlock::new(false, 0),
            mod_fx: FxBlock::new(false, 26),
            fx: FxBlock::new(false, 17),
            delay: DelayState::default(),
            reverb: ReverbState::default(),
            ns: NsState::default(),
            snap_a: None,
            snap_b: None,
        }
    }
}

impl ToneApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let cfg = load_config();
        cfg.palette.apply();
        apply_theme(&cc.egui_ctx);

        let mut app = Self::default();
        app.palette = cfg.palette;

        if let Some(p) = cfg.library_path {
            app.library_path = Some(p);
        } else {
            let default_dir = std::env::current_dir().unwrap_or_default().join("patches");
            if default_dir.exists() {
                app.library_path = Some(default_dir);
            }
        }

        app.refresh_library();
        app.refresh_midi_ports();
        app
    }

    pub fn save_config(&self) {
        save_config(&crate::data::config::AppConfig {
            library_path: self.library_path.clone(),
            palette: self.palette,
        });
    }

    pub fn refresh_midi_ports(&mut self) {
        self.available_ports.clear();
        if let Ok(m) = MidiOutput::new("tone-mk2-discover") {
            for p in m.ports().iter() {
                if let Ok(n) = m.port_name(p) {
                    self.available_ports.push(n);
                }
            }
        }
    }

    pub fn refresh_library(&mut self) {
        self.patches.clear();
        if let Some(path) = &self.library_path {
            find_tsl_files(path, &mut self.patches);
            self.patches.sort();
            self.library_meta = LibraryMeta::load(path);
        } else {
            self.library_meta = LibraryMeta::default();
        }
        if !self
            .patches
            .iter()
            .any(|p| Some(p) == self.selected_patch.as_ref())
        {
            self.selected_patch = None;
            self.selected_patch_info = None;
        }
        let known: Vec<String> = self
            .library_meta
            .tags
            .iter()
            .map(|t| t.name.clone())
            .collect();
        self.active_tag_filters.retain(|t| known.contains(t));
    }

    pub fn rel_path(&self, p: &Path) -> Option<String> {
        let base = self.library_path.as_ref()?;
        p.strip_prefix(base)
            .ok()
            .map(|r| r.to_string_lossy().replace('\\', "/"))
    }

    pub fn save_library_meta(&self) {
        if let Some(base) = &self.library_path {
            self.library_meta.save(base);
        }
    }

    pub fn connect_katana(&mut self) {
        let target: Option<String> = {
            let m = match MidiOutput::new("tone-mk2-discover") {
                Ok(m) => m,
                Err(_) => {
                    self.status_msg = "MIDI init error.".into();
                    return;
                }
            };
            m.ports().iter().find_map(|p| {
                m.port_name(p)
                    .ok()
                    .filter(|n| n.to_uppercase().contains("KATANA"))
            })
        };
        if let Some(name) = target {
            self.connect_to_port_name(&name);
        } else {
            self.refresh_midi_ports();
            self.status_msg = "Katana not found — choose a port from the menu.".into();
        }
    }

    pub fn connect_to_port_name(&mut self, name: &str) {
        let midi_out = match MidiOutput::new("Tone Mark II") {
            Ok(m) => m,
            Err(_) => {
                self.status_msg = "MIDI init error.".into();
                return;
            }
        };
        let mut target_port = None;
        for p in midi_out.ports().iter() {
            if let Ok(n) = midi_out.port_name(p) {
                if n == name {
                    target_port = Some(p.clone());
                    break;
                }
            }
        }
        let Some(p) = target_port else {
            self.status_msg = format!("Port '{}' not found.", name);
            return;
        };
        match midi_out.connect(&p, "tone-mk2-tx") {
            Ok(conn) => {
                self.midi_conn = Some(conn);
                self.amp_name = Some(name.to_string());
                self.status_msg = format!("Connected to {}.", name);
            }
            Err(_) => self.status_msg = "MIDI port busy.".into(),
        }
    }

    pub fn disconnect(&mut self) {
        if let Some(c) = self.midi_conn.take() {
            let _ = c.close();
        }
        self.amp_name = None;
        self.status_msg = "Disconnected.".into();
    }

    pub fn send_param(&mut self, address_offset: [u8; 3], value: u8) {
        if self.midi_conn.is_none() {
            return;
        }
        let msb = self.target.msb();
        let address = [msb, address_offset[0], address_offset[1], address_offset[2]];
        let sysex = build_sysex(address, &[value]);
        if let Some(conn) = &mut self.midi_conn {
            let _ = conn.send(&sysex);
        }
    }

    pub fn send_program_change(&mut self, pc: u8) {
        if let Some(conn) = &mut self.midi_conn {
            let _ = conn.send(&[0xC0, pc & 0x7F]);
        }
    }

    pub fn select_target(&mut self, t: TargetChannel) {
        self.target = t;
        if let Some(pc) = t.program_change() {
            self.send_program_change(pc);
        }
        self.status_msg = format!("Switched to {}.", t.short());
    }

    pub fn get_base_address(key: &str) -> Option<[u8; 4]> {
        match key {
            "UserPatch%Patch_0" => Some([0x60, 0x00, 0x00, 0x10]),
            "UserPatch%Eq(2)" => Some([0x60, 0x00, 0x01, 0x00]),
            "UserPatch%Fx(1)" => Some([0x60, 0x00, 0x01, 0x20]),
            "UserPatch%Fx(2)" => Some([0x60, 0x00, 0x03, 0x00]),
            "UserPatch%Delay(1)" => Some([0x60, 0x00, 0x05, 0x00]),
            "UserPatch%Delay(2)" => Some([0x60, 0x00, 0x05, 0x20]),
            "UserPatch%Patch_1" => Some([0x60, 0x00, 0x05, 0x40]),
            "UserPatch%Patch_2" => Some([0x60, 0x00, 0x06, 0x00]),
            "UserPatch%Status" => Some([0x60, 0x00, 0x06, 0x10]),
            _ => None,
        }
    }

    pub fn apply_patch(&mut self) {
        if self.midi_conn.is_none() || self.selected_patch.is_none() {
            self.status_msg = "Connect the amp first.".into();
            return;
        }
        let patch_path = self.selected_patch.as_ref().unwrap().clone();
        let raw = match fs::read_to_string(&patch_path) {
            Ok(s) => s,
            Err(_) => {
                self.status_msg = "Cannot read file.".into();
                return;
            }
        };
        let json: Value = match serde_json::from_str(&raw) {
            Ok(j) => j,
            Err(_) => {
                self.status_msg = "Invalid TSL JSON.".into();
                return;
            }
        };

        let msb = self.target.msb();
        let mut wrote = 0_usize;
        if let Some(param_set) = json["data"][0][0]["paramSet"].as_object() {
            for (key, hex_array) in param_set {
                if let Some(mut base_address) = Self::get_base_address(key) {
                    base_address[0] = msb;
                    let mut payload = Vec::new();
                    if let Some(arr) = hex_array.as_array() {
                        for hex_val in arr {
                            if let Some(s) = hex_val.as_str() {
                                if let Ok(byte) = u8::from_str_radix(s, 16) {
                                    payload.push(byte);
                                }
                            }
                        }
                    }
                    if !payload.is_empty() {
                        let sysex = build_sysex(base_address, &payload);
                        if let Some(conn) = &mut self.midi_conn {
                            let _ = conn.send(&sysex);
                            thread::sleep(Duration::from_millis(20));
                            wrote += 1;
                        }
                    }
                }
            }
        }
        self.status_msg = format!("Patch loaded ({} blocks to {}).", wrote, self.target.short());
    }

    pub fn save_state_as_tsl(&self, path: &Path) -> Result<(), String> {
        let dump = serde_json::json!({
            "exportedFromToneMarkII": true,
            "amp": {
                "type": AMP_TYPES.get(self.amp.type_idx).copied().unwrap_or(""),
                "gain": self.amp.gain, "bass": self.amp.bass, "mid": self.amp.mid,
                "treble": self.amp.treble, "presence": self.amp.pres,
                "volume": self.amp.vol, "bright": self.amp.bright,
                "sag": self.amp.sag, "resonance": self.amp.res,
            },
            "booster": { "on": self.booster.on, "type": BOOSTER_TYPES.get(self.booster.type_idx).copied().unwrap_or(""),
                         "drive": self.booster.p1, "bottom": self.booster.p2,
                         "tone": self.booster.p3, "level": self.booster.p4 },
            "mod":     { "on": self.mod_fx.on, "type": MOD_TYPES.get(self.mod_fx.type_idx).copied().unwrap_or("") },
            "fx":      { "on": self.fx.on, "type": FX_TYPES.get(self.fx.type_idx).copied().unwrap_or("") },
            "delay":   { "on": self.delay.on, "type": DELAY_TYPES.get(self.delay.type_idx).copied().unwrap_or(""),
                         "time": self.delay.time, "feedback": self.delay.feedback, "level": self.delay.level },
            "reverb":  { "on": self.reverb.on, "type": REVERB_TYPES.get(self.reverb.type_idx).copied().unwrap_or(""),
                         "time": self.reverb.time, "preDelay": self.reverb.pre,
                         "density": self.reverb.density, "level": self.reverb.level },
            "noiseSuppressor": { "on": self.ns.on, "threshold": self.ns.threshold, "release": self.ns.release },
        });
        fs::write(path, serde_json::to_string_pretty(&dump).unwrap()).map_err(|e| e.to_string())
    }

    pub fn make_snapshot(&self) -> Snapshot {
        Snapshot {
            amp: self.amp,
            booster: self.booster,
            mod_fx: self.mod_fx,
            fx: self.fx,
            delay: self.delay,
            reverb: self.reverb,
            ns: self.ns,
        }
    }

    pub fn load_snapshot(&mut self, s: Snapshot) {
        self.amp = s.amp;
        self.booster = s.booster;
        self.mod_fx = s.mod_fx;
        self.fx = s.fx;
        self.delay = s.delay;
        self.reverb = s.reverb;
        self.ns = s.ns;
        self.resend_all();
    }

    pub fn reset_defaults(&mut self) {
        self.amp = AmpState::default();
        self.booster = FxBlock::new(false, 0);
        self.mod_fx = FxBlock::new(false, 26);
        self.fx = FxBlock::new(false, 17);
        self.delay = DelayState::default();
        self.reverb = ReverbState::default();
        self.ns = NsState::default();
        if self.midi_conn.is_some() {
            self.resend_all();
        }
        self.status_msg = "Editor reset to defaults.".into();
    }

    pub fn parse_patch_info(path: &Path) -> Option<PatchInfo> {
        let raw = fs::read_to_string(path).ok()?;
        let json: Value = serde_json::from_str(&raw).ok()?;
        let entry = &json["data"][0][0];
        let name = entry["patchName"].as_str().unwrap_or("").trim().to_string();
        let memo = entry["memo"].as_str().unwrap_or("").trim().to_string();
        let block_count = entry["paramSet"].as_object().map(|m| m.len()).unwrap_or(0);
        let display_name = if name.is_empty() {
            path.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("?")
                .to_string()
        } else {
            name
        };
        Some(PatchInfo {
            name: display_name,
            memo,
            block_count,
        })
    }

    pub fn resend_all(&mut self) {
        self.send_param(
            ad::AMP_TYPE,
            AMP_TYPE_VALUES
                .get(self.amp.type_idx)
                .copied()
                .unwrap_or(self.amp.type_idx as u8),
        );
        self.send_param(ad::AMP_GAIN, self.amp.gain);
        self.send_param(ad::AMP_BASS, self.amp.bass);
        self.send_param(ad::AMP_MID, self.amp.mid);
        self.send_param(ad::AMP_TREB, self.amp.treble);
        self.send_param(ad::AMP_PRES, self.amp.pres);
        self.send_param(ad::AMP_VOL, self.amp.vol);
        self.send_param(ad::AMP_BRIGHT, self.amp.bright as u8);
        self.send_param(ad::AMP_SAG, self.amp.sag);
        self.send_param(ad::AMP_RES, self.amp.res);

        self.send_param(ad::BST_ON, self.booster.on as u8);
        self.send_param(
            ad::BST_TYPE,
            BOOSTER_TYPE_VALUES
                .get(self.booster.type_idx)
                .copied()
                .unwrap_or(self.booster.type_idx as u8),
        );
        self.send_param(ad::BST_DRIVE, self.booster.p1);
        self.send_param(ad::BST_BOTTOM, self.booster.p2);
        self.send_param(ad::BST_TONE, self.booster.p3);
        self.send_param(ad::BST_LEVEL, self.booster.p4);

        self.send_param(ad::MOD_ON, self.mod_fx.on as u8);
        self.send_param(
            ad::MOD_TYPE,
            MOD_TYPE_VALUES.get(self.mod_fx.type_idx).copied().unwrap_or(0),
        );

        self.send_param(ad::FX_ON, self.fx.on as u8);
        self.send_param(
            ad::FX_TYPE,
            FX_TYPE_VALUES.get(self.fx.type_idx).copied().unwrap_or(0),
        );

        self.send_param(ad::DLY1_ON, self.delay.on as u8);
        self.send_param(
            ad::DLY1_TYPE,
            DELAY_TYPE_VALUES
                .get(self.delay.type_idx)
                .copied()
                .unwrap_or(0),
        );
        self.send_param(ad::DLY1_TIME, self.delay.time);
        self.send_param(ad::DLY1_FB, self.delay.feedback);
        self.send_param(ad::DLY1_LVL, self.delay.level);

        self.send_param(ad::REV_ON, self.reverb.on as u8);
        self.send_param(
            ad::REV_TYPE,
            REVERB_TYPE_VALUES
                .get(self.reverb.type_idx)
                .copied()
                .unwrap_or(0),
        );
        self.send_param(ad::REV_TIME, self.reverb.time);
        self.send_param(ad::REV_PRE, self.reverb.pre);
        self.send_param(ad::REV_DENS, self.reverb.density);
        self.send_param(ad::REV_LVL, self.reverb.level);

        self.send_param(ad::NS_ON, self.ns.on as u8);
        self.send_param(ad::NS_THR, self.ns.threshold);
        self.send_param(ad::NS_REL, self.ns.release);
    }

    fn render_top_bar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label(
                RichText::new("Tone Mark II")
                    .size(26.0)
                    .strong()
                    .extra_letter_spacing(2.0)
                    .color(col::TEXT),
            );
            let tabs = [
                (AppTab::Library, "LIBRARY"),
                (AppTab::Editor, "EDITOR"),
                (AppTab::Settings, "SETTINGS"),
            ];
            for (t, n) in tabs {
                let sel = self.current_tab == t;
                let txt = RichText::new(n)
                    .size(11.5)
                    .extra_letter_spacing(2.0)
                    .color(if sel { col::accent() } else { col::TEXT_DIM });
                let btn = egui::Button::new(txt)
                    .fill(if sel { col::CARD } else { Color32::TRANSPARENT })
                    .stroke(Stroke::new(
                        1.0,
                        if sel { col::accent() } else { Color32::TRANSPARENT },
                    ))
                    .rounding(Rounding::same(5.0));
                if ui.add(btn).clicked() {
                    self.current_tab = t;
                }
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if let Some(name) = self.amp_name.clone() {
                    if ui
                        .add(
                            egui::Button::new(
                                RichText::new("DISCONNECT").size(10.0).color(col::TEXT_DIM),
                            )
                            .fill(col::PANEL)
                            .stroke(Stroke::new(1.0, col::STROKE))
                            .rounding(Rounding::same(5.0)),
                        )
                        .clicked()
                    {
                        self.disconnect();
                    }
                    ui.add_space(6.0);
                    ui.label(RichText::new(name).color(col::TEXT_DIM).size(10.5));
                    ui.add_space(4.0);
                    crate::ui::widgets::status_dot(ui, col::OK, true);
                    ui.label(
                        RichText::new("CONNECTED")
                            .color(col::OK)
                            .size(11.0)
                            .extra_letter_spacing(1.5),
                    );
                } else {
                    let connect_btn = egui::Button::new(
                        RichText::new("CONNECT").size(10.5).color(col::accent()),
                    )
                    .fill(col::CARD)
                    .stroke(Stroke::new(1.0, col::accent()))
                    .rounding(Rounding::same(5.0));
                    if ui.add(connect_btn).clicked() {
                        self.refresh_midi_ports();
                        self.connect_katana();
                    }
                    ui.add_space(4.0);

                    let label = if self.available_ports.is_empty() {
                        "No MIDI ports"
                    } else {
                        "MIDI port..."
                    };
                    let mut chosen: Option<String> = None;
                    egui::ComboBox::from_id_source("topbar_midi_port")
                        .selected_text(RichText::new(label).color(col::TEXT).size(11.0))
                        .width(220.0)
                        .show_ui(ui, |ui| {
                            if ui.button("Refresh ports").clicked() {
                                chosen = Some("__refresh__".into());
                            }
                            ui.separator();
                            if self.available_ports.is_empty() {
                                ui.label(
                                    RichText::new("(no ports detected)")
                                        .color(col::TEXT_FAINT)
                                        .size(10.5),
                                );
                            } else {
                                for port in &self.available_ports {
                                    if ui.selectable_label(false, port).clicked() {
                                        chosen = Some(port.clone());
                                    }
                                }
                            }
                        });
                    if let Some(c) = chosen {
                        if c == "__refresh__" {
                            self.refresh_midi_ports();
                        } else {
                            self.connect_to_port_name(&c);
                        }
                    }
                    ui.add_space(4.0);
                    crate::ui::widgets::status_dot(ui, col::TEXT_FAINT, false);
                    ui.label(
                        RichText::new("OFFLINE")
                            .color(col::TEXT_FAINT)
                            .size(11.0)
                            .extra_letter_spacing(1.5),
                    );
                }
            });
        });
    }

    fn render_bottom_bar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label(
                RichText::new(&self.status_msg)
                    .color(col::TEXT_DIM)
                    .size(11.0),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let opts = [
                    TargetChannel::Panel,
                    TargetChannel::Ch1,
                    TargetChannel::Ch2,
                    TargetChannel::Ch3,
                    TargetChannel::Ch4,
                ];
                for (i, t) in opts.iter().enumerate().rev() {
                    let sel = self.target == *t;
                    let txt = RichText::new(t.short())
                        .size(10.5)
                        .extra_letter_spacing(1.5)
                        .color(if sel { col::accent() } else { col::TEXT_DIM });
                    let btn = egui::Button::new(txt)
                        .fill(if sel { col::CARD } else { Color32::TRANSPARENT })
                        .stroke(Stroke::new(
                            1.0,
                            if sel { col::accent() } else { col::STROKE },
                        ))
                        .rounding(Rounding::same(4.0));
                    if ui.add(btn).clicked() {
                        self.select_target(*t);
                    }
                    if i > 0 {
                        ui.add_space(4.0);
                    }
                }
            });
        });
    }
}

impl eframe::App for ToneApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.pending_theme_refresh {
            apply_theme(ctx);
            self.pending_theme_refresh = false;
        }

        egui::TopBottomPanel::top("top_panel")
            .frame(egui::Frame::none().fill(col::PANEL).inner_margin(16.0))
            .show(ctx, |ui| self.render_top_bar(ui));

        egui::TopBottomPanel::bottom("footer_panel")
            .exact_height(34.0)
            .frame(
                egui::Frame::none()
                    .fill(col::PANEL)
                    .inner_margin(egui::Margin::symmetric(14.0, 8.0)),
            )
            .show(ctx, |ui| self.render_bottom_bar(ui));

        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(col::BG).inner_margin(28.0))
            .show(ctx, |ui| match self.current_tab {
                AppTab::Library => self.ui_library(ui),
                AppTab::Editor => self.ui_editor(ui),
                AppTab::Settings => self.ui_settings(ui),
            });
    }
}
