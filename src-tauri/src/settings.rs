use tauri::{AppHandle};
use serde::{Serialize, Deserialize};
use crate::helper::{jhelper, fshelper::{self, PathBuf}};

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct OllamaSettings {
    pub ollama_default_path: String,
    pub ollama_custom_path: String,
    pub ollama_use_custom_path: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct MarkdownSettings {
    pub markdown_default_path: String,
    pub markdown_custom_path: String,
    pub markdown_use_custom_path: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default)]
pub struct AppSettings {
    pub urgent_threshold_days: i32,
    pub warning_threshold_days: i32,
    pub theme: String,
    pub ollama: OllamaSettings,
    pub markdown: MarkdownSettings
}

fn get_settings_path() -> PathBuf {
    let mut path = fshelper::get_feather_documents_dir();
    path.push("Settings");
    path.push("settings.json");
    path
}

fn resolve_default_ollama_dir() -> String {
    let mut path = std::env::var("LOCALAPPDATA")
        .or_else(|_| std::env::var("HOME"))
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."));

    #[cfg(target_os = "windows")]
    {
        path.push("Programs");
        path.push("Ollama");
        path.push("Ollama App.exe");
        return path.to_string_lossy().into_owned();
    }
    #[cfg(not(target_os = "windows"))]
    {
        String::from("ollama")
    }
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            urgent_threshold_days: 1,
            warning_threshold_days: 3,
            theme: String::from("default"),
            ollama: OllamaSettings {
                ollama_default_path: resolve_default_ollama_dir(),
                ollama_custom_path: String::new(),
                ollama_use_custom_path: false,
            },
            markdown: MarkdownSettings {
                markdown_default_path: fshelper::get_feather_documents_dir().to_string_lossy().into_owned(),
                markdown_custom_path: String::new(),
                markdown_use_custom_path: false,
            },
        }
    }
}

#[tauri::command]
pub async fn load_app_settings(_app: AppHandle) -> Result<AppSettings, String> {
    let path = get_settings_path();
    
    if !path.exists() {
        let default_settings = AppSettings::default();
        jhelper::save_json(&path, &default_settings)?;
        return Ok(default_settings);
    }

    let settings_option = jhelper::load_json::<AppSettings>(&path)?;
    
    let final_settings = settings_option.unwrap_or_default();

    jhelper::save_json(&path, &final_settings)?;

    Ok(final_settings)
}

#[tauri::command]
pub async fn save_app_settings(_app: AppHandle, settings: AppSettings) -> Result<(), String> {
    let path = get_settings_path();
    jhelper::save_json(&path, &settings)?;
    Ok(())
}