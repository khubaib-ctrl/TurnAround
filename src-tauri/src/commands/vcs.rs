use std::path::Path;
use tauri::State;
use crate::AppState;
use crate::db::schema::{self, Branch, Commit, FileSnapshot};
use crate::error::AppError;
use crate::vcs;
use crate::vcs::object_store::ObjectStore;
use crate::vcs::commit::{RestoreReport, ExportReport};
use serde::Serialize;

#[derive(Serialize)]
pub struct CommitDetail {
    pub commit: Commit,
    pub files: Vec<FileSnapshot>,
}

#[tauri::command]
pub fn create_commit(
    state: State<AppState>,
    message: String,
    is_milestone: bool,
) -> Result<Commit, AppError> {
    let project_path = state.active_project_path.lock().clone()
        .ok_or(AppError::NoActiveProject)?;

    let db = state.db.lock();
    let project = schema::get_project_by_path(&db.conn, &project_path)?
        .ok_or(AppError::ProjectNotFound)?;

    let turnaround_dir = Path::new(&project_path).join(".turnaround");
    let obj_store = ObjectStore::new(&turnaround_dir);

    let commit = vcs::commit::create_commit(
        &db.conn,
        &project.id,
        Path::new(&project_path),
        &message,
        is_milestone,
        &obj_store,
    )?;

    drop(db);
    if let Err(e) = crate::backup::backup_project(&project.name, &project_path) {
        log::warn!("Background backup failed: {e}");
    }

    Ok(commit)
}

#[tauri::command]
pub fn get_history(
    state: State<AppState>,
    branch_id: String,
    limit: u32,
) -> Result<Vec<Commit>, AppError> {
    let db = state.db.lock();
    Ok(vcs::history::get_branch_history(&db.conn, &branch_id, limit)?)
}

#[tauri::command]
pub fn get_commit_detail(
    state: State<AppState>,
    commit_id: String,
) -> Result<CommitDetail, AppError> {
    let db = state.db.lock();
    let (commit, files) = vcs::commit::get_detail(&db.conn, &commit_id)?;
    Ok(CommitDetail { commit, files })
}

#[tauri::command]
pub fn get_branches(
    state: State<AppState>,
) -> Result<Vec<Branch>, AppError> {
    let project_path = state.active_project_path.lock().clone()
        .ok_or(AppError::NoActiveProject)?;
    let db = state.db.lock();
    let project = schema::get_project_by_path(&db.conn, &project_path)?
        .ok_or(AppError::ProjectNotFound)?;
    Ok(vcs::branch::get_all(&db.conn, &project.id)?)
}

#[tauri::command]
pub fn create_branch(
    state: State<AppState>,
    name: String,
) -> Result<Branch, AppError> {
    let project_path = state.active_project_path.lock().clone()
        .ok_or(AppError::NoActiveProject)?;
    let db = state.db.lock();
    let project = schema::get_project_by_path(&db.conn, &project_path)?
        .ok_or(AppError::ProjectNotFound)?;
    Ok(vcs::branch::create_branch(&db.conn, &project.id, &name)?)
}

#[tauri::command]
pub fn delete_branch(
    state: State<AppState>,
    branch_id: String,
) -> Result<(), AppError> {
    let project_path = state.active_project_path.lock().clone()
        .ok_or(AppError::NoActiveProject)?;
    let db = state.db.lock();
    let project = schema::get_project_by_path(&db.conn, &project_path)?
        .ok_or(AppError::ProjectNotFound)?;
    Ok(vcs::branch::delete_branch(&db.conn, &project.id, &branch_id)?)
}

#[tauri::command]
pub fn delete_commit(
    state: State<AppState>,
    commit_id: String,
) -> Result<(), AppError> {
    let project_path = state.active_project_path.lock().clone()
        .ok_or(AppError::NoActiveProject)?;
    let db = state.db.lock();
    let project = schema::get_project_by_path(&db.conn, &project_path)?
        .ok_or(AppError::ProjectNotFound)?;
    let turnaround_dir = Path::new(&project_path).join(".turnaround");
    let obj_store = ObjectStore::new(&turnaround_dir);
    Ok(vcs::commit::delete_commit(&db.conn, &project.id, &commit_id, &obj_store)?)
}

#[tauri::command]
pub fn restore_commit(
    state: State<AppState>,
    commit_id: String,
) -> Result<RestoreReport, AppError> {
    let project_path = state.active_project_path.lock().clone()
        .ok_or(AppError::NoActiveProject)?;
    let db = state.db.lock();
    let turnaround_dir = Path::new(&project_path).join(".turnaround");
    let obj_store = ObjectStore::new(&turnaround_dir);
    let report = vcs::commit::restore_commit(&db.conn, &commit_id, Path::new(&project_path), &obj_store)?;

    let restored_resolve_db = Path::new(&project_path).join("ResolveProject.db");
    if restored_resolve_db.exists() {
        let resolve_db_path = state.resolve_db_path.lock().clone();
        if let Some(resolve_dest) = resolve_db_path {
            let dest_path = Path::new(&resolve_dest);
            if let Some(parent) = dest_path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            if let Err(e) = std::fs::copy(&restored_resolve_db, dest_path) {
                log::warn!("Failed to push restored DB back to Resolve: {e}");
            } else {
                log::info!("Restored Resolve DB to {resolve_dest}");
            }
        }
    }

    Ok(report)
}

#[tauri::command]
pub fn export_commit(
    state: State<AppState>,
    commit_id: String,
    dest_path: String,
) -> Result<ExportReport, AppError> {
    let project_path = state.active_project_path.lock().clone()
        .ok_or(AppError::NoActiveProject)?;
    let db = state.db.lock();
    let turnaround_dir = Path::new(&project_path).join(".turnaround");
    let obj_store = ObjectStore::new(&turnaround_dir);
    Ok(vcs::commit::export_commit(&db.conn, &commit_id, Path::new(&dest_path), &obj_store)?)
}

#[tauri::command]
pub fn switch_branch(
    state: State<AppState>,
    branch_id: String,
) -> Result<Branch, AppError> {
    let project_path = state.active_project_path.lock().clone()
        .ok_or(AppError::NoActiveProject)?;
    let db = state.db.lock();
    let project = schema::get_project_by_path(&db.conn, &project_path)?
        .ok_or(AppError::ProjectNotFound)?;
    Ok(vcs::branch::switch_branch(&db.conn, &project.id, &branch_id)?)
}

#[tauri::command]
pub fn get_changed_files(state: State<AppState>) -> Result<Vec<String>, AppError> {
    let project_path = state.active_project_path.lock().clone()
        .ok_or(AppError::NoActiveProject)?;
    let db = state.db.lock();
    let project = schema::get_project_by_path(&db.conn, &project_path)?
        .ok_or(AppError::ProjectNotFound)?;
    Ok(vcs::commit::get_changed_files(
        &db.conn,
        &project.id,
        Path::new(&project_path),
    )?)
}
