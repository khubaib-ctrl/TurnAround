use crate::db::schema::{self, Branch};
use rusqlite::Connection;
use uuid::Uuid;

pub fn create_branch(conn: &Connection, project_id: &str, name: &str) -> Result<Branch, super::VcsError> {
    let active = schema::get_active_branch(conn, project_id)?;
    let head = active.as_ref().and_then(|b| b.head_commit_id.clone());

    let branch = Branch {
        id: Uuid::new_v4().to_string(),
        project_id: project_id.to_string(),
        name: name.to_string(),
        head_commit_id: head,
        is_active: false,
    };
    schema::insert_branch(conn, &branch)?;
    Ok(branch)
}

pub fn switch_branch(conn: &Connection, project_id: &str, branch_id: &str) -> Result<Branch, super::VcsError> {
    let branches = schema::get_branches(conn, project_id)?;
    let target = branches
        .into_iter()
        .find(|b| b.id == branch_id)
        .ok_or_else(|| super::VcsError::BranchNotFound(branch_id.to_string()))?;

    schema::set_active_branch(conn, project_id, branch_id)?;

    Ok(Branch {
        is_active: true,
        ..target
    })
}

pub fn delete_branch(conn: &Connection, project_id: &str, branch_id: &str) -> Result<(), super::VcsError> {
    let branches = schema::get_branches(conn, project_id)?;
    let target = branches
        .iter()
        .find(|b| b.id == branch_id)
        .ok_or_else(|| super::VcsError::BranchNotFound(branch_id.to_string()))?;

    if target.is_active {
        return Err(super::VcsError::CannotDeleteActiveBranch);
    }

    if branches.len() <= 1 {
        return Err(super::VcsError::CannotDeleteLastBranch);
    }

    schema::delete_branch(conn, branch_id)?;
    Ok(())
}

pub fn get_all(conn: &Connection, project_id: &str) -> Result<Vec<Branch>, super::VcsError> {
    Ok(schema::get_branches(conn, project_id)?)
}

pub fn get_active(conn: &Connection, project_id: &str) -> Result<Option<Branch>, super::VcsError> {
    Ok(schema::get_active_branch(conn, project_id)?)
}
