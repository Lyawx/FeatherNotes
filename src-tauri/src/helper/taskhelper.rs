use std::fs;
use std::path::PathBuf;
use serde::{Serialize, Deserialize};
use crate::helper::fshelper;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TaskItem {
    pub id: usize,
    pub text: String,
    pub completed: bool,
    pub due_date: Option<String>, // <-- Ajout de la date de la tâche
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Milestone {
    pub name: String,
    pub path: String,
    pub date: Option<String>,     // <-- Ajout de la date de la milestone
    pub tasks: Vec<TaskItem>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TaskProject {
    pub name: String,
    pub path: String,
    pub milestones: Vec<Milestone>,
}

#[tauri::command]
pub fn get_tasks_structure() -> Result<Vec<TaskProject>, String> {
    let mut root_path = fshelper::get_feather_documents_dir();
    root_path.push("Tasks");

    if !root_path.exists() {
        return Ok(vec![]);
    }

    let mut projects = Vec::new();

    if let Ok(entries) = fs::read_dir(root_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let project_name = path.file_name().unwrap_or_default().to_string_lossy().into_owned();
                let mut milestones = Vec::new();

                if let Ok(sub_entries) = fs::read_dir(&path) {
                    for sub_entry in sub_entries.flatten() {
                        let sub_path = sub_entry.path();
                        if sub_path.is_file() && sub_path.extension().map_or(false, |ext| ext == "md") {
                            let milestone_name = sub_path.file_stem().unwrap_or_default().to_string_lossy().into_owned();
                            
                            // Récupération combinée (Front matter + Tâches)
                            let (milestone_date, tasks) = parse_milestone_file(&sub_path)?;

                            milestones.push(Milestone {
                                name: milestone_name,
                                path: sub_path.to_string_lossy().into_owned(),
                                date: milestone_date,
                                tasks,
                            });
                        }
                    }
                }

                projects.push(TaskProject {
                    name: project_name,
                    path: path.to_string_lossy().into_owned(),
                    milestones,
                });
            }
        }
    }

    Ok(projects)
}

fn parse_milestone_file(path: &PathBuf) -> Result<(Option<String>, Vec<TaskItem>), String> {
    let content = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let mut tasks = Vec::new();
    let mut id_counter = 0;
    let mut milestone_date = None;
    
    let mut in_front_matter = false;
    let mut front_matter_count = 0;

    for line in content.lines() {
        let trimmed = line.trim();

        // Gestion du Front Matter (YAML) en haut de fichier
        if trimmed == "---" {
            front_matter_count += 1;
            if front_matter_count == 1 {
                in_front_matter = true;
                continue;
            } else if front_matter_count == 2 {
                in_front_matter = false;
                continue;
            }
        }

        if in_front_matter {
            if trimmed.starts_with("date:") {
                let date_val = trimmed["date:".len()..].trim().to_string();
                milestone_date = Some(date_val);
            }
            continue;
        }

        // Parsing des lignes de tâches
        if trimmed.starts_with("- [ ]") || trimmed.starts_with("- [x]") || trimmed.starts_with("- [X]") {
            let completed = !trimmed.starts_with("- [ ]");
            let mut text = trimmed[5..].trim().to_string();
            let mut due_date = None;

            // Détection du tag [due: YYYY-MM-DD]
            if let Some(start_idx) = text.find("[due:") {
                if let Some(end_idx) = text[start_idx..].find(']') {
                    let absolute_end = start_idx + end_idx;
                    let date_part = &text[start_idx + 5..absolute_end].trim();
                    due_date = Some(date_part.to_string());
                    
                    // On nettoie le texte pour enlever la date de l'affichage de l'intitulé
                    text = format!("{}{}", &text[..start_idx], &text[absolute_end + 1..]).trim().to_string();
                }
            }

            tasks.push(TaskItem {
                id: id_counter,
                text,
                completed,
                due_date,
            });
            id_counter += 1;
        }
    }

    Ok((milestone_date, tasks))
}

#[tauri::command]
pub fn toggle_task_in_file(file_path: String, task_index: usize, completed: bool) -> Result<(), String> {
    let path = PathBuf::from(&file_path);
    let content = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let mut lines: Vec<String> = content.lines().map(|l| l.to_string()).collect();
    
    let mut current_task_idx = 0;
    let mut in_front_matter = false;
    let mut front_matter_count = 0;

    for line in lines.iter_mut() {
        let trimmed = line.trim();

        if trimmed == "---" {
            front_matter_count += 1;
            if front_matter_count == 1 { in_front_matter = true; }
            else if front_matter_count == 2 { in_front_matter = false; }
            continue;
        }

        if in_front_matter { continue; }

        let trimmed_start = line.trim_start();
        if trimmed_start.starts_with("- [ ]") || trimmed_start.starts_with("- [x]") || trimmed_start.starts_with("- [X]") {
            if current_task_idx == task_index {
                let leading_spaces = line.len() - trimmed_start.len();
                let indent = &line[..leading_spaces];
                let text = &trimmed_start[5..];
                
                if completed {
                    *line = format!("{}- [x]{}", indent, text);
                } else {
                    *line = format!("{}- [ ]{}", indent, text);
                }
                break;
            }
            current_task_idx += 1;
        }
    }

    fs::write(&path, lines.join("\n")).map_err(|e| e.to_string())?;
    Ok(())
}