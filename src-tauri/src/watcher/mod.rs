pub mod filter;
pub mod resolve;

use notify::{RecommendedWatcher, RecursiveMode, Watcher, Config};
use std::path::PathBuf;
use std::sync::mpsc;
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager};
use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct FileChangeEvent {
    pub path: String,
    pub kind: String,
}

pub struct WatcherHandle {
    _project_watcher: RecommendedWatcher,
    _resolve_watcher: Option<RecommendedWatcher>,
}

/// Start watching the project folder for tracked-file changes.
/// If `resolve_db` is provided, also poll that file and copy it into
/// `project_dir/project.drp` whenever it changes.
pub fn start_watching(
    app_handle: AppHandle,
    project_dir: PathBuf,
    resolve_db: Option<PathBuf>,
) -> Result<WatcherHandle, String> {
    let project_watcher = start_project_watcher(app_handle.clone(), &project_dir)?;
    let resolve_watcher = match resolve_db {
        Some(db_path) => {
            Some(start_resolve_watcher(app_handle, db_path, project_dir)?)
        }
        None => None,
    };

    Ok(WatcherHandle {
        _project_watcher: project_watcher,
        _resolve_watcher: resolve_watcher,
    })
}

fn start_project_watcher(app: AppHandle, watch_path: &PathBuf) -> Result<RecommendedWatcher, String> {
    let (tx, rx) = mpsc::channel();

    let mut watcher = RecommendedWatcher::new(
        move |res: Result<notify::Event, notify::Error>| {
            if let Ok(event) = res {
                let _ = tx.send(event);
            }
        },
        Config::default(),
    )
    .map_err(|e| format!("Failed to create project watcher: {e}"))?;

    watcher
        .watch(watch_path, RecursiveMode::Recursive)
        .map_err(|e| format!("Failed to watch project path: {e}"))?;

    let app_clone = app.clone();
    std::thread::spawn(move || {
        let mut last_emit = std::time::Instant::now();
        let debounce = Duration::from_millis(300);

        while let Ok(event) = rx.recv() {
            let dominated_paths: Vec<_> = event
                .paths
                .iter()
                .filter(|p| filter::is_tracked_file(p))
                .collect();

            if dominated_paths.is_empty() {
                continue;
            }

            let now = std::time::Instant::now();
            if now.duration_since(last_emit) < debounce {
                continue;
            }
            last_emit = now;

            for path in &dominated_paths {
                let kind = match event.kind {
                    notify::EventKind::Create(_) => "created",
                    notify::EventKind::Modify(_) => "modified",
                    notify::EventKind::Remove(_) => "removed",
                    _ => continue,
                };

                let payload = FileChangeEvent {
                    path: path.to_string_lossy().to_string(),
                    kind: kind.to_string(),
                };

                let _ = app_clone.emit("editgit://file-changed", &payload);
            }

        }
    });

    Ok(watcher)
}

fn start_resolve_watcher(
    app: AppHandle,
    resolve_db: PathBuf,
    project_dir: PathBuf,
) -> Result<RecommendedWatcher, String> {
    let (tx, rx) = mpsc::channel();

    let db_parent = resolve_db.parent()
        .ok_or_else(|| "Resolve DB has no parent directory".to_string())?
        .to_path_buf();

    let mut watcher = RecommendedWatcher::new(
        move |res: Result<notify::Event, notify::Error>| {
            if let Ok(event) = res {
                let _ = tx.send(event);
            }
        },
        Config::default(),
    )
    .map_err(|e| format!("Failed to create Resolve watcher: {e}"))?;

    watcher
        .watch(&db_parent, RecursiveMode::NonRecursive)
        .map_err(|e| format!("Failed to watch Resolve DB: {e}"))?;

    let app_clone = app.clone();
    let db_path = resolve_db.clone();
    let db_filename = resolve_db
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();
    std::thread::spawn(move || {
        let mut last_emit = std::time::Instant::now();
        let debounce = Duration::from_secs(1);

        while let Ok(event) = rx.recv() {
            let is_related = event.paths.iter().any(|p| {
                let name = p.file_name().unwrap_or_default().to_string_lossy();
                name == db_filename
                    || name.starts_with(&format!("{db_filename}-"))
                    || p == &db_path
            });
            if !is_related {
                continue;
            }

            let now = std::time::Instant::now();
            if now.duration_since(last_emit) < debounce {
                continue;
            }
            last_emit = now;

            let dest = project_dir.join("ResolveProject.db");
            if let Err(e) = std::fs::copy(&db_path, &dest) {
                log::warn!("Failed to copy Resolve DB to project folder: {e}");
                continue;
            }

            log::info!("Resolve DB change detected, copied to {}", dest.display());
            let payload = FileChangeEvent {
                path: dest.to_string_lossy().to_string(),
                kind: "modified".to_string(),
            };
            let _ = app_clone.emit("editgit://file-changed", &payload);
        }
    });

    Ok(watcher)
}
