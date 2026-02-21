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
    let (temp_dir, _repo) = init_test_repo();

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

    assert!(
        head.contains("detached"),
        "HEAD should be detached: {}",
        head
    );
}
#[test]
fn test_working_dir_status_clean() {
    let (_temp_dir, repo) = init_test_repo();

    // Clean working directory should have no changes
    let status = repo.get_working_dir_status().unwrap();
    assert_eq!(status.len(), 0, "Clean repo should have no changes");
}

#[test]
fn test_working_dir_status_untracked() {
    let (temp_dir, repo) = init_test_repo();

    // Create new untracked file
    std::fs::write(temp_dir.path().join("new_file.txt"), "content").unwrap();

    let status = repo.get_working_dir_status().unwrap();
    assert_eq!(status.len(), 1);
    assert_eq!(status[0].path.to_str().unwrap(), "new_file.txt");
    assert_eq!(
        status[0].status,
        crabontree_git::WorkingDirStatus::Untracked
    );
    assert!(!status[0].is_staged);
}

#[test]
fn test_working_dir_status_modified() {
    let (temp_dir, repo) = init_test_repo();

    // Modify existing file
    std::fs::write(temp_dir.path().join("README.md"), "# Modified\n").unwrap();

    let status = repo.get_working_dir_status().unwrap();
    assert_eq!(status.len(), 1);
    assert_eq!(status[0].path.to_str().unwrap(), "README.md");
    assert_eq!(status[0].status, crabontree_git::WorkingDirStatus::Modified);
    assert!(!status[0].is_staged);
}

#[test]
fn test_working_dir_status_staged() {
    let (temp_dir, repo) = init_test_repo();
    let repo_path = temp_dir.path();

    // Create and stage new file
    std::fs::write(repo_path.join("staged.txt"), "staged content").unwrap();
    Command::new("git")
        .args(["add", "staged.txt"])
        .current_dir(repo_path)
        .output()
        .expect("Failed to stage file");

    let status = repo.get_working_dir_status().unwrap();
    assert_eq!(status.len(), 1);
    assert_eq!(status[0].path.to_str().unwrap(), "staged.txt");
    assert!(status[0].is_staged, "File should be marked as staged");
}

#[test]
fn test_working_dir_status_mixed() {
    let (temp_dir, repo) = init_test_repo();
    let repo_path = temp_dir.path();

    // Create untracked file
    std::fs::write(repo_path.join("untracked.txt"), "untracked").unwrap();

    // Modify existing file
    std::fs::write(repo_path.join("README.md"), "# Modified\n").unwrap();

    // Stage modified file
    Command::new("git")
        .args(["add", "README.md"])
        .current_dir(repo_path)
        .output()
        .expect("Failed to stage file");

    let status = repo.get_working_dir_status().unwrap();
    assert_eq!(status.len(), 2, "Should have 2 changed files");

    // Check for both files
    let readme = status
        .iter()
        .find(|f| f.path.to_str().unwrap() == "README.md");
    let untracked = status
        .iter()
        .find(|f| f.path.to_str().unwrap() == "untracked.txt");

    assert!(readme.is_some(), "README.md should be in status");
    assert!(untracked.is_some(), "untracked.txt should be in status");

    assert!(readme.unwrap().is_staged, "README.md should be staged");
    assert!(
        !untracked.unwrap().is_staged,
        "untracked.txt should not be staged"
    );
}

#[test]
fn test_get_author_identity() {
    let (_temp_dir, repo) = init_test_repo();

    // Should get configured identity
    let (name, email) = repo.get_author_identity().unwrap();
    assert_eq!(name, "Test User");
    assert_eq!(email, "test@example.com");
}

#[test]
fn test_has_staged_changes_false() {
    let (_temp_dir, repo) = init_test_repo();

    // Clean repo should have no staged changes
    let has_changes = repo.has_staged_changes().unwrap();
    assert!(!has_changes, "Clean repo should not have staged changes");
}

#[test]
fn test_has_staged_changes_true() {
    let (temp_dir, repo) = init_test_repo();
    let repo_path = temp_dir.path();

    // Create and stage a file
    std::fs::write(repo_path.join("new.txt"), "new content").unwrap();
    Command::new("git")
        .args(["add", "new.txt"])
        .current_dir(repo_path)
        .output()
        .expect("Failed to stage file");

    let has_changes = repo.has_staged_changes().unwrap();
    assert!(has_changes, "Repo with staged file should have changes");
}

#[test]
fn test_create_commit_success() {
    let (temp_dir, repo) = init_test_repo();
    let repo_path = temp_dir.path();

    // Create and stage a file
    std::fs::write(repo_path.join("file1.txt"), "content 1").unwrap();
    std::fs::write(repo_path.join("file2.txt"), "content 2").unwrap();
    Command::new("git")
        .args(["add", "."])
        .current_dir(repo_path)
        .output()
        .expect("Failed to stage files");

    // Create commit
    let commit_message = "Add two new files\n\nThis is a detailed commit message.";
    let commit_hash = repo.create_commit(commit_message).unwrap();

    // Verify commit hash is valid (40 char hex string)
    assert_eq!(commit_hash.len(), 40);
    assert!(commit_hash.chars().all(|c| c.is_ascii_hexdigit()));

    // Verify commit exists using git log
    let output = Command::new("git")
        .args(["log", "--oneline", "-n", "1"])
        .current_dir(repo_path)
        .output()
        .expect("Failed to get log");

    let log = String::from_utf8(output.stdout).unwrap();
    assert!(log.contains("Add two new files"));

    // Verify no staged changes remain
    let has_changes = repo.has_staged_changes().unwrap();
    assert!(!has_changes, "Should have no staged changes after commit");
}

#[test]
fn test_create_commit_no_staged_files() {
    let (_temp_dir, repo) = init_test_repo();

    // Try to commit without staging anything
    let result = repo.create_commit("Test commit");
    assert!(result.is_err());

    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("No staged changes"));
}

#[test]
fn test_create_commit_empty_message() {
    let (temp_dir, repo) = init_test_repo();
    let repo_path = temp_dir.path();

    // Create and stage a file
    std::fs::write(repo_path.join("file.txt"), "content").unwrap();
    Command::new("git")
        .args(["add", "file.txt"])
        .current_dir(repo_path)
        .output()
        .expect("Failed to stage file");

    // Try to commit with empty message
    let result = repo.create_commit("");
    assert!(result.is_err());

    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("empty"));
}

#[test]
fn test_create_commit_multiline_message() {
    let (temp_dir, repo) = init_test_repo();
    let repo_path = temp_dir.path();

    // Create and stage a file
    std::fs::write(repo_path.join("multi.txt"), "multi-line test").unwrap();
    Command::new("git")
        .args(["add", "multi.txt"])
        .current_dir(repo_path)
        .output()
        .expect("Failed to stage file");

    // Create commit with multi-line message
    let message = "Short summary line\n\nDetailed explanation:\n- Point 1\n- Point 2\n- Point 3";
    let commit_hash = repo.create_commit(message).unwrap();

    assert_eq!(commit_hash.len(), 40);

    // Verify full message was saved
    let output = Command::new("git")
        .args(["log", "--format=%B", "-n", "1"])
        .current_dir(repo_path)
        .output()
        .expect("Failed to get log");

    let log_message = String::from_utf8(output.stdout).unwrap();
    assert!(log_message.contains("Short summary line"));
    assert!(log_message.contains("Point 1"));
    assert!(log_message.contains("Point 3"));
}
