pub mod filter;

use notify::{RecommendedWatcher, RecursiveMode, Watcher, Config};
use std::path::PathBuf;
use std::sync::mpsc;
use std::time::Duration;
use tauri::{AppHandle, Emitter};
use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct FileChangeEvent {
    pub path: String,
    pub kind: String,
}

pub struct WatcherHandle {
    _watcher: RecommendedWatcher,
}

pub fn start_watching(
    app_handle: AppHandle,
    watch_path: PathBuf,
) -> Result<WatcherHandle, String> {
    let (tx, rx) = mpsc::channel();

    let mut watcher = RecommendedWatcher::new(
        move |res: Result<notify::Event, notify::Error>| {
            if let Ok(event) = res {
                let _ = tx.send(event);
            }
        },
        Config::default().with_poll_interval(Duration::from_millis(500)),
    )
    .map_err(|e| format!("Failed to create watcher: {e}"))?;

    watcher
        .watch(&watch_path, RecursiveMode::Recursive)
        .map_err(|e| format!("Failed to watch path: {e}"))?;

    let app = app_handle.clone();
    std::thread::spawn(move || {
        let mut last_emit = std::time::Instant::now();
        let debounce = Duration::from_millis(500);

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

            for path in dominated_paths {
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

                let _ = app.emit("editgit://file-changed", &payload);
            }
        }
    });

    Ok(WatcherHandle { _watcher: watcher })
}
