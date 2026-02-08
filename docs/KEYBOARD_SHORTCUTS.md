# Keyboard Shortcuts Reference

Complete reference for all keyboard shortcuts in CrabOnTree. Press `?` in the application to view this help.

## Quick Reference Card

```
┌─────────────────────────────────────────────────────────────┐
│                    CRABONTREE SHORTCUTS                     │
├─────────────────────────────────────────────────────────────┤
│ GLOBAL                                                      │
│  1, 2, 3      Switch to panel (Working Dir/Message/History)│
│  Tab          Cycle through panels                          │
│  r            Refresh repository status                     │
│  ?            Show/hide this help                           │
├─────────────────────────────────────────────────────────────┤
│ NAVIGATION (File List / Commit List)                       │
│  ↑ / k        Move up (previous item)                       │
│  ↓ / j        Move down (next item)                         │
│  g g          Jump to top (vim-style, press g twice)        │
│  G            Jump to bottom (vim-style)                    │
│  Home         Jump to first item                            │
│  End          Jump to last item                             │
├─────────────────────────────────────────────────────────────┤
│ ACTIONS                                                     │
│  Space        Toggle file staging (in Working Directory)    │
│  Enter        View commit details (in Commit History)       │
│  a            Stage all files                               │
│  u            Unstage all files                             │
│  c            Focus commit message input                    │
│  Ctrl+Enter   Create commit (when message focused)          │
├─────────────────────────────────────────────────────────────┤
│ SEARCH                                                      │
│  /            Focus file search (Working Directory)         │
│  Ctrl+F       Focus commit search (Commit History)          │
│  Esc          Clear search / blur input                     │
└─────────────────────────────────────────────────────────────┘
```

---

## Detailed Reference

### Global Shortcuts

These shortcuts work from anywhere in the application, regardless of which panel is focused.

| Shortcut | Action | Description |
|----------|--------|-------------|
| `1` | Focus Working Directory | Jump directly to the Working Directory panel to view and stage files |
| `2` | Focus Commit Message | Jump directly to the Commit Message panel to write your commit |
| `3` | Focus Commit History | Jump directly to the Commit History panel to browse commits |
| `Tab` | Cycle Panels | Move through panels in order: Working Dir → Message → History → Working Dir |
| `r` | Refresh | Reload repository status from disk (working dir, branches, HEAD) |
| `?` | Toggle Help | Show or hide the keyboard shortcuts help dialog |

**Usage Examples**:
- Press `1` to quickly return to file staging after writing a commit message
- Press `3` to immediately view your commit after creating it
- Press `r` after making changes in your editor to see them in CrabOnTree

---

### Navigation Shortcuts

These shortcuts work within lists (file list in Working Directory, commit list in Commit History).

#### Arrow Key Navigation

| Shortcut | Action | Description |
|----------|--------|-------------|
| `↑` | Previous Item | Move focus to the previous item in the list |
| `↓` | Next Item | Move focus to the next item in the list |
| `Home` | Jump to Top | Move focus to the first item in the list |
| `End` | Jump to Bottom | Move focus to the last item in the list |

**Visual Feedback**: The focused item shows a `>` marker at the beginning of the line.

#### Vim-Style Navigation

For users who prefer vim keybindings:

| Shortcut | Action | Description |
|----------|--------|-------------|
| `j` | Previous Item | Same as `↑`, move to previous item |
| `k` | Next Item | Same as `↓`, move to next item |
| `g g` | Jump to Top | Press `g` twice quickly to jump to the first item |
| `G` | Jump to Bottom | Jump to the last item in the list |

**Note**: The `gg` shortcut requires pressing `g` twice in quick succession (within ~500ms).

#### Context-Specific Navigation

**In Working Directory**:
- Navigation moves through the file list
- Focused file shows `>` marker
- Files show staging status: `[U]` unstaged, `[S]` staged, `[?]` untracked

**In Commit History**:
- Navigation moves through commit list
- Focused commit shows `>` marker
- Selected commit (diff shown) is in bold

---

### Action Shortcuts

#### Working Directory Actions

| Shortcut | Action | Description | Context |
|----------|--------|-------------|---------|
| `Space` | Toggle Staging | Stage file if unstaged, unstage if staged | Working Directory (file focused) |
| `a` | Stage All | Stage all modified and untracked files | Working Directory |
| `u` | Unstage All | Unstage all currently staged files | Working Directory |

**Staging Workflow**:
```
1. Navigate to file with j/k or ↑/↓
2. Press Space to stage it
3. Repeat for other files
4. Press 'a' to stage remaining files (if desired)
```

**Unstaging Workflow**:
```
1. Navigate to staged file (marked [S])
2. Press Space to unstage it
3. Or press 'u' to unstage all files at once
```

#### Commit Message Actions

| Shortcut | Action | Description | Context |
|----------|--------|-------------|---------|
| `c` | Focus Message | Jump to commit message input and place cursor | Anywhere |
| `Ctrl+Enter` | Create Commit | Create a commit with the current message and staged files | Commit Message (input focused) |
| `Esc` | Blur Message | Remove focus from commit message input | Commit Message (input focused) |

**Commit Workflow**:
```
1. Stage files (Space or 'a')
2. Press 'c' to focus commit message
3. Type your commit message
4. Press Ctrl+Enter to create commit
5. Message clears and UI refreshes automatically
```

**Commit Button Requirements**:
- At least one file must be staged
- Message must not be empty (at least 1 non-whitespace character)
- If requirements not met, `Ctrl+Enter` has no effect

#### Commit History Actions

| Shortcut | Action | Description | Context |
|----------|--------|-------------|---------|
| `Enter` | View Commit Details | Load and display the diff for the focused commit | Commit History (commit focused) |

**History Browsing Workflow**:
```
1. Press '3' to focus commit history
2. Navigate with j/k to browse commits
3. Press Enter on interesting commit to view diff
4. Diff displays below commit list
5. Continue navigating to view other commits
```

---

### Search Shortcuts

#### File Search (Working Directory)

| Shortcut | Action | Description |
|----------|--------|-------------|
| `/` | Focus File Search | Place cursor in file search box |
| `Esc` | Clear/Blur Search | Clear search text and exit search mode |

**Search Behavior**:
- **Real-time filtering**: Files filter as you type
- **Case-insensitive**: Matches regardless of case
- **Substring matching**: Matches anywhere in file path
- **Count displayed**: Shows "N files (M hidden by filter)"

**Search Examples**:
- `"src"` - Shows all files in src/ directory
- `"test"` - Shows all test files
- `".rs"` - Shows all Rust files
- `"auth"` - Shows all files with "auth" in path

**Workflow**:
```
1. Press '/' to focus search
2. Type search term
3. List filters instantly
4. Navigate with j/k through filtered results
5. Press Esc to clear and see all files again
```

#### Commit Search (Commit History)

| Shortcut | Action | Description |
|----------|--------|-------------|
| `Ctrl+F` | Focus Commit Search | Place cursor in commit search box |
| `Esc` | Clear/Blur Search | Clear search text and exit search mode |

**Search Behavior**:
- **Multi-field search**: Searches commit message, author name, and commit hash
- **Real-time filtering**: Commits filter as you type
- **Case-insensitive**: Matches regardless of case
- **Substring matching**: Matches anywhere in searched fields
- **Count displayed**: Shows "N commits (M hidden by filter)"

**Search Examples**:
- `"bug fix"` - Find commits mentioning bug fixes
- `"alice"` - Find commits by author Alice
- `"feature"` - Find feature commits
- `"a3f2b1"` - Find commit by partial hash

**Workflow**:
```
1. Press Ctrl+F to focus search
2. Type search term
3. Commit list filters instantly
4. Navigate with j/k through filtered results
5. Press Enter to view commit diff
6. Press Esc to clear and see all commits again
```

---

## Shortcut Combinations

### Fast Commit Workflow

```
a                  Stage all files
c                  Focus commit message
[type message]     Write your commit message
Ctrl+Enter         Create commit
3                  View commit in history
```

**Time**: ~5-10 seconds for a complete commit!

### Selective Staging Workflow

```
1                  Focus working directory
/                  Focus search
[type filter]      Filter to specific files
j j j              Navigate to desired file
Space              Stage it
Space              Stage another
Esc                Clear search
a                  Stage remaining files
c                  Write commit message
Ctrl+Enter         Commit
```

### Review Workflow

```
3                  Focus commit history
j j j              Navigate through recent commits
Enter              View commit details
j                  Next commit
Enter              View its details
/                  Search for specific commit
[type search]      Find it
Enter              View details
Esc                Clear search
```

### Search and Stage Workflow

```
1                  Focus working directory
/                  Search files
test               Find test files
Space Space Space  Stage focused test files
Esc                Clear search
/                  Search again
doc                Find documentation
Space Space        Stage doc files
Esc                Clear search
c                  Write commit message
Ctrl+Enter         Commit
```

---

## Keyboard Navigation Tips

### 1. Stay on Home Row

Use vim-style keys (`j`, `k`, `gg`, `G`) to keep your fingers on the home row for faster navigation.

### 2. Use Number Keys for Panel Switching

Faster than clicking or tabbing:
- `1` for files
- `2` for message
- `3` for history

### 3. Combine Search with Navigation

1. Filter with search (`/` or `Ctrl+F`)
2. Navigate filtered results with `j`/`k`
3. Take action (`Space` or `Enter`)
4. Clear search with `Esc`

### 4. Focus vs. Selection

**Focus** (navigation):
- Shows `>` marker
- Doesn't load data
- Fast to move through list

**Selection** (action):
- For commits: Press `Enter` to select and load diff
- For files: Press `Space` to stage/unstage
- Allows browsing without loading costs

### 5. Quick Peek Workflow

Browse commits quickly without loading each diff:
```
3            Focus history
j j j j      Navigate through commits (just look at messages)
Enter        Load diff only for interesting commits
```

---

## Accessibility Features

### Visual Indicators

- **`>` marker**: Shows keyboard focus position
- **Bold text**: Shows selected commit (diff displayed)
- **Color coding**: Staged [S] vs Unstaged [U] files
- **Status badges**: [U], [S], [?] for file staging status

### Keyboard-Only Operation

CrabOnTree is fully operable without a mouse:
- All features accessible via keyboard
- No hidden mouse-only actions
- Logical tab order
- Clear focus indicators

### Screen Reader Support

While not optimized for screen readers yet, basic structure is accessible:
- Clear text labels
- Logical layout structure
- Text-based status indicators

---

## Platform-Specific Notes

### macOS

- `Ctrl` means `⌘ Command` key on macOS
- `Ctrl+Enter` is `⌘+Return`
- `Ctrl+F` is `⌘+F`
- All other shortcuts work as documented

### Windows

- All shortcuts work as documented
- `Ctrl` is the Control key

### Linux

- All shortcuts work as documented
- `Ctrl` is the Control key

---

## Customization

**Note**: Keyboard shortcuts are currently not customizable. Custom keybinding support may be added in a future release.

**Workaround**: If you need different shortcuts, you can:
1. Fork the repository
2. Edit `crates/ui_egui/src/main.rs`
3. Modify the keyboard handling code
4. Build your custom version

---

## Troubleshooting

### Shortcuts Not Working

**Problem**: Pressing keys doesn't trigger expected actions.

**Solutions**:

1. **Check panel focus**: Some shortcuts require specific panel focus
   - Press `1`, `2`, or `3` to focus correct panel
   - Global shortcuts (`r`, `?`) work from anywhere

2. **Exit text input mode**: If cursor is in a text box:
   - Press `Esc` to exit input mode
   - Then try the shortcut again

3. **Close help dialog**: If help is open:
   - Press `?` or click X to close
   - Then try the shortcut again

4. **Check conflicts**: Some OS shortcuts may conflict:
   - **macOS**: System shortcuts override app shortcuts
   - **Linux**: Window manager shortcuts may conflict
   - **Windows**: System shortcuts generally don't conflict

5. **Restart application**: If shortcuts stop working unexpectedly

### Common Conflicts

| Shortcut | Conflict | Solution |
|----------|----------|----------|
| `Ctrl+F` | Browser find (if running in browser) | Use native app instead |
| `/` | Quick find (Firefox) | Use native app instead |
| `Space` | Page down (some browsers) | Use native app instead |

---

## Learning the Shortcuts

### Beginner Strategy

Start with these essential shortcuts:

**Week 1**: Basic navigation
- `↑` / `↓` for navigation
- `Space` for staging
- Click buttons for commit

**Week 2**: Add efficiency
- `a` for stage all
- `c` for focus message
- `Ctrl+Enter` for commit

**Week 3**: Add power features
- `/` for file search
- `Ctrl+F` for commit search
- `3` to view history

**Week 4**: Master vim-style
- `j` / `k` for navigation
- `gg` / `G` for jumping
- `1`, `2`, `3` for panel switching

### Advanced Strategy

Once comfortable with basics, add:

1. **Vim-style navigation** (`j`/`k`, `gg`/`G`)
2. **Search workflows** (filter → navigate → action)
3. **Panel switching** (`1`/`2`/`3` instead of clicking)
4. **Focus management** (navigate without loading)

### Practice Exercise

Try this complete workflow without the mouse:

```
1. Press 'r' to refresh
2. Press '1' to focus working directory
3. Press '/' and search for ".rs" files
4. Press 'j' to navigate
5. Press 'Space' to stage a file
6. Press 'Esc' to clear search
7. Press 'a' to stage remaining
8. Press 'c' to focus message
9. Type "Practice commit"
10. Press 'Ctrl+Enter' to commit
11. Press '3' to view in history
12. Press 'j' to navigate commits
13. Press 'Enter' to view diff
```

If you can complete this smoothly, you've mastered the keyboard interface!

---

## Cheat Sheet (Printable)

```
╔══════════════════════════════════════════════════════════════╗
║              CRABONTREE KEYBOARD SHORTCUTS                   ║
╠══════════════════════════════════════════════════════════════╣
║ PANELS:  1=Files  2=Message  3=History  Tab=Cycle           ║
║ NAV:     ↑↓jk=Move  gg=Top  G=Bottom  Home/End=Edges        ║
║ ACTION:  Space=Stage  Enter=View  a=StageAll  u=UnstageAll  ║
║ COMMIT:  c=Focus  Ctrl+Enter=Commit                          ║
║ SEARCH:  /=Files  Ctrl+F=Commits  Esc=Clear                  ║
║ GLOBAL:  r=Refresh  ?=Help                                   ║
╚══════════════════════════════════════════════════════════════╝
```

---

## Additional Resources

- **User Guide**: Complete usage instructions
- **Architecture**: Understanding the design helps know what shortcuts do
- **Contributing**: Add new shortcuts by modifying the UI layer

---

## Feedback

Have suggestions for new shortcuts or changes to existing ones?

- **GitHub Issues**: Report shortcut problems
- **GitHub Discussions**: Suggest new shortcuts
- **Discord**: Chat about keyboard workflows (coming soon)

---

**Master the keyboard, master your Git workflow! ⌨️🦀🌳**
