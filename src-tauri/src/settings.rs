use tauri::{AppHandle, Manager};
use std::path::PathBuf;
use serde::{Serialize, Deserialize};
use crate::helper::jhelper;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct OllamaSettings {
    pub ollama_default_path: String,
    pub ollama_custom_path: String,
    pub ollama_use_custom_path: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MarkdownSettings {
    pub markdown_default_path: String,
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
    
    // Si le fichier n'existe pas (premier lancement), on l'initialise avec les valeurs par défaut
    if !path.exists() {
        let default_settings = AppSettings::default();
        jhelper::save_json(&path, &default_settings)?;
        return Ok(default_settings);
    }

    let settings = jhelper::load_json::<AppSettings>(&path)?;
    Ok(settings.unwrap_or_default())
}

#[tauri::command]
pub async fn save_app_settings(app: AppHandle, settings: AppSettings) -> Result<(), String> {
    let path = get_settings_path(&app)?;
    jhelper::save_json(&path, &settings)?;
    Ok(())
}

// --- FONCTIONS UTILITAIRES POUR GÉNÉRER LES CHEMINS D'USINE ---

fn resolve_default_documents_dir() -> String {
    let base_path = std::env::var("USERPROFILE") 
        .or_else(|_| std::env::var("HOME"))    
        .map(PathBuf::from)
        .ok();

    if let Some(mut path) = base_path {
        path.push("Documents");
        path.push("FeatherNotes");
        return path.to_string_lossy().into_owned();
    }
    String::new()
}

fn resolve_default_ollama_dir() -> String {
    let base_path = std::env::var("LOCALAPPDATA")
        .or_else(|_| std::env::var("HOME"))
        .map(PathBuf::from)
        .ok();

    if let Some(mut path) = base_path {
        #[cfg(target_os = "windows")]
        {
            path.push("Programs");
            path.push("Ollama");
            path.push("Ollama App.exe");
            return path.to_string_lossy().into_owned();
        }
    }
    String::from("ollama") // Fallback Mac/Linux ou binaire global dans le PATH
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            ollama: OllamaSettings {
                ollama_default_path: resolve_default_ollama_dir(),
                ollama_custom_path: String::new(),
                ollama_use_custom_path: false,
            },
            markdown: MarkdownSettings {
                markdown_default_path: resolve_default_documents_dir(),
                markdown_custom_path: String::new(),
                markdown_use_custom_path: false,
            },
        }
    }
}