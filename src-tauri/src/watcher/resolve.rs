use std::path::PathBuf;
use serde::Serialize;

const RESOLVE_LIBRARY_REL: &str =
    "Library/Application Support/Blackmagic Design/DaVinci Resolve/Resolve Project Library/Resolve Projects";

#[derive(Debug, Clone, Serialize)]
pub struct ResolveProject {
    pub name: String,
    pub db_path: String,
}

/// Locate Resolve's Project Library root for the current user.
fn resolve_projects_root() -> Option<PathBuf> {
    dirs::home_dir().map(|h| h.join(RESOLVE_LIBRARY_REL))
}

/// Scan the Resolve library and return all discovered projects with their
/// display name (the folder name) and the full path to `Project.db`.
pub fn list_resolve_projects() -> Vec<ResolveProject> {
    let root = match resolve_projects_root() {
        Some(r) if r.is_dir() => r,
        _ => return Vec::new(),
    };

    let users_dir = root.join("Users");
    let mut projects = Vec::new();

    let user_entries = match std::fs::read_dir(&users_dir) {
        Ok(e) => e,
        Err(_) => return projects,
    };

    for user in user_entries.flatten() {
        let projects_dir = user.path().join("Projects");
        if !projects_dir.is_dir() {
            continue;
        }
        let proj_entries = match std::fs::read_dir(&projects_dir) {
            Ok(e) => e,
            Err(_) => continue,
        };
        for proj in proj_entries.flatten() {
            let db = proj.path().join("Project.db");
            if db.is_file() {
                let name = proj.file_name().to_string_lossy().to_string();
                projects.push(ResolveProject {
                    name,
                    db_path: db.to_string_lossy().to_string(),
                });
            }
        }
    }

    projects.sort_by(|a, b| a.name.cmp(&b.name));
    projects
}
