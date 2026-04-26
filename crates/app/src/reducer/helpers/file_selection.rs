use crate::ChangedFilesState;
use std::collections::HashSet;

use std::path::{Path, PathBuf};

pub(in crate::reducer) fn collect_files_to_stage(files: &ChangedFilesState) -> Vec<PathBuf> {
    files
        .selected_files
        .iter()
        .filter(|path| {
            files.unstaged.iter().any(|f| &f.path == *path)
                || files.untracked.iter().any(|f| &f.path == *path)
        })
        .cloned()
        .collect()
}

pub(in crate::reducer) fn collect_files_to_unstage(files: &ChangedFilesState) -> Vec<PathBuf> {
    files
        .selected_files
        .iter()
        .filter(|path| files.staged.iter().any(|f| &f.path == *path))
        .cloned()
        .collect()
}

pub(in crate::reducer) fn should_apply_single_file_result(
    changed_files: Option<&ChangedFilesState>,
    path: &Path,
) -> bool {
    let Some(files) = changed_files else {
        return false;
    };

    files.selected_files.len() == 1
        && files
            .selected_file
            .as_ref()
            .is_some_and(|selected| selected == path)
        && files.selected_files.contains(path)
}

pub(in crate::reducer) fn should_apply_multi_file_result(
    changed_files: Option<&ChangedFilesState>,
    selected_paths: &[PathBuf],
) -> bool {
    let Some(files) = changed_files else {
        return false;
    };

    if files.selected_files.len() <= 1 {
        return false;
    }

    let result_set: HashSet<_> = selected_paths.iter().collect();
    files.selected_files.len() == result_set.len()
        && files.selected_files.iter().all(|p| result_set.contains(p))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::ChangedFilesState;
    use crate::{WorkingDirFile, WorkingDirStatus};

    fn file(path: &str) -> WorkingDirFile {
        WorkingDirFile {
            path: PathBuf::from(path),
            status: WorkingDirStatus::Modified,
            is_staged: false,
        }
    }

    fn changed_files() -> ChangedFilesState {
        ChangedFilesState {
            staged: vec![],
            unstaged: vec![file("a.rs")],
            untracked: vec![file("b.rs")],
            conflicted: vec![],
            selected_file: None,
            selected_files: HashSet::new(),
            last_clicked_file: None,
            commit_message: String::new(),
            is_commit_view: false,
            commit_info: None,
            commit_summary: String::new(),
            commit_description: String::new(),
            amend_last_commit: false,
            push_after_commit: false,
        }
    }

    #[test]
    fn collect_files_to_stage_filters_by_unstaged_or_untracked() {
        let mut files = changed_files();
        files.selected_files.insert(PathBuf::from("a.rs"));
        files.selected_files.insert(PathBuf::from("c.rs"));

        let paths = collect_files_to_stage(&files);
        assert_eq!(paths, vec![PathBuf::from("a.rs")]);
    }

    #[test]
    fn should_apply_single_file_result_rejects_stale_path() {
        let mut files = changed_files();
        files.selected_file = Some(PathBuf::from("a.rs"));
        files.selected_files.insert(PathBuf::from("a.rs"));
        assert!(!should_apply_single_file_result(
            Some(&files),
            Path::new("b.rs")
        ));
    }

    #[test]
    fn should_apply_multi_file_result_requires_exact_match() {
        let mut files = changed_files();
        files.selected_files.insert(PathBuf::from("a.rs"));
        files.selected_files.insert(PathBuf::from("b.rs"));

        assert!(should_apply_multi_file_result(
            Some(&files),
            &[PathBuf::from("b.rs"), PathBuf::from("a.rs")]
        ));
        assert!(!should_apply_multi_file_result(
            Some(&files),
            &[PathBuf::from("a.rs")]
        ));
    }
}
