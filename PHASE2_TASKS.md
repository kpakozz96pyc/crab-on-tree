# Phase 2: Task Checklist

## Sprint 1: Foundation (Week 1) ⏱️ 24-30h

### 1.1 Working Directory Status ⭐ Critical
**Time**: 6-8h

#### Git Layer
- [ ] Create `FileStatus` enum
- [ ] Create `WorkingDirFile` struct
- [ ] Implement `get_working_dir_status()`
- [ ] Add tests

#### App Layer
- [ ] Add working dir files to `RepoState`
- [ ] Add messages for loading status
- [ ] Add job and effect
- [ ] Implement handlers

#### UI Layer
- [ ] None (UI in task 1.3)

---

### 1.2 Full Diff (Line-by-Line) ⭐ Critical
**Time**: 10-12h

#### Git Layer
- [ ] Create `LineDiff` struct
- [ ] Create `FileDiffDetail` struct
- [ ] Implement `get_file_diff()`
- [ ] Implement `get_working_dir_file_diff()`
- [ ] Add tests

#### App Layer
- [ ] Add file diff to `RepoState`
- [ ] Add messages for loading diff
- [ ] Add job and effect
- [ ] Implement handlers

#### UI Layer
- [ ] Create `render_line_diff()` function
- [ ] Implement unified diff view
- [ ] Color-code additions/deletions
- [ ] Show line numbers
- [ ] Make scrollable

---

### 1.3 Working Directory UI ⭐ Critical
**Time**: 6-8h

#### UI Layer
- [ ] Create "Changes" tab/section
- [ ] Implement `render_working_dir_status()`
- [ ] Show staged vs unstaged sections
- [ ] Add file status icons
- [ ] Make files clickable to view diff
- [ ] Add refresh button

---

## Sprint 2: Core Workflow (Week 2) ⏱️ 22-28h

### 2.1 Staging Operations ⭐ Critical
**Time**: 8-10h

#### Git Layer
- [ ] Implement `stage_file()`
- [ ] Implement `unstage_file()`
- [ ] Implement `stage_all()`
- [ ] Implement `unstage_all()`
- [ ] Add tests

#### App Layer
- [ ] Add staging messages
- [ ] Add staging jobs and effects
- [ ] Implement handlers
- [ ] Auto-refresh after staging

#### UI Layer
- [ ] Add Stage/Unstage buttons
- [ ] Add Stage All / Unstage All buttons
- [ ] Show loading indicators

---

### 2.2 Commit Creation ⭐ Critical
**Time**: 6-8h

#### Git Layer
- [ ] Implement `create_commit()`
- [ ] Support multi-line messages
- [ ] Add tests

#### App Layer
- [ ] Add commit message to state
- [ ] Add commit creation messages
- [ ] Add job and effect
- [ ] Clear message after commit

#### UI Layer
- [ ] Create commit message input (multi-line)
- [ ] Add Commit button
- [ ] Enable only with staged files
- [ ] Show success notification
- [ ] Clear input after commit

---

### 2.3 Commit Graph Visualization 🔶 High
**Time**: 8-10h

#### Git Layer
- [ ] Add branch refs to `Commit`
- [ ] Add merge flag to `Commit`
- [ ] Compute parent relationships

#### UI Layer
- [ ] Implement graph drawing with egui shapes
- [ ] Draw parent-child lines
- [ ] Use different colors for branches
- [ ] Show branch/tag labels
- [ ] Make compact and readable

---

## Sprint 3: Enhancement (Week 3) ⏱️ 16-22h

### 3.1 Search and Filter 🔶 High
**Time**: 4-6h

#### UI Layer
- [ ] Add search input field
- [ ] Filter by message (case-insensitive)
- [ ] Add author filter dropdown
- [ ] Add date range filter
- [ ] Show "X of Y commits"
- [ ] Add clear filters button

---

### 3.2 Keyboard Shortcuts 🔶 High
**Time**: 3-4h

#### UI Layer
- [ ] Implement keyboard event handling
- [ ] Add navigation shortcuts (j/k, arrows)
- [ ] Add selection shortcut (Enter)
- [ ] Add staging shortcut (Space)
- [ ] Add commit shortcut (c, Ctrl+Enter)
- [ ] Add refresh shortcut (r, F5)
- [ ] Add search shortcut (Ctrl+f)
- [ ] Create help dialog (?)
- [ ] Show tooltips on buttons

---

### 3.3 Performance Optimizations 🔶 High
**Time**: 6-8h

#### Implementation
- [ ] Virtual scrolling for commits
- [ ] Cache diffs (LRU)
- [ ] Lazy-load diffs
- [ ] Add pagination
- [ ] Debounce file system watches
- [ ] Profile hot paths

#### Testing
- [ ] Test with 10k+ commits
- [ ] Test with large files (1MB+ diffs)
- [ ] Measure performance metrics
- [ ] Ensure 60fps UI

---

### 3.4 File History View 🟡 Medium
**Time**: 3-4h

#### Git Layer
- [ ] Implement `get_file_history()`

#### UI Layer
- [ ] Add History button/menu
- [ ] Show commits for file
- [ ] Click to see file state
- [ ] Add back button

---

## Sprint 4: Quality (Week 4) ⏱️ 8-10h

### 4.1 Testing ⭐ Critical
**Time**: 4-6h

- [ ] Integration tests for staging
- [ ] Integration tests for commits
- [ ] UI interaction tests
- [ ] Test edge cases
- [ ] Test error scenarios
- [ ] Add benchmarks
- [ ] Achieve 50+ total tests

---

### 4.2 Documentation ⭐ Critical
**Time**: 4-4h

- [ ] Update README with Phase 2 features
- [ ] Create USER_GUIDE.md
- [ ] Update architecture docs
- [ ] Document keyboard shortcuts
- [ ] Add API docs
- [ ] Create PHASE2_COMPLETE.md
- [ ] Update QUICKSTART.md
- [ ] Add screenshots

---

## Progress Tracking

### Week 1
- [ ] Sprint 1 Complete (Tasks 1.1-1.3)

### Week 2
- [ ] Sprint 2 Complete (Tasks 2.1-2.3)

### Week 3
- [ ] Sprint 3 Complete (Tasks 3.1-3.4)

### Week 4
- [ ] Sprint 4 Complete (Tasks 4.1-4.2)
- [ ] Phase 2 Complete ✅

---

## Quick Stats

- **Total Tasks**: 12 major tasks
- **Estimated Time**: 70-90 hours
- **Target Duration**: 4 weeks
- **Critical Tasks**: 6
- **High Priority**: 4
- **Medium Priority**: 2

---

## Daily Goals (Suggested)

Assuming 4-5 hour work days:

**Week 1:**
- Day 1-2: Task 1.1 (Working Dir Status)
- Day 3-4: Task 1.2 (Full Diff)
- Day 5: Task 1.3 (Working Dir UI)

**Week 2:**
- Day 1-2: Task 2.1 (Staging)
- Day 3: Task 2.2 (Commits)
- Day 4-5: Task 2.3 (Graph)

**Week 3:**
- Day 1: Task 3.1 (Search)
- Day 2: Task 3.2 (Shortcuts)
- Day 3-4: Task 3.3 (Performance)
- Day 5: Task 3.4 (File History)

**Week 4:**
- Day 1-2: Task 4.1 (Testing)
- Day 3-4: Task 4.2 (Documentation)
- Day 5: Final review and release

---

**Last Updated**: 2026-02-07
**Status**: Ready to start
