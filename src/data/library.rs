use eframe::egui::Color32;
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

const META_FILENAME: &str = ".tone_mk2_library.json";

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum TagColor {
    Crimson,
    Amber,
    Gold,
    Lime,
    Teal,
    Sky,
    Indigo,
    Violet,
    Pink,
    Slate,
}

impl TagColor {
    pub const ALL: [TagColor; 10] = [
        TagColor::Crimson,
        TagColor::Amber,
        TagColor::Gold,
        TagColor::Lime,
        TagColor::Teal,
        TagColor::Sky,
        TagColor::Indigo,
        TagColor::Violet,
        TagColor::Pink,
        TagColor::Slate,
    ];

    pub fn id(self) -> &'static str {
        match self {
            TagColor::Crimson => "crimson",
            TagColor::Amber => "amber",
            TagColor::Gold => "gold",
            TagColor::Lime => "lime",
            TagColor::Teal => "teal",
            TagColor::Sky => "sky",
            TagColor::Indigo => "indigo",
            TagColor::Violet => "violet",
            TagColor::Pink => "pink",
            TagColor::Slate => "slate",
        }
    }

    pub fn from_id(s: &str) -> TagColor {
        match s {
            "crimson" => TagColor::Crimson,
            "amber" => TagColor::Amber,
            "gold" => TagColor::Gold,
            "lime" => TagColor::Lime,
            "teal" => TagColor::Teal,
            "sky" => TagColor::Sky,
            "indigo" => TagColor::Indigo,
            "violet" => TagColor::Violet,
            "pink" => TagColor::Pink,
            _ => TagColor::Slate,
        }
    }

    pub fn pair(self) -> (Color32, Color32) {
        match self {
            TagColor::Crimson => (Color32::from_rgb(255, 120, 120), Color32::from_rgb(58, 26, 30)),
            TagColor::Amber => (Color32::from_rgb(255, 175, 85), Color32::from_rgb(58, 38, 20)),
            TagColor::Gold => (Color32::from_rgb(240, 215, 95), Color32::from_rgb(52, 46, 20)),
            TagColor::Lime => (Color32::from_rgb(150, 220, 120), Color32::from_rgb(28, 50, 28)),
            TagColor::Teal => (Color32::from_rgb(100, 220, 200), Color32::from_rgb(22, 50, 48)),
            TagColor::Sky => (Color32::from_rgb(120, 190, 245), Color32::from_rgb(22, 42, 62)),
            TagColor::Indigo => (Color32::from_rgb(155, 150, 245), Color32::from_rgb(30, 28, 60)),
            TagColor::Violet => (Color32::from_rgb(205, 135, 235), Color32::from_rgb(45, 26, 56)),
            TagColor::Pink => (Color32::from_rgb(245, 140, 200), Color32::from_rgb(58, 26, 46)),
            TagColor::Slate => (Color32::from_rgb(170, 180, 200), Color32::from_rgb(40, 44, 52)),
        }
    }
}

#[derive(Clone)]
pub struct Tag {
    pub name: String,
    pub color: TagColor,
}

pub struct LibraryMeta {
    pub tags: Vec<Tag>,
    pub assignments: HashMap<String, Vec<String>>,
}

impl Default for LibraryMeta {
    fn default() -> Self {
        Self {
            tags: vec![
                Tag { name: "Clean".into(),    color: TagColor::Lime },
                Tag { name: "Crunch".into(),   color: TagColor::Amber },
                Tag { name: "Lead".into(),     color: TagColor::Crimson },
                Tag { name: "Metal".into(),    color: TagColor::Violet },
                Tag { name: "Ambient".into(),  color: TagColor::Sky },
                Tag { name: "Funk".into(),     color: TagColor::Gold },
                Tag { name: "Acoustic".into(), color: TagColor::Teal },
                Tag { name: "Live".into(),     color: TagColor::Pink },
            ],
            assignments: HashMap::new(),
        }
    }
}

impl LibraryMeta {
    pub fn load(library: &Path) -> Self {
        let path = library.join(META_FILENAME);
        let Ok(raw) = fs::read_to_string(&path) else {
            return Self::default();
        };
        let Ok(v) = serde_json::from_str::<Value>(&raw) else {
            return Self::default();
        };
        let mut meta = Self {
            tags: vec![],
            assignments: HashMap::new(),
        };

        if let Some(arr) = v["tags"].as_array() {
            for t in arr {
                if let (Some(n), Some(c)) = (t["name"].as_str(), t["color"].as_str()) {
                    meta.tags.push(Tag {
                        name: n.into(),
                        color: TagColor::from_id(c),
                    });
                }
            }
        }
        if meta.tags.is_empty() {
            meta.tags = Self::default().tags;
        }

        if let Some(obj) = v["assignments"].as_object() {
            for (k, val) in obj {
                if let Some(arr) = val.as_array() {
                    let names: Vec<String> = arr
                        .iter()
                        .filter_map(|x| x.as_str().map(String::from))
                        .collect();
                    if !names.is_empty() {
                        meta.assignments.insert(k.clone(), names);
                    }
                }
            }
        }
        meta
    }

    pub fn save(&self, library: &Path) {
        let tags: Vec<_> = self
            .tags
            .iter()
            .map(|t| {
                serde_json::json!({
                    "name": t.name,
                    "color": t.color.id(),
                })
            })
            .collect();
        let assignments: serde_json::Map<String, Value> = self
            .assignments
            .iter()
            .map(|(k, v)| {
                (
                    k.clone(),
                    Value::Array(v.iter().map(|s| Value::String(s.clone())).collect()),
                )
            })
            .collect();
        let v = serde_json::json!({ "tags": tags, "assignments": assignments });
        let _ = fs::write(
            library.join(META_FILENAME),
            serde_json::to_string_pretty(&v).unwrap_or_default(),
        );
    }

    pub fn tag_by_name(&self, name: &str) -> Option<&Tag> {
        self.tags.iter().find(|t| t.name == name)
    }

    pub fn tags_for(&self, rel: &str) -> Vec<&Tag> {
        self.assignments
            .get(rel)
            .map(|names| names.iter().filter_map(|n| self.tag_by_name(n)).collect())
            .unwrap_or_default()
    }

    pub fn toggle_assignment(&mut self, rel: &str, tag_name: &str) {
        let list = self.assignments.entry(rel.to_string()).or_default();
        if let Some(pos) = list.iter().position(|n| n == tag_name) {
            list.remove(pos);
            if list.is_empty() {
                self.assignments.remove(rel);
            }
        } else {
            list.push(tag_name.to_string());
        }
    }
}

pub fn find_tsl_files(dir: &Path, patches: &mut Vec<PathBuf>) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                find_tsl_files(&path, patches);
            } else if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                if ext.eq_ignore_ascii_case("tsl") {
                    patches.push(path);
                }
            }
        }
    }
}
