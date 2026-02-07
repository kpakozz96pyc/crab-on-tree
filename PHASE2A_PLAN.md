# Phase 2a: Core Workflow Implementation

## Overview

**Goal**: Complete the essential Git workflow - view changes, stage files, and create commits.

**Status**: Ready to Start
**Estimated Time**: 50-70 hours (6-8 weeks at 8-10h/week)
**Priority**: Core features only, polish deferred to Phase 2b

## Scope Definition

### ✅ In Scope (Phase 2a)
- Working directory status (staged/unstaged files)
- Full line-by-line diff viewer
- Stage/unstage individual files
- Commit creation with multi-line messages
- Basic search and filtering
- Essential keyboard shortcuts
- Manual refresh only (no auto-watch)
- Text files only (binary files show "binary file changed")

### ⏸️ Deferred to Phase 2b or Phase 3
- Commit graph visualization
- File history view
- Auto-refresh with file watchers
- Advanced performance optimizations
- Syntax highlighting in diffs
- Binary/image file preview
- Partial staging (hunks)

---

## Implementation Plan

### Phase 0: API Feasibility Spike ⚡ (Week 0, Day 1)
**Time**: 2-3 hours
**Priority**: ⭐ Critical - Do this FIRST

**Goal**: Verify gix can do what we need before committing to full implementation.

**Tasks**:
- [ ] Create spike branch: `git checkout -b spike/phase2a-feasibility`
- [ ] Test gix status API:
  - [ ] Can get modified files
  - [ ] Can distinguish staged vs unstaged
  - [ ] Can get untracked files
- [ ] Test gix diff API:
  - [ ] Can compute line-by-line diffs
  - [ ] Can get hunks with context lines
  - [ ] Can diff working tree vs HEAD
  - [ ] Can diff working tree vs index
- [ ] Test gix staging API:
  - [ ] Can add files to index
  - [ ] Can remove files from index
  - [ ] Can manipulate index programmatically
- [ ] Test gix commit API:
  - [ ] Can create commits
  - [ ] Can get author from config
  - [ ] Can write commit messages
- [ ] Document findings in `docs/api-spike-results.md`

**Success Criteria**:
- All APIs exist and work as expected
- Any limitations are documented
- Plan adjusted if needed

**If APIs are insufficient**: Evaluate alternatives or adjust scope.

---

### Sprint 1: Working Directory & Diff (Weeks 1-2)
**Time**: 18-24 hours
**Focus**: See what needs to be committed

#### Task 1.1: Working Directory Status (Git + App)
**Time**: 6-8 hours

**Git Layer** (`crates/git/src/repo.rs`):
```rust
// New types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkingDirStatus {
    Modified,
    Added,
    Deleted,
    Untracked,
    Renamed,
}

#[derive(Debug, Clone)]
pub struct WorkingDirFile {
    pub path: PathBuf,
    pub status: WorkingDirStatus,
    pub is_staged: bool,
}

// New method
impl GitRepository {
    pub fn get_working_dir_status(&self) -> Result<Vec<WorkingDirFile>, GitError>
}
```

**Implementation**:
- [ ] Add types to `repo.rs`
- [ ] Implement `get_working_dir_status()` using gix status
- [ ] Walk working tree and compare with index and HEAD
- [ ] Categorize files as staged or unstaged
- [ ] Add integration tests (3-4 test cases)
- [ ] Export types in `lib.rs`

**App Layer** (`crates/app/`):
- [ ] Add `working_dir_files: Vec<WorkingDirFile>` to `RepoState`
- [ ] Add `LoadWorkingDirStatus` message
- [ ] Add `WorkingDirStatusLoaded(Vec<WorkingDirFile>)` message
- [ ] Add `LoadWorkingDirStatus(PathBuf)` job
- [ ] Add `LoadWorkingDirStatus(PathBuf)` effect
- [ ] Implement reducer handler
- [ ] Implement job executor function
- [ ] Update tests

**Success Criteria**:
- Can get list of all changed files
- Staged and unstaged are distinguished
- Tests pass

---

#### Task 1.2: Full Diff Viewer (Git + App + UI)
**Time**: 12-16 hours

**Git Layer** (`crates/git/src/repo.rs`):
```rust
// New types
#[derive(Debug, Clone)]
pub enum DiffLineType {
    Context,
    Addition,
    Deletion,
}

#[derive(Debug, Clone)]
pub struct DiffLine {
    pub old_line_num: Option<usize>,
    pub new_line_num: Option<usize>,
    pub content: String,
    pub line_type: DiffLineType,
}

#[derive(Debug, Clone)]
pub struct DiffHunk {
    pub old_start: usize,
    pub old_lines: usize,
    pub new_start: usize,
    pub new_lines: usize,
    pub lines: Vec<DiffLine>,
}

#[derive(Debug, Clone)]
pub struct FileDiffDetail {
    pub path: String,
    pub hunks: Vec<DiffHunk>,
    pub is_binary: bool,
}

// New methods
impl GitRepository {
    pub fn get_commit_file_diff(&self, commit_hash: &str, file_path: &str)
        -> Result<FileDiffDetail, GitError>;

    pub fn get_working_dir_file_diff(&self, file_path: &str)
        -> Result<FileDiffDetail, GitError>;
}
```

**Implementation**:
- [ ] Add types to `repo.rs`
- [ ] Implement `get_commit_file_diff()` using gix diff
- [ ] Implement `get_working_dir_file_diff()`
- [ ] Parse diff hunks into structured format
- [ ] Detect binary files and skip line diff
- [ ] Add integration tests (4-5 test cases)
- [ ] Export types in `lib.rs`

**App Layer**:
- [ ] Add `selected_file_diff: Option<FileDiffDetail>` to `RepoState`
- [ ] Add `LoadFileDiff { path, is_working_dir }` message
- [ ] Add `FileDiffLoaded(FileDiffDetail)` message
- [ ] Create job and effect
- [ ] Implement handlers

**UI Layer** (`crates/ui_egui/src/main.rs`):
```rust
fn render_diff_view(&self, ui: &mut egui::Ui, diff: &FileDiffDetail) {
    // Render hunks with line numbers and colors
}
```

**Implementation**:
- [ ] Create `render_diff_view()` function
- [ ] Show hunk headers (`@@ -10,5 +10,6 @@`)
- [ ] Render lines with line numbers
- [ ] Color code: green for additions, red for deletions
- [ ] Context lines in default color
- [ ] Handle binary files (show message only)
- [ ] Make scrollable with unique ID
- [ ] Add copy button (optional)

**Success Criteria**:
- Can view line diffs for committed files
- Can view line diffs for working dir files
- Colors and line numbers are correct
- Binary files handled gracefully

---

#### Task 1.3: Working Directory UI Panel
**Time**: 4-6 hours

**UI Layer** (`crates/ui_egui/src/main.rs`):

**Layout**:
```
┌──────────────────────────────────────────────────────┐
│  Repository Info (collapsible)                       │
├──────────────────────────────────────────────────────┤
│  [Changes] [History] tabs                            │
├─────────────────────┬────────────────────────────────┤
│  Staged (2)         │  Diff View                     │
│  ✓ src/main.rs      │  @@ -42,5 +42,8 @@            │
│  ✓ Cargo.toml       │  - old line                    │
│                     │  + new line                    │
│  Unstaged (3)       │  + another new line            │
│  ~ src/lib.rs       │                                │
│  + README.md        │  (line numbers on left)        │
│  - old_file.rs      │                                │
│                     │                                │
│  [Refresh] [Stage All]                               │
└─────────────────────┴────────────────────────────────┘
```

**Implementation**:
- [ ] Add tab state to track "Changes" vs "History" view
- [ ] Create `render_changes_tab()` function
- [ ] Show staged files section (collapsible)
- [ ] Show unstaged files section (collapsible)
- [ ] Add file status icons (✓ staged, ~ modified, + added, - deleted, ? untracked)
- [ ] Make files clickable to view diff
- [ ] Highlight selected file
- [ ] Add "Refresh" button
- [ ] Show file counts in section headers

**Success Criteria**:
- Can switch between Changes and History tabs
- Staged and unstaged files clearly separated
- Clicking file shows its diff
- UI is clean and intuitive

---

### Sprint 2: Staging & Commits (Weeks 3-4)
**Time**: 16-22 hours
**Focus**: Make commits through the UI

#### Task 2.1: Staging Operations (Git + App + UI)
**Time**: 8-10 hours

**Git Layer** (`crates/git/src/repo.rs`):
```rust
impl GitRepository {
    pub fn stage_file(&self, path: &Path) -> Result<(), GitError>;
    pub fn unstage_file(&self, path: &Path) -> Result<(), GitError>;
    pub fn stage_all(&self) -> Result<(), GitError>;
    pub fn unstage_all(&self) -> Result<(), GitError>;
}
```

**Implementation**:
- [ ] Implement `stage_file()` using gix index
- [ ] Implement `unstage_file()`
- [ ] Implement `stage_all()` (stage all modified/new files)
- [ ] Implement `unstage_all()` (reset index to HEAD)
- [ ] Add integration tests (4-5 test cases)

**App Layer**:
- [ ] Add `StageFile(PathBuf)` message
- [ ] Add `UnstageFile(PathBuf)` message
- [ ] Add `StageAll` message
- [ ] Add `UnstageAll` message
- [ ] Add `FileStaged` message (for success feedback)
- [ ] Create jobs and effects
- [ ] Implement handlers
- [ ] Auto-refresh working dir status after staging

**UI Layer**:
- [ ] Add "Stage" button next to unstaged files
- [ ] Add "Unstage" button next to staged files
- [ ] Add "Stage All" button in unstaged section
- [ ] Add "Unstage All" button in staged section
- [ ] Show loading spinner during operations
- [ ] Update UI immediately after operation
- [ ] Show brief success notification (optional)

**Success Criteria**:
- Can stage/unstage individual files by clicking
- Can stage/unstage all files with one click
- UI updates immediately
- Operations are async and non-blocking

---

#### Task 2.2: Commit Creation (Git + App + UI)
**Time**: 8-12 hours

**Git Layer** (`crates/git/src/repo.rs`):
```rust
impl GitRepository {
    pub fn create_commit(&self, message: &str) -> Result<String, GitError>;
    pub fn get_author_identity(&self) -> Result<(String, String), GitError>;
}
```

**Implementation**:
- [ ] Implement `get_author_identity()` from git config
- [ ] Implement `create_commit()` using gix commit
- [ ] Support multi-line commit messages
- [ ] Return new commit hash
- [ ] Handle errors gracefully (no staged files, etc.)
- [ ] Add integration tests (3-4 test cases)

**App Layer**:
- [ ] Add `commit_message: String` to `RepoState`
- [ ] Add `UpdateCommitMessage(String)` message
- [ ] Add `CreateCommit` message
- [ ] Add `CommitCreated(String)` message (with hash)
- [ ] Create job and effect
- [ ] Implement handlers
- [ ] Clear message after successful commit
- [ ] Refresh history and working dir after commit

**UI Layer**:
- [ ] Add commit message input (multi-line TextEdit)
- [ ] Add character/line count indicator
- [ ] Add "Commit" button (only enabled with staged files)
- [ ] Show staged file count on button: "Commit (3 files)"
- [ ] Add commit message placeholder text
- [ ] Show success message with commit hash after commit
- [ ] Clear input field after commit
- [ ] Focus back to file list after commit

**Commit Message UI**:
```
┌──────────────────────────────────────┐
│  Commit Message                      │
│  ┌────────────────────────────────┐  │
│  │ Add user authentication        │  │
│  │                                │  │
│  │ - Implement login/logout       │  │
│  │ - Add session management       │  │
│  └────────────────────────────────┘  │
│  [Commit (3 files)]                  │
└──────────────────────────────────────┘
```

**Success Criteria**:
- Can type multi-line commit messages
- Commit button enabled only with staged files
- Successfully creates commits
- UI refreshes showing new commit
- Message is cleared after commit

---

### Sprint 3: UX Enhancements (Weeks 5-6)
**Time**: 10-14 hours
**Focus**: Make it efficient to use

#### Task 3.1: Search and Filter
**Time**: 4-6 hours

**UI Layer**:
- [ ] Add search input above commit list
- [ ] Filter commits by message (case-insensitive substring match)
- [ ] Add author dropdown filter (populate from commits)
- [ ] Show "Showing X of Y commits" when filtered
- [ ] Add "Clear" button to reset filters
- [ ] Highlight search terms in commit messages (optional)

**Implementation**:
- [ ] Add `commit_filter: String` to UI state
- [ ] Add `author_filter: Option<String>` to UI state
- [ ] Filter commits in `render_commit_history()`
- [ ] Update count display
- [ ] Add unique IDs to avoid widget conflicts

**Success Criteria**:
- Can search commits by message content
- Can filter by author
- Filters apply in real-time
- Can clear filters easily

---

#### Task 3.2: Keyboard Shortcuts
**Time**: 3-4 hours

**UI Layer**:
- [ ] Implement keyboard event handling in `update()`
- [ ] Add navigation shortcuts:
  - `j` / `k` or `↓` / `↑` - Navigate commits
  - `Enter` - Select/view commit
- [ ] Add staging shortcuts:
  - `Space` - Stage/unstage selected file
  - `s` - Stage file
  - `u` - Unstage file
- [ ] Add commit shortcuts:
  - `c` - Focus commit message input
  - `Ctrl+Enter` - Create commit (when message focused)
- [ ] Add utility shortcuts:
  - `r` or `F5` - Refresh
  - `Ctrl+f` - Focus search
  - `Esc` - Clear search/deselect
  - `?` - Show keyboard shortcuts help
- [ ] Create help dialog showing all shortcuts
- [ ] Add tooltips to buttons showing shortcuts

**Help Dialog**:
```
┌────────────────────────────────────┐
│  Keyboard Shortcuts                │
│                                    │
│  Navigation                        │
│  j/k or ↓/↑    Navigate commits    │
│  Enter         Select commit       │
│                                    │
│  Staging                           │
│  Space         Stage/unstage       │
│  s             Stage file          │
│  u             Unstage file        │
│                                    │
│  Committing                        │
│  c             Focus message       │
│  Ctrl+Enter    Create commit       │
│                                    │
│  Utility                           │
│  r / F5        Refresh             │
│  Ctrl+f        Search              │
│  Esc           Clear/deselect      │
│  ?             This help           │
└────────────────────────────────────┘
```

**Success Criteria**:
- All shortcuts work without conflicts
- Help dialog accessible and helpful
- Keyboard navigation is smooth

---

#### Task 3.3: Performance Testing & Basic Optimization
**Time**: 3-4 hours

**Goal**: Ensure app is usable with large repositories

**Testing**:
- [ ] Test with Linux kernel repo (100k+ commits)
- [ ] Test with repo containing large files (>1MB)
- [ ] Measure key metrics:
  - Application startup time
  - Commit list scroll performance
  - Diff loading time
  - Staging/commit operation time
- [ ] Document performance in `docs/performance.md`

**Basic Optimizations** (only if needed):
- [ ] Limit initial commit load to 500 (add "Load more" button)
- [ ] Skip diff computation for very large files (>5MB)
- [ ] Cache last viewed diff
- [ ] Debounce search input (300ms delay)

**Performance Targets**:
- Startup: <3 seconds
- Scroll: 60fps with 500 commits visible
- Diff load: <200ms for typical files (<1000 lines)
- Staging: <50ms
- Commit: <300ms

**Success Criteria**:
- App is usable with large repos
- No noticeable lag in common operations
- Performance metrics documented

---

### Sprint 4: Testing & Documentation (Week 7)
**Time**: 8-12 hours
**Focus**: Production readiness

#### Task 4.1: Comprehensive Testing
**Time**: 4-6 hours

**Integration Tests** (`crates/git/tests/`):
- [ ] Test working dir status with mixed changes
- [ ] Test staging/unstaging operations
- [ ] Test commit creation with various messages
- [ ] Test diff computation for edge cases:
  - Empty files
  - New files
  - Deleted files
  - Binary files
  - Large files
- [ ] Test error scenarios:
  - No staged files when committing
  - Invalid file paths
  - Git operation failures

**App Layer Tests** (`crates/app/tests/`):
- [ ] Test reducer with new messages
- [ ] Test state transitions for staging workflow
- [ ] Test commit workflow (stage → message → commit → refresh)
- [ ] Test error handling

**Test Coverage Goals**:
- Git layer: 75%+
- App layer: 80%+
- Total tests: 40-45 (up from 26)

**Success Criteria**:
- All tests pass
- Critical workflows tested
- Edge cases covered

---

#### Task 4.2: Documentation
**Time**: 4-6 hours

**Documents to Create/Update**:

1. **`PHASE2A_COMPLETE.md`**:
   - [ ] Summary of features implemented
   - [ ] Test results
   - [ ] Known limitations
   - [ ] Next steps (Phase 2b/3)

2. **`README.md`**:
   - [ ] Update feature list with Phase 2a features
   - [ ] Add screenshots (working dir, diff, commit)
   - [ ] Update status section

3. **`docs/user-guide.md`** (new):
   - [ ] How to view changes
   - [ ] How to stage files
   - [ ] How to create commits
   - [ ] Keyboard shortcuts reference
   - [ ] Screenshots for each workflow

4. **`docs/architecture.md`**:
   - [ ] Document new Git operations
   - [ ] Document staging workflow
   - [ ] Update state diagram

5. **`docs/api.md`** (new):
   - [ ] Document public Git layer APIs
   - [ ] Code examples for each operation

**Success Criteria**:
- All features documented
- User guide complete with screenshots
- Architecture docs updated

---

## Success Criteria for Phase 2a

### Functional Requirements
- ✅ Can view all files in working directory (staged/unstaged)
- ✅ Can see full line-by-line diffs
- ✅ Can stage and unstage individual files
- ✅ Can stage/unstage all files at once
- ✅ Can create commits with multi-line messages
- ✅ Can search commits by message
- ✅ Can filter commits by author
- ✅ Keyboard shortcuts work
- ✅ Manual refresh updates all data

### Quality Requirements
- ✅ 40-45 tests passing (60% increase from 26)
- ✅ All tests pass with no warnings
- ✅ Clippy shows zero warnings
- ✅ Documentation complete with screenshots

### Performance Requirements
- ✅ Startup: <3 seconds (on reasonable repo)
- ✅ Commit list: 60fps scroll
- ✅ Diff load: <200ms for typical files
- ✅ Staging: <50ms per file
- ✅ Commit: <300ms total

### User Experience
- ✅ Workflow is intuitive (can use without manual)
- ✅ Keyboard navigation is smooth
- ✅ No UI freezing during operations
- ✅ Error messages are helpful

---

## What's NOT in Phase 2a

These are intentionally deferred:

### Deferred to Phase 2b/3
- ❌ Commit graph visualization
- ❌ File history view
- ❌ Auto-refresh with file watchers
- ❌ Partial staging (stage hunks, not whole files)
- ❌ Syntax highlighting in diffs
- ❌ Binary/image file preview
- ❌ Virtual scrolling for 10k+ commits
- ❌ Advanced diff algorithms
- ❌ Commit amending
- ❌ Branch operations (checkout, create, delete)
- ❌ Remote operations (fetch, pull, push)
- ❌ Merge/rebase support
- ❌ Stash operations

---

## Risk Mitigation

### Risk 1: gix API Limitations
**Mitigation**: API Feasibility Spike (Week 0)
- If APIs insufficient, evaluate alternatives early
- Can use `git` CLI as fallback if needed

### Risk 2: Time Overrun
**Mitigation**: Aggressive scope management
- Each sprint delivers working feature
- Can stop early and still have usable tool
- Phase 2a is minimum viable, not comprehensive

### Risk 3: Performance Issues
**Mitigation**: Test early and often
- Performance testing in Sprint 3
- Basic optimizations only if needed
- Advanced optimization deferred to Phase 2b

### Risk 4: UX Complexity
**Mitigation**: Focus on common workflows
- Design for simple case first (stage all → commit)
- Advanced use cases can be manual
- User testing before finalizing UI

---

## Development Guidelines

### Code Quality
- Follow existing patterns from Phase 0/1
- All public APIs must have doc comments
- Add unit tests for complex logic
- Integration tests for Git operations
- Run `cargo clippy` before committing
- Keep UI code in separate functions

### Git Commits
- Commit after each subtask completion
- Use conventional commit format
- Include Co-Authored-By for AI assistance
- Push to main after each sprint

### Testing Strategy
- Write tests alongside implementation
- Test happy path first, edge cases second
- Integration tests for Git layer
- Unit tests for App layer
- Manual testing for UI

---

## Timeline

| Week | Focus | Deliverable |
|------|-------|-------------|
| 0 | API Spike | Verified gix capabilities |
| 1 | Working Dir Status | Can see changed files |
| 2 | Diff Viewer | Can see line changes |
| 3 | Staging | Can stage/unstage files |
| 4 | Commits | Can create commits |
| 5 | Search & Shortcuts | Efficient navigation |
| 6 | Performance | Fast with large repos |
| 7 | Testing & Docs | Production ready |

**Total**: 7 weeks at 8-10 hours/week = **56-70 hours**

---

## Next Steps

1. **Start with API Spike** (2-3 hours)
   - Verify gix can do what we need
   - Document any limitations
   - Get green light to proceed

2. **Begin Sprint 1** (after spike)
   - Task 1.1: Working Directory Status
   - Task 1.2: Full Diff Viewer
   - Task 1.3: Working Directory UI

3. **Review after Sprint 1**
   - Validate approach
   - Adjust timeline if needed
   - Confirm scope

---

**Ready to start? Let's begin with the API Feasibility Spike!**

---

**Document Status**: Ready for Implementation
**Created**: 2026-02-07
**Estimated Total**: 50-70 hours
**Target Completion**: 7 weeks
