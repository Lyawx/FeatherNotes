use tauri::{AppHandle, Manager};
use std::path::PathBuf;
use serde::{Serialize, Deserialize};
use crate::jhelper;

#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct OllamaSettings {
    pub ollama_custom_path: String,
    pub ollama_use_custom_path: bool,
}

#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct MarkdownSettings {
    pub markdown_custom_path: String,
    pub markdown_use_custom_path: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AppSettings {
    pub ollama: OllamaSettings,
    pub markdown: MarkdownSettings
}

fn get_settings_path(app: &AppHandle) -> Result<PathBuf, String> {
    let mut path = app.path().config_dir().map_err(|e| e.to_string())?;
    path.push("FeatherNotes");
    path.push("settings.json");
    Ok(path)
}

#[tauri::command]
pub async fn load_app_settings(app: AppHandle) -> Result<AppSettings, String> {
    let path = get_settings_path(&app)?;
    let settings = jhelper::load_json::<AppSettings>(&path)?;
    Ok(settings.unwrap_or_default())
}

#[tauri::command]
pub async fn save_app_settings(app: AppHandle, settings: AppSettings) -> Result<(), String> {
    let path = get_settings_path(&app)?;
    jhelper::save_json(&path, &settings)?;
    Ok(())
}

fn resolve_default_documents_dir() -> String {
    let base_path = std::env::var("USERPROFILE") 
        .or_else(|_| std::env::var("HOME"))    
        .map(PathBuf::from)
        .ok();

    if let Some(mut path) = base_path {
        path.push("Documents");
        if path.exists() {
            return path.to_string_lossy().into_owned();
        }
    }
    String::new()
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            ollama: OllamaSettings {
                ollama_custom_path: String::new(),
                ollama_use_custom_path: false,
            },
            markdown: MarkdownSettings {
                // Utilisation de la helper func statique pour garder le bloc Default ultra-clean
                markdown_custom_path: resolve_default_documents_dir(),
                markdown_use_custom_path: false,
            },
        }
    }
}