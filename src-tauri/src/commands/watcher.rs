use std::path::PathBuf;
use tauri::{AppHandle, Manager, State};
use crate::AppState;
use crate::error::AppError;
use crate::watcher::resolve;
use crate::db::schema;

#[tauri::command]
pub fn start_watching(
    app_handle: AppHandle,
    state: State<AppState>,
) -> Result<String, AppError> {
    let project_path = state.active_project_path.lock().clone()
        .ok_or(AppError::NoActiveProject)?;

    let watch_path = PathBuf::from(&project_path);
    let watch_dir = if watch_path.is_file() {
        watch_path.parent().map(PathBuf::from).unwrap_or(watch_path.clone())
    } else {
        watch_path.clone()
    };

    let linked_path = {
        let db = state.db.lock();
        schema::get_config(&db.conn, "resolve_db_path")
            .ok()
            .flatten()
            .map(PathBuf::from)
    };

    if let Some(ref path) = linked_path {
        let mut rdb = state.resolve_db_path.lock();
        *rdb = Some(path.to_string_lossy().to_string());
    }

    let resolve_db = linked_path.filter(|p| p.is_file());

    if let Some(ref db) = resolve_db {
        log::info!("Using linked Resolve DB: {}", db.display());
    } else {
        log::info!("No Resolve project linked (or DB not on disk) — watching project folder only");
    }

    let handle = crate::watcher::start_watching(
        app_handle,
        watch_dir,
        resolve_db,
    ).map_err(AppError::Watcher)?;

    let mut watcher_lock = state.watcher_handle.lock();
    *watcher_lock = Some(handle);

    Ok(format!("Watching: {project_path}"))
}

#[tauri::command]
pub fn stop_watching(state: State<AppState>) -> Result<(), AppError> {
    let mut watcher_lock = state.watcher_handle.lock();
    *watcher_lock = None;
    let mut rdb = state.resolve_db_path.lock();
    *rdb = None;
    Ok(())
}

#[tauri::command]
pub fn focus_window(app_handle: AppHandle) -> Result<(), AppError> {
    if let Some(win) = app_handle.get_webview_window("main") {
        let _ = win.set_focus();
    }
    Ok(())
}

#[tauri::command]
pub fn list_resolve_projects() -> Vec<resolve::ResolveProject> {
    resolve::list_resolve_projects()
}

#[tauri::command]
pub fn link_resolve_project(
    state: State<AppState>,
    db_path: String,
) -> Result<(), AppError> {
    if !std::path::Path::new(&db_path).is_file() {
        return Err(AppError::Watcher(format!("Resolve DB not found: {db_path}")));
    }
    let db = state.db.lock();
    schema::set_config(&db.conn, "resolve_db_path", &db_path)?;
    Ok(())
}

#[tauri::command]
pub fn unlink_resolve_project(state: State<AppState>) -> Result<(), AppError> {
    let db = state.db.lock();
    schema::delete_config(&db.conn, "resolve_db_path")?;
    Ok(())
}

#[tauri::command]
pub fn get_linked_resolve_project(state: State<AppState>) -> Result<Option<String>, AppError> {
    let db = state.db.lock();
    Ok(schema::get_config(&db.conn, "resolve_db_path")?)
}
