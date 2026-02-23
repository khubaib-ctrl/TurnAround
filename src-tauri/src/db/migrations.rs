use rusqlite::Connection;
use super::DbError;

const MIGRATIONS: &[&str] = &[
    // V1: Initial schema
    r#"
    CREATE TABLE IF NOT EXISTS projects (
        id TEXT PRIMARY KEY,
        name TEXT NOT NULL,
        root_path TEXT NOT NULL UNIQUE,
        created_at TEXT NOT NULL
    );

    CREATE TABLE IF NOT EXISTS branches (
        id TEXT PRIMARY KEY,
        project_id TEXT NOT NULL REFERENCES projects(id),
        name TEXT NOT NULL,
        head_commit_id TEXT,
        is_active INTEGER DEFAULT 0
    );

    CREATE TABLE IF NOT EXISTS commits (
        id TEXT PRIMARY KEY,
        project_id TEXT NOT NULL REFERENCES projects(id),
        branch_id TEXT NOT NULL REFERENCES branches(id),
        parent_id TEXT,
        message TEXT NOT NULL,
        is_milestone INTEGER DEFAULT 0,
        created_at TEXT NOT NULL
    );

    CREATE TABLE IF NOT EXISTS file_snapshots (
        id TEXT PRIMARY KEY,
        commit_id TEXT NOT NULL REFERENCES commits(id),
        file_path TEXT NOT NULL,
        content_hash TEXT NOT NULL,
        file_size INTEGER NOT NULL,
        file_type TEXT NOT NULL
    );

    CREATE TABLE IF NOT EXISTS objects (
        hash TEXT PRIMARY KEY,
        size INTEGER NOT NULL,
        stored_path TEXT NOT NULL,
        ref_count INTEGER DEFAULT 1
    );

    CREATE TABLE IF NOT EXISTS schema_version (
        version INTEGER PRIMARY KEY
    );
    INSERT OR IGNORE INTO schema_version (version) VALUES (1);
    "#,
];

pub fn run_all(conn: &Connection) -> Result<(), DbError> {
    let current_version = get_version(conn);

    for (i, migration) in MIGRATIONS.iter().enumerate() {
        let version = (i + 1) as i64;
        if version > current_version {
            conn.execute_batch(migration)
                .map_err(|e| DbError::Migration(format!("Migration v{version} failed: {e}")))?;
        }
    }

    Ok(())
}

fn get_version(conn: &Connection) -> i64 {
    conn.query_row(
        "SELECT COALESCE(MAX(version), 0) FROM schema_version",
        [],
        |row| row.get(0),
    )
    .unwrap_or(0)
}
