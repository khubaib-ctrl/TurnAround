pub mod object_store;
pub mod commit;
pub mod branch;
pub mod history;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum VcsError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Database error: {0}")]
    Db(#[from] crate::db::DbError),
    #[error("No active project")]
    NoActiveProject,
    #[error("No active branch")]
    NoActiveBranch,
    #[error("Branch not found: {0}")]
    BranchNotFound(String),
    #[error("Cannot delete the active branch — switch to another branch first")]
    CannotDeleteActiveBranch,
    #[error("Cannot delete the last remaining branch")]
    CannotDeleteLastBranch,
    #[error("Commit not found: {0}")]
    CommitNotFound(String),
    #[error("Can only delete the latest version on the current branch")]
    CannotDeleteNonHeadCommit,
    #[error("No changes to commit")]
    NoChanges,
}
