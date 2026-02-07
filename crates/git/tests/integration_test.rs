//! Integration tests for Git operations.

use crabontree_git::{GitError, GitRepository};
use std::process::Command;
use tempfile::TempDir;

/// Helper to initialize a test repository.
fn init_test_repo() -> (TempDir, GitRepository) {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path();

    // Initialize git repository
    Command::new("git")
        .args(["init"])
        .current_dir(repo_path)
        .output()
        .expect("Failed to init git repo");

    // Configure git
    Command::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(repo_path)
        .output()
        .expect("Failed to configure git");

    Command::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(repo_path)
        .output()
        .expect("Failed to configure git");

    // Create initial commit
    std::fs::write(repo_path.join("README.md"), "# Test Repo\n").unwrap();
    Command::new("git")
        .args(["add", "README.md"])
        .current_dir(repo_path)
        .output()
        .expect("Failed to add file");

    Command::new("git")
        .args(["commit", "-m", "Initial commit"])
        .current_dir(repo_path)
        .output()
        .expect("Failed to commit");

    let repo = GitRepository::open(repo_path).unwrap();
    (temp_dir, repo)
}

#[test]
fn test_open_valid_repository() {
    let (temp_dir, repo) = init_test_repo();
    assert_eq!(repo.path(), temp_dir.path());
}

#[test]
fn test_open_invalid_path() {
    let result = GitRepository::open("/nonexistent/path/to/repo");
    assert!(matches!(result, Err(GitError::RepoNotFound(_))));
}

#[test]
fn test_open_invalid_repository() {
    let temp_dir = TempDir::new().unwrap();
    // Create a directory but don't initialize git
    let result = GitRepository::open(temp_dir.path());
    assert!(matches!(result, Err(GitError::InvalidRepo(_))));
}

#[test]
fn test_get_head() {
    let (_temp_dir, repo) = init_test_repo();
    let head = repo.get_head().unwrap();

    // Should be on main or master branch
    assert!(head == "main" || head == "master", "HEAD is: {}", head);
}

#[test]
fn test_get_branches() {
    let (temp_dir, repo) = init_test_repo();

    let branches = repo.get_branches().unwrap();
    assert_eq!(branches.len(), 1);
    assert!(branches[0] == "main" || branches[0] == "master");

    // Create a new branch
    Command::new("git")
        .args(["branch", "feature/test"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to create branch");

    // Re-open repository to get fresh data
    let repo = GitRepository::open(temp_dir.path()).unwrap();
    let branches = repo.get_branches().unwrap();
    assert_eq!(branches.len(), 2);
    assert!(branches.contains(&"feature/test".to_string()));
}

#[test]
fn test_get_status() {
    let (_temp_dir, repo) = init_test_repo();
    let status = repo.get_status().unwrap();

    // For Phase 0, status is simplified
    // Just verify it doesn't crash
    assert_eq!(status.modified, 0);
    assert_eq!(status.added, 0);
    assert_eq!(status.deleted, 0);
    assert_eq!(status.untracked, 0);
}

#[test]
fn test_detached_head() {
    let (temp_dir, repo) = init_test_repo();

    // Get the current commit hash
    let output = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to get commit hash");

    let commit_hash = String::from_utf8(output.stdout).unwrap();
    let commit_hash = commit_hash.trim();

    // Checkout the commit directly (detached HEAD)
    Command::new("git")
        .args(["checkout", commit_hash])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to checkout commit");

    // Re-open repository
    let repo = GitRepository::open(temp_dir.path()).unwrap();
    let head = repo.get_head().unwrap();

    assert!(head.contains("detached"), "HEAD should be detached: {}", head);
}
