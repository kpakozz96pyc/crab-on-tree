# Contributing to CrabOnTree

Thank you for your interest in contributing to CrabOnTree! This guide will help you get started with development.

## Table of Contents

1. [Prerequisites](#prerequisites)
2. [Development Setup](#development-setup)
3. [Project Structure](#project-structure)
4. [Development Workflow](#development-workflow)
5. [Code Style](#code-style)
6. [Testing Guidelines](#testing-guidelines)
7. [Pull Request Process](#pull-request-process)
8. [Adding Features](#adding-features)
9. [Release Process](#release-process)

---

## Prerequisites

### Required

- **Rust**: 1.70 or later
  ```bash
  # Install via rustup
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

  # Update to latest stable
  rustup update stable
  ```

- **Git**: 2.30 or later
  ```bash
  git --version
  ```

### Platform-Specific

**Linux**:
```bash
# Ubuntu/Debian
sudo apt-get install libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev \
    libxkbcommon-dev libssl-dev

# Fedora
sudo dnf install libxcb-devel libxkbcommon-devel openssl-devel
```

**macOS**:
```bash
# Usually no additional dependencies needed
# If issues, install Xcode Command Line Tools
xcode-select --install
```

**Windows**:
```powershell
# Install Visual Studio C++ Build Tools
# Or Visual Studio Community with "Desktop development with C++"
```

### Recommended Tools

- **rust-analyzer**: LSP for IDE integration
- **cargo-watch**: Auto-rebuild on changes
  ```bash
  cargo install cargo-watch
  ```
- **cargo-nextest**: Faster test runner
  ```bash
  cargo install cargo-nextest
  ```

---

## Development Setup

### 1. Fork and Clone

```bash
# Fork on GitHub, then clone your fork
git clone https://github.com/YOUR_USERNAME/crabontree.git
cd crabontree

# Add upstream remote
git remote add upstream https://github.com/ORIGINAL_OWNER/crabontree.git
```

### 2. Build

```bash
# Debug build (faster compilation, slower runtime)
cargo build

# Release build (slower compilation, faster runtime)
cargo build --release

# Build specific crate
cargo build -p crabontree-git
```

### 3. Run

```bash
# Run debug build
cargo run

# Run release build (recommended for performance testing)
cargo run --release

# Run with logging
RUST_LOG=debug cargo run

# Run with specific log level per crate
RUST_LOG=crabontree_git=debug,crabontree_app=info cargo run
```

### 4. Test

```bash
# Run all tests
cargo test --workspace

# Run specific crate tests
cargo test -p crabontree-git
cargo test -p crabontree-app

# Run with output
cargo test -- --nocapture

# Run with logging
RUST_LOG=debug cargo test -- --nocapture

# Run specific test
cargo test test_stage_file

# Run tests with nextest (faster)
cargo nextest run --workspace
```

### 5. Development Loop

```bash
# Watch and auto-rebuild on changes
cargo watch -x 'run'

# Watch and auto-test
cargo watch -x 'test --workspace'

# Watch, check, and test
cargo watch -x check -x 'test --workspace'
```

---

## Project Structure

```
crabontree/
├── crates/
│   ├── git/              # Git operations layer
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── repo.rs       # Main GitRepository implementation
│   │   │   └── error.rs      # Error types
│   │   ├── tests/
│   │   │   └── integration_test.rs
│   │   └── Cargo.toml
│   │
│   ├── app/              # Application state layer
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── state.rs      # AppState definition
│   │   │   ├── message.rs    # AppMessage enum
│   │   │   ├── effect.rs     # Effect enum
│   │   │   ├── job.rs        # Job enum
│   │   │   ├── reducer.rs    # Pure state update function
│   │   │   └── executor.rs   # Async effect executor
│   │   ├── tests/
│   │   │   ├── reducer_test.rs
│   │   │   └── integration_test.rs
│   │   └── Cargo.toml
│   │
│   ├── ui_core/          # Framework-agnostic UI primitives
│   │   ├── src/
│   │   │   └── lib.rs
│   │   └── Cargo.toml
│   │
│   └── ui_egui/          # egui-based frontend (binary)
│       ├── src/
│       │   └── main.rs       # Main application
│       └── Cargo.toml
│
├── docs/                 # Documentation
│   ├── ARCHITECTURE.md
│   ├── USER_GUIDE.md
│   ├── KEYBOARD_SHORTCUTS.md
│   └── CONTRIBUTING.md (this file)
│
├── scripts/              # Utility scripts
│   └── create_test_repos.sh
│
├── test-repos/           # Generated test repositories (gitignored)
│
├── Cargo.toml            # Workspace configuration
├── Cargo.lock            # Locked dependencies
└── README.md             # Project overview
```

### Crate Responsibilities

**crates/git/**:
- Git operations (status, staging, commits, diffs)
- Repository abstraction over gix + git2
- No app/UI dependencies

**crates/app/**:
- Application state management
- Elm-inspired architecture (reducer, effects, jobs)
- Framework-agnostic (no UI code)

**crates/ui_core/**:
- Shared UI primitives (future)
- Currently minimal

**crates/ui_egui/**:
- Concrete UI implementation with egui
- Main binary entry point
- User interaction handling

---

## Development Workflow

### Typical Feature Development

1. **Create a branch**:
   ```bash
   git checkout -b feature/my-feature
   ```

2. **Make changes** following the architecture:
   - Start with git layer if new git operations needed
   - Add app layer messages/effects/jobs
   - Implement UI layer

3. **Add tests**:
   - Unit tests for git operations
   - Reducer tests for state management
   - Integration tests for workflows

4. **Run tests**:
   ```bash
   cargo test --workspace
   ```

5. **Check code**:
   ```bash
   # Check compilation
   cargo check --workspace

   # Format code
   cargo fmt --all

   # Lint code
   cargo clippy --workspace -- -D warnings
   ```

6. **Commit changes**:
   ```bash
   git add .
   git commit -m "Add feature: brief description"
   ```

7. **Push and create PR**:
   ```bash
   git push origin feature/my-feature
   # Open PR on GitHub
   ```

### Architecture Flow

When adding a feature, follow this order:

**1. Git Layer** (if needed):
```rust
// crates/git/src/repo.rs
impl GitRepository {
    pub fn new_operation(&self) -> Result<Data, GitError> {
        // Implementation
    }
}
```

**2. App Layer**:
```rust
// crates/app/src/message.rs
pub enum AppMessage {
    NewOperationRequested,
    NewOperationCompleted(Data),
}

// crates/app/src/effect.rs
pub enum Effect {
    PerformNewOperation(Params),
}

// crates/app/src/job.rs
pub enum Job {
    PerformNewOperation(Params),
}

// crates/app/src/reducer.rs
pub fn reduce(state: &mut AppState, message: AppMessage) -> Effect {
    match message {
        AppMessage::NewOperationRequested => {
            Effect::PerformNewOperation(params)
        }
        AppMessage::NewOperationCompleted(data) => {
            state.data = data;
            Effect::None
        }
    }
}

// crates/app/src/executor.rs
async fn execute_new_operation(params: Params) -> anyhow::Result<AppMessage> {
    let result = tokio::task::spawn_blocking(move || {
        let repo = GitRepository::open(&path)?;
        repo.new_operation()
    }).await??;
    Ok(AppMessage::NewOperationCompleted(result))
}
```

**3. UI Layer**:
```rust
// crates/ui_egui/src/main.rs
impl CrabOnTreeApp {
    fn render_new_feature(&mut self, ui: &mut egui::Ui) {
        if ui.button("New Operation").clicked() {
            self.handle_message(AppMessage::NewOperationRequested);
        }
    }
}
```

---

## Code Style

### Rust Conventions

- **Follow** [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- **Use** `cargo fmt` for formatting (runs automatically on save if configured)
- **Fix** all `cargo clippy` warnings
- **Prefer** explicit types in public APIs
- **Use** descriptive variable names (avoid single letters except in closures)

### Naming Conventions

```rust
// Types: PascalCase
struct GitRepository { }
enum AppMessage { }

// Functions/variables: snake_case
fn stage_file() { }
let commit_message = "...";

// Constants: SCREAMING_SNAKE_CASE
const MAX_COMMITS: usize = 100;

// Modules: snake_case
mod working_directory;
```

### Documentation

**Public APIs** must have doc comments:
```rust
/// Stages a file for commit.
///
/// # Arguments
/// * `path` - Path to the file, relative to repository root
///
/// # Returns
/// * `Ok(())` if successful
/// * `Err(GitError)` if file doesn't exist or staging fails
///
/// # Examples
/// ```
/// let repo = GitRepository::open(".")?;
/// repo.stage_file(Path::new("src/main.rs"))?;
/// ```
pub fn stage_file(&self, path: &Path) -> Result<(), GitError> {
    // Implementation
}
```

**Internal code** should have explanatory comments for complex logic:
```rust
// Use gitoxide for reading status (faster than git2)
let status = self.gix_repo.status()?;

// But use git2 for staging (more reliable for writes)
let mut index = self.git2_repo.index()?;
index.add_path(path)?;
index.write()?;
```

### Error Handling

**Use appropriate error types**:
```rust
// Git layer: Custom error type
pub enum GitError {
    RepoNotFound(PathBuf),
    OperationFailed(String),
}

// App layer: anyhow for flexibility
use anyhow::{Context, Result};

async fn execute_job() -> Result<AppMessage> {
    let result = operation()
        .context("Failed to perform operation")?;
    Ok(result)
}
```

**Provide context**:
```rust
GitRepository::open(&path)
    .with_context(|| format!("Failed to open repository at {}", path.display()))?
```

### Logging

**Use structured logging**:
```rust
use tracing::{debug, info, warn, error};

// Info for important events
tracing::info!("Created commit: {}", commit_hash);

// Debug for detailed information
tracing::debug!("Staging file: {:?}", path);

// Warn for recoverable issues
tracing::warn!("Author not configured, using fallback");

// Error for failures
tracing::error!("Failed to stage file: {}", error);
```

**Add timing for performance-critical operations**:
```rust
let start = std::time::Instant::now();
let result = expensive_operation()?;
let elapsed = start.elapsed();
tracing::info!("operation completed: {} items in {:?}", count, elapsed);
```

---

## Testing Guidelines

### Test Structure

```
crates/git/tests/
    integration_test.rs   # Integration tests for git operations

crates/app/tests/
    reducer_test.rs       # Unit tests for reducer
    integration_test.rs   # Integration tests for workflows
```

### Writing Tests

**Git Layer Tests**:
```rust
#[test]
fn test_stage_file() {
    // Setup: Create test repository
    let (temp_dir, repo) = init_test_repo();
    let repo_path = temp_dir.path();

    // Create test file
    std::fs::write(repo_path.join("test.txt"), "content").unwrap();

    // Execute: Stage the file
    repo.stage_file(Path::new("test.txt")).unwrap();

    // Verify: Check file is staged
    let status = repo.get_working_dir_status().unwrap();
    let staged = status.iter().find(|f| f.path.ends_with("test.txt")).unwrap();
    assert!(staged.is_staged);
}
```

**App Layer Tests**:
```rust
#[test]
fn test_reducer_stage_file() {
    // Setup: Initial state
    let mut state = AppState::new();
    state.current_repo = Some(test_repo_state());

    // Execute: Send message
    let effect = reduce(&mut state, AppMessage::StageFileRequested(PathBuf::from("test.txt")));

    // Verify: Check effect
    match effect {
        Effect::StageFile { file_path, .. } => {
            assert_eq!(file_path, PathBuf::from("test.txt"));
        }
        _ => panic!("Expected StageFile effect"),
    }
}
```

### Test Helpers

**Use helper functions** for common setup:
```rust
// crates/git/tests/integration_test.rs
fn init_test_repo() -> (TempDir, GitRepository) {
    let temp_dir = tempfile::tempdir().unwrap();
    let repo_path = temp_dir.path();

    // Initialize git repo
    std::process::Command::new("git")
        .args(["init"])
        .current_dir(repo_path)
        .output()
        .unwrap();

    // Set config
    std::process::Command::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(repo_path)
        .output()
        .unwrap();

    let repo = GitRepository::open(repo_path).unwrap();
    (temp_dir, repo)
}
```

### Test Coverage Goals

- **Critical paths**: 100% (staging, commits, diffs)
- **Error handling**: Cover major error cases
- **Edge cases**: Empty repos, large data, concurrent operations
- **Integration**: End-to-end workflows

### Running Tests

```bash
# All tests
cargo test --workspace

# With logging
RUST_LOG=debug cargo test -- --nocapture

# Specific test
cargo test test_stage_file

# All tests in a crate
cargo test -p crabontree-git

# Integration tests only
cargo test --test integration_test
```

---

## Pull Request Process

### Before Submitting

**1. Ensure tests pass**:
```bash
cargo test --workspace
```

**2. Format code**:
```bash
cargo fmt --all
```

**3. Check for warnings**:
```bash
cargo clippy --workspace -- -D warnings
```

**4. Update documentation** if needed:
- Update README.md for user-facing changes
- Update relevant docs/ files
- Add doc comments to new public APIs

**5. Add changelog entry** (if applicable):
- Note new features in commit message
- Breaking changes must be clearly documented

### PR Template

**Title**: Brief, descriptive (e.g., "Add file unstaging functionality")

**Description**:
```markdown
## Summary
Brief description of changes

## Motivation
Why this change is needed

## Changes
- Bullet point list of changes
- Key files modified
- New features/fixes

## Testing
- Tests added/modified
- Manual testing performed

## Screenshots (if UI changes)
[Include screenshots]

## Checklist
- [ ] Tests pass locally
- [ ] Code formatted with `cargo fmt`
- [ ] No clippy warnings
- [ ] Documentation updated
- [ ] Changelog entry added (if needed)
```

### Review Process

1. **Automated checks** run (CI):
   - Build on all platforms
   - Run all tests
   - Check formatting and linting

2. **Code review** by maintainer:
   - Architecture fit
   - Code quality
   - Test coverage
   - Documentation

3. **Address feedback**:
   - Make requested changes
   - Push updates to PR branch
   - Respond to comments

4. **Merge**:
   - Maintainer merges when approved
   - PR branch automatically deleted

---

## Adding Features

### Feature Request Process

1. **Check existing issues** to avoid duplicates
2. **Open a discussion** for major features
3. **Create an issue** with:
   - Clear description
   - Use cases
   - Proposed approach
   - Potential challenges

### Implementation Phases

For major features, break into phases:

**Phase 1: Git Layer**
- Implement git operations
- Add tests
- PR and merge

**Phase 2: App Layer**
- Add messages, effects, jobs
- Implement reducer logic
- Add tests
- PR and merge

**Phase 3: UI Layer**
- Implement user interface
- Add keyboard shortcuts
- Manual testing
- PR and merge

**Phase 4: Polish**
- Documentation
- Performance optimization
- Edge case handling

### Example: Adding Branch Creation

**1. Git Layer** (`crates/git/src/repo.rs`):
```rust
pub fn create_branch(&self, name: &str) -> Result<(), GitError> {
    let head = self.git2_repo.head()?;
    let commit = head.peel_to_commit()?;
    self.git2_repo.branch(name, &commit, false)?;
    Ok(())
}
```

**2. App Layer** (multiple files):
```rust
// message.rs
CreateBranchRequested(String),
BranchCreated(String),

// effect.rs
CreateBranch { repo_path: PathBuf, name: String },

// reducer.rs
AppMessage::CreateBranchRequested(name) => {
    Effect::CreateBranch { repo_path, name }
}

// executor.rs
async fn execute_create_branch(path: PathBuf, name: String) -> Result<AppMessage> {
    // ...
}
```

**3. UI Layer** (`crates/ui_egui/src/main.rs`):
```rust
// Add branch name input
egui::TextEdit::singleline(&mut self.new_branch_name)
    .hint_text("Branch name...")
    .show(ui);

// Add create button
if ui.button("Create Branch").clicked() {
    self.handle_message(AppMessage::CreateBranchRequested(
        self.new_branch_name.clone()
    ));
}
```

---

## Release Process

### Versioning

We follow [Semantic Versioning](https://semver.org/):
- **Major** (1.0.0): Breaking changes
- **Minor** (0.1.0): New features, backwards compatible
- **Patch** (0.0.1): Bug fixes

### Release Checklist

1. **Update version** in all `Cargo.toml` files
2. **Update CHANGELOG.md** with release notes
3. **Update README.md** if needed
4. **Run full test suite**:
   ```bash
   cargo test --workspace --release
   ```
5. **Build release binaries**:
   ```bash
   cargo build --release
   ```
6. **Create git tag**:
   ```bash
   git tag -a v0.2.0 -m "Release v0.2.0"
   git push origin v0.2.0
   ```
7. **Create GitHub release** with:
   - Release notes
   - Binary attachments (per platform)
   - Checksums

### Release Cadence

- **Major releases**: When breaking changes accumulate
- **Minor releases**: Every 4-8 weeks with new features
- **Patch releases**: As needed for critical bugs

---

## Getting Help

### Resources

- **Documentation**: Check `docs/` directory
- **Architecture**: See [ARCHITECTURE.md](ARCHITECTURE.md)
- **User Guide**: See [USER_GUIDE.md](USER_GUIDE.md)
- **Code**: Read existing implementations as examples

### Communication

- **Issues**: Bug reports and feature requests
- **Discussions**: Questions and brainstorming
- **Discord**: Real-time chat (coming soon)

### Common Questions

**Q: How do I debug the UI?**
A: Run with `RUST_LOG=debug cargo run` and check terminal output. egui has built-in debug tools (press F12).

**Q: How do I test with large repositories?**
A: Use `scripts/create_test_repos.sh` to generate test repos with 1000+ files.

**Q: My PR was rejected, why?**
A: Check feedback carefully. Common reasons: missing tests, not following architecture, insufficient documentation.

**Q: Can I work on multiple features at once?**
A: Yes, but use separate branches and PRs for each feature. Makes review easier.

---

## Code of Conduct

### Our Pledge

We are committed to providing a welcoming and inclusive environment for all contributors, regardless of experience level, gender, gender identity, sexual orientation, disability, personal appearance, race, ethnicity, age, religion, or nationality.

### Expected Behavior

- Be respectful and constructive in feedback
- Welcome newcomers and help them learn
- Focus on what is best for the community
- Show empathy towards other contributors

### Unacceptable Behavior

- Harassment or discriminatory language
- Trolling, insulting comments, or personal attacks
- Publishing others' private information
- Other conduct inappropriate in a professional setting

### Enforcement

Violations can be reported to project maintainers. All complaints will be reviewed and investigated, resulting in a response deemed necessary and appropriate.

---

## License

By contributing to CrabOnTree, you agree that your contributions will be licensed under the same terms as the project (MIT OR Apache-2.0).

---

## Thank You!

Thank you for contributing to CrabOnTree! Your efforts help make this project better for everyone.

**Happy coding! 🦀🌳**
