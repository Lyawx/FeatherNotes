pub mod helper;
pub mod ollama;
pub mod settings;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let _ = helper::fshelper::ensure_app_directories();
    let _ = helper::themehelper::inject_app_themes();

    tauri::Builder::default()
        .setup(|_app| {
            #[cfg(not(dev))]
            {
                use tauri::Manager;
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window
                        .eval("window.addEventListener('contextmenu', e => e.preventDefault());");
                }
            }
            Ok(())
        })
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_positioner::init())
        .invoke_handler(tauri::generate_handler![
            settings::load_app_settings,
            settings::save_app_settings,
            ollama::status::check_ollama_status,
            ollama::status::start_ollama_process,
            ollama::status::stop_ollama_process,
            ollama::rq::fetch_models,
            ollama::rq::process_brain_dump,
            helper::fshelper::ensure_app_directories,
            helper::fshelper::get_vault_tree,
            helper::fshelper::rename_item,
            helper::fshelper::create_dir,
            helper::themehelper::inject_app_themes,
            helper::themehelper::get_available_themes,
            helper::themehelper::load_theme_raw,
            helper::mdhelper::convert_md_to_html,
            helper::mdhelper::save_markdown_file,
            helper::mdhelper::read_raw_markdown,
            helper::mdhelper::create_vault_directory,
            helper::taskhelper::get_tasks_structure,
            helper::taskhelper::toggle_task_in_file
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
