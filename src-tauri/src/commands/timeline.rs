use std::path::Path;
use tauri::State;
use crate::AppState;
use crate::db::schema;
use crate::error::AppError;
use crate::timeline::{self, diff, parser};
use crate::vcs::object_store::ObjectStore;

#[tauri::command]
pub fn get_timeline_diff(
    state: State<AppState>,
    commit_a: String,
    commit_b: String,
) -> Result<diff::TimelineDiff, AppError> {
    let project_path = state.active_project_path.lock().clone()
        .ok_or(AppError::NoActiveProject)?;

    let db = state.db.lock();
    let editgit_dir = Path::new(&project_path).join(".editgit");
    let obj_store = ObjectStore::new(&editgit_dir);

    let snapshots_a = schema::get_snapshots_for_commit(&db.conn, &commit_a)?;
    let snapshots_b = schema::get_snapshots_for_commit(&db.conn, &commit_b)?;

    let timeline_file_a = snapshots_a.iter()
        .find(|s| is_timeline_type(&s.file_type))
        .ok_or_else(|| AppError::Timeline("No timeline file found in version A".into()))?;
    let timeline_file_b = snapshots_b.iter()
        .find(|s| is_timeline_type(&s.file_type))
        .ok_or_else(|| AppError::Timeline("No timeline file found in version B".into()))?;

    let path_a = obj_store.retrieve_path(&timeline_file_a.content_hash);
    let path_b = obj_store.retrieve_path(&timeline_file_b.content_hash);

    let tl_a = parser::parse_timeline_from_path(&path_a)
        .map_err(AppError::Timeline)?;
    let tl_b = parser::parse_timeline_from_path(&path_b)
        .map_err(AppError::Timeline)?;

    Ok(diff::diff_timelines(&tl_a, &tl_b))
}

#[tauri::command]
pub fn parse_timeline_file(path: String) -> Result<timeline::Timeline, AppError> {
    parser::parse_timeline_from_path(Path::new(&path))
        .map_err(AppError::Timeline)
}

fn is_timeline_type(file_type: &str) -> bool {
    matches!(file_type, "otio" | "fcpxml" | "xml" | "edl")
}
