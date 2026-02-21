//! Pure state reducer function.

mod branch_handlers;
mod commit_handlers;
mod file_handlers;
mod repo_handlers;

use crate::{AppMessage, AppState, Effect};

/// Pure reducer function that updates state based on messages.
///
/// This function is deterministic and performs no I/O.
pub fn reduce(state: &mut AppState, msg: AppMessage) -> Effect {
    match msg {
        m @ (AppMessage::OpenRepoRequested(_)
        | AppMessage::RepoOpened { .. }
        | AppMessage::CloseRepo
        | AppMessage::RefreshRepo
        | AppMessage::RepoRefreshed { .. }
        | AppMessage::Error(_)
        | AppMessage::ClearError
        | AppMessage::LoadWorkingDirStatusRequested
        | AppMessage::WorkingDirStatusLoaded(_)) => repo_handlers::handle(state, m),

        m @ (AppMessage::LoadCommitHistoryRequested
        | AppMessage::CommitHistoryLoaded(_)
        | AppMessage::CommitSelected(_)
        | AppMessage::CommitDeselected
        | AppMessage::CommitDiffLoaded { .. }
        | AppMessage::StageFileRequested(_)
        | AppMessage::UnstageFileRequested(_)
        | AppMessage::StageAllRequested
        | AppMessage::UnstageAllRequested
        | AppMessage::StagingCompleted
        | AppMessage::StagingProgress { .. }
        | AppMessage::CommitMessageUpdated(_)
        | AppMessage::CreateCommitRequested
        | AppMessage::CommitCreated { .. }
        | AppMessage::AuthorIdentityLoaded { .. }
        | AppMessage::CommitSummaryUpdated(_)
        | AppMessage::CommitDescriptionUpdated(_)
        | AppMessage::AmendLastCommitToggled(_)
        | AppMessage::PushAfterCommitToggled(_)
        | AppMessage::CommitChangesRequested { .. }) => commit_handlers::handle(state, m),

        m @ (AppMessage::LoadBranchTreeRequested
        | AppMessage::BranchTreeLoaded(_)
        | AppMessage::BranchSectionToggled(_)
        | AppMessage::BranchSelected { .. }
        | AppMessage::BranchCheckoutRequested { .. }
        | AppMessage::ShowCheckoutWithChangesDialog { .. }
        | AppMessage::CheckoutWithStash { .. }
        | AppMessage::CheckoutWithDiscard { .. }
        | AppMessage::ShowRemoteBranchConflictDialog { .. }
        | AppMessage::CheckoutRemoteOverride { .. }
        | AppMessage::CheckoutRemoteRename { .. }
        | AppMessage::ChangesStashed { .. }
        | AppMessage::ChangesDiscarded
        | AppMessage::BranchCheckedOut(_)) => branch_handlers::handle(state, m),

        m @ (AppMessage::LoadChangedFilesRequested
        | AppMessage::ChangedFilesLoaded(_)
        | AppMessage::ChangedFileSelected(_)
        | AppMessage::SelectFileWithModifiers { .. }
        | AppMessage::StageSelectedFilesRequested
        | AppMessage::UnstageSelectedFilesRequested
        | AppMessage::FileContentRequested(_)
        | AppMessage::FileContentLoaded { .. }
        | AppMessage::FileDiffRequested(_)
        | AppMessage::FileDiffLoaded { .. }
        | AppMessage::MultipleFileDiffsLoaded { .. }
        | AppMessage::BinaryFileDetected { .. }
        | AppMessage::DiffViewModeChanged(_)) => file_handlers::handle(state, m),
    }
}
