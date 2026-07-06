use std::fs;
use std::path::{Path, PathBuf};
use serde::Serialize;

#[derive(Serialize, Debug, Clone)]
pub struct FileNode {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub children: Vec<FileNode>,
}

pub fn get_feather_documents_dir() -> PathBuf {
    let mut path = std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."));
    path.push("Documents");
    path.push("FeatherNotes");
    path
}

pub fn ensure_parent_dir(path: &Path) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

pub fn read_file_to_string<P: AsRef<Path>>(path: P) -> Result<String, String> {
    fs::read_to_string(path).map_err(|e| format!("Impossible de lire le fichier : {}", e))
}

pub fn write_string_to_file<P: AsRef<Path>>(path: P, content: &str) -> Result<(), String> {
    let p = path.as_ref();
    ensure_parent_dir(p)?;
    fs::write(p, content).map_err(|e| format!("Échec de l'écriture : {}", e))
}

#[tauri::command]
pub fn ensure_app_directories() -> Result<(), String> {
    let base = get_feather_documents_dir();
    let sub_dirs = [
        "",
        "Logbook",
        "Tasks",
        "MoodTracker",
        "LifeWiki",
        "Themes",
        "Settings",
    ];

    for sub in sub_dirs.iter() {
        let full_path = if sub.is_empty() {
            base.clone()
        } else {
            base.join(sub)
        };
        
        if !full_path.exists() {
            fs::create_dir_all(&full_path).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

pub fn build_safe_tree(path: &Path) -> Result<FileNode, String> {
    let name = path.file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();

    if name.starts_with('.') || name.starts_with('$') {
        return Err("Fichier système ignoré".to_string());
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

                // AJOUT : On accepte l'extension .md ET l'extension .css désormais
                let is_valid_ext = entry_path.extension().map_or(false, |ext| ext == "md" || ext == "css");

                if entry_path.is_dir() || is_valid_ext {
                    if let Ok(child_node) = build_safe_tree(&entry_path) {
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