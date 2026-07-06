use crate::helper::fshelper;

fn get_themes_list() -> Vec<(&'static str, &'static str)> {
    vec![
        ("slate_nord.css", r#":root {
  --bg-00: #1e2530;
  --bg-01: #2b3545;
  --bg-02: #48556b;
  --text-00: #ffffff;
  --text-01: #e1e7f0;
  --text-02: #7fa3c4;
  --border-width: 1px;
  --bg-active: #3b485c;
  --color-blue: #60a5fa;
  --color-blue-hover: #93c5fd;
  --color-blue-surface: #20293a;
  --color-blue-dark: #1e3a8a;
  --color-green: #34d399;
  --color-green-hover: #6ee7b7;
  --color-green-surface: #182c25;
  --color-red: #f87171;
  --color-red-hover: #fca5a5;
  --color-red-surface: #2d1a1a;
}"#),
    ]
}

#[tauri::command]
pub fn inject_app_themes() -> Result<(), String> {
    let mut base_path = fshelper::get_feather_documents_dir();
    base_path.push("Themes");

    // Supprime d'abord les anciens fichiers CSS pour éviter les résidus des autres thèmes
    if base_path.exists() {
        if let Ok(entries) = std::fs::read_dir(&base_path) {
            for entry in entries.flatten() {
                let entry_path = entry.path();
                if entry_path.is_file() && entry_path.extension().map_or(false, |ext| ext == "css") {
                    let _ = std::fs::remove_file(entry_path);
                }
            }
        }
    }

    for (filename, css_content) in get_themes_list() {
        let mut theme_file = base_path.clone();
        theme_file.push(filename);

        if !theme_file.exists() {
            fshelper::write_string_to_file(&theme_file, css_content)?;
        }
    }
    Ok(())
}

#[tauri::command]
pub fn get_available_themes() -> Result<Vec<String>, String> {
    let mut path = fshelper::get_feather_documents_dir();
    path.push("Themes");

    let root_node = fshelper::build_safe_tree(&path)?;
    let mut themes = vec![String::from("default")];

    for child in root_node.children {
        if !child.is_dir && child.name.ends_with(".css") {
            let name_without_ext = child.name.strip_suffix(".css").unwrap_or(&child.name);
            themes.push(name_without_ext.to_string());
        }
    }

    Ok(themes)
}

#[tauri::command]
pub fn load_theme_raw(theme_name: String) -> Result<String, String> {
    let mut path = fshelper::get_feather_documents_dir();
    path.push("Themes");
    path.push(format!("{}.css", theme_name));

    if !path.exists() {
        return Err(format!("Le fichier de thème {}.css n'existe pas.", theme_name));
    }

    fshelper::read_file_to_string(path)
}