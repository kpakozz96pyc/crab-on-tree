# CrabOnTree Quick Start Guide

**Version**: v0.1.0 (Phase 2a Complete)

Get started with CrabOnTree in 5 minutes!

## Installation

### Build from Source

```bash
# Clone the repository
git clone https://github.com/yourusername/crabontree.git
cd crabontree

# Build release version (recommended)
cargo build --release

# Run
./target/release/crabontree
```

### Prerequisites

- Rust 1.70+ (`rustup install stable`)
- Git 2.30+

## First Use

### 1. Open a Repository

- Launch CrabOnTree
- Click "📂 Open Repository" button
- Navigate to any Git repository
- Select the directory

### 2. View Your Changes

The Working Directory panel shows all modified files:
- `[U]` - Unstaged (modified but not staged)
- `[S]` - Staged (ready to commit)
- `[?]` - Untracked (new file)

### 3. Stage Files

**Individual files**:
- Click `+` button next to file
- Or press `Space` with keyboard navigation

**All files**:
- Click "Stage All" button
- Or press `a` key

### 4. Write Commit Message

- Click in Commit Message box (or press `c`)
- Type your message
- First line: brief summary (< 50 chars)
- Add blank line and details if needed

### 5. Create Commit

- Click "📝 Commit" button
- Or press `Ctrl+Enter` in message box
- Commit appears in history automatically

## Essential Keyboard Shortcuts

CrabOnTree is designed for keyboard-first workflow:

```
GLOBAL
  1, 2, 3       Switch panels (Working Dir / Message / History)
  Tab           Cycle through panels
  r             Refresh repository
  ?             Show help dialog

NAVIGATION
  ↑ / ↓         Navigate items
  j / k         Navigate (vim-style)
  g g           Jump to top
  G             Jump to bottom

ACTIONS
  Space         Toggle file staging
  a             Stage all files
  u             Unstage all files
  c             Focus commit message
  Ctrl+Enter    Create commit
  Enter         View commit details (in history)

SEARCH
  /             Search files
  Ctrl+F        Search commits
  Esc           Clear search
```

Press `?` in the app for complete reference!

## Common Workflows

### Quick Commit (5 seconds)
```
a                  Stage all
c                  Focus message
[type message]     Write commit
Ctrl+Enter         Commit
```

### Selective Staging
```
/                  Search files
[filter]           Find specific files
Space              Stage focused file
Esc                Clear search
c                  Write message
Ctrl+Enter         Commit
```

### Browse History
```
3                  Focus history panel
j / k              Navigate commits
Enter              View commit details
Ctrl+F             Search commits
```

## Testing

```bash
# Run all tests
cargo test --workspace
```

Expected results:
- `crabontree-git`: 19 tests ✅
- `crabontree-app`: 14 tests ✅
- `crabontree-ui-core`: 5 tests ✅
- **Total**: 38 tests ✅

## Troubleshooting

### Application Won't Launch

**Check for errors:**
```bash
RUST_LOG=debug cargo run --bin crabontree
```

**Ensure all dependencies are installed:**
```bash
cargo clean
cargo build --workspace
```

### Can't Open Repository

**Common issues:**
- Path is not a Git repository → Make sure `.git` folder exists
- Permissions issue → Check file system permissions
- Invalid repository → Run `git status` in the folder to verify

**View detailed errors:**
```bash
RUST_LOG=crabontree_git=trace cargo run
```

### UI Appears Frozen

The async architecture ensures UI never freezes. If this happens:
1. Check logs: `RUST_LOG=debug cargo run`
2. Verify async job system is working
3. Report as a bug on GitHub

### Tests Failing

```bash
# Clean and rebuild
cargo clean
cargo test --workspace

# Run specific failing test
cargo test test_name -- --nocapture

# Check for Git availability
git --version
```

## Configuration

Configuration file location:
- **Linux**: `~/.config/crabontree/CrabOnTree/config.toml`
- **macOS**: `~/Library/Application Support/com.crabontree.CrabOnTree/config.toml`
- **Windows**: `%APPDATA%\crabontree\CrabOnTree\config\config.toml`

### Change Theme

Edit `config.toml`:
```toml
theme = "light"  # or "dark"
```

Or delete the config file to reset to defaults.

### Adjust Recent Repositories Limit

Edit `config.toml`:
```toml
max_recent = 20  # default is 10
```

## Performance Tips

CrabOnTree is optimized for large repositories:
- ✅ Handles 1,000+ files smoothly
- ✅ 500+ commits load in < 500ms
- ✅ Search is instant (< 100ms)
- ✅ Virtual scrolling ensures smooth UI
- ✅ Async operations keep UI responsive

**For best performance**:
- Use release build: `cargo build --release`
- Use search to filter large file lists
- Keep diffs collapsed when not viewing

## Development Mode

### Watch for Changes

```bash
# Install cargo-watch
cargo install cargo-watch

# Auto-run tests on changes
cargo watch -x "test --workspace"

# Auto-run app on changes
cargo watch -x "run --bin crabontree"
```

### Enable All Logging

```bash
RUST_LOG=trace cargo run --bin crabontree
```

### Check Code

```bash
# Check compilation without building
cargo check --workspace

# Check with all features
cargo check --workspace --all-features

# Format code
cargo fmt --all

# Lint code
cargo clippy --workspace
```

## Performance Tips

### Faster Compilation

```bash
# Use debug builds for development
cargo build

# Only build what changed
cargo build -p crabontree
```

### Faster Execution

```bash
# Use release builds for real usage
cargo build --release
./target/release/crabontree
```

## Getting Help

1. **Check Documentation**
   - [README.md](README.md) - Project overview
   - [docs/architecture.md](docs/architecture.md) - Architecture details
   - [docs/development.md](docs/development.md) - Development guide
   - [docs/logging.md](docs/logging.md) - Logging information

2. **Enable Debug Logging**
   ```bash
   RUST_LOG=debug cargo run --bin crabontree
   ```

3. **Check Test Results**
   ```bash
   cargo test --workspace
   ```

4. **Examine the Code**
   - Code is well-commented
   - Start with `crates/ui_egui/src/main.rs`
   - Follow the data flow through the layers

## Documentation

**For Users**:
- [USER_GUIDE.md](docs/USER_GUIDE.md) - Complete usage guide with workflows
- [KEYBOARD_SHORTCUTS.md](docs/KEYBOARD_SHORTCUTS.md) - Full shortcut reference

**For Developers**:
- [ARCHITECTURE.md](docs/ARCHITECTURE.md) - System design and decisions
- [CONTRIBUTING.md](docs/CONTRIBUTING.md) - How to contribute
- [TESTING.md](docs/TESTING.md) - Testing strategy and guide

## Getting Help

- **Press `?`** in the app for keyboard shortcuts
- **GitHub Issues**: Report bugs
- **GitHub Discussions**: Ask questions
- **Documentation**: Complete guides in `docs/`

## What's Next?

**Current Version**: v0.1.0 (Phase 2a Complete)

**Available Now**:
- ✅ Complete commit workflow (stage → commit → history)
- ✅ Search and filter (files and commits)
- ✅ Comprehensive keyboard shortcuts
- ✅ Excellent performance with large repos

**Coming Soon** (Phase 2b):
- Branch visualization
- Branch creation/switching
- Stash management

**Future** (Phase 2c & 3):
- Remote operations (fetch/pull/push)
- Merge and rebase
- Conflict resolution

---

**Ready to start?**

```bash
# Build and run
cargo build --release
./target/release/crabontree

# Or run directly
cargo run --release
```

**Happy coding! 🦀🌳**
