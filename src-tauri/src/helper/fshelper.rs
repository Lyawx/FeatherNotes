use std::fs;
pub use std::path::{Path, PathBuf};
use serde::Serialize;
use std::time::SystemTime;

#[derive(Serialize, Debug, Clone)]
pub struct FileNode {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    #[serde(rename = "createdAt")]
    pub created_at: u64,
    #[serde(rename = "updatedAt")]
    pub updated_at: u64,
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

pub fn exists<P: AsRef<Path>>(path: P) -> bool {
    path.as_ref().exists()
}

pub fn create_dir_all<P: AsRef<Path>>(path: P) -> Result<(), String> {
    fs::create_dir_all(path.as_ref()).map_err(|e| format!("Impossible de créer le dossier : {}", e))
}

#[tauri::command]
pub fn create_dir(dir_path: String) -> Result<(), String> {
    let path: PathBuf = PathBuf::from(dir_path);
    fs::create_dir_all(&path).map_err(|e| format!("Impossible de créer le dossier : {}", e))
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

pub fn read_dir_entries<P: AsRef<Path>>(path: P) -> Result<Vec<PathBuf>, String> {
    let entries = fs::read_dir(path.as_ref())
        .map_err(|e| format!("Impossible de lire le répertoire : {}", e))?;
    
    Ok(entries.flatten().map(|e| e.path()).collect())
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

    let mut created_at = 0;
    let mut updated_at = 0;

    if let Ok(metadata) = fs::metadata(path) {
        created_at = metadata.created()
            .or_else(|_| metadata.modified())
            .ok()
            .and_then(|t| t.duration_since(SystemTime::UNIX_EPOCH).ok())
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);

        updated_at = metadata.modified()
            .ok()
            .and_then(|t| t.duration_since(SystemTime::UNIX_EPOCH).ok())
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);
    }

    if is_dir {
        if let Ok(paths) = read_dir_entries(path) {
            for entry_path in paths {
                if let Some(entry_name) = entry_path.file_name().map(|n| n.to_string_lossy()) {
                    if entry_name.starts_with('.') || entry_name.starts_with('$') {
                        continue;
                    }
                }

                let is_valid_ext = entry_path.extension()
                    .map_or(false, |ext| ext == "md" || ext == "css");

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
        created_at,
        updated_at,
        children,
    })
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

#[tauri::command]
pub fn rename_item(old_path: String, new_name: String) -> Result<(), String> {
    let old_path_obj = Path::new(&old_path);
    let parent = old_path_obj.parent().ok_or("Chemin invalide")?;
    let new_path = parent.join(new_name);

    fs::rename(old_path, new_path).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_vault_tree(vault_path: String) -> Result<FileNode, String> {
    let root_path = PathBuf::from(&vault_path);
    if !root_path.exists() || !root_path.is_dir() {
        return Err("Le dossier spécifié n'existe pas ou n'est pas un répertoire.".to_string());
    }
    build_safe_tree(&root_path)
}