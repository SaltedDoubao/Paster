use std::fs;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub hotkey: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            hotkey: "CmdOrCtrl+Alt+V".to_string(),
        }
    }
}

impl AppConfig {
    fn get_config_path() -> PathBuf {
        let mut path = dirs_next::config_dir().unwrap_or_else(|| PathBuf::from("."));
        path.push("paster");
        fs::create_dir_all(&path).ok();
        path.push("config.json");
        path
    }

    pub fn load() -> Self {
        let path = Self::get_config_path();
        if let Ok(content) = fs::read_to_string(&path) {
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            Self::default()
        }
    }

    pub fn save(&self) -> Result<(), String> {
        let path = Self::get_config_path();
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| format!("序列化配置失败: {}", e))?;
        fs::write(&path, content)
            .map_err(|e| format!("写入配置文件失败: {}", e))?;
        Ok(())
    }
}

// Tauri commands
#[tauri::command]
pub fn get_hotkey() -> String {
    AppConfig::load().hotkey
}

#[tauri::command]
pub fn set_hotkey(hotkey: String) -> Result<(), String> {
    let mut config = AppConfig::load();
    config.hotkey = hotkey;
    config.save()?;
    Ok(())
}

