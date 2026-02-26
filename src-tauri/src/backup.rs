use std::path::{Path, PathBuf};
use std::fs;
use serde::{Serialize, Deserialize};

fn backup_root() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".turnaround")
        .join("backups")
}

fn registry_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".turnaround")
        .join("registry.json")
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectEntry {
    pub name: String,
    pub original_path: String,
    pub backup_path: String,
    pub last_backup: String,
}

pub fn get_registry() -> Vec<ProjectEntry> {
    let path = registry_path();
    if !path.exists() {
        return Vec::new();
    }
    match fs::read_to_string(&path) {
        Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
        Err(_) => Vec::new(),
    }
}

fn save_registry(entries: &[ProjectEntry]) -> Result<(), String> {
    let path = registry_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("{e}"))?;
    }
    let json = serde_json::to_string_pretty(entries).map_err(|e| format!("{e}"))?;
    fs::write(&path, json).map_err(|e| format!("{e}"))?;
    Ok(())
}

pub fn backup_project(project_name: &str, project_path: &str) -> Result<ProjectEntry, String> {
    let turnaround_dir = Path::new(project_path).join(".turnaround");
    let db_path = turnaround_dir.join("editgit.db");

    if !db_path.exists() {
        return Err("No .turnaround database found to back up".to_string());
    }

    let safe_name = project_path
        .replace(['/', '\\', ':'], "_")
        .trim_matches('_')
        .to_string();

    let dest_dir = backup_root().join(&safe_name);
    fs::create_dir_all(&dest_dir).map_err(|e| format!("Failed to create backup dir: {e}"))?;

    let dest_db = dest_dir.join("editgit.db");
    fs::copy(&db_path, &dest_db).map_err(|e| format!("Failed to copy database: {e}"))?;

    let objects_dir = turnaround_dir.join("objects");
    let dest_objects = dest_dir.join("objects");
    if objects_dir.exists() {
        copy_dir_recursive(&objects_dir, &dest_objects)
            .map_err(|e| format!("Failed to backup objects: {e}"))?;
    }

    let now = chrono::Utc::now().to_rfc3339();
    let entry = ProjectEntry {
        name: project_name.to_string(),
        original_path: project_path.to_string(),
        backup_path: dest_dir.to_string_lossy().to_string(),
        last_backup: now,
    };

    let mut registry = get_registry();
    registry.retain(|e| e.original_path != project_path);
    registry.push(entry.clone());
    save_registry(&registry)?;

    Ok(entry)
}

pub fn recover_project(original_path: &str, target_path: &str) -> Result<(), String> {
    let registry = get_registry();
    let entry = registry.iter()
        .find(|e| e.original_path == original_path)
        .ok_or("No backup found for this project")?;

    let backup_dir = Path::new(&entry.backup_path);
    if !backup_dir.exists() {
        return Err("Backup directory no longer exists".to_string());
    }

    let target = Path::new(target_path);
    let turnaround_dir = target.join(".turnaround");
    fs::create_dir_all(&turnaround_dir).map_err(|e| format!("{e}"))?;

    let src_db = backup_dir.join("editgit.db");
    if src_db.exists() {
        fs::copy(&src_db, turnaround_dir.join("editgit.db")).map_err(|e| format!("{e}"))?;
    }

    let src_objects = backup_dir.join("objects");
    if src_objects.exists() {
        let dest_objects = turnaround_dir.join("objects");
        copy_dir_recursive(&src_objects, &dest_objects)
            .map_err(|e| format!("Failed to restore objects: {e}"))?;
    }

    Ok(())
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<(), std::io::Error> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}
