pub mod backup;
pub mod commands;
pub mod db;
pub mod error;
pub mod hasher;
pub mod registry;
pub mod timeline;
pub mod vcs;
pub mod watcher;

use std::sync::Arc;
use parking_lot::Mutex;

pub struct AppState {
    pub registry: Arc<Mutex<registry::Registry>>,
    pub db: Arc<Mutex<db::Database>>,
    pub watcher_handle: Arc<Mutex<Option<watcher::WatcherHandle>>>,
    pub active_project_path: Arc<Mutex<Option<String>>>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::init();

    let registry_dir = registry::default_registry_dir();
    let registry = registry::Registry::open(&registry_dir)
        .expect("Failed to open registry database");

    let app_state = AppState {
        registry: Arc::new(Mutex::new(registry)),
        db: Arc::new(Mutex::new(db::Database::new_in_memory().expect("Failed to init database"))),
        watcher_handle: Arc::new(Mutex::new(None)),
        active_project_path: Arc::new(Mutex::new(None)),
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            commands::registry::get_user_profile,
            commands::registry::save_user_profile,
            commands::registry::list_projects,
            commands::registry::rename_project,
            commands::registry::update_project_description,
            commands::registry::archive_project,
            commands::registry::unarchive_project,
            commands::registry::delete_project_from_registry,
            commands::registry::get_project_stats_live,
            commands::project::init_project,
            commands::project::open_project,
            commands::project::close_project,
            commands::project::get_project_info,
            commands::project::get_project_tree,
            commands::project::backup_project,
            commands::project::get_backup_registry,
            commands::project::recover_project_from_backup,
            commands::vcs::create_commit,
            commands::vcs::get_history,
            commands::vcs::get_commit_detail,
            commands::vcs::get_branches,
            commands::vcs::create_branch,
            commands::vcs::delete_commit,
            commands::vcs::delete_branch,
            commands::vcs::restore_commit,
            commands::vcs::export_commit,
            commands::vcs::switch_branch,
            commands::watcher::start_watching,
            commands::watcher::stop_watching,
            commands::timeline::get_timeline_diff,
            commands::timeline::parse_timeline_file,
        ])
        .run(tauri::generate_context!())
        .expect("error while running EditGit");
}
