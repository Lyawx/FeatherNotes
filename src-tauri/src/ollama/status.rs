use std::process::Command;
use tauri::{AppHandle};
use crate::settings::load_app_settings; //

#[tauri::command]
pub async fn check_ollama_status() -> Result<bool, String> {
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
pub async fn start_ollama_process(app: AppHandle) -> Result<(), String> {
    // 1. On charge la configuration GLOBALE de l'application
    let app_settings = load_app_settings(app.clone()).await.unwrap_or_default();
    // 2. On isole la partie Ollama
    let settings = app_settings.ollama;

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

// Forcefully terminates any running Ollama background processes using native OS tools
#[tauri::command]
pub async fn stop_ollama_process() -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        // We call taskkill to terminate both potential binary runners (/F forces it, /IM specifies image name)
        let _ = Command::new("taskkill")
            .args(["/F", "/IM", "ollama.exe"])
            .spawn();

        let _ = Command::new("taskkill")
            .args(["/F", "/IM", "Ollama App.exe"])
            .spawn();
    }

    #[cfg(target_os = "macos")]
    {
        let _ = Command::new("killall")
            .arg("Ollama")
            .spawn();
    }

    #[cfg(target_os = "linux")]
    {
        let _ = Command::new("pkill")
            .arg("ollama")
            .spawn();
    }

    // Small delay to let the OS release the process and the network port
    std::thread::sleep(std::time::Duration::from_millis(500));
    Ok(())
}