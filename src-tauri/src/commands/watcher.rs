use std::path::PathBuf;
use tauri::{AppHandle, State};
use crate::AppState;
use crate::error::AppError;

#[tauri::command]
pub fn start_watching(
    app_handle: AppHandle,
    state: State<AppState>,
) -> Result<String, AppError> {
    let project_path = state.active_project_path.lock().clone()
        .ok_or(AppError::NoActiveProject)?;

    let handle = crate::watcher::start_watching(
        app_handle,
        PathBuf::from(&project_path),
    ).map_err(AppError::Watcher)?;

    let mut watcher_lock = state.watcher_handle.lock();
    *watcher_lock = Some(handle);

    Ok(format!("Watching: {project_path}"))
}

#[tauri::command]
pub fn stop_watching(state: State<AppState>) -> Result<(), AppError> {
    let mut watcher_lock = state.watcher_handle.lock();
    *watcher_lock = None;
    Ok(())
}
