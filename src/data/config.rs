use crate::ui::theme::PaletteId;
use serde_json::Value;
use std::fs;
use std::path::PathBuf;

const CONFIG_FILENAME: &str = ".tone_mk2_config.json";

pub struct AppConfig {
    pub library_path: Option<PathBuf>,
    pub palette: PaletteId,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            library_path: None,
            palette: PaletteId::Amber,
        }
    }
}

pub fn load_config() -> AppConfig {
    let mut cfg = AppConfig::default();
    let Ok(raw) = fs::read_to_string(CONFIG_FILENAME) else {
        return cfg;
    };
    let Ok(json) = serde_json::from_str::<Value>(&raw) else {
        return cfg;
    };

    if let Some(p) = json["library_path"].as_str() {
        let pb = PathBuf::from(p);
        if pb.exists() {
            cfg.library_path = Some(pb);
        }
    }
    if let Some(p) = json["palette"].as_str() {
        cfg.palette = PaletteId::from_id(p);
    }
    cfg
}

pub fn save_config(cfg: &AppConfig) {
    let v = serde_json::json!({
        "library_path": cfg.library_path.as_ref().map(|p| p.to_string_lossy().to_string()),
        "palette": cfg.palette.id(),
    });
    let _ = fs::write(
        CONFIG_FILENAME,
        serde_json::to_string_pretty(&v).unwrap_or_default(),
    );
}
