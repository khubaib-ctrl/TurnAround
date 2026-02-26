use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RegistryError {
    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Project not found: {0}")]
    ProjectNotFound(String),
    #[error("Project already registered at: {0}")]
    AlreadyRegistered(String),
    #[error("Profile not found")]
    ProfileNotFound,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    pub id: String,
    pub display_name: String,
    pub email: String,
    pub avatar_path: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectEntry {
    pub id: String,
    pub name: String,
    pub description: String,
    pub root_path: String,
    pub tags: String,
    pub is_archived: bool,
    pub last_opened_at: String,
    pub created_at: String,
    pub disk_usage_bytes: i64,
    pub commit_count: i64,
    pub branch_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectStats {
    pub commit_count: i64,
    pub branch_count: i64,
    pub disk_usage_bytes: i64,
}

pub struct Registry {
    pub conn: Connection,
}

impl Registry {
    pub fn open(app_data_dir: &Path) -> Result<Self, RegistryError> {
        std::fs::create_dir_all(app_data_dir)?;
        let db_path = app_data_dir.join("registry.db");
        let conn = Connection::open(&db_path)?;
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;
        let registry = Self { conn };
        registry.run_migrations()?;
        Ok(registry)
    }

    fn run_migrations(&self) -> Result<(), RegistryError> {
        self.conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS registry_version (
                version INTEGER PRIMARY KEY
            );
            INSERT OR IGNORE INTO registry_version (version) VALUES (0);

            CREATE TABLE IF NOT EXISTS user_profile (
                id TEXT PRIMARY KEY,
                display_name TEXT NOT NULL,
                email TEXT NOT NULL DEFAULT '',
                avatar_path TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS registered_projects (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                description TEXT NOT NULL DEFAULT '',
                root_path TEXT NOT NULL UNIQUE,
                tags TEXT NOT NULL DEFAULT '',
                is_archived INTEGER NOT NULL DEFAULT 0,
                last_opened_at TEXT NOT NULL,
                created_at TEXT NOT NULL,
                disk_usage_bytes INTEGER NOT NULL DEFAULT 0,
                commit_count INTEGER NOT NULL DEFAULT 0,
                branch_count INTEGER NOT NULL DEFAULT 1
            );
            "#,
        )?;
        Ok(())
    }

    // ── User Profile ──

    pub fn get_profile(&self) -> Result<Option<UserProfile>, RegistryError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, display_name, email, avatar_path, created_at, updated_at FROM user_profile LIMIT 1",
        )?;
        let mut rows = stmt.query([])?;
        if let Some(row) = rows.next()? {
            Ok(Some(UserProfile {
                id: row.get(0)?,
                display_name: row.get(1)?,
                email: row.get(2)?,
                avatar_path: row.get(3)?,
                created_at: row.get(4)?,
                updated_at: row.get(5)?,
            }))
        } else {
            Ok(None)
        }
    }

    pub fn save_profile(&self, profile: &UserProfile) -> Result<(), RegistryError> {
        self.conn.execute(
            r#"INSERT INTO user_profile (id, display_name, email, avatar_path, created_at, updated_at)
               VALUES (?1, ?2, ?3, ?4, ?5, ?6)
               ON CONFLICT(id) DO UPDATE SET
                 display_name = excluded.display_name,
                 email = excluded.email,
                 avatar_path = excluded.avatar_path,
                 updated_at = excluded.updated_at"#,
            params![
                profile.id,
                profile.display_name,
                profile.email,
                profile.avatar_path,
                profile.created_at,
                profile.updated_at,
            ],
        )?;
        Ok(())
    }

    // ── Projects ──

    pub fn list_projects(&self) -> Result<Vec<ProjectEntry>, RegistryError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, description, root_path, tags, is_archived, last_opened_at, created_at, disk_usage_bytes, commit_count, branch_count
             FROM registered_projects ORDER BY last_opened_at DESC",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(ProjectEntry {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                root_path: row.get(3)?,
                tags: row.get(4)?,
                is_archived: row.get::<_, i32>(5)? != 0,
                last_opened_at: row.get(6)?,
                created_at: row.get(7)?,
                disk_usage_bytes: row.get(8)?,
                commit_count: row.get(9)?,
                branch_count: row.get(10)?,
            })
        })?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    pub fn get_project(&self, project_id: &str) -> Result<ProjectEntry, RegistryError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, description, root_path, tags, is_archived, last_opened_at, created_at, disk_usage_bytes, commit_count, branch_count
             FROM registered_projects WHERE id = ?1",
        )?;
        stmt.query_row(params![project_id], |row| {
            Ok(ProjectEntry {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                root_path: row.get(3)?,
                tags: row.get(4)?,
                is_archived: row.get::<_, i32>(5)? != 0,
                last_opened_at: row.get(6)?,
                created_at: row.get(7)?,
                disk_usage_bytes: row.get(8)?,
                commit_count: row.get(9)?,
                branch_count: row.get(10)?,
            })
        })
        .map_err(|_| RegistryError::ProjectNotFound(project_id.to_string()))
    }

    pub fn get_project_by_path(&self, root_path: &str) -> Result<Option<ProjectEntry>, RegistryError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, description, root_path, tags, is_archived, last_opened_at, created_at, disk_usage_bytes, commit_count, branch_count
             FROM registered_projects WHERE root_path = ?1",
        )?;
        let mut rows = stmt.query(params![root_path])?;
        if let Some(row) = rows.next()? {
            Ok(Some(ProjectEntry {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                root_path: row.get(3)?,
                tags: row.get(4)?,
                is_archived: row.get::<_, i32>(5)? != 0,
                last_opened_at: row.get(6)?,
                created_at: row.get(7)?,
                disk_usage_bytes: row.get(8)?,
                commit_count: row.get(9)?,
                branch_count: row.get(10)?,
            }))
        } else {
            Ok(None)
        }
    }

    pub fn register_project(&self, entry: &ProjectEntry) -> Result<(), RegistryError> {
        if let Some(existing) = self.get_project_by_path(&entry.root_path)? {
            return Err(RegistryError::AlreadyRegistered(existing.root_path));
        }
        self.conn.execute(
            r#"INSERT INTO registered_projects
               (id, name, description, root_path, tags, is_archived, last_opened_at, created_at, disk_usage_bytes, commit_count, branch_count)
               VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)"#,
            params![
                entry.id, entry.name, entry.description, entry.root_path,
                entry.tags, entry.is_archived as i32, entry.last_opened_at,
                entry.created_at, entry.disk_usage_bytes, entry.commit_count, entry.branch_count,
            ],
        )?;
        Ok(())
    }

    pub fn rename_project(&self, project_id: &str, new_name: &str) -> Result<(), RegistryError> {
        let rows = self.conn.execute(
            "UPDATE registered_projects SET name = ?1 WHERE id = ?2",
            params![new_name, project_id],
        )?;
        if rows == 0 {
            return Err(RegistryError::ProjectNotFound(project_id.to_string()));
        }
        Ok(())
    }

    pub fn update_description(&self, project_id: &str, description: &str) -> Result<(), RegistryError> {
        self.conn.execute(
            "UPDATE registered_projects SET description = ?1 WHERE id = ?2",
            params![description, project_id],
        )?;
        Ok(())
    }

    pub fn update_tags(&self, project_id: &str, tags: &str) -> Result<(), RegistryError> {
        self.conn.execute(
            "UPDATE registered_projects SET tags = ?1 WHERE id = ?2",
            params![tags, project_id],
        )?;
        Ok(())
    }

    pub fn archive_project(&self, project_id: &str) -> Result<(), RegistryError> {
        let rows = self.conn.execute(
            "UPDATE registered_projects SET is_archived = 1 WHERE id = ?1",
            params![project_id],
        )?;
        if rows == 0 {
            return Err(RegistryError::ProjectNotFound(project_id.to_string()));
        }
        Ok(())
    }

    pub fn unarchive_project(&self, project_id: &str) -> Result<(), RegistryError> {
        let rows = self.conn.execute(
            "UPDATE registered_projects SET is_archived = 0 WHERE id = ?1",
            params![project_id],
        )?;
        if rows == 0 {
            return Err(RegistryError::ProjectNotFound(project_id.to_string()));
        }
        Ok(())
    }

    pub fn delete_project(&self, project_id: &str) -> Result<ProjectEntry, RegistryError> {
        let entry = self.get_project(project_id)?;
        self.conn.execute("DELETE FROM registered_projects WHERE id = ?1", params![project_id])?;
        Ok(entry)
    }

    pub fn touch_project(&self, project_id: &str, now: &str) -> Result<(), RegistryError> {
        self.conn.execute(
            "UPDATE registered_projects SET last_opened_at = ?1 WHERE id = ?2",
            params![now, project_id],
        )?;
        Ok(())
    }

    pub fn update_stats(&self, project_id: &str, stats: &ProjectStats) -> Result<(), RegistryError> {
        self.conn.execute(
            "UPDATE registered_projects SET commit_count = ?1, branch_count = ?2, disk_usage_bytes = ?3 WHERE id = ?4",
            params![stats.commit_count, stats.branch_count, stats.disk_usage_bytes, project_id],
        )?;
        Ok(())
    }
}

pub fn default_registry_dir() -> PathBuf {
    let base = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));
    base.join("com.turnaround.desktop")
}
