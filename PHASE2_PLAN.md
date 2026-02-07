# Phase 2 Implementation Plan

## Overview

**Goal**: Enhance CrabOnTree with full diff viewing, working directory management, and staging capabilities.

**Status**: Planning Phase
**Target Completion**: 70-90 hours
**Priority**: High-value features that complete the core Git workflow

## Current State (Phase 1 Complete)

✅ Repository operations (open, refresh, close)
✅ Branch viewing (local + remote)
✅ Commit history with interactive selection
✅ Commit details display
✅ File-level diff (list of changed files with status)
✅ Clean architecture with async operations
✅ Theme support and configuration

## Phase 2 Goals

1. **Full diff viewing** - See actual line-by-line changes
2. **Working directory status** - View uncommitted changes
3. **Staging operations** - Stage/unstage files for commit
4. **Commit creation** - Create commits with messages
5. **Enhanced UX** - Better navigation and visual improvements

---

## Implementation Roadmap

### Sprint 1: Foundation (Week 1) - 24-30 hours

Focus: Core Git operations and data structures

#### Task 1.1: Working Directory Status Implementation
**Priority**: ⭐ Critical
**Estimated Time**: 6-8 hours

**Git Layer (crates/git/):**
- [ ] Create `FileStatus` enum (Modified, Added, Deleted, Untracked, etc.)
- [ ] Create `WorkingDirFile` struct with path, status, and metadata
- [ ] Implement `get_working_dir_status() -> Result<Vec<WorkingDirFile>>`
- [ ] Use gix status API to walk working tree
- [ ] Compare working tree with index (staged changes)
- [ ] Compare index with HEAD (unstaged changes)
- [ ] Add unit tests for various status scenarios

**App Layer (crates/app/):**
- [ ] Add `working_dir_files: Vec<WorkingDirFile>` to `RepoState`
- [ ] Add `LoadWorkingDirStatus` message
- [ ] Add `WorkingDirStatusLoaded` message
- [ ] Add `LoadWorkingDirStatus` job
- [ ] Add `LoadWorkingDirStatus` effect
- [ ] Implement reducer handlers
- [ ] Implement job executor

**Success Criteria:**
- Can list all modified files in working directory
- Can distinguish between staged and unstaged changes
- Status updates correctly after file changes

---

#### Task 1.2: Full Diff Implementation (Line-by-Line)
**Priority**: ⭐ Critical
**Estimated Time**: 10-12 hours

**Git Layer (crates/git/):**
- [ ] Create `LineDiff` struct with line numbers, content, and diff type
- [ ] Create `FileDiffDetail` struct with hunks and line changes
- [ ] Implement `get_file_diff(commit_hash, file_path) -> Result<FileDiffDetail>`
- [ ] Implement `get_working_dir_file_diff(file_path) -> Result<FileDiffDetail>`
- [ ] Use gix diff API to compute line-by-line changes
- [ ] Parse diff hunks with context lines
- [ ] Add tests for various diff scenarios

**App Layer (crates/app/):**
- [ ] Add `selected_file_diff: Option<FileDiffDetail>` to `RepoState`
- [ ] Add `LoadFileDiff { commit_hash, file_path }` message
- [ ] Add `FileDiffLoaded` message
- [ ] Create corresponding job and effect
- [ ] Implement reducer handlers

**UI Layer (crates/ui_egui/):**
- [ ] Create `render_line_diff()` function
- [ ] Implement unified diff view (like git diff)
- [ ] Color-code additions (green) and deletions (red)
- [ ] Show line numbers for old and new versions
- [ ] Add context lines between changes
- [ ] Make diff scrollable with unique ID
- [ ] Add "Copy" button for diff content

**Success Criteria:**
- Can view line-by-line diff for any commit
- Additions and deletions are clearly color-coded
- Line numbers are accurate
- Diff is scrollable and readable

---

#### Task 1.3: Working Directory UI Panel
**Priority**: ⭐ Critical
**Estimated Time**: 6-8 hours

**UI Layer (crates/ui_egui/):**
- [ ] Create new "Changes" tab/section
- [ ] Implement `render_working_dir_status()` function
- [ ] Show list of modified files with status icons
- [ ] Show staged vs unstaged sections
- [ ] Click file to view diff
- [ ] Add "Refresh" button for working directory
- [ ] Show file count summary

**Layout Structure:**
```
┌─────────────────────────────────────────────────────┐
│  Changes Tab | History Tab                          │
├─────────────────────┬───────────────────────────────┤
│  Staged (2)         │  Diff View                    │
│  ✓ file1.rs         │  @@ -10,5 +10,6 @@           │
│  ✓ file2.rs         │  - old line                   │
│                     │  + new line                   │
│  Unstaged (3)       │                               │
│  ~ file3.rs         │                               │
│  + file4.rs         │                               │
│  - file5.rs         │                               │
└─────────────────────┴───────────────────────────────┘
```

**Success Criteria:**
- Working directory changes are visible in dedicated panel
- Can click files to see their diffs
- Staged and unstaged changes are separated
- UI updates when files change

---

### Sprint 2: Staging & Commits (Week 2) - 22-28 hours

Focus: Interactive Git operations

#### Task 2.1: Staging Operations
**Priority**: ⭐ Critical
**Estimated Time**: 8-10 hours

**Git Layer (crates/git/):**
- [ ] Implement `stage_file(path: &Path) -> Result<()>`
- [ ] Implement `unstage_file(path: &Path) -> Result<()>`
- [ ] Implement `stage_all() -> Result<()>`
- [ ] Implement `unstage_all() -> Result<()>`
- [ ] Use gix index manipulation APIs
- [ ] Add proper error handling
- [ ] Add tests for staging operations

**App Layer (crates/app/):**
- [ ] Add `StageFile(PathBuf)` message
- [ ] Add `UnstageFile(PathBuf)` message
- [ ] Add `StageAll` message
- [ ] Add `UnstageAll` message
- [ ] Create corresponding jobs and effects
- [ ] Implement reducer handlers
- [ ] Auto-refresh working dir status after staging

**UI Layer (crates/ui_egui/):**
- [ ] Add "Stage" button next to unstaged files
- [ ] Add "Unstage" button next to staged files
- [ ] Add "Stage All" / "Unstage All" buttons
- [ ] Show loading indicator during operations
- [ ] Update UI immediately after operations

**Success Criteria:**
- Can stage individual files
- Can unstage individual files
- Can stage/unstage all files
- UI reflects changes immediately

---

#### Task 2.2: Commit Creation Dialog
**Priority**: ⭐ Critical
**Estimated Time**: 6-8 hours

**Git Layer (crates/git/):**
- [ ] Implement `create_commit(message: &str) -> Result<String>`
- [ ] Use gix commit creation API
- [ ] Support multi-line commit messages
- [ ] Get author info from git config
- [ ] Return commit hash on success
- [ ] Add tests for commit creation

**App Layer (crates/app/):**
- [ ] Add `commit_message: String` to `RepoState`
- [ ] Add `UpdateCommitMessage(String)` message
- [ ] Add `CreateCommit` message
- [ ] Add `CommitCreated(String)` message
- [ ] Create corresponding job and effect
- [ ] Clear message and refresh after commit

**UI Layer (crates/ui_egui/):**
- [ ] Create commit message input (multi-line TextEdit)
- [ ] Add "Commit" button (enabled only with staged files)
- [ ] Show commit button with staged file count
- [ ] Add commit message placeholder/hints
- [ ] Show success notification after commit
- [ ] Clear input after successful commit

**Success Criteria:**
- Can write multi-line commit messages
- Commit button disabled when no staged changes
- Successfully creates commits
- UI refreshes after commit creation

---

#### Task 2.3: Enhanced Commit History with Graph
**Priority**: 🔶 High
**Estimated Time**: 8-10 hours

**Git Layer (crates/git/):**
- [ ] Add `branch_refs: Vec<String>` to `Commit` struct
- [ ] Add `is_merge: bool` to `Commit` struct
- [ ] Compute parent relationships for graph
- [ ] Identify branch points and merges

**UI Layer (crates/ui_egui/):**
- [ ] Implement simple graph visualization using egui shapes
- [ ] Draw lines connecting parent-child commits
- [ ] Use different colors for different branches
- [ ] Show branch/tag labels on commits
- [ ] Make graph compact and readable

**Graph Algorithm:**
```
Commit1 ●─┐
        │ │
Commit2 ● │  (main)
        │ │
Commit3 │ ●  (feature)
        │ │
Commit4 ●─┘  (merge)
```

**Success Criteria:**
- Visual graph shows commit relationships
- Branch lines are distinguishable
- Merges are clearly visible
- Graph is performant with 100+ commits

---

### Sprint 3: Polish & Performance (Week 3) - 16-22 hours

Focus: User experience and optimization

#### Task 3.1: Search and Filter
**Priority**: 🔶 High
**Estimated Time**: 4-6 hours

**UI Layer (crates/ui_egui/):**
- [ ] Add search input field above commit list
- [ ] Filter commits by message content (case-insensitive)
- [ ] Add author filter dropdown
- [ ] Add date range filter
- [ ] Show "X of Y commits" when filtered
- [ ] Add "Clear filters" button

**Success Criteria:**
- Can search commits by message
- Can filter by author
- Filters work in real-time
- Clear indication when filters are active

---

#### Task 3.2: Keyboard Shortcuts
**Priority**: 🔶 High
**Estimated Time**: 3-4 hours

**UI Layer (crates/ui_egui/):**
- [ ] Implement keyboard event handling
- [ ] Add shortcuts for common operations:
  - `j/k` or `↑/↓` - Navigate commits
  - `Enter` - Select/deselect commit
  - `Space` - Stage/unstage selected file
  - `c` - Focus commit message input
  - `Ctrl+Enter` - Create commit
  - `r` or `F5` - Refresh
  - `Ctrl+f` - Focus search
  - `?` - Show help dialog
- [ ] Create help dialog listing all shortcuts
- [ ] Show shortcuts as tooltips on buttons

**Success Criteria:**
- Keyboard navigation works smoothly
- Shortcuts don't conflict with egui defaults
- Help dialog shows all available shortcuts

---

#### Task 3.3: Performance Optimizations
**Priority**: 🔶 High
**Estimated Time**: 6-8 hours

**Optimizations:**
- [ ] Implement virtual scrolling for commit list (only render visible commits)
- [ ] Cache computed diffs in memory (LRU cache)
- [ ] Lazy-load diffs only when commit is selected
- [ ] Add pagination for commit history (load more button)
- [ ] Debounce file system watches for working dir
- [ ] Profile and optimize hot paths

**Testing:**
- [ ] Test with repository containing 10,000+ commits
- [ ] Test with large files (1MB+ diffs)
- [ ] Measure and document performance metrics
- [ ] Ensure UI stays responsive during heavy operations

**Success Criteria:**
- Scrolling through 10k commits is smooth (60fps)
- Diff loading is fast (<100ms)
- Memory usage stays reasonable (<200MB for large repos)
- No UI freezing under any operation

---

#### Task 3.4: File History View
**Priority**: 🟡 Medium
**Estimated Time**: 3-4 hours

**Git Layer (crates/git/):**
- [ ] Implement `get_file_history(path: &Path) -> Result<Vec<Commit>>`
- [ ] Use gix log with pathspec filter

**UI Layer (crates/ui_egui/):**
- [ ] Add "History" button in file list (context menu)
- [ ] Show commits that modified the selected file
- [ ] Click commit to see file state at that point
- [ ] Add "Back" button to return to main view

**Success Criteria:**
- Can view all commits that modified a specific file
- Can see file content at any point in history

---

### Sprint 4: Testing & Documentation (Week 4) - 8-10 hours

Focus: Quality assurance and documentation

#### Task 4.1: Comprehensive Testing
**Priority**: ⭐ Critical
**Estimated Time**: 4-6 hours

**Tests to Add:**
- [ ] Integration tests for staging operations
- [ ] Integration tests for commit creation
- [ ] UI interaction tests (if possible with egui)
- [ ] Test edge cases (empty commits, merge commits, etc.)
- [ ] Test error scenarios (permissions, conflicts, etc.)
- [ ] Add benchmark tests for performance-critical paths

**Target Coverage:**
- Git layer: 80%+
- App layer: 85%+
- Total tests: 50+ (currently 26)

---

#### Task 4.2: Documentation Updates
**Priority**: ⭐ Critical
**Estimated Time**: 4-4 hours

**Documentation:**
- [ ] Update README.md with Phase 2 features
- [ ] Create USER_GUIDE.md with screenshots
- [ ] Update docs/architecture.md with new components
- [ ] Document keyboard shortcuts
- [ ] Add API documentation for new Git operations
- [ ] Create PHASE2_COMPLETE.md with summary
- [ ] Update QUICKSTART.md

**Screenshots:**
- [ ] Working directory view with staged/unstaged files
- [ ] Full diff view with line numbers
- [ ] Commit creation dialog
- [ ] Commit graph visualization

---

## Implementation Order Summary

### Week 1: Foundation
1. Working directory status (Git + App + UI) - 6-8h
2. Full diff implementation (Git + App + UI) - 10-12h
3. Working directory UI panel - 6-8h

### Week 2: Core Workflow
4. Staging operations - 8-10h
5. Commit creation - 6-8h
6. Commit graph visualization - 8-10h

### Week 3: Enhancement
7. Search and filter - 4-6h
8. Keyboard shortcuts - 3-4h
9. Performance optimizations - 6-8h
10. File history view - 3-4h

### Week 4: Quality
11. Comprehensive testing - 4-6h
12. Documentation updates - 4-4h

---

## Success Criteria for Phase 2

### Functional Requirements
- ✅ Can view all uncommitted changes in working directory
- ✅ Can see full line-by-line diffs for any file
- ✅ Can stage and unstage individual files
- ✅ Can create commits with multi-line messages
- ✅ Can view commit history with graph visualization
- ✅ Can search and filter commits
- ✅ Keyboard shortcuts work for common operations

### Quality Requirements
- ✅ All tests pass (target: 50+ tests)
- ✅ No performance regression with large repositories
- ✅ UI remains responsive during all operations
- ✅ Complete documentation with screenshots
- ✅ Zero clippy warnings

### Performance Targets
- ✅ Load 10,000 commits: <2 seconds
- ✅ Display diff: <100ms
- ✅ Stage/unstage file: <50ms
- ✅ Commit creation: <200ms
- ✅ UI frame rate: 60fps maintained

---

## Risk Assessment

### High Risk
- **Diff Performance**: Large diffs may be slow
  - *Mitigation*: Implement streaming, virtual scrolling

- **Git Edge Cases**: Merge conflicts, submodules, etc.
  - *Mitigation*: Comprehensive testing, graceful error handling

### Medium Risk
- **Graph Complexity**: Graph layout algorithm may be complex
  - *Mitigation*: Start with simple implementation, iterate

- **Memory Usage**: Caching diffs may use too much memory
  - *Mitigation*: Implement LRU cache with size limits

### Low Risk
- **UI Responsiveness**: Operations may block UI
  - *Mitigation*: Already using async pattern from Phase 1

---

## Dependencies

### External Crates (New)
- None required (using existing gix capabilities)

### Optional Enhancements
- `syntect` - Syntax highlighting for diffs (future)
- `similar` - Better diff algorithms (if gix insufficient)

---

## Milestones

### M1: Working Directory (End of Week 1)
- Can view uncommitted changes
- Can see full diffs
- Demo: Show working directory panel with diffs

### M2: Complete Workflow (End of Week 2)
- Can stage/unstage files
- Can create commits
- Demo: Full workflow from edit to commit

### M3: Enhanced UX (End of Week 3)
- Visual commit graph
- Search and shortcuts
- Demo: Navigate large repo efficiently

### M4: Production Ready (End of Week 4)
- All tests passing
- Documentation complete
- Demo: Ready for daily use

---

## Post-Phase 2 Considerations

After Phase 2, the application will have a complete local Git workflow. Phase 3 could focus on:
- Branch management (create, delete, merge)
- Remote operations (fetch, pull, push)
- Conflict resolution
- Stash operations
- Advanced rebase support

---

**Document Version**: 1.0
**Created**: 2026-02-07
**Status**: Ready for Implementation
**Total Estimated Effort**: 70-90 hours
