use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use super::DbError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub root_path: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Branch {
    pub id: String,
    pub project_id: String,
    pub name: String,
    pub head_commit_id: Option<String>,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Commit {
    pub id: String,
    pub project_id: String,
    pub branch_id: String,
    pub parent_id: Option<String>,
    pub message: String,
    pub is_milestone: bool,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileSnapshot {
    pub id: String,
    pub commit_id: String,
    pub file_path: String,
    pub content_hash: String,
    pub file_size: i64,
    pub file_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredObject {
    pub hash: String,
    pub size: i64,
    pub stored_path: String,
    pub ref_count: i64,
}

pub fn insert_project(conn: &Connection, project: &Project) -> Result<(), DbError> {
    conn.execute(
        "INSERT INTO projects (id, name, root_path, created_at) VALUES (?1, ?2, ?3, ?4)",
        params![project.id, project.name, project.root_path, project.created_at],
    )?;
    Ok(())
}

pub fn get_project_by_path(conn: &Connection, path: &str) -> Result<Option<Project>, DbError> {
    let mut stmt = conn.prepare("SELECT id, name, root_path, created_at FROM projects WHERE root_path = ?1")?;
    let mut rows = stmt.query(params![path])?;
    if let Some(row) = rows.next()? {
        Ok(Some(Project {
            id: row.get(0)?,
            name: row.get(1)?,
            root_path: row.get(2)?,
            created_at: row.get(3)?,
        }))
    } else {
        Ok(None)
    }
}

pub fn insert_branch(conn: &Connection, branch: &Branch) -> Result<(), DbError> {
    conn.execute(
        "INSERT INTO branches (id, project_id, name, head_commit_id, is_active) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![branch.id, branch.project_id, branch.name, branch.head_commit_id, branch.is_active as i32],
    )?;
    Ok(())
}

pub fn get_branches(conn: &Connection, project_id: &str) -> Result<Vec<Branch>, DbError> {
    let mut stmt = conn.prepare(
        "SELECT id, project_id, name, head_commit_id, is_active FROM branches WHERE project_id = ?1 ORDER BY name"
    )?;
    let rows = stmt.query_map(params![project_id], |row| {
        Ok(Branch {
            id: row.get(0)?,
            project_id: row.get(1)?,
            name: row.get(2)?,
            head_commit_id: row.get(3)?,
            is_active: row.get::<_, i32>(4)? != 0,
        })
    })?;
    Ok(rows.filter_map(|r| r.ok()).collect())
}

pub fn get_active_branch(conn: &Connection, project_id: &str) -> Result<Option<Branch>, DbError> {
    let mut stmt = conn.prepare(
        "SELECT id, project_id, name, head_commit_id, is_active FROM branches WHERE project_id = ?1 AND is_active = 1"
    )?;
    let mut rows = stmt.query(params![project_id])?;
    if let Some(row) = rows.next()? {
        Ok(Some(Branch {
            id: row.get(0)?,
            project_id: row.get(1)?,
            name: row.get(2)?,
            head_commit_id: row.get(3)?,
            is_active: true,
        }))
    } else {
        Ok(None)
    }
}

pub fn delete_branch(conn: &Connection, branch_id: &str) -> Result<(), DbError> {
    conn.execute("DELETE FROM file_snapshots WHERE commit_id IN (SELECT id FROM commits WHERE branch_id = ?1)", params![branch_id])?;
    conn.execute("DELETE FROM commits WHERE branch_id = ?1", params![branch_id])?;
    conn.execute("DELETE FROM branches WHERE id = ?1", params![branch_id])?;
    Ok(())
}

pub fn set_active_branch(conn: &Connection, project_id: &str, branch_id: &str) -> Result<(), DbError> {
    conn.execute(
        "UPDATE branches SET is_active = 0 WHERE project_id = ?1",
        params![project_id],
    )?;
    conn.execute(
        "UPDATE branches SET is_active = 1 WHERE id = ?1",
        params![branch_id],
    )?;
    Ok(())
}

pub fn update_branch_head(conn: &Connection, branch_id: &str, commit_id: &str) -> Result<(), DbError> {
    conn.execute(
        "UPDATE branches SET head_commit_id = ?1 WHERE id = ?2",
        params![commit_id, branch_id],
    )?;
    Ok(())
}

pub fn insert_commit(conn: &Connection, commit: &Commit) -> Result<(), DbError> {
    conn.execute(
        "INSERT INTO commits (id, project_id, branch_id, parent_id, message, is_milestone, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            commit.id, commit.project_id, commit.branch_id,
            commit.parent_id, commit.message, commit.is_milestone as i32,
            commit.created_at
        ],
    )?;
    Ok(())
}

pub fn get_history(conn: &Connection, branch_id: &str, limit: u32) -> Result<Vec<Commit>, DbError> {
    let mut stmt = conn.prepare(
        "SELECT id, project_id, branch_id, parent_id, message, is_milestone, created_at FROM commits WHERE branch_id = ?1 ORDER BY created_at DESC LIMIT ?2"
    )?;
    let rows = stmt.query_map(params![branch_id, limit], |row| {
        Ok(Commit {
            id: row.get(0)?,
            project_id: row.get(1)?,
            branch_id: row.get(2)?,
            parent_id: row.get(3)?,
            message: row.get(4)?,
            is_milestone: row.get::<_, i32>(5)? != 0,
            created_at: row.get(6)?,
        })
    })?;
    Ok(rows.filter_map(|r| r.ok()).collect())
}

pub fn get_commit(conn: &Connection, commit_id: &str) -> Result<Option<Commit>, DbError> {
    let mut stmt = conn.prepare(
        "SELECT id, project_id, branch_id, parent_id, message, is_milestone, created_at FROM commits WHERE id = ?1"
    )?;
    let mut rows = stmt.query(params![commit_id])?;
    if let Some(row) = rows.next()? {
        Ok(Some(Commit {
            id: row.get(0)?,
            project_id: row.get(1)?,
            branch_id: row.get(2)?,
            parent_id: row.get(3)?,
            message: row.get(4)?,
            is_milestone: row.get::<_, i32>(5)? != 0,
            created_at: row.get(6)?,
        }))
    } else {
        Ok(None)
    }
}

pub fn delete_commit(conn: &Connection, commit_id: &str) -> Result<(), DbError> {
    conn.execute("DELETE FROM file_snapshots WHERE commit_id = ?1", params![commit_id])?;
    conn.execute("DELETE FROM commits WHERE id = ?1", params![commit_id])?;
    Ok(())
}

pub fn get_content_hashes_for_commit(conn: &Connection, commit_id: &str) -> Result<Vec<String>, DbError> {
    let mut stmt = conn.prepare("SELECT content_hash FROM file_snapshots WHERE commit_id = ?1")?;
    let rows = stmt.query_map(params![commit_id], |row| row.get(0))?;
    Ok(rows.filter_map(|r| r.ok()).collect())
}

pub fn insert_file_snapshot(conn: &Connection, snapshot: &FileSnapshot) -> Result<(), DbError> {
    conn.execute(
        "INSERT INTO file_snapshots (id, commit_id, file_path, content_hash, file_size, file_type) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![snapshot.id, snapshot.commit_id, snapshot.file_path, snapshot.content_hash, snapshot.file_size, snapshot.file_type],
    )?;
    Ok(())
}

pub fn get_snapshots_for_commit(conn: &Connection, commit_id: &str) -> Result<Vec<FileSnapshot>, DbError> {
    let mut stmt = conn.prepare(
        "SELECT id, commit_id, file_path, content_hash, file_size, file_type FROM file_snapshots WHERE commit_id = ?1"
    )?;
    let rows = stmt.query_map(params![commit_id], |row| {
        Ok(FileSnapshot {
            id: row.get(0)?,
            commit_id: row.get(1)?,
            file_path: row.get(2)?,
            content_hash: row.get(3)?,
            file_size: row.get(4)?,
            file_type: row.get(5)?,
        })
    })?;
    Ok(rows.filter_map(|r| r.ok()).collect())
}

pub fn insert_object(conn: &Connection, obj: &StoredObject) -> Result<(), DbError> {
    conn.execute(
        "INSERT INTO objects (hash, size, stored_path, ref_count) VALUES (?1, ?2, ?3, ?4)",
        params![obj.hash, obj.size, obj.stored_path, obj.ref_count],
    )?;
    Ok(())
}

pub fn get_object(conn: &Connection, hash: &str) -> Result<Option<StoredObject>, DbError> {
    let mut stmt = conn.prepare("SELECT hash, size, stored_path, ref_count FROM objects WHERE hash = ?1")?;
    let mut rows = stmt.query(params![hash])?;
    if let Some(row) = rows.next()? {
        Ok(Some(StoredObject {
            hash: row.get(0)?,
            size: row.get(1)?,
            stored_path: row.get(2)?,
            ref_count: row.get(3)?,
        }))
    } else {
        Ok(None)
    }
}

pub fn increment_object_ref(conn: &Connection, hash: &str) -> Result<(), DbError> {
    conn.execute(
        "UPDATE objects SET ref_count = ref_count + 1 WHERE hash = ?1",
        params![hash],
    )?;
    Ok(())
}

pub fn decrement_object_ref(conn: &Connection, hash: &str) -> Result<i64, DbError> {
    conn.execute(
        "UPDATE objects SET ref_count = ref_count - 1 WHERE hash = ?1",
        params![hash],
    )?;
    let count: i64 = conn.query_row(
        "SELECT ref_count FROM objects WHERE hash = ?1",
        params![hash],
        |row| row.get(0),
    )?;
    Ok(count)
}

pub fn delete_object(conn: &Connection, hash: &str) -> Result<(), DbError> {
    conn.execute("DELETE FROM objects WHERE hash = ?1", params![hash])?;
    Ok(())
}
