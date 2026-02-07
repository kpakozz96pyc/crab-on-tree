# Development Guide

This guide covers the development workflow, project structure, and how to contribute to CrabOnTree.

## Project Structure

```
crab-on-tree/
├── crates/
│   ├── git/              # Git operations layer
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── error.rs
│   │   │   └── repo.rs
│   │   └── tests/
│   │       └── integration_test.rs
│   │
│   ├── app/              # Application layer
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── config.rs
│   │   │   ├── effect.rs
│   │   │   ├── executor.rs
│   │   │   ├── job.rs
│   │   │   ├── message.rs
│   │   │   ├── reducer.rs
│   │   │   └── state.rs
│   │   └── tests/
│   │       └── reducer_test.rs
│   │
│   ├── ui_core/          # Framework-agnostic UI
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── color.rs
│   │       ├── shortcuts.rs
│   │       └── theme.rs
│   │
│   └── ui_egui/          # egui frontend
│       └── src/
│           └── main.rs
│
├── docs/                 # Documentation
│   ├── architecture.md
│   ├── development.md
│   └── logging.md
│
├── assets/               # Assets (future use)
│   ├── fonts/
│   └── icons/
│
└── Cargo.toml           # Workspace root
```

## Development Workflow

### Initial Setup

```bash
# Clone the repository
git clone <repository-url>
cd crab-on-tree/crab-on-tree

# Build the project
cargo build --workspace

# Run tests
cargo test --workspace

# Run the application
cargo run --bin crabontree
```

### Development Cycle

```bash
# 1. Make changes to code

# 2. Check compilation
cargo check --workspace

# 3. Run tests
cargo test --workspace

# 4. Run the application
cargo run --bin crabontree

# 5. Build release version
cargo build --release
```

### Continuous Development

```bash
# Install cargo-watch for auto-recompilation
cargo install cargo-watch

# Watch for changes and run tests
cargo watch -x "test --workspace"

# Watch for changes and run the application
cargo watch -x "run --bin crabontree"
```

## Code Style Guidelines

### General Principles

1. **Simplicity**: Keep code simple and readable
2. **Type Safety**: Leverage Rust's type system
3. **Error Handling**: Use `Result` types, never panic in library code
4. **Documentation**: Document public APIs and complex logic
5. **Testing**: Write tests for new functionality

### Naming Conventions

- **Types**: `PascalCase` (e.g., `GitRepository`, `AppState`)
- **Functions**: `snake_case` (e.g., `get_branches`, `load_config`)
- **Constants**: `SCREAMING_SNAKE_CASE` (e.g., `MAX_RECENT_REPOS`)
- **Modules**: `snake_case` (e.g., `executor`, `ui_core`)

### Code Organization

```rust
// Module structure
pub mod error;    // Error types
pub mod types;    // Data types
pub mod ops;      // Operations

// Re-export public API
pub use error::MyError;
pub use types::{TypeA, TypeB};
```

### Error Handling

```rust
// Git layer: Use thiserror
#[derive(Error, Debug)]
pub enum GitError {
    #[error("Repository not found: {0}")]
    RepoNotFound(String),
}

// App layer: Use anyhow with context
use anyhow::Context;

let result = operation()
    .context("Failed to perform operation")?;
```

### Logging

```rust
use tracing::{debug, info, warn, error, instrument};

// Instrument public functions
#[instrument(skip(self))]
pub fn important_function(&self, arg: String) -> Result<()> {
    info!("Starting operation");
    debug!("Processing: {}", arg);
    // ...
}

// Use structured logging
debug!(
    count = items.len(),
    duration_ms = elapsed.as_millis(),
    "Operation completed"
);
```

## How to Add New Features

### Example: Adding a New Git Operation

1. **Add to Git Layer** (`crates/git/src/repo.rs`):

```rust
impl GitRepository {
    #[instrument(skip(self))]
    pub fn get_tags(&self) -> Result<Vec<String>, GitError> {
        // Implementation
        let tags = vec![];
        tracing::debug!("Found {} tags", tags.len());
        Ok(tags)
    }
}
```

2. **Add Job Type** (`crates/app/src/job.rs`):

```rust
pub enum Job {
    OpenRepo(PathBuf),
    RefreshRepo(PathBuf),
    GetTags(PathBuf),  // New job
}
```

3. **Add Message** (`crates/app/src/message.rs`):

```rust
pub enum AppMessage {
    // ... existing messages
    TagsLoaded(Vec<String>),
}
```

4. **Update State** (`crates/app/src/state.rs`):

```rust
pub struct RepoState {
    // ... existing fields
    pub tags: Vec<String>,
}
```

5. **Update Reducer** (`crates/app/src/reducer.rs`):

```rust
pub fn reduce(state: &mut AppState, msg: AppMessage) -> Effect {
    match msg {
        // ... existing cases
        AppMessage::TagsLoaded(tags) => {
            if let Some(repo) = &mut state.current_repo {
                repo.tags = tags;
            }
            Effect::None
        }
    }
}
```

6. **Add Job Executor Handler** (`crates/app/src/executor.rs`):

```rust
match job {
    Job::OpenRepo(path) => execute_open_repo(path).await,
    Job::RefreshRepo(path) => execute_refresh_repo(path).await,
    Job::GetTags(path) => execute_get_tags(path).await,
}

async fn execute_get_tags(path: PathBuf) -> anyhow::Result<AppMessage> {
    let tags = tokio::task::spawn_blocking(move || {
        let repo = GitRepository::open(&path)?;
        repo.get_tags()
    }).await??;

    Ok(AppMessage::TagsLoaded(tags))
}
```

7. **Update UI** (`crates/ui_egui/src/main.rs`):

```rust
// Add UI elements to display tags
if !repo.tags.is_empty() {
    ui.heading("Tags");
    for tag in &repo.tags {
        ui.label(tag);
    }
}
```

8. **Add Tests**:

```rust
// Git layer test
#[test]
fn test_get_tags() {
    let (_, repo) = init_test_repo();
    let tags = repo.get_tags().unwrap();
    assert!(tags.is_empty()); // No tags initially
}

// Reducer test
#[test]
fn test_tags_loaded() {
    let mut state = state_with_open_repo();
    let tags = vec!["v1.0.0".to_string()];

    reduce(&mut state, AppMessage::TagsLoaded(tags.clone()));

    assert_eq!(state.current_repo.unwrap().tags, tags);
}
```

## Testing

### Running Tests

```bash
# All tests
cargo test --workspace

# Specific crate
cargo test -p crabontree-git

# Specific test
cargo test test_get_branches

# With logging output
RUST_LOG=debug cargo test -- --nocapture

# With backtrace
RUST_BACKTRACE=1 cargo test
```

### Writing Tests

#### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature() {
        let result = my_function();
        assert_eq!(result, expected);
    }
}
```

#### Integration Tests

```rust
// In tests/integration_test.rs
use crabontree_git::GitRepository;
use tempfile::TempDir;

#[test]
fn test_integration() {
    let temp = TempDir::new().unwrap();
    // Test with temporary directory
}
```

## Debugging

### Using RUST_LOG

```bash
# Debug all CrabOnTree code
RUST_LOG=crabontree=debug cargo run

# Trace Git operations
RUST_LOG=crabontree_git=trace cargo run

# Multiple targets
RUST_LOG=crabontree_git=debug,crabontree_app=debug cargo run
```

### Using rust-gdb or rust-lldb

```bash
# Build with debug symbols
cargo build

# Run with debugger
rust-gdb target/debug/crabontree
(gdb) run
(gdb) break main
(gdb) continue
```

### Print Debugging

```rust
// Use dbg! macro
let value = dbg!(expensive_calculation());

// Print with Debug trait
println!("{:?}", complex_structure);

// Pretty print
println!("{:#?}", complex_structure);
```

## Performance Profiling

### Using cargo-flamegraph

```bash
# Install
cargo install flamegraph

# Generate flamegraph
cargo flamegraph --bin crabontree

# Open flamegraph.svg in browser
```

### Using perf (Linux)

```bash
# Build with release + debug symbols
cargo build --release

# Record
perf record --call-graph=dwarf target/release/crabontree

# Report
perf report
```

## Common Issues and Solutions

### Issue: UI Freezing

**Symptom**: UI becomes unresponsive during Git operations

**Solution**:
- Ensure all Git operations are in async jobs
- Check that jobs use `spawn_blocking`
- Verify UI uses `try_recv()` not `recv()`

### Issue: Jobs Not Completing

**Symptom**: Repository doesn't open, no error message

**Solution**:
- Check logs with `RUST_LOG=debug`
- Verify worker thread is running
- Check for panics in job executor
- Ensure message channel is not dropped

### Issue: State Not Updating

**Symptom**: UI doesn't reflect state changes

**Solution**:
- Add logging to reducer to verify messages
- Check that effects are being executed
- Verify message is being sent from job
- Ensure `poll_messages()` is called in `update()`

### Issue: Configuration Not Saving

**Symptom**: Recent repos list doesn't persist

**Solution**:
- Check file permissions on config directory
- Verify `save_config()` is called in effect execution
- Check logs for save errors
- Ensure config directory exists

## Code Review Checklist

Before submitting code:

- [ ] Code compiles without warnings
- [ ] All tests pass
- [ ] New functionality has tests
- [ ] Public APIs are documented
- [ ] Logging added for important operations
- [ ] Error handling is appropriate
- [ ] No blocking operations in UI thread
- [ ] Git operations use `spawn_blocking`
- [ ] Code follows style guidelines
- [ ] No unnecessary dependencies added

## Git Workflow

### Commit Messages

Follow conventional commits format:

```
feat: Add tag listing functionality
fix: Correct branch sorting order
docs: Update architecture documentation
test: Add integration tests for status
refactor: Simplify error handling in Git layer
```

### Branch Strategy

- `main`: Stable code
- `develop`: Integration branch
- `feature/name`: Feature development
- `fix/name`: Bug fixes

## Resources

- [Rust Book](https://doc.rust-lang.org/book/)
- [gitoxide Documentation](https://docs.rs/gix/)
- [egui Documentation](https://docs.rs/egui/)
- [tokio Documentation](https://docs.rs/tokio/)
- [tracing Documentation](https://docs.rs/tracing/)

## Getting Help

- Check documentation in `docs/`
- Read the code - it's well-commented
- Check logs with `RUST_LOG=debug`
- Search issues on GitHub
- Ask questions in discussions
