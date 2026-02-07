//! Error types for Git operations.

use thiserror::Error;

/// Errors that can occur during Git operations.
#[derive(Error, Debug)]
pub enum GitError {
    #[error("Repository not found at path: {0}")]
    RepoNotFound(String),

    #[error("Invalid repository: {0}")]
    InvalidRepo(String),

    #[error("Git operation failed: {0}")]
    OperationFailed(String),

    #[error("Reference not found: {0}")]
    RefNotFound(String),
}
