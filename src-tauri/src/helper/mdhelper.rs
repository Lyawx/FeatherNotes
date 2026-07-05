use std::fs;
use std::path::{Path, PathBuf};
use serde::Serialize;
use pulldown_cmark::{Parser, Options, html};

#[derive(Serialize, Debug, Clone)]
pub struct FileNode {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub children: Vec<FileNode>,
}

#[tauri::command]
pub fn convert_md_to_html(file_path: String) -> Result<String, String> {
    let path = Path::new(&file_path);
    if !path.exists() || !path.is_file() {
        return Err("Le fichier spécifié n'existe pas ou n'est pas valide.".to_string());
    }

    let markdown_input = fs::read_to_string(path)
        .map_err(|e| format!("Impossible de lire le fichier : {}", e))?;

    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TASKLISTS);

    let parser = Parser::new_ext(&markdown_input, options);

    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);

    Ok(html_output)
}

#[tauri::command]
pub fn get_vault_tree(vault_path: String) -> Result<FileNode, String> {
    let root_path = PathBuf::from(&vault_path);

    if !root_path.exists() || !root_path.is_dir() {
        return Err("Le dossier du Vault spécifié n'existe pas.".to_string());
    }

    build_tree(&root_path)
}

#[tauri::command]
pub fn create_vault_directory(dir_path: String) -> Result<(), String> {
    let path = Path::new(&dir_path);
    if !path.exists() {
        fs::create_dir_all(path).map_err(|e| format!("Échec de création du dossier : {}", e))?;
    }
    Ok(())
}

fn build_tree(path: &Path) -> Result<FileNode, String> {
    let name = path.file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();

    if name.starts_with('.') || name.starts_with('$') {
        return Err("Dossier système ou caché ignoré".to_string());
    }

    let is_dir = path.is_dir();
    let mut children = Vec::new();

    if is_dir {
        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries.flatten() {
                let entry_path = entry.path();
                let entry_name = entry_path.file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_default();

                if entry_name.starts_with('.') || entry_name.starts_with('$') {
                    continue;
                }

                if entry_path.is_dir() || entry_path.extension().map_or(false, |ext| ext == "md") {
                    if let Ok(child_node) = build_tree(&entry_path) {
                        children.push(child_node);
                    }
                }
            }
        }
        children.sort_by(|a, b| b.is_dir.cmp(&a.is_dir).then_with(|| a.name.cmp(&b.name)));
    }

    Ok(FileNode {
        name,
        path: path.to_string_lossy().into_owned(),
        is_dir,
        children,
    })
}