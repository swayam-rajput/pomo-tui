use std::fs;

use crate::app::Settings; // or define it here
const SETTINGS_PATH:&str = "src/settings.json";

pub fn save_settings(settings: &Settings) {
    if let Ok(json) = serde_json::to_string_pretty(settings) {
        let _ = fs::write(SETTINGS_PATH, json);
    }
}

pub fn load_settings() -> Option<Settings> {
    let data = fs::read_to_string(SETTINGS_PATH).ok()?;
    serde_json::from_str(&data).ok()
}