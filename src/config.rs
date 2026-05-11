use std::fs;
use std::path::PathBuf;
use crate::app::Settings;
//const SETTINGS_PATH:&str = "src/pomo-tui-qsettings.json";

pub fn config_path() -> PathBuf {
    let mut path = std::env::current_exe().expect("failed to get exe path");

    path.pop();
    path.push("pomo-tui-settings.json");
    path
}
pub fn save_settings(settings: &Settings) {
    let path = config_path();

    if let Ok(json) = serde_json::to_string_pretty(settings) {
        let _ = fs::write(path, json);
    }
}

pub fn load_settings() -> Option<Settings> {
    let path = config_path();

    let data = fs::read_to_string(path).ok()?;
    serde_json::from_str(&data).ok()
}
