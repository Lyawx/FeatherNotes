use std::process::Command;
use serde::{Deserialize, Serialize};
use tauri::Manager;
use tauri::AppHandle;
use std::fs::{create_dir_all, File};
use std::io::{Read, Write};
use std::path::PathBuf;

// We define a simple structure to hold our Ollama path configurations
#[derive(Serialize, Deserialize, Clone, Debug)]
struct OllamaSettings {
    ollama_custom_path: String,
    ollama_use_custom_path: bool,
}

// We provide default values so the app doesn't crash if the settings file doesn't exist yet
impl Default for OllamaSettings {
    fn default() -> Self {
        Self {
            ollama_custom_path: String::new(),
            ollama_use_custom_path: false,
        }
    }
}

// Tauri command to load settings from disk
#[tauri::command]
async fn load_ollama_settings(app: AppHandle) -> Result<OllamaSettings, String> {
    let path = get_settings_path(&app)?;
    
    // If the file doesn't exist yet, return the default values
    if !path.exists() {
        return Ok(OllamaSettings::default());
    }

    // Open and read the file
    let mut file = File::open(path).map_err(|e| e.to_string())?;
    let mut contents = String::new();
    file.read_to_string(&mut contents).map_err(|e| e.to_string())?;

    // Parse JSON into our struct, fallback to default if corrupt
    let settings: OllamaSettings = serde_json::from_str(&contents).unwrap_or_else(|_| OllamaSettings::default());
    Ok(settings)
}

// Tauri command to save settings to disk
#[tauri::command]
async fn save_ollama_settings(app: AppHandle, settings: OllamaSettings) -> Result<(), String> {
    let path = get_settings_path(&app)?;
    
    // Convert struct to pretty formatted JSON string
    let json_data = serde_json::to_string_pretty(&settings).map_err(|e| e.to_string())?;
    
    // Write data to file
    let mut file = File::create(path).map_err(|e| e.to_string())?;
    file.write_all(json_data.as_bytes()).map_err(|e| e.to_string())?;
    Ok(())
}

// Helper function to get the absolute path to settings.json in a clean 'FeatherNotes' directory
fn get_settings_path(app: &AppHandle) -> Result<PathBuf, String> {
    // Get the base roaming/config profile directory path (e.g., AppData/Roaming)
    let mut path = app.path().config_dir().map_err(|e| e.to_string())?;
    
    // Explicitly push our preferred clean application folder name
    path.push("FeatherNotes");
    
    // Ensure the directory exists, create it if missing
    create_dir_all(&path).map_err(|e| e.to_string())?;
    
    // Append the file name
    path.push("settings.json");
    Ok(path)
}

#[derive(Serialize, Deserialize, Debug)]
struct OllamaModel {
    name: String,
}

#[derive(Deserialize, Debug)]
struct OllamaTagsResponse {
    models: Vec<OllamaModel>,
}

#[derive(Serialize)]
struct OllamaGenerateRequest {
    model: String,
    prompt: String,
    system: String,
    stream: bool,
}

#[derive(Deserialize)]
struct OllamaGenerateResponse {
    response: String,
}

#[tauri::command]
async fn check_ollama_status() -> Result<bool, String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(1))
        .build()
        .map_err(|e| e.to_string())?;

    let res = client.get("http://localhost:11434/").send().await;
    match res {
        Ok(response) => Ok(response.status().is_success()),
        Err(_) => Ok(false),
    }
}

// Starts Ollama, respecting the custom executable path if defined and active in settings
#[tauri::command]
async fn start_ollama(app: AppHandle) -> Result<(), String> {
    // 1. Load local settings to check for custom executable overrides
    let settings = load_ollama_settings(app.clone()).await.unwrap_or_default();

    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        use std::path::Path;
        const CREATE_NO_WINDOW: u32 = 0x08000000;

        // Determine executable based on user configuration toggle
        let exe = if settings.ollama_use_custom_path && !settings.ollama_custom_path.is_empty() {
            settings.ollama_custom_path
        } else {
            let local_app_data = std::env::var("LOCALAPPDATA").unwrap_or_default();
            let app_path = format!(r"{}\Programs\Ollama\Ollama App.exe", local_app_data);
            if Path::new(&app_path).exists() { app_path } else { "ollama".to_string() }
        };

        Command::new(exe)
            .creation_flags(CREATE_NO_WINDOW)
            .spawn()
            .map_err(|e| format!("Failed to start Ollama executable: {}", e))?;
    }

    #[cfg(target_os = "macos")]
    {
        if settings.ollama_use_custom_path && !settings.ollama_custom_path.is_empty() {
            Command::new(settings.ollama_custom_path)
                .spawn()
                .map_err(|e| e.to_string())?;
        } else {
            Command::new("open")
                .args(["-a", "Ollama"])
                .spawn()
                .map_err(|e| e.to_string())?;
        }
    }

    #[cfg(target_os = "linux")]
    {
        let exe = if settings.ollama_use_custom_path && !settings.ollama_custom_path.is_empty() {
            settings.ollama_custom_path
        } else {
            "ollama".to_string()
        };

        Command::new(exe)
            .arg("serve")
            .spawn()
            .map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[tauri::command]
async fn get_installed_models() -> Result<Vec<String>, String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(2))
        .build()
        .map_err(|e| e.to_string())?;

    let res = client.get("http://localhost:11434/api/tags").send().await;

    match res {
        Ok(response) => {
            if response.status().is_success() {
                let tags: OllamaTagsResponse = response.json().await.map_err(|e| e.to_string())?;
                let model_names = tags.models.into_iter().map(|m| m.name).collect();
                Ok(model_names)
            } else {
                Err("Ollama returned an error status".to_string())
            }
        }
        Err(_) => Err("Could not connect to Ollama to fetch models".to_string()),
    }
}

// Sends the text and returns the complete formatted string response once finished
#[tauri::command]
async fn process_brain_dump(selected_model: String, raw_dump: String) -> Result<String, String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(60))
        .build()
        .map_err(|e| e.to_string())?;

    let system_prompt = String::from(
        "You are FeatherNotes, a precise organization assistant. \
        Analyze the user's raw, unstructured brain dump and categorize the information into exactly 3 sections: \
        1. TASKS (Actionable items with estimated priority) \
        2. KNOWLEDGE (Facts, ideas, or notes for long-term storage) \
        3. MOOD ANALYSIS (A quick assessment of the user's current mental state or stress level based on the text) \
        Be concise, use clean formatting, and do not add any conversational filler introduction or conclusion."
    );

    let payload = OllamaGenerateRequest {
        model: selected_model,
        prompt: raw_dump,
        system: system_prompt,
        stream: false, // Disabling stream to wait for complete output
    };

    let res = client.post("http://localhost:11434/api/generate")
        .json(&payload)
        .send()
        .await
        .map_err(|e| format!("Failed to connect to Ollama: {}", e))?;

    if res.status().is_success() {
        let json_res: OllamaGenerateResponse = res.json().await
            .map_err(|e| format!("Failed to parse response: {}", e))?;
        Ok(json_res.response)
    } else {
        Err(format!("Ollama API error status: {}", res.status()))
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            check_ollama_status, 
            start_ollama,
            get_installed_models,
            process_brain_dump,
            load_ollama_settings,
            save_ollama_settings 
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}