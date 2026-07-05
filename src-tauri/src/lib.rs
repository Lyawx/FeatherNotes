pub mod jhelper;
pub mod settings;
pub mod ollama;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            settings::load_app_settings,
            settings::save_app_settings,
            ollama::status::check_ollama_status,
            ollama::status::start_ollama_process,
            ollama::status::stop_ollama_process,
            ollama::rq::fetch_models,
            ollama::rq::process_brain_dump
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}