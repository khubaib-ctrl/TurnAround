use serde::ser::{SerializeStruct, Serializer};
use thiserror::Error;

use crate::db::DbError;
use crate::registry::RegistryError;
use crate::vcs::VcsError;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("{0}")]
    Vcs(#[from] VcsError),

    #[error("{0}")]
    Db(#[from] DbError),

    #[error("{0}")]
    Registry(#[from] RegistryError),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("No active project")]
    NoActiveProject,

    #[error("Project not found in database")]
    ProjectNotFound,

    #[error("Project path does not exist")]
    ProjectPathNotExists,

    #[error("Not an EditGit project (no .editgit directory found)")]
    NotEditGitProject,

    #[error("{0}")]
    Backup(String),

    #[error("{0}")]
    Timeline(String),

    #[error("{0}")]
    Watcher(String),
}

impl From<rusqlite::Error> for AppError {
    fn from(e: rusqlite::Error) -> Self {
        AppError::Db(DbError::Sqlite(e))
    }
}

impl AppError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::Vcs(e) => match e {
                VcsError::Io(_) => "VCS_IO",
                VcsError::Db(_) => "VCS_DB",
                VcsError::NoActiveProject => "VCS_NO_ACTIVE_PROJECT",
                VcsError::NoActiveBranch => "VCS_NO_ACTIVE_BRANCH",
                VcsError::BranchNotFound(_) => "VCS_BRANCH_NOT_FOUND",
                VcsError::CannotDeleteActiveBranch => "VCS_CANNOT_DELETE_ACTIVE_BRANCH",
                VcsError::CannotDeleteLastBranch => "VCS_CANNOT_DELETE_LAST_BRANCH",
                VcsError::CommitNotFound(_) => "VCS_COMMIT_NOT_FOUND",
                VcsError::CannotDeleteNonHeadCommit => "VCS_CANNOT_DELETE_NON_HEAD",
                VcsError::NoChanges => "VCS_NO_CHANGES",
            },
            Self::Db(e) => match e {
                DbError::Sqlite(_) => "DB_SQLITE",
                DbError::Migration(_) => "DB_MIGRATION",
            },
            Self::Registry(e) => match e {
                RegistryError::Sqlite(_) => "REGISTRY_SQLITE",
                RegistryError::Io(_) => "REGISTRY_IO",
                RegistryError::ProjectNotFound(_) => "REGISTRY_PROJECT_NOT_FOUND",
                RegistryError::AlreadyRegistered(_) => "REGISTRY_ALREADY_REGISTERED",
                RegistryError::ProfileNotFound => "REGISTRY_PROFILE_NOT_FOUND",
            },
            Self::Io(_) => "IO_ERROR",
            Self::NoActiveProject => "PROJECT_NO_ACTIVE",
            Self::ProjectNotFound => "PROJECT_NOT_FOUND",
            Self::ProjectPathNotExists => "PROJECT_PATH_NOT_EXISTS",
            Self::NotEditGitProject => "PROJECT_NOT_EDITGIT",
            Self::Backup(_) => "BACKUP_ERROR",
            Self::Timeline(_) => "TIMELINE_ERROR",
            Self::Watcher(_) => "WATCHER_ERROR",
        }
    }
}

impl serde::Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("AppError", 2)?;
        state.serialize_field("code", self.code())?;
        state.serialize_field("message", &self.to_string())?;
        state.end()
    }
}
