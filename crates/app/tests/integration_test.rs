//! Integration tests for the full application flow.

use crabontree_app::{
    reduce, AppConfig, AppMessage, AppState, Effect, Job, JobExecutor,
};
use std::path::PathBuf;

#[test]
fn test_full_open_repository_flow() {
    // Initialize state
    let mut state = AppState {
        current_repo: None,
        loading: false,
        error: None,
        config: AppConfig::default(),
        staging_progress: None,
    };

    // Create job executor
    let (executor, mut message_rx) = JobExecutor::new();

    // Get the current repository path (CrabOnTree itself)
    let repo_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf();

    println!("Testing with repository at: {}", repo_path.display());

    // Simulate user clicking "Open Repository"
    let effect = reduce(&mut state, AppMessage::OpenRepoRequested(repo_path.clone()));

    // Verify state changes
    assert!(state.loading, "State should be loading");
    assert!(state.error.is_none(), "No error should be present");

    // Verify effect is to open the repository
    match effect {
        Effect::OpenRepo(path) => {
            assert_eq!(path, repo_path);

            // Execute the effect (submit job)
            executor.submit(Job::OpenRepo(path));
        }
        _ => panic!("Expected Effect::OpenRepo"),
    }

    // Wait for job to complete and receive message
    let message = message_rx.blocking_recv().expect("Should receive message");

    println!("Received message: {:?}", message);

    // Apply the result message
    let effect = reduce(&mut state, message);

    // Verify state after opening
    assert!(!state.loading, "Loading should be done");
    assert!(state.error.is_none(), "No error should occur");
    assert!(state.current_repo.is_some(), "Repository should be open");

    let repo = state.current_repo.as_ref().unwrap();
    assert_eq!(repo.path, repo_path);
    assert!(!repo.head.is_empty(), "HEAD should be set");
    assert!(!repo.branches.is_empty(), "Should have at least one branch");

    println!("Repository opened successfully!");
    println!("  Path: {}", repo.path.display());
    println!("  HEAD: {}", repo.head);
    println!("  Branches: {:?}", repo.branches);

    // Verify the Batch effect (SaveConfig + LoadCommitHistory + LoadWorkingDirStatus + LoadAuthorIdentity)
    match effect {
        Effect::Batch(effects) => {
            assert_eq!(effects.len(), 4, "Expected 4 effects in batch");
            // We don't need to check the exact order, just that they're there
        }
        _ => panic!("Expected Effect::Batch with SaveConfig, LoadCommitHistory, LoadWorkingDirStatus, and LoadAuthorIdentity"),
    }

    // Verify recent repos was updated
    assert_eq!(state.config.recent_repos.len(), 1);
    assert_eq!(state.config.recent_repos[0], repo_path);
}

#[test]
fn test_refresh_repository_flow() {
    // Initialize state with an open repository
    let repo_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf();

    let (executor, mut message_rx) = JobExecutor::new();

    // First, open the repository
    let mut state = AppState {
        current_repo: None,
        loading: false,
        error: None,
        config: AppConfig::default(),
        staging_progress: None,
    };

    let effect = reduce(&mut state, AppMessage::OpenRepoRequested(repo_path.clone()));
    match effect {
        Effect::OpenRepo(path) => {
            executor.submit(Job::OpenRepo(path));
        }
        _ => panic!("Expected Effect::OpenRepo"),
    }

    let message = message_rx.blocking_recv().expect("Should receive message");
    reduce(&mut state, message);

    assert!(state.current_repo.is_some(), "Repository should be open");

    // Now test refresh
    let effect = reduce(&mut state, AppMessage::RefreshRepo);

    assert!(state.loading, "Should be loading during refresh");

    match effect {
        Effect::RefreshRepo(path) => {
            assert_eq!(path, repo_path);
            executor.submit(Job::RefreshRepo(path));
        }
        _ => panic!("Expected Effect::RefreshRepo"),
    }

    // Wait for refresh to complete
    let message = message_rx.blocking_recv().expect("Should receive refresh message");

    let effect = reduce(&mut state, message);

    assert!(!state.loading, "Loading should be done after refresh");
    assert!(state.current_repo.is_some(), "Repository should still be open");
    assert!(matches!(effect, Effect::None));

    println!("Repository refreshed successfully!");
}

#[test]
fn test_open_invalid_repository() {
    let mut state = AppState {
        current_repo: None,
        loading: false,
        error: None,
        config: AppConfig::default(),
        staging_progress: None,
    };

    let (executor, mut message_rx) = JobExecutor::new();

    let invalid_path = PathBuf::from("/nonexistent/invalid/path");

    let effect = reduce(&mut state, AppMessage::OpenRepoRequested(invalid_path.clone()));

    assert!(state.loading);

    match effect {
        Effect::OpenRepo(path) => {
            executor.submit(Job::OpenRepo(path));
        }
        _ => panic!("Expected Effect::OpenRepo"),
    }

    // Wait for error message
    let message = message_rx.blocking_recv().expect("Should receive message");

    // Should be an error message
    match &message {
        AppMessage::Error(err) => {
            println!("Received expected error: {}", err);
            assert!(err.contains("Repository not found") || err.contains("Failed to open"));
        }
        _ => panic!("Expected AppMessage::Error, got: {:?}", message),
    }

    let effect = reduce(&mut state, message);

    assert!(!state.loading, "Loading should be done");
    assert!(state.error.is_some(), "Error should be set");
    assert!(state.current_repo.is_none(), "Repository should not be open");
    assert!(matches!(effect, Effect::None));

    println!("Invalid repository correctly rejected!");
}

#[test]
fn test_worker_thread_continues_after_error() {
    let (executor, mut message_rx) = JobExecutor::new();

    // Submit an invalid job
    executor.submit(Job::OpenRepo(PathBuf::from("/invalid/path")));

    // Should receive error
    let msg1 = message_rx.blocking_recv().expect("Should receive first message");
    assert!(matches!(msg1, AppMessage::Error(_)));

    // Submit a valid job - worker should still be running
    let valid_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf();

    executor.submit(Job::OpenRepo(valid_path));

    // Should receive success message
    let msg2 = message_rx
        .blocking_recv()
        .expect("Should receive second message");

    match msg2 {
        AppMessage::RepoOpened { .. } => {
            println!("Worker thread correctly recovered from error!");
        }
        _ => panic!("Expected RepoOpened message"),
    }
}
