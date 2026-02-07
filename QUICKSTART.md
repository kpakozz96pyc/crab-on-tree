# CrabOnTree Quick Start Guide

## Building and Running

### Build the Application

```bash
# Debug build (faster compilation)
cargo build --workspace

# Release build (optimized)
cargo build --release
```

### Run the Application

```bash
# Run in debug mode
cargo run --bin crabontree

# Run release version
cargo run --release --bin crabontree

# Run with debug logging
RUST_LOG=debug cargo run --bin crabontree
```

## First Steps

1. **Launch CrabOnTree**
   ```bash
   cargo run --bin crabontree
   ```

2. **Open a Repository**
   - Click "📂 Open Repository" button
   - Navigate to any Git repository folder
   - Select the folder

3. **View Repository Information**
   - Current branch displays at the top
   - All branches are listed below
   - Status summary shows file counts

4. **Try with CrabOnTree Itself**
   ```bash
   # CrabOnTree is a Git repository!
   cargo run --bin crabontree
   # Click "Open Repository"
   # Navigate to the crab-on-tree/crab-on-tree folder
   ```

## Basic Operations

### Open a Repository
- Click "📂 Open Repository" button in the top panel
- Or select from recent repositories on the welcome screen

### Refresh Repository Data
- Click "🔄 Refresh" button (appears when a repository is open)
- Updates branch list and status

### Close Repository
- Click "✖ Close" button
- Returns to welcome screen

### View Recent Repositories
- Recent repositories appear on the welcome screen
- Click any path to quickly reopen

## Testing

```bash
# Run all tests
cargo test --workspace

# Run with output
cargo test --workspace -- --nocapture

# Run with debug logging
RUST_LOG=debug cargo test --workspace
```

Expected results:
- `crabontree-git`: 7 tests ✅
- `crabontree-app`: 10 tests ✅
- `crabontree-ui-core`: 5 tests ✅
- **Total**: 22 tests ✅

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

Phase 0 architecture ensures UI never freezes. If this happens:
1. Check logs: `RUST_LOG=debug cargo run`
2. Verify async job system is working
3. Report as a bug

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

## Keyboard Shortcuts (Phase 0)

Currently no keyboard shortcuts are implemented in the UI, but they are defined in `ui_core`:
- Ctrl+O: Open Repository (planned)
- Ctrl+R: Refresh Repository (planned)
- Ctrl+Q: Quit (planned)

These will be activated in a future phase.

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

## What's Next?

After Phase 0, you can:
- Explore the codebase structure
- Read the architecture documentation
- Try opening different repositories
- Check the logs to understand the flow
- Look forward to Phase 1 features (commit history)

---

**Ready to start?**

```bash
cargo run --bin crabontree
```

Enjoy using CrabOnTree! 🦀
