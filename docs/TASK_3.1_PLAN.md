# Task 3.1: Search and Filter - Implementation Plan

**Estimated Time**: 4-6 hours
**Priority**: HIGH - Major UX improvement
**Status**: Starting implementation

## Overview

Add search and filter capabilities to improve user experience when working with large numbers of files and commits.

## Features to Implement

### 1. Working Directory Search (2-3 hours)

**UI Changes**:
- Add search input box above working directory file list
- Filter files by path/name as user types
- Show count: "Showing X of Y files"
- Clear button to reset filter

**Implementation**:
- Add `working_dir_search: String` to UI state
- Filter files in render function based on search term
- Case-insensitive substring matching
- Keyboard shortcut: `/` to focus search box

### 2. Commit History Search (2-3 hours)

**UI Changes**:
- Add search input box above commit history
- Filter commits by:
  - Commit message (primary)
  - Author name
  - Commit hash
- Show count: "Showing X of Y commits"
- Clear button to reset filter

**Implementation**:
- Add `commit_search: String` to UI state
- Filter commits in render function
- Multi-field search (message, author, hash)
- Case-insensitive matching
- Keyboard shortcut: `Ctrl+F` to focus search

### 3. File Status Filter (1 hour)

**UI Changes**:
- Add filter buttons for file status:
  - "All", "Staged", "Unstaged", "Modified", "Untracked"
- Toggle filters on/off
- Combine with text search

**Implementation**:
- Add `file_status_filter: FileStatusFilter` enum to UI state
- Apply status filter before text search
- Visual indication of active filter

## Implementation Steps

### Phase 1: Working Directory Search

1. Add UI state field for search query
2. Add search input box to working directory panel
3. Implement filter logic
4. Add clear button
5. Add keyboard shortcut (/)
6. Test with large file lists

### Phase 2: Commit History Search

1. Add UI state field for commit search query
2. Add search input box to commit history panel
3. Implement multi-field search (message, author, hash)
4. Add clear button
5. Add keyboard shortcut (Ctrl+F)
6. Test with long commit history

### Phase 3: Status Filters

1. Add filter enum and state
2. Add filter button row
3. Implement filter logic
4. Combine with text search
5. Test all combinations

## Success Criteria

- ✅ Can search files by name/path
- ✅ Can search commits by message/author/hash
- ✅ Can filter files by status
- ✅ Search is fast and responsive
- ✅ Keyboard shortcuts work
- ✅ Clear buttons work correctly
- ✅ Filtered counts display correctly

## Technical Details

**No Backend Changes Required**:
- All filtering done in UI layer
- No new messages/jobs/effects needed
- Pure UI enhancement

**Performance**:
- Filtering happens on already-loaded data
- Should be instant even with 1000+ items
- No need for debouncing (data is local)

## UI Design

```
┌─────────────────────────────────────────────┐
│ 🔨 Working Directory                        │
│ ┌─────────────────────────────────────────┐ │
│ │ 🔍 Search files... [X]                  │ │
│ │ [All] [Staged] [Unstaged] [Modified]   │ │
│ └─────────────────────────────────────────┘ │
│ Showing 3 of 12 files                       │
│                                             │
│ [+] [U] src/main.rs                         │
│ [−] [S] src/lib.rs                          │
│ [+] [U] README.md                           │
└─────────────────────────────────────────────┘

┌─────────────────────────────────────────────┐
│ Commit History                               │
│ ┌─────────────────────────────────────────┐ │
│ │ 🔍 Search commits... [X]                │ │
│ └─────────────────────────────────────────┘ │
│ Showing 5 of 100 commits                    │
│                                             │
│ abc1234 Fix bug in parser                   │
│ def5678 Add new feature                     │
└─────────────────────────────────────────────┘
```

## Keyboard Shortcuts

- `/` - Focus working directory search
- `Ctrl+F` - Focus commit history search
- `Esc` - Clear search and blur input

---

**Ready to implement!**
