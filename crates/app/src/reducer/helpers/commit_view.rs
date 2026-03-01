use crate::state::{BranchTreeState, ChangedFilesState, CommitInfo};
use crate::{Commit, FileDiff, FileStatus, WorkingDirFile, WorkingDirStatus};
use std::collections::HashSet;
use std::path::PathBuf;

pub(in crate::reducer) fn build_commit_info(
    commits: &[Commit],
    branch_tree: Option<&BranchTreeState>,
    commit_hash: &str,
) -> Option<CommitInfo> {
    commits.iter().find(|c| c.hash == commit_hash).map(|c| {
        let branches = branch_tree
            .map(|bt| {
                bt.local_branches
                    .iter()
                    .filter(|b| b.commit_hash == c.hash)
                    .map(|b| b.name.clone())
                    .collect()
            })
            .unwrap_or_default();

        let tags = branch_tree
            .map(|bt| {
                bt.tags
                    .iter()
                    .filter(|t| t.commit_hash == c.hash)
                    .map(|t| t.name.clone())
                    .collect()
            })
            .unwrap_or_default();

        CommitInfo {
            hash: c.hash.clone(),
            author_name: c.author_name.clone(),
            author_email: c.author_email.clone(),
            author_date: c.author_date,
            parent_hashes: c
                .parent_hashes
                .iter()
                .map(|h| h.chars().take(10).collect())
                .collect(),
            branches,
            tags,
        }
    })
}

pub(in crate::reducer) fn commit_message(commits: &[Commit], commit_hash: &str) -> String {
    commits
        .iter()
        .find(|c| c.hash == commit_hash)
        .map(|c| c.message.clone())
        .unwrap_or_default()
}

pub(in crate::reducer) fn build_commit_view_changed_files(
    diff: &[FileDiff],
    commit_message: String,
    commit_info: Option<CommitInfo>,
) -> ChangedFilesState {
    let staged = diff
        .iter()
        .map(|file_diff| WorkingDirFile {
            path: PathBuf::from(&file_diff.path),
            status: map_diff_status_to_working_status(&file_diff.status),
            is_staged: true,
        })
        .collect();

    ChangedFilesState {
        staged,
        unstaged: Vec::new(),
        untracked: Vec::new(),
        conflicted: Vec::new(),
        selected_file: None,
        selected_files: HashSet::new(),
        last_clicked_file: None,
        commit_message,
        is_commit_view: true,
        commit_info,
        commit_summary: String::new(),
        commit_description: String::new(),
        amend_last_commit: false,
        push_after_commit: false,
    }
}

fn map_diff_status_to_working_status(status: &FileStatus) -> WorkingDirStatus {
    match status {
        FileStatus::Modified => WorkingDirStatus::Modified,
        FileStatus::Added => WorkingDirStatus::Untracked,
        FileStatus::Deleted => WorkingDirStatus::Deleted,
        FileStatus::Renamed => WorkingDirStatus::Renamed,
        FileStatus::Copied => WorkingDirStatus::Modified,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::BranchInfo;

    #[test]
    fn build_commit_info_collects_branches_tags_and_short_parents() {
        let commit = Commit {
            hash: "0123456789abcdef".into(),
            hash_short: "0123456".into(),
            author_name: "A".into(),
            author_email: "a@example.com".into(),
            author_date: 1,
            committer_name: "A".into(),
            committer_email: "a@example.com".into(),
            committer_date: 1,
            message: "msg".into(),
            message_summary: "msg".into(),
            parent_hashes: vec!["1234567890abcdef".into()],
        };
        let tree = BranchTreeState {
            local_branches: vec![BranchInfo {
                name: "main".into(),
                commit_hash: commit.hash.clone(),
                is_current: true,
                upstream: None,
            }],
            remote_branches: Default::default(),
            tags: vec![crate::state::TagInfo {
                name: "v1.0.0".into(),
                commit_hash: commit.hash.clone(),
                message: None,
            }],
            current_branch: "main".into(),
            expanded_sections: Default::default(),
            selected_branch: None,
        };

        let info = build_commit_info(&[commit], Some(&tree), "0123456789abcdef").unwrap();
        assert_eq!(info.branches, vec!["main".to_string()]);
        assert_eq!(info.tags, vec!["v1.0.0".to_string()]);
        assert_eq!(info.parent_hashes, vec!["1234567890".to_string()]);
    }

    #[test]
    fn build_commit_view_changed_files_maps_status_and_sets_commit_mode() {
        let diff = vec![FileDiff {
            path: "src/lib.rs".into(),
            status: FileStatus::Added,
            additions: 1,
            deletions: 0,
            hunks: vec![],
        }];

        let changed = build_commit_view_changed_files(&diff, "message".into(), None);
        assert!(changed.is_commit_view);
        assert_eq!(changed.staged.len(), 1);
        assert_eq!(changed.staged[0].status, WorkingDirStatus::Untracked);
        assert_eq!(changed.commit_message, "message");
    }
}
