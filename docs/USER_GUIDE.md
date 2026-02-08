# CrabOnTree User Guide

Welcome to CrabOnTree! This guide will help you get started and make the most of your Git workflow.

## Table of Contents

1. [Getting Started](#getting-started)
2. [Basic Workflows](#basic-workflows)
3. [Keyboard Shortcuts Reference](#keyboard-shortcuts-reference)
4. [Tips and Tricks](#tips-and-tricks)
5. [Troubleshooting](#troubleshooting)
6. [FAQ](#faq)

---

## Getting Started

### First Launch

When you first launch CrabOnTree, you'll see the welcome screen with:
- **Open Repository** button - Opens a file browser to select a Git repository
- **Recent Repositories** - Quick access to previously opened repos (appears after first use)

### Opening a Repository

1. Click **"📂 Open Repository"** button
2. Navigate to any directory containing a Git repository (.git folder)
3. Select the directory and click **Open**

The application will immediately load:
- Working directory status (modified files)
- Current branch and HEAD commit
- Available branches

### Interface Overview

CrabOnTree has three main panels:

```
┌─────────────────────────────────────────────┐
│ 🔨 Working Directory                        │
│  - View modified/staged/untracked files     │
│  - Stage/unstage files                      │
│  - Search files by path                     │
├─────────────────────────────────────────────┤
│ 💬 Commit Message                           │
│  - Write multi-line commit messages         │
│  - View staged file count                   │
│  - See author identity                      │
├─────────────────────────────────────────────┤
│ 📜 Commit History                           │
│  - Browse commit history                    │
│  - Search commits                           │
│  - View commit diffs                        │
└─────────────────────────────────────────────┘
```

---

## Basic Workflows

### Workflow 1: Making a Commit

This is the most common workflow - viewing changes, staging files, and creating a commit.

**Step-by-step**:

1. **View Your Changes**
   - Open your repository
   - The working directory panel shows all modified files
   - Files are marked as:
     - `[U]` - Unstaged (modified but not staged)
     - `[S]` - Staged (ready to commit)
     - `[?]` - Untracked (new file)

2. **Stage Files**
   - **Individual file**: Click the `+` button next to the file
   - **Keyboard**: Press `Space` when a file is focused
   - **All files**: Click **"Stage All"** or press `a`

3. **Write Commit Message**
   - Click in the commit message text box (or press `c`)
   - Type your commit message
   - First line should be a brief summary (< 50 chars recommended)
   - Add blank line and detailed description if needed

4. **Create Commit**
   - Click **"📝 Commit"** button
   - **Keyboard**: Press `Ctrl+Enter` when in commit message box
   - The commit is created and the UI refreshes automatically

**Example**:
```
Add user authentication feature

- Implement login/logout endpoints
- Add session management with Redis
- Add password hashing with bcrypt
- Update user model with auth fields
```

### Workflow 2: Browsing History

View past commits and their changes.

**Step-by-step**:

1. **Load Commit History**
   - History loads automatically when you open a repository
   - Shows the most recent 100 commits by default

2. **Navigate Commits**
   - **Scroll**: Use mouse wheel or scrollbar
   - **Keyboard**: Press `↑`/`↓` or `j`/`k` to navigate
   - **Jump**: Press `g g` for top, `G` for bottom

3. **View Commit Details**
   - Click on any commit
   - **Keyboard**: Press `Enter` on focused commit
   - The diff panel shows all files changed in that commit

4. **Explore Changes**
   - Each file in the diff is collapsible
   - Click the `▼` arrow to expand/collapse
   - View line-by-line changes with +/- indicators

### Workflow 3: Searching

Find specific files or commits quickly.

**Search Files** (Working Directory):
1. Press `/` to focus the file search box
2. Type any part of the file path
3. Files filter in real-time
4. Press `Esc` to clear and exit search

**Search Commits** (History):
1. Press `Ctrl+F` to focus commit search
2. Type search term - searches in:
   - Commit message
   - Author name
   - Commit hash
3. Commits filter in real-time
4. Press `Esc` to clear and exit search

**Examples**:
- File search: `"src/auth"` finds all files in src/auth/
- Commit search: `"bug fix"` finds commits mentioning bug fixes
- Commit search: `"alice"` finds commits by author Alice
- Commit search: `"a3f2b1"` finds commit with that hash

### Workflow 4: Unstaging Files

Changed your mind about what to commit?

**Unstage Individual File**:
1. Find the staged file (marked with `[S]`)
2. Click the `-` button next to it
3. **Keyboard**: Press `Space` on the focused file

**Unstage All Files**:
1. Click **"Unstage All"** button
2. **Keyboard**: Press `u`

### Workflow 5: Keyboard-First Navigation

Work without touching the mouse!

**Quick Example Session**:
```
1. Open repository (mouse required for first time)
2. Press `1` to focus working directory
3. Press `j` to move down through files
4. Press `Space` to stage focused file
5. Repeat `j` + `Space` for more files
6. Press `c` to focus commit message
7. Type your message
8. Press `Ctrl+Enter` to commit
9. Press `3` to view commit history
10. Press `j`/`k` to browse commits
11. Press `Enter` to view commit diff
```

---

## Keyboard Shortcuts Reference

### Global Shortcuts

These work from anywhere in the application:

| Key | Action |
|-----|--------|
| `1` | Focus Working Directory panel |
| `2` | Focus Commit Message panel |
| `3` | Focus Commit History panel |
| `Tab` | Cycle through panels (1→2→3→1) |
| `r` | Refresh repository status |
| `?` | Show this keyboard shortcuts help |

### Navigation

Navigate within lists (files or commits):

| Key | Action |
|-----|--------|
| `↑` | Previous item |
| `↓` | Next item |
| `j` | Previous item (vim-style) |
| `k` | Next item (vim-style) |
| `g g` | Jump to top (vim-style) |
| `G` | Jump to bottom (vim-style) |
| `Home` | Jump to first item |
| `End` | Jump to last item |

### Actions

Context-specific actions:

| Key | Action | Context |
|-----|--------|---------|
| `Space` | Toggle file staging | Working Directory |
| `Enter` | View commit details | Commit History |
| `c` | Focus commit message | Anywhere |
| `Ctrl+Enter` | Create commit | Commit Message |
| `a` | Stage all files | Working Directory |
| `u` | Unstage all files | Working Directory |

### Search

| Key | Action |
|-----|--------|
| `/` | Focus file search | Working Directory |
| `Ctrl+F` | Focus commit search | Commit History |
| `Esc` | Clear search / blur input | Search boxes |

### Visual Indicators

- **`>`** marker - Shows currently focused item (for keyboard navigation)
- **Bold text** - Shows selected commit (diff being displayed)
- **Blue highlight** - Shows text input has keyboard focus

---

## Tips and Tricks

### 1. Efficient Staging

**Selective Staging**:
- Don't stage everything at once if you want to make focused commits
- Use search (`/`) to find related files, then stage them together
- Example: Search `"test"` to stage only test files

**Batch Operations**:
- Stage All (`a`) is optimized for large numbers of files
- Chunked processing keeps UI responsive even with 1,000+ files

### 2. Writing Good Commit Messages

**Best Practices**:
```
Short summary (50 chars or less)

More detailed explanatory text, if necessary. Wrap it to
about 72 characters or so. The blank line separating the
summary from the body is critical.

- Bullet points are fine
- Use imperative mood: "Fix bug" not "Fixed bug"
- Explain what and why, not how
```

**Character/Line Counter**:
- Watch the counter below the text box
- First line should be < 50 chars (summary)
- Body lines should be < 72 chars (readability)

### 3. Fast Navigation

**Panel Switching**:
- Use number keys (`1`, `2`, `3`) to jump directly to panels
- Much faster than clicking or tabbing

**Vim-Style Navigation**:
- `j`/`k` for single-item movement (keeps fingers on home row)
- `gg`/`G` for jumping to ends
- These work in both file lists and commit history

**Focus Without Selection**:
- Navigate with `j`/`k` to highlight items (shows `>` marker)
- Only loads commit diff when you press `Enter`
- Allows fast browsing without loading costs

### 4. Search Power

**File Search Tips**:
- Search is case-insensitive
- Matches anywhere in path: `"auth"` finds `src/auth/login.rs`
- Use `/` from anywhere to quickly filter files

**Commit Search Tips**:
- Searches across message, author, and hash
- Use author name to find all your commits
- Use partial hash to find specific commit
- Searches are case-insensitive

### 5. Performance Tips

**Large Repositories** (1,000+ files):
- Use search/filter to narrow down file lists
- Virtual scrolling keeps UI smooth
- Collapse diff sections you're not viewing

**Long History** (500+ commits):
- Default 100 commits loads fast
- Use search to find specific commits
- Only selected commit's diff is loaded

**Large Diffs** (50+ files):
- Files are collapsed by default
- Only expand sections you need to review
- Scrolling is smooth even with large diffs

### 6. Workflow Optimization

**Quick Commit Cycle**:
```
1. Make changes in your editor
2. Press `r` to refresh (see changes)
3. Press `a` to stage all (or selectively stage)
4. Press `c` to focus message
5. Type message
6. Press `Ctrl+Enter` to commit
7. Press `3` to view in history
```

**Review Before Commit**:
```
1. Press `3` to view history
2. Press `Enter` on latest commit
3. Review changes carefully
4. Press `1` to return to working directory
5. Stage and commit
```

### 7. Author Identity

**Automatic Detection**:
- CrabOnTree reads from `git config`
- Shows "Committing as: Name <email>" below commit message
- Commits use the same identity as command-line git

**If Not Set**:
- Falls back to system username and hostname
- Configure properly with:
  ```bash
  git config --global user.name "Your Name"
  git config --global user.email "you@example.com"
  ```

---

## Troubleshooting

### Repository Won't Open

**Problem**: Error opening repository or no repositories shown.

**Solutions**:
1. **Verify it's a Git repository**: Must have a `.git` folder
2. **Check permissions**: Ensure you have read access to the directory
3. **Try command line**: If `git status` fails, CrabOnTree will too
4. **Clone fresh**: If repository is corrupted, re-clone it

### Can't Stage Files

**Problem**: Files show as modified but can't stage them.

**Solutions**:
1. **Check file permissions**: Ensure files are readable
2. **Refresh**: Press `r` to reload status
3. **Check git**: Run `git status` to verify git sees the changes
4. **Large binary files**: Git may have issues with very large files

### Commit Button Disabled

**Problem**: Can't click the commit button.

**Reasons**:
- **No staged files**: Stage at least one file first
- **Empty message**: Type a commit message (at least 1 non-whitespace character)
- **Operation in progress**: Wait for current operation to complete

### Slow Performance

**Problem**: UI feels sluggish with large repository.

**Solutions**:
1. **Use search**: Filter files/commits to reduce visible items
2. **Collapse diffs**: Close sections you're not viewing
3. **Close other apps**: Free up system resources
4. **Check disk**: SSD recommended for large repositories
5. **Update Rust**: Build with latest stable Rust compiler

### Commit Not Appearing

**Problem**: Created commit but don't see it in history.

**Solutions**:
1. **Refresh**: Press `r` or click refresh button
2. **Check branch**: Ensure you're viewing the correct branch
3. **Load history**: Click "Load Commit History" if not auto-loaded
4. **Verify with git**: Run `git log` to confirm commit exists

### Keyboard Shortcuts Not Working

**Problem**: Pressing keys doesn't trigger expected actions.

**Solutions**:
1. **Check focus**: Some shortcuts require specific panel focus
2. **Text input active**: `Esc` out of search/message boxes first
3. **Modal open**: Close help dialog (`?`) if open
4. **Operating system**: Some OS shortcuts may conflict

### Search Not Finding Items

**Problem**: Know an item exists but search doesn't show it.

**Solutions**:
1. **Clear search**: Press `Esc` and try again
2. **Check spelling**: Search is exact substring match
3. **Case doesn't matter**: Searches are case-insensitive
4. **Full text search**: For commits, searches message/author/hash
5. **Refresh data**: Press `r` to reload repository data

---

## FAQ

### General Questions

**Q: Does CrabOnTree work with any Git repository?**
A: Yes! CrabOnTree works with any standard Git repository, including GitHub, GitLab, Bitbucket, or local repos.

**Q: Can I use CrabOnTree alongside command-line git?**
A: Absolutely! CrabOnTree is fully compatible with command-line git. Changes made with either tool are visible to the other.

**Q: Does it support Git LFS (Large File Storage)?**
A: Yes, through git2's LFS support. Large files are handled transparently.

**Q: What platforms does CrabOnTree support?**
A: CrabOnTree is built with Rust and egui, supporting Linux, macOS, and Windows.

### Features

**Q: Can I create branches?**
A: Not yet. Branch creation/switching is planned for Phase 2b. You can currently view branches but must use command-line to switch.

**Q: Does it support push/pull/fetch?**
A: Not yet. Remote operations are planned for Phase 2c. Currently focuses on local operations.

**Q: Can I merge branches?**
A: Not yet. Merge operations are planned for Phase 3.

**Q: Can I rebase or cherry-pick?**
A: Not yet. Advanced operations are planned for Phase 3.

**Q: Can I view/edit stashes?**
A: Not yet. Stash management is planned for Phase 2b.

**Q: Can I stage partial files (hunks)?**
A: Not yet. Currently supports full-file staging. Partial staging may be added in future releases.

### Workflow Questions

**Q: How do I undo a commit?**
A: Currently use command-line: `git reset HEAD~1` (keeps changes) or `git reset --hard HEAD~1` (discards changes). UI support coming in future release.

**Q: Can I amend the last commit?**
A: Currently use command-line: `git commit --amend`. UI support planned for future release.

**Q: How do I discard changes in a file?**
A: Currently use command-line: `git checkout -- <file>`. UI support planned for future release.

**Q: Can I compare two commits?**
A: Not directly yet. You can view each commit's diff individually. Commit comparison may be added in future release.

**Q: How do I resolve merge conflicts?**
A: Currently use command-line or dedicated merge tools. Conflict resolution UI is planned for Phase 2c.

### Performance Questions

**Q: How many files can CrabOnTree handle?**
A: Tested with 1,000+ files. Virtual scrolling ensures smooth performance even with 10,000+ files.

**Q: How much commit history can it load?**
A: Default limit is 100 commits (fast loading). Architecture supports much larger histories with smooth scrolling.

**Q: Does it work with monorepos?**
A: Yes! Performance optimizations (virtual scrolling, chunked operations, lazy loading) are designed for large repositories.

**Q: Why is the first load slow?**
A: Git operations (reading status, history) take time on first load. Subsequent operations are faster due to caching.

**Q: Can I speed it up?**
A: Use SSD storage, close unused applications, use search/filter to reduce visible items, build in release mode (`--release`).

### Technical Questions

**Q: What Git library does it use?**
A: Hybrid approach: gitoxide (gix) for read operations, git2 for write operations. Best of both worlds.

**Q: Why not just use libgit2?**
A: gitoxide is pure Rust (faster, safer) for reads. git2 is mature and reliable for writes.

**Q: Does it store any data outside the git repository?**
A: Only recent repositories list (stored in user config directory). All git data remains in standard .git folder.

**Q: Can I customize the UI theme?**
A: Basic dark/light theme support through egui. More customization may be added in future releases.

**Q: Is there a configuration file?**
A: Not yet. Configuration options planned for future release.

### Troubleshooting Questions

**Q: Why does my commit have the wrong author?**
A: CrabOnTree uses git config (`user.name` and `user.email`). Set with:
```bash
git config --global user.name "Your Name"
git config --global user.email "you@example.com"
```

**Q: Why can't I see my recent commits?**
A: Press `r` to refresh, or ensure you're on the correct branch. Check with `git log` if issue persists.

**Q: The UI froze, what happened?**
A: Very rare. Try pressing `Esc` to cancel current operation. If persists, restart the application.

**Q: How do I report a bug?**
A: Open an issue at: https://github.com/yourusername/crabontree/issues

**Q: How do I request a feature?**
A: Open a discussion at: https://github.com//crabontree/discussions

---

## Need More Help?

### Documentation

- **README**: Project overview and quick start
- **Architecture**: System design and components
- **Contributing**: Developer guide and code style
- **Keyboard Shortcuts**: Complete shortcut reference
- **Testing**: Testing strategy and guidelines

### Community

- **GitHub Issues**: Bug reports and feature requests
- **GitHub Discussions**: Questions and community help
- **Discord**: Real-time chat (coming soon)

### Command-Line Git Resources

CrabOnTree is a complement to git, not a replacement. For advanced operations, you'll use command-line git:

- [Official Git Documentation](https://git-scm.com/doc)
- [Pro Git Book](https://git-scm.com/book/en/v2)
- [Git Cheat Sheet](https://training.github.com/downloads/github-git-cheat-sheet/)

---

**Happy committing! 🦀🌳**
