//! Git operations layer using gitoxide (gix).
//!
//! This crate provides a high-level wrapper around gitoxide for common Git operations
//! needed by CrabOnTree. It is designed to be UI-agnostic and focused purely on Git
//! repository interactions.

pub mod error;
pub mod repo;

pub use error::GitError;
pub use repo::{Commit, FileDiff, FileStatus, GitRepository, StatusSummary};
