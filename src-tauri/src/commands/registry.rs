use std::path::Path;
use tauri::State;
use crate::AppState;
use crate::error::AppError;
use crate::registry::{UserProfile, ProjectEntry, ProjectStats};
use chrono::Utc;
use uuid::Uuid;

// ── User Profile ──

#[tauri::command]
pub fn get_user_profile(state: State<AppState>) -> Result<Option<UserProfile>, AppError> {
    let reg = state.registry.lock();
    Ok(reg.get_profile()?)
}

#[tauri::command]
pub fn save_user_profile(
    state: State<AppState>,
    display_name: String,
    email: String,
) -> Result<UserProfile, AppError> {
    let reg = state.registry.lock();
    let now = Utc::now().to_rfc3339();

    let existing = reg.get_profile()?;

    let profile = UserProfile {
        id: existing.as_ref().map(|p| p.id.clone()).unwrap_or_else(|| Uuid::new_v4().to_string()),
        display_name,
        email,
        avatar_path: existing.and_then(|p| p.avatar_path),
        created_at: now.clone(),
        updated_at: now,
    };

    reg.save_profile(&profile)?;
    Ok(profile)
}

// ── Projects ──

#[tauri::command]
pub fn list_projects(state: State<AppState>) -> Result<Vec<ProjectEntry>, AppError> {
    let reg = state.registry.lock();
    Ok(reg.list_projects()?)
}

#[tauri::command]
pub fn rename_project(
    state: State<AppState>,
    project_id: String,
    new_name: String,
) -> Result<(), AppError> {
    let reg = state.registry.lock();
    Ok(reg.rename_project(&project_id, &new_name)?)
}

#[tauri::command]
pub fn update_project_description(
    state: State<AppState>,
    project_id: String,
    description: String,
) -> Result<(), AppError> {
    let reg = state.registry.lock();
    Ok(reg.update_description(&project_id, &description)?)
}

#[tauri::command]
pub fn archive_project(
    state: State<AppState>,
    project_id: String,
) -> Result<(), AppError> {
    let reg = state.registry.lock();
    Ok(reg.archive_project(&project_id)?)
}

#[tauri::command]
pub fn unarchive_project(
    state: State<AppState>,
    project_id: String,
) -> Result<(), AppError> {
    let reg = state.registry.lock();
    Ok(reg.unarchive_project(&project_id)?)
}

#[tauri::command]
pub fn delete_project_from_registry(
    state: State<AppState>,
    project_id: String,
    delete_data: bool,
) -> Result<(), AppError> {
    let reg = state.registry.lock();
    let entry = reg.delete_project(&project_id)?;

    if delete_data {
        let editgit_dir = Path::new(&entry.root_path).join(".editgit");
        if editgit_dir.exists() {
            if let Err(e) = std::fs::remove_dir_all(&editgit_dir) {
                log::warn!("Failed to remove .editgit directory during deletion: {e}");
            }
        }
    }

    Ok(())
}

#[tauri::command]
pub fn get_project_stats_live(
    state: State<AppState>,
    project_id: String,
) -> Result<ProjectStats, AppError> {
    let reg = state.registry.lock();
    let entry = reg.get_project(&project_id)?;

    let editgit_dir = Path::new(&entry.root_path).join(".editgit");
    let db_path = editgit_dir.join("editgit.db");

    if !db_path.exists() {
        return Ok(ProjectStats {
            commit_count: 0,
            branch_count: 0,
            disk_usage_bytes: 0,
        });
    }

    let conn = rusqlite::Connection::open(&db_path)?;

    let commit_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM commits", [], |row| row.get(0))?;

    let branch_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM branches", [], |row| row.get(0))?;

    let disk_usage_bytes = dir_size(&editgit_dir);

    let stats = ProjectStats {
        commit_count,
        branch_count,
        disk_usage_bytes,
    };

    if let Err(e) = reg.update_stats(&project_id, &stats) {
        log::warn!("Failed to persist project stats: {e}");
    }

    Ok(stats)
}

fn dir_size(path: &Path) -> i64 {
    let mut total: u64 = 0;
    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_dir() {
                total += dir_size(&p) as u64;
            } else if let Ok(meta) = p.metadata() {
                total += meta.len();
            }
        }
    }
    total as i64
}
