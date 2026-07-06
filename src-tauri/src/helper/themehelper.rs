use crate::helper::fshelper;
use std::collections::HashMap;

fn get_builtin_themes() -> HashMap<&'static str, &'static str> {
    let mut m = HashMap::new();
    m.insert("default_dark", include_str!("../../../src/themes/default_dark.css"));
    m.insert("light_theme", include_str!("../../../src/themes/default_light.css"));
    m.insert("slate_nord", include_str!("../../../src/themes/slate_nord.css"));
    m
}

const USER_TEMPLATE_CSS: &str = include_str!("../../../src/themes/user_template.css");

#[tauri::command]
pub fn inject_app_themes() -> Result<(), String> {
    let mut base_path = fshelper::get_feather_documents_dir();
    base_path.push("Themes");

    if !base_path.exists() {
        std::fs::create_dir_all(&base_path).map_err(|e| e.to_string())?;
    }

    let mut template_file = base_path.clone();
    template_file.push("user_template.css.example");
    
    if !template_file.exists() {
        fshelper::write_string_to_file(&template_file, USER_TEMPLATE_CSS)?;
    }

    Ok(())
}

#[tauri::command]
pub fn get_available_themes() -> Result<Vec<String>, String> {
    let mut themes = vec![];

    for name in get_builtin_themes().keys() {
        themes.push(name.to_string());
    }

    let mut path = fshelper::get_feather_documents_dir();
    path.push("Themes");

    if path.exists() {
        if let Ok(root_node) = fshelper::build_safe_tree(&path) {
            for child in root_node.children {
                if !child.is_dir && child.name.ends_with(".css") && !child.name.ends_with(".example") {
                    let name_without_ext = child.name.strip_suffix(".css").unwrap_or(&child.name);
                    
                    if !themes.contains(&name_without_ext.to_string()) {
                        themes.push(name_without_ext.to_string());
                    }
                }
            }
        }
    }

    Ok(themes)
}

#[tauri::command]
pub fn load_theme_raw(theme_name: String) -> Result<String, String> {
    let builtin_themes = get_builtin_themes();
    if let Some(css_content) = builtin_themes.get(theme_name.as_str()) {
        return Ok(css_content.to_string());
    }

    let mut path = fshelper::get_feather_documents_dir();
    path.push("Themes");
    path.push(format!("{}.css", theme_name));

    if !path.exists() {
        return Err(format!(
            "Le thème '{}' n'existe ni en interne ni dans ton dossier utilisateur.",
            theme_name
        ));
    }

    fshelper::read_file_to_string(path)
}