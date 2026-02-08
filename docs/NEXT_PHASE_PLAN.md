# Next Phase: Sprint 2 - Task 2.2: Commit Creation

## Current Status

### ✅ Completed Tasks
- **Sprint 1** (18-24 hours) - ✅ COMPLETE
  - Task 1.1: Working Directory Status - ✅ DONE
  - Task 1.2: Full Diff Implementation - ✅ DONE
  - Task 1.3: Working Directory UI Panel - ✅ DONE

- **Sprint 2** (16-22 hours) - ✅ COMPLETE
  - Task 2.1: Staging Operations - ✅ DONE
    - ✅ Stage/unstage individual files
    - ✅ Stage/unstage all files
    - ✅ Chunked batch operations for performance
    - ✅ Progress tracking infrastructure
    - ✅ Virtual scrolling optimizations
  - Task 2.2: Commit Creation - ✅ DONE
    - ✅ Git layer: get_author_identity(), create_commit(), has_staged_changes()
    - ✅ App layer: Full message/job/effect/reducer chain
    - ✅ UI layer: Commit message input, button, keyboard shortcuts
    - ✅ 7 new git tests, all passing
    - ✅ Author identity auto-loaded on repo open

- **Sprint 3** (10-14 hours) - ✅ COMPLETE
  - Task 3.1: Search and Filter - ✅ DONE
    - ✅ Working directory file search by path
    - ✅ Commit history search (message/author/hash)
    - ✅ Keyboard shortcuts (/ for files, Ctrl+F for commits)
    - ✅ Real-time filtering with counts
    - ✅ Clear buttons and Esc support
  - Task 3.2: Enhanced Keyboard Shortcuts - ✅ DONE
    - ✅ Arrow/vim key navigation (↑↓jk) for commits and files
    - ✅ Enter to select commit, Space to toggle file staging
    - ✅ Panel switching (1,2,3) and Tab cycling
    - ✅ Vim-style gg/G for top/bottom
    - ✅ Global actions (a=stage all, u=unstage all, r=refresh)
    - ✅ Help dialog (?) with comprehensive shortcut reference
    - ✅ Visual focus indicators (> marker)
  - Task 3.3: Performance Testing - ✅ DONE
    - ✅ Test repository generation script
    - ✅ Performance logging for key operations
    - ✅ 3 test scenarios (large files, long history, large diff)
    - ✅ Performance documentation and recommendations
    - ✅ Architecture validated for production use

### 📊 Progress Summary
- **Completed**: ~60-65 hours of work (Sprint 1 + Sprint 2 + Sprint 3)
- **Remaining in Phase 2a**: ~8-12 hours (Sprint 4: Testing & Documentation)
- **Current Status**: Sprint 3 Complete! Ready for Sprint 4

---

## Next Task: Task 2.2 - Commit Creation

**Estimated Time**: 8-12 hours
**Priority**: HIGH - Core workflow completion
**Status**: Ready to implement

### Overview

Enable users to create commits directly from the UI with proper commit messages.

### Implementation Plan

#### Phase 1: Git Layer (2-3 hours)

**File**: `crates/git/src/repo.rs`

**New Methods**:
```rust
impl GitRepository {
    /// Get author identity from git config.
    pub fn get_author_identity(&self) -> Result<(String, String), GitError> {
        // Read user.name and user.email from config
        // Return (name, email)
    }

    /// Create a commit with the current staged changes.
    pub fn create_commit(&self, message: &str) -> Result<String, GitError> {
        // Validate staged changes exist
        // Get author identity
        // Create commit object
        // Update HEAD
        // Return commit hash
    }

    /// Check if there are staged changes ready to commit.
    pub fn has_staged_changes(&self) -> Result<bool, GitError> {
        // Check if index differs from HEAD
    }
}
```

**Implementation Steps**:
1. [ ] Implement `get_author_identity()`
   - Read from git2 config
   - Handle missing config gracefully
   - Use fallback values if needed

2. [ ] Implement `has_staged_changes()`
   - Compare index with HEAD tree
   - Return true if differences exist

3. [ ] Implement `create_commit()`
   - Validate message is not empty
   - Check has_staged_changes()
   - Get author identity
   - Create commit using git2
   - Update HEAD reference
   - Return new commit SHA

4. [ ] Add error handling
   - No staged files error
   - Invalid author config
   - Commit write failures

5. [ ] Write integration tests
   - Test successful commit creation
   - Test commit with no staged files (should fail)
   - Test multi-line commit messages
   - Test author identity retrieval

**Success Criteria**:
- ✅ Can read author from git config
- ✅ Can create commits with staged files
- ✅ Returns commit hash
- ✅ Validates preconditions
- ✅ 3-4 integration tests passing

---

#### Phase 2: App Layer (2-3 hours)

**Files**:
- `crates/app/src/state.rs`
- `crates/app/src/message.rs`
- `crates/app/src/job.rs`
- `crates/app/src/effect.rs`
- `crates/app/src/reducer.rs`
- `crates/app/src/executor.rs`

**State Changes**:
```rust
// In RepoState
pub struct RepoState {
    // ... existing fields ...
    pub commit_message: String,
    pub author_name: String,
    pub author_email: String,
}
```

**New Messages**:
```rust
pub enum AppMessage {
    // ... existing variants ...

    /// User updated the commit message.
    CommitMessageUpdated(String),

    /// User requested to create a commit.
    CreateCommitRequested,

    /// Commit was created successfully.
    CommitCreated {
        hash: String,
        message: String,
    },

    /// Author identity was loaded.
    AuthorIdentityLoaded {
        name: String,
        email: String,
    },
}
```

**New Jobs**:
```rust
pub enum Job {
    // ... existing variants ...

    /// Create a commit with the given message.
    CreateCommit {
        repo_path: PathBuf,
        message: String,
    },

    /// Load author identity from git config.
    LoadAuthorIdentity(PathBuf),
}
```

**New Effects**:
```rust
pub enum Effect {
    // ... existing variants ...

    /// Create a commit.
    CreateCommit {
        repo_path: PathBuf,
        message: String,
    },

    /// Load author identity.
    LoadAuthorIdentity(PathBuf),
}
```

**Implementation Steps**:
1. [ ] Add fields to RepoState
2. [ ] Add new message types
3. [ ] Add new job types
4. [ ] Add new effect types
5. [ ] Implement reducer handlers
   - CommitMessageUpdated: Update state
   - CreateCommitRequested: Trigger effect if message not empty
   - CommitCreated: Clear message, refresh repo
   - AuthorIdentityLoaded: Update state
6. [ ] Implement executor functions
   - execute_create_commit()
   - execute_load_author_identity()
7. [ ] Update effect handler in UI
8. [ ] Load author identity when repo opens
9. [ ] Update tests

**Success Criteria**:
- ✅ Commit message state managed correctly
- ✅ Create commit message triggers job
- ✅ Successful commit refreshes UI
- ✅ Author identity loaded on repo open

---

#### Phase 3: UI Layer (4-6 hours)

**File**: `crates/ui_egui/src/main.rs`

**UI Design**:
```
┌─────────────────────────────────────────────┐
│ 🔨 Working Directory                        │
│ ┌─────────────────────────────────────────┐ │
│ │ Stage All (12) | Unstage All (3)        │ │
│ └─────────────────────────────────────────┘ │
│                                             │
│ [S] ~/src/main.rs                           │
│ [S] ~/src/lib.rs                            │
│ [U] ~/README.md                             │
│                                             │
├─────────────────────────────────────────────┤
│ 💬 Commit Message                           │
│ ┌─────────────────────────────────────────┐ │
│ │ Add user authentication                 │ │
│ │                                         │ │
│ │ - Implement login/logout                │ │
│ │ - Add session management                │ │
│ │ - Add password hashing                  │ │
│ └─────────────────────────────────────────┘ │
│ 72 chars | 5 lines | 3 files staged        │
│                                             │
│ [📝 Commit] [⟲ Refresh]                     │
└─────────────────────────────────────────────┘
```

**Implementation Steps**:

1. [ ] Add commit message input section
   ```rust
   fn render_commit_panel(&mut self, ui: &mut egui::Ui, repo: &RepoState) {
       // Commit message section
       ui.heading("Commit Message");

       // Multi-line text editor
       let response = egui::TextEdit::multiline(&mut self.commit_message_buffer)
           .desired_width(f32::INFINITY)
           .desired_rows(5)
           .hint_text("Enter commit message...")
           .show(ui);

       // Update state on change
       if response.changed() {
           self.handle_message(AppMessage::CommitMessageUpdated(
               self.commit_message_buffer.clone()
           ));
       }
   }
   ```

2. [ ] Add commit message buffer to UI state
   ```rust
   struct CrabOnTreeApp {
       // ... existing fields ...
       commit_message_buffer: String,
   }
   ```

3. [ ] Add commit statistics display
   - Character count
   - Line count
   - Number of staged files
   - Show in muted color below text editor

4. [ ] Add commit button
   - Only enabled when:
     - Staged files exist
     - Message is not empty (at least 1 non-whitespace char)
   - Show staged file count on button
   - Format: "Commit (N files)"

5. [ ] Add keyboard shortcuts
   - `Ctrl+Enter`: Create commit (when in text editor)
   - `c`: Focus commit message input

6. [ ] Add success notification
   - Show commit hash after successful commit
   - Format: "✓ Committed: abc1234"
   - Auto-dismiss after 3 seconds (or make dismissible)

7. [ ] Clear commit message after commit
   - Reset buffer to empty
   - Sync with state

8. [ ] Add validation feedback
   - Show error if trying to commit with no staged files
   - Show error if message is empty
   - Red border or error text below input

9. [ ] Add author identity display
   - Show "Committing as: Name <email>" in small text
   - Let user know who will be the author

10. [ ] Handle edge cases
    - Disable commit button during commit operation
    - Show loading spinner on button when committing
    - Handle commit failures gracefully

**Keyboard Shortcut Summary**:
- `c` - Focus commit message input
- `Ctrl+Enter` - Create commit (when message focused)
- `Esc` - Clear/blur commit message input

**Success Criteria**:
- ✅ Can type multi-line commit messages
- ✅ Shows character/line count
- ✅ Commit button only enabled when valid
- ✅ Shows staged file count
- ✅ Keyboard shortcuts work
- ✅ Shows success notification after commit
- ✅ Clears message after successful commit
- ✅ Shows author identity

---

## Testing Plan

### Integration Tests (Git Layer)
```rust
#[test]
fn test_get_author_identity() {
    // Setup test repo with config
    // Call get_author_identity()
    // Verify returns expected values
}

#[test]
fn test_create_commit_success() {
    // Setup test repo
    // Stage some files
    // Create commit
    // Verify commit exists and has correct message
}

#[test]
fn test_create_commit_no_staged_files() {
    // Setup test repo
    // Try to commit without staging
    // Verify returns error
}

#[test]
fn test_has_staged_changes() {
    // Test with no changes
    // Stage a file
    // Verify returns true
}
```

### Manual Testing Checklist
- [ ] Create commit with single-line message
- [ ] Create commit with multi-line message
- [ ] Try to commit with no staged files (should fail)
- [ ] Try to commit with empty message (should be prevented)
- [ ] Verify commit appears in history after creation
- [ ] Verify working directory clears after commit
- [ ] Test keyboard shortcuts (c, Ctrl+Enter)
- [ ] Verify author identity displays correctly
- [ ] Test with very long commit messages (200+ chars)
- [ ] Test with special characters in message

---

## Risk Assessment

### Risk 1: Git Config Missing
**Probability**: Medium
**Impact**: High
**Mitigation**:
- Provide fallback values (user@hostname)
- Show warning to user if config missing
- Link to git config documentation

### Risk 2: UI Complexity
**Probability**: Low
**Impact**: Medium
**Mitigation**:
- Keep UI simple and focused
- Follow established patterns from staging UI
- Test early with real use cases

### Risk 3: Commit Operation Failures
**Probability**: Medium
**Impact**: Medium
**Mitigation**:
- Comprehensive error handling
- Clear error messages to user
- Rollback state on failure

---

## Timeline

| Phase | Task | Time | Status |
|-------|------|------|--------|
| 1 | Git Layer Implementation | 2-3h | ⏸️ Next |
| 2 | App Layer Integration | 2-3h | ⏸️ Pending |
| 3 | UI Implementation | 4-6h | ⏸️ Pending |
| - | Testing & Refinement | 1-2h | ⏸️ Pending |

**Total Estimated Time**: 8-12 hours

---

## After This Task

Upon completion of Task 2.2, the core commit workflow will be complete:
1. ✅ View working directory changes
2. ✅ Stage/unstage files
3. ✅ Create commits

Next up would be:
- **Sprint 3**: UX Enhancements (search, keyboard shortcuts, performance)
- **Sprint 4**: Testing & Documentation

---

## Quick Start Guide

To implement this task:

1. **Start with Git Layer** (safest, most isolated)
   ```bash
   # Edit: crates/git/src/repo.rs
   # Add: get_author_identity(), create_commit(), has_staged_changes()
   # Test: crates/git/tests/integration_test.rs
   cargo test -p crabontree-git
   ```

2. **Move to App Layer** (plumbing)
   ```bash
   # Edit: state, messages, jobs, effects, reducer, executor
   # Test: crates/app/tests/
   cargo test -p crabontree-app
   ```

3. **Finish with UI Layer** (visible results)
   ```bash
   # Edit: crates/ui_egui/src/main.rs
   # Run: cargo run
   # Test manually in app
   ```

4. **Integration Test** (end-to-end)
   ```bash
   # Open app
   # Open a repo
   # Make changes
   # Stage files
   # Write commit message
   # Click Commit
   # Verify commit appears in history
   ```

---

**Ready to implement? Start with Phase 1: Git Layer!**
