# Task 3.2: Enhanced Keyboard Shortcuts - Implementation Plan

**Estimated Time**: 3-4 hours
**Priority**: MEDIUM-HIGH - Great for power users
**Status**: Starting implementation

## Overview

Add comprehensive keyboard navigation to enable users to work efficiently without using the mouse. This is especially valuable for developers who prefer keyboard-driven workflows.

## Features to Implement

### 1. Commit Navigation (1-2 hours)

**Keyboard Shortcuts**:
- `↑` / `k` - Select previous commit
- `↓` / `j` - Select next commit
- `Enter` - View selected commit details
- `Home` - Jump to first commit
- `End` - Jump to last commit

**Implementation**:
- Track which commit is "focused" (different from selected)
- Up/Down arrows move focus
- Enter selects the focused commit
- Visual indicator for focused vs selected
- Works with filtered commit list

### 2. File Staging Shortcuts (1 hour)

**Keyboard Shortcuts**:
- `Space` - Toggle staging for focused file
- `a` - Stage all files
- `u` - Unstage all files
- `↑` / `k` - Focus previous file
- `↓` / `j` - Focus next file

**Implementation**:
- Track which file is "focused" in working directory
- Space bar toggles staging for focused file
- Up/Down arrows move focus
- Visual indicator for focused file
- Works with filtered file list

### 3. Panel Navigation (30 min)

**Keyboard Shortcuts**:
- `1` - Focus working directory panel
- `2` - Focus commit message panel
- `3` - Focus commit history panel
- `Tab` - Cycle through panels

**Implementation**:
- Track active panel
- Number keys jump to specific panels
- Tab cycles through in order
- Visual indicator for active panel (subtle border?)

### 4. General Navigation (30 min)

**Keyboard Shortcuts**:
- `r` - Refresh repository
- `?` - Show keyboard shortcuts help
- `g g` - Go to top (vim-style)
- `G` - Go to bottom (vim-style)

**Implementation**:
- Simple message triggers for actions
- Help dialog showing all shortcuts
- Vim-style navigation for power users

## Implementation Steps

### Phase 1: Commit List Navigation

1. Add focused_commit_index to UI state
2. Handle Up/Down/j/k arrow keys
3. Implement Home/End keys
4. Add visual focus indicator
5. Handle Enter to select focused commit
6. Test with keyboard-only navigation

### Phase 2: File List Navigation

1. Add focused_file_index to UI state
2. Handle Up/Down/j/k arrow keys
3. Implement Space to toggle staging
4. Add visual focus indicator
5. Implement 'a' and 'u' shortcuts
6. Test with keyboard-only navigation

### Phase 3: Panel Navigation

1. Add active_panel enum to UI state
2. Implement number key navigation (1, 2, 3)
3. Implement Tab cycling
4. Add visual panel indicators
5. Test panel switching

### Phase 4: General & Help

1. Implement 'r' for refresh
2. Create help dialog component
3. Implement '?' to show help
4. Add vim-style g g and G
5. Test all shortcuts together

## Success Criteria

- ✅ Can navigate commits with arrow keys
- ✅ Can select commits with Enter
- ✅ Can navigate files with arrow keys
- ✅ Can stage/unstage with Space
- ✅ Can switch panels with number keys
- ✅ Can cycle panels with Tab
- ✅ Vim-style navigation works (j/k)
- ✅ Help dialog shows all shortcuts
- ✅ All shortcuts work together harmoniously
- ✅ Visual feedback for focused elements

## Technical Details

**UI State Changes**:
```rust
struct CrabOnTreeApp {
    // ... existing fields ...
    focused_commit_index: Option<usize>,
    focused_file_index: Option<usize>,
    active_panel: ActivePanel,
    show_shortcuts_help: bool,
}

enum ActivePanel {
    WorkingDirectory,
    CommitMessage,
    CommitHistory,
}
```

**Performance**:
- All shortcuts handled locally (no I/O)
- Should be instant response
- No impact on existing functionality

## UI Design

### Focus Indicators

```
Working Directory (focused file has subtle background):
  [+] [U] src/main.rs           <- Regular
> [+] [U] src/lib.rs             <- Focused (with arrow or bg color)
  [−] [S] README.md

Commit History (focused commit has border):
┌─────────────────────────────┐
│ abc1234 - Fix parser        │ <- Focused (with border)
└─────────────────────────────┘
  def5678 - Add feature
  ghi9012 - Update docs
```

### Keyboard Shortcuts Help Dialog

```
┌──────────────────────────────────────────┐
│          Keyboard Shortcuts              │
├──────────────────────────────────────────┤
│ Navigation                               │
│  ↑/k        Previous item                │
│  ↓/j        Next item                    │
│  Enter      Select/Open                  │
│  Space      Toggle (files)               │
│                                          │
│ Search                                   │
│  /          Focus file search            │
│  Ctrl+F     Focus commit search          │
│  Esc        Clear search                 │
│                                          │
│ Panels                                   │
│  1          Working directory            │
│  2          Commit message               │
│  3          Commit history               │
│  Tab        Cycle panels                 │
│                                          │
│ Actions                                  │
│  c          Focus commit message         │
│  Ctrl+Enter Create commit                │
│  a          Stage all                    │
│  u          Unstage all                  │
│  r          Refresh                      │
│                                          │
│  ?          Show this help               │
└──────────────────────────────────────────┘
```

## Keyboard Shortcuts Summary

### Navigation
- `↑` / `k` - Previous (commit/file)
- `↓` / `j` - Next (commit/file)
- `Enter` - Select/Open
- `Home` - First item
- `End` - Last item
- `g g` - Go to top
- `G` - Go to bottom

### Actions
- `Space` - Toggle staging (when file focused)
- `a` - Stage all files
- `u` - Unstage all files
- `c` - Focus commit message
- `Ctrl+Enter` - Create commit
- `r` - Refresh repository

### Panels
- `1` - Working directory
- `2` - Commit message
- `3` - Commit history
- `Tab` - Cycle panels

### Search
- `/` - Focus file search
- `Ctrl+F` - Focus commit search
- `Esc` - Clear/blur

### Help
- `?` - Show keyboard shortcuts

## Edge Cases to Handle

1. **No commits loaded**: Arrow keys should do nothing gracefully
2. **No files in working dir**: Arrow keys should do nothing gracefully
3. **Filtered lists**: Navigation works on filtered items only
4. **Focus vs Selection**: Make clear distinction
5. **Multiple shortcuts pressed**: Handle gracefully
6. **Text input focused**: Don't capture shortcuts meant for typing

## Implementation Notes

**Focus Management**:
- Need to track focus separately from selection
- Focus is visual only (highlighted but not selected)
- Selection is actual state change (loads commit diff)

**Event Handling Priority**:
1. If any text input is focused, let it handle keys (except Esc)
2. Otherwise, check for global shortcuts
3. Then check for panel-specific shortcuts

**Vim-Style Navigation**:
- `j`/`k` for up/down (same as arrows)
- `g g` for top (press g twice)
- `G` for bottom
- Optional: `h`/`l` for left/right (not needed in this UI)

---

**Ready to implement!**
