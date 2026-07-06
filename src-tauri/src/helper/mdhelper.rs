use std::path::{Path, PathBuf};
use pulldown_cmark::{Parser, Options, html};
use crate::helper::fshelper::{self, FileNode};

#[tauri::command]
pub fn save_markdown_file(file_path: String, content: String) -> Result<(), String> {
    fshelper::write_string_to_file(Path::new(&file_path), &content)
}

#[tauri::command]
pub fn read_raw_markdown(file_path: String) -> Result<String, String> {
    fshelper::read_file_to_string(Path::new(&file_path))
}

#[tauri::command]
pub fn get_vault_tree(vault_path: String) -> Result<FileNode, String> {
    let root_path = PathBuf::from(&vault_path);
    if !root_path.exists() || !root_path.is_dir() {
        return Err("Le dossier spécifié n'existe pas ou n'est pas un répertoire.".to_string());
    }
    fshelper::build_safe_tree(&root_path)
}

#[tauri::command]
pub fn convert_md_to_html(file_path: String) -> Result<String, String> {
    let markdown_input = fshelper::read_file_to_string(Path::new(&file_path))?;
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
pub fn create_vault_directory(dir_path: String) -> Result<(), String> {
    let path = Path::new(&dir_path);
    if !path.exists() {
        let mut base_path = fshelper::get_feather_documents_dir();
        base_path.push(path);
        fshelper::ensure_parent_dir(&base_path)?;
    }
    Ok(())
}