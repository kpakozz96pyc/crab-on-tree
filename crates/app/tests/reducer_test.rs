//! Tests for the state reducer.

use crabontree_app::{reduce, AppMessage, AppState, ChangedFilesState, Effect, RepoState};
use crabontree_git::{DiffHunk, StatusSummary, WorkingDirFile, WorkingDirStatus};
use std::collections::HashSet;
use std::path::PathBuf;

fn default_state() -> AppState {
    AppState::default()
}

fn changed_file(path: &str, is_staged: bool, status: WorkingDirStatus) -> WorkingDirFile {
    WorkingDirFile {
        path: PathBuf::from(path),
        status,
        is_staged,
    }
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

#[test]
fn test_stage_selected_files_uses_batch_effect() {
    let mut state = default_state();
    let mut repo = RepoState::new(
        PathBuf::from("/test/repo"),
        "main".to_string(),
        vec!["main".to_string()],
        StatusSummary::default(),
    );

    let mut selected = HashSet::new();
    selected.insert(PathBuf::from("src/a.rs"));
    selected.insert(PathBuf::from("src/b.rs"));

    repo.changed_files = Some(ChangedFilesState {
        staged: vec![],
        unstaged: vec![changed_file("src/a.rs", false, WorkingDirStatus::Modified)],
        untracked: vec![changed_file("src/b.rs", false, WorkingDirStatus::Untracked)],
        conflicted: vec![],
        selected_file: Some(PathBuf::from("src/a.rs")),
        selected_files: selected,
        last_clicked_file: Some(PathBuf::from("src/a.rs")),
        commit_message: String::new(),
        is_commit_view: false,
        commit_info: None,
        commit_summary: String::new(),
        commit_description: String::new(),
        amend_last_commit: false,
        push_after_commit: false,
    });
    state.current_repo = Some(repo);

    let effect = reduce(&mut state, AppMessage::StageSelectedFilesRequested);
    assert!(state.loading);
    match effect {
        Effect::StageFiles {
            repo_path,
            file_paths,
        } => {
            assert_eq!(repo_path, PathBuf::from("/test/repo"));
            assert_eq!(file_paths.len(), 2);
        }
        other => panic!("Expected Effect::StageFiles, got {:?}", other),
    }
}

#[test]
fn test_stale_file_diff_result_is_ignored() {
    let mut state = default_state();
    let mut repo = RepoState::new(
        PathBuf::from("/test/repo"),
        "main".to_string(),
        vec!["main".to_string()],
        StatusSummary::default(),
    );

    let mut selected = HashSet::new();
    selected.insert(PathBuf::from("new.rs"));

    repo.changed_files = Some(ChangedFilesState {
        staged: vec![],
        unstaged: vec![changed_file("new.rs", false, WorkingDirStatus::Modified)],
        untracked: vec![],
        conflicted: vec![],
        selected_file: Some(PathBuf::from("new.rs")),
        selected_files: selected,
        last_clicked_file: Some(PathBuf::from("new.rs")),
        commit_message: String::new(),
        is_commit_view: false,
        commit_info: None,
        commit_summary: String::new(),
        commit_description: String::new(),
        amend_last_commit: false,
        push_after_commit: false,
    });
    state.current_repo = Some(repo);

    reduce(
        &mut state,
        AppMessage::FileDiffLoaded {
            path: PathBuf::from("old.rs"),
            hunks: Vec::<DiffHunk>::new(),
        },
    );

    let repo = state.current_repo.as_ref().unwrap();
    assert!(matches!(
        repo.file_view,
        crabontree_app::FileViewState::None
    ));
}
