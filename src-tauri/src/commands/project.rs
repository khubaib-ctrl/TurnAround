use std::path::Path;
use tauri::State;
use crate::AppState;
use crate::db::{Database, schema};
use crate::error::AppError;
use crate::registry::ProjectEntry;
use crate::vcs::object_store::ObjectStore;
use uuid::Uuid;
use chrono::Utc;
use serde::Serialize;

#[derive(Serialize, Clone)]
pub struct FileNode {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub size: Option<u64>,
    pub children: Option<Vec<FileNode>>,
}

#[derive(Serialize)]
pub struct ProjectInfo {
    pub id: String,
    pub name: String,
    pub root_path: String,
    pub created_at: String,
    pub active_branch: Option<schema::Branch>,
}

#[tauri::command]
pub fn init_project(
    state: State<AppState>,
    path: String,
    name: String,
) -> Result<ProjectInfo, AppError> {
    let project_path = Path::new(&path);
    std::fs::create_dir_all(&project_path)?;

    let editgit_dir = project_path.join(".editgit");
    std::fs::create_dir_all(&editgit_dir)?;

    let db_path = editgit_dir.join("editgit.db");
    let db = Database::new(&db_path)?;

    let obj_store = ObjectStore::new(&editgit_dir);
    obj_store.init()?;

    if let Some(existing) = schema::get_project_by_path(&db.conn, &path)? {
        let active_branch = schema::get_active_branch(&db.conn, &existing.id)?;

        {
            let mut db_lock = state.db.lock();
            *db_lock = db;
        }
        {
            let mut path_lock = state.active_project_path.lock();
            *path_lock = Some(path);
        }

        {
            let reg = state.registry.lock();
            let now = Utc::now().to_rfc3339();
            if let Err(e) = reg.touch_project(&existing.id, &now) {
                log::warn!("Failed to update last-opened timestamp: {e}");
            }
        }

        return Ok(ProjectInfo {
            id: existing.id,
            name: existing.name,
            root_path: existing.root_path,
            created_at: existing.created_at,
            active_branch,
        });
    }

    let project_id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();

    let project = schema::Project {
        id: project_id.clone(),
        name: name.clone(),
        root_path: path.clone(),
        created_at: now.clone(),
    };
    schema::insert_project(&db.conn, &project)?;

    let main_branch = schema::Branch {
        id: Uuid::new_v4().to_string(),
        project_id: project_id.clone(),
        name: "main".to_string(),
        head_commit_id: None,
        is_active: true,
    };
    schema::insert_branch(&db.conn, &main_branch)?;

    {
        let mut db_lock = state.db.lock();
        *db_lock = db;
    }
    {
        let mut path_lock = state.active_project_path.lock();
        *path_lock = Some(path.clone());
    }

    {
        let reg = state.registry.lock();
        let project_entry = ProjectEntry {
            id: project_id.clone(),
            name: name.clone(),
            description: String::new(),
            root_path: path.clone(),
            tags: String::new(),
            is_archived: false,
            last_opened_at: now.clone(),
            created_at: now.clone(),
            disk_usage_bytes: 0,
            commit_count: 0,
            branch_count: 1,
        };
        if let Err(e) = reg.register_project(&project_entry) {
            log::warn!("Failed to register project in registry: {e}");
        }
    }

    Ok(ProjectInfo {
        id: project_id,
        name,
        root_path: path,
        created_at: now,
        active_branch: Some(main_branch),
    })
}

#[tauri::command]
pub fn open_project(
    state: State<AppState>,
    path: String,
) -> Result<ProjectInfo, AppError> {
    let project_path = Path::new(&path);
    let editgit_dir = project_path.join(".editgit");
    let db_path = editgit_dir.join("editgit.db");

    if !db_path.exists() {
        return Err(AppError::NotEditGitProject);
    }

    let db = Database::new(&db_path)?;

    let project = schema::get_project_by_path(&db.conn, &path)?
        .ok_or(AppError::ProjectNotFound)?;

    let active_branch = schema::get_active_branch(&db.conn, &project.id)?;

    {
        let mut db_lock = state.db.lock();
        *db_lock = db;
    }
    {
        let mut path_lock = state.active_project_path.lock();
        *path_lock = Some(path);
    }

    {
        let reg = state.registry.lock();
        let now = Utc::now().to_rfc3339();
        if reg.get_project_by_path(&project.root_path).ok().flatten().is_none() {
            let project_entry = ProjectEntry {
                id: project.id.clone(),
                name: project.name.clone(),
                description: String::new(),
                root_path: project.root_path.clone(),
                tags: String::new(),
                is_archived: false,
                last_opened_at: now,
                created_at: project.created_at.clone(),
                disk_usage_bytes: 0,
                commit_count: 0,
                branch_count: 1,
            };
            if let Err(e) = reg.register_project(&project_entry) {
                log::warn!("Failed to register project in registry: {e}");
            }
        } else if let Err(e) = reg.touch_project(&project.id, &now) {
            log::warn!("Failed to update last-opened timestamp: {e}");
        }
    }

    Ok(ProjectInfo {
        id: project.id,
        name: project.name,
        root_path: project.root_path,
        created_at: project.created_at,
        active_branch,
    })
}

#[tauri::command]
pub fn close_project(state: State<AppState>) -> Result<(), AppError> {
    {
        let mut watcher = state.watcher_handle.lock();
        *watcher = None;
    }
    {
        let mut path_lock = state.active_project_path.lock();
        *path_lock = None;
    }
    Ok(())
}

#[tauri::command]
pub fn get_project_info(state: State<AppState>) -> Result<Option<ProjectInfo>, AppError> {
    let path = state.active_project_path.lock().clone();
    let Some(project_path) = path else {
        return Ok(None);
    };

    let db = state.db.lock();
    let project = schema::get_project_by_path(&db.conn, &project_path)?;

    match project {
        Some(p) => {
            let active_branch = schema::get_active_branch(&db.conn, &p.id)?;
            Ok(Some(ProjectInfo {
                id: p.id,
                name: p.name,
                root_path: p.root_path,
                created_at: p.created_at,
                active_branch,
            }))
        }
        None => Ok(None),
    }
}

#[tauri::command]
pub fn get_project_tree(state: State<AppState>) -> Result<FileNode, AppError> {
    let project_path = state.active_project_path.lock().clone()
        .ok_or(AppError::NoActiveProject)?;
    let root = Path::new(&project_path);
    let name = root.file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "Project".to_string());

    Ok(build_tree(root, &name, "")?)
}

#[tauri::command]
pub fn backup_project(state: State<AppState>) -> Result<crate::backup::ProjectEntry, AppError> {
    let project_path = state.active_project_path.lock().clone()
        .ok_or(AppError::NoActiveProject)?;
    let db = state.db.lock();
    let project = schema::get_project_by_path(&db.conn, &project_path)?
        .ok_or(AppError::ProjectNotFound)?;
    crate::backup::backup_project(&project.name, &project_path)
        .map_err(AppError::Backup)
}

#[tauri::command]
pub fn get_backup_registry() -> Result<Vec<crate::backup::ProjectEntry>, AppError> {
    Ok(crate::backup::get_registry())
}

#[tauri::command]
pub fn recover_project_from_backup(
    state: State<AppState>,
    original_path: String,
    target_path: String,
) -> Result<ProjectInfo, AppError> {
    crate::backup::recover_project(&original_path, &target_path)
        .map_err(AppError::Backup)?;

    let editgit_dir = Path::new(&target_path).join(".editgit");
    let db_path = editgit_dir.join("editgit.db");
    let db = Database::new(&db_path)?;

    let project = {
        let mut stmt = db.conn.prepare(
            "SELECT id, name, root_path, created_at FROM projects LIMIT 1",
        )?;
        stmt.query_row([], |row| {
            Ok(schema::Project {
                id: row.get(0)?,
                name: row.get(1)?,
                root_path: row.get(2)?,
                created_at: row.get(3)?,
            })
        }).map_err(|_| AppError::Backup(
            "No project found in recovered database".into(),
        ))?
    };

    if project.root_path != target_path {
        db.conn.execute(
            "UPDATE projects SET root_path = ?1 WHERE id = ?2",
            rusqlite::params![target_path, project.id],
        )?;
    }

    let active_branch = schema::get_active_branch(&db.conn, &project.id)?;

    {
        let mut db_lock = state.db.lock();
        *db_lock = db;
    }
    {
        let mut path_lock = state.active_project_path.lock();
        *path_lock = Some(target_path.clone());
    }

    Ok(ProjectInfo {
        id: project.id,
        name: project.name,
        root_path: target_path,
        created_at: project.created_at,
        active_branch,
    })
}

fn build_tree(path: &Path, name: &str, rel_path: &str) -> Result<FileNode, std::io::Error> {
    if path.is_dir() {
        let mut children = Vec::new();

        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let entry_path = entry.path();
            let entry_name = entry.file_name().to_string_lossy().to_string();

            if entry_name.starts_with('.') {
                continue;
            }

            if entry_name == "node_modules" || entry_name == "target" || entry_name == "__pycache__" {
                continue;
            }

            let child_rel = if rel_path.is_empty() {
                entry_name.clone()
            } else {
                format!("{}/{}", rel_path, entry_name)
            };

            children.push(build_tree(&entry_path, &entry_name, &child_rel)?);
        }

        children.sort_by(|a, b| {
            match (a.is_dir, b.is_dir) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
            }
        });

        Ok(FileNode {
            name: name.to_string(),
            path: rel_path.to_string(),
            is_dir: true,
            size: None,
            children: Some(children),
        })
    } else {
        let size = std::fs::metadata(path).map(|m| m.len()).ok();
        Ok(FileNode {
            name: name.to_string(),
            path: rel_path.to_string(),
            is_dir: false,
            size,
            children: None,
        })
    }
}
