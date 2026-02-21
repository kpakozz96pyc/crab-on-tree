//! Tests for the state reducer.

use crabontree_app::{reduce, AppMessage, AppState, Effect, RepoState};
use crabontree_git::StatusSummary;
use std::path::PathBuf;

fn default_state() -> AppState {
    AppState::default()
}

#[test]
fn test_open_repo_requested() {
    let mut state = default_state();
    let path = PathBuf::from("/test/repo");

    let effect = reduce(&mut state, AppMessage::OpenRepoRequested(path.clone()));

    assert!(state.loading);
    assert!(state.error.is_none());
    assert!(matches!(effect, Effect::OpenRepo(p) if p == path));
}

#[test]
fn test_repo_opened() {
    let mut state = default_state();
    let path = PathBuf::from("/test/repo");

    let effect = reduce(
        &mut state,
        AppMessage::RepoOpened {
            path: path.clone(),
            head: "main".to_string(),
            branches: vec!["main".to_string(), "develop".to_string()],
            status: StatusSummary::default(),
        },
    );

    assert!(!state.loading);
    assert!(state.current_repo.is_some());

    let repo = state.current_repo.as_ref().unwrap();
    assert_eq!(repo.path, path);
    assert_eq!(repo.head, "main");
    assert_eq!(repo.branches.len(), 2);
    assert_eq!(repo.commits.len(), 0);
    assert!(repo.selected_commit.is_none());

    // Should add to recent repos
    assert_eq!(state.config.recent_repos.len(), 1);
    assert_eq!(state.config.recent_repos[0], path);

    // Should return batch effect with SaveConfig and LoadCommitHistory
    assert!(matches!(effect, Effect::Batch(_)));
}

#[test]
fn test_repo_opened_duplicate_recent() {
    let mut state = default_state();
    let path = PathBuf::from("/test/repo");

    // Add to recent repos first
    state.config.recent_repos.push(path.clone());

    reduce(
        &mut state,
        AppMessage::RepoOpened {
            path: path.clone(),
            head: "main".to_string(),
            branches: vec!["main".to_string()],
            status: StatusSummary::default(),
        },
    );

    // Should not duplicate
    assert_eq!(state.config.recent_repos.len(), 1);
}

#[test]
fn test_close_repo() {
    let mut state = default_state();
    state.current_repo = Some(RepoState {
        ..RepoState::new(
            PathBuf::from("/test/repo"),
            "main".to_string(),
            vec!["main".to_string()],
            StatusSummary::default(),
        )
    });

    let effect = reduce(&mut state, AppMessage::CloseRepo);

    assert!(state.current_repo.is_none());
    assert!(matches!(effect, Effect::None));
}

#[test]
fn test_refresh_repo() {
    let mut state = default_state();
    let path = PathBuf::from("/test/repo");

    state.current_repo = Some(RepoState {
        ..RepoState::new(
            path.clone(),
            "main".to_string(),
            vec!["main".to_string()],
            StatusSummary::default(),
        )
    });

    let effect = reduce(&mut state, AppMessage::RefreshRepo);

    assert!(state.loading);
    assert!(state.error.is_none());
    assert!(matches!(effect, Effect::RefreshRepo(p) if p == path));
}

#[test]
fn test_refresh_repo_no_repo() {
    let mut state = default_state();

    let effect = reduce(&mut state, AppMessage::RefreshRepo);

    assert!(!state.loading);
    assert!(matches!(effect, Effect::None));
}

#[test]
fn test_repo_refreshed() {
    let mut state = default_state();
    state.current_repo = Some(RepoState {
        ..RepoState::new(
            PathBuf::from("/test/repo"),
            "main".to_string(),
            vec!["main".to_string()],
            StatusSummary::default(),
        )
    });

    state.loading = true;

    let effect = reduce(
        &mut state,
        AppMessage::RepoRefreshed {
            head: "develop".to_string(),
            branches: vec![
                "main".to_string(),
                "develop".to_string(),
                "feature".to_string(),
            ],
            status: StatusSummary::default(),
        },
    );

    assert!(!state.loading);

    let repo = state.current_repo.as_ref().unwrap();
    assert_eq!(repo.head, "develop");
    assert_eq!(repo.branches.len(), 3);

    assert!(matches!(effect, Effect::None));
}

#[test]
fn test_error() {
    let mut state = default_state();
    state.loading = true;

    let error_msg = "Failed to open repository";
    let effect = reduce(&mut state, AppMessage::Error(error_msg.to_string()));

    assert!(!state.loading);
    assert_eq!(state.error, Some(error_msg.to_string()));
    assert!(matches!(effect, Effect::None));
}

#[test]
fn test_clear_error() {
    let mut state = default_state();
    state.error = Some("Some error".to_string());

    let effect = reduce(&mut state, AppMessage::ClearError);

    assert!(state.error.is_none());
    assert!(matches!(effect, Effect::None));
}

#[test]
fn test_max_recent_repos() {
    let mut state = default_state();
    state.config.max_recent = 3;

    // Add 4 repositories
    for i in 0..4 {
        let path = PathBuf::from(format!("/test/repo{}", i));
        reduce(
            &mut state,
            AppMessage::RepoOpened {
                path,
                head: "main".to_string(),
                branches: vec!["main".to_string()],
                status: StatusSummary::default(),
            },
        );
    }

    // Should only keep the last 3
    assert_eq!(state.config.recent_repos.len(), 3);
}
