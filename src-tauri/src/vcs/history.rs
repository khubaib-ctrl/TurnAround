use crate::db::schema::{self, Commit};
use rusqlite::Connection;

pub fn get_branch_history(conn: &Connection, branch_id: &str, limit: u32) -> Result<Vec<Commit>, super::VcsError> {
    Ok(schema::get_history(conn, branch_id, limit)?)
}

pub fn walk_history(conn: &Connection, start_commit_id: &str, max_depth: usize) -> Result<Vec<Commit>, super::VcsError> {
    let mut result = Vec::new();
    let mut current_id = Some(start_commit_id.to_string());

    while let Some(id) = current_id {
        if result.len() >= max_depth {
            break;
        }
        if let Some(commit) = schema::get_commit(conn, &id)? {
            current_id = commit.parent_id.clone();
            result.push(commit);
        } else {
            break;
        }
    }

    Ok(result)
}
