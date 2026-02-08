# Testing Guide

This document describes the testing strategy, infrastructure, and guidelines for CrabOnTree.

## Table of Contents

1. [Overview](#overview)
2. [Testing Strategy](#testing-strategy)
3. [Test Structure](#test-structure)
4. [Running Tests](#running-tests)
5. [Writing Tests](#writing-tests)
6. [Test Coverage](#test-coverage)
7. [Performance Testing](#performance-testing)
8. [Manual Testing](#manual-testing)
9. [CI/CD Integration](#cicd-integration)

---

## Overview

CrabOnTree has a comprehensive test suite covering:
- **38 automated tests** across the workspace
- **Unit tests** for individual functions
- **Integration tests** for complete workflows
- **Performance tests** for large repositories
- **Manual test scenarios** for UI validation

### Test Distribution

| Crate | Tests | Focus |
|-------|-------|-------|
| `crabontree-git` | 19 | Git operations, staging, commits, diffs |
| `crabontree-app` | 14 | State management, reducers, effects, jobs |
| `crabontree-ui_core` | 5 | UI primitives (minimal currently) |
| **Total** | **38** | **Complete workflow coverage** |

---

## Testing Strategy

### Testing Pyramid

```
           ┌─────────────┐
           │   Manual    │  UI/UX validation
           │   Testing   │  Edge cases
           └─────────────┘
          ┌───────────────┐
          │  Integration  │  Complete workflows
          │     Tests     │  End-to-end scenarios
          └───────────────┘
        ┌───────────────────┐
        │    Unit Tests     │  Individual functions
        │                   │  Pure logic
        └───────────────────┘
```

**Unit Tests** (Base):
- Test individual functions in isolation
- Fast, focused, deterministic
- No external dependencies
- Example: Testing reducer pure functions

**Integration Tests** (Middle):
- Test complete workflows
- Multiple components working together
- Real git operations with temp repositories
- Example: Stage file → commit → verify

**Manual Tests** (Top):
- UI/UX validation
- Visual verification
- Performance with real repositories
- Example: Testing with 1,000+ file repository

### Test Types

**1. Unit Tests**

Test individual functions in isolation:
```rust
#[test]
fn test_reducer_stage_file() {
    let mut state = AppState::new();
    // Test pure reducer logic
    let effect = reduce(&mut state, AppMessage::StageFileRequested(path));
    assert!(matches!(effect, Effect::StageFile { .. }));
}
```

**2. Integration Tests**

Test complete workflows:
```rust
#[test]
fn test_complete_commit_workflow() {
    // 1. Create test repository
    let (temp_dir, repo) = init_test_repo();

    // 2. Make changes
    create_test_file(&temp_dir, "test.txt", "content");

    // 3. Stage files
    repo.stage_file(Path::new("test.txt")).unwrap();

    // 4. Create commit
    let hash = repo.create_commit("Test commit").unwrap();

    // 5. Verify commit exists
    assert_eq!(hash.len(), 40);

    // 6. Verify working dir clean
    let status = repo.get_working_dir_status().unwrap();
    assert!(status.is_empty());
}
```

**3. Performance Tests**

Validate performance with large data:
```rust
#[test]
fn test_large_repository_performance() {
    // Create repo with 1,000 files
    let repo = create_large_test_repo(1000);

    // Measure load time
    let start = Instant::now();
    let status = repo.get_working_dir_status().unwrap();
    let elapsed = start.elapsed();

    // Verify performance
    assert!(elapsed < Duration::from_secs(2));
    assert_eq!(status.len(), 1000);
}
```

---

## Test Structure

### Directory Layout

```
crates/git/
├── src/
│   ├── lib.rs
│   ├── repo.rs
│   └── error.rs
└── tests/
    └── integration_test.rs    # Git operation tests

crates/app/
├── src/
│   ├── lib.rs
│   ├── state.rs
│   ├── reducer.rs
│   └── ...
└── tests/
    ├── reducer_test.rs        # Unit tests for reducer
    └── integration_test.rs    # Workflow tests

scripts/
└── create_test_repos.sh       # Generate test repositories

test-repos/                    # Generated (gitignored)
├── large-files/               # 1,000 files
├── long-history/              # 500 commits
└── large-diff/                # Large commit
```

### Test Organization

**Git Layer Tests** (`crates/git/tests/integration_test.rs`):
- Repository operations
- Working directory status
- Staging/unstaging
- Commit creation
- Diff generation
- Error scenarios

**App Layer Tests**:
- `reducer_test.rs`: Pure reducer function tests
- `integration_test.rs`: Complete message → effect → job flows

---

## Running Tests

### Basic Commands

```bash
# Run all tests
cargo test --workspace

# Run tests for specific crate
cargo test -p crabontree-git
cargo test -p crabontree-app

# Run specific test
cargo test test_stage_file

# Run with output (see println!, tracing)
cargo test -- --nocapture

# Run with logging
RUST_LOG=debug cargo test -- --nocapture

# Run in release mode (faster, for performance tests)
cargo test --workspace --release
```

### Advanced Commands

```bash
# Run tests matching pattern
cargo test stage

# Run only integration tests
cargo test --test integration_test

# Run single-threaded (for debugging)
cargo test -- --test-threads=1

# Show test execution time
cargo test -- --nocapture --show-output

# Run ignored tests
cargo test -- --ignored

# List tests without running
cargo test -- --list
```

### Using cargo-nextest (Faster)

```bash
# Install nextest
cargo install cargo-nextest

# Run all tests (faster than cargo test)
cargo nextest run --workspace

# Run with retries for flaky tests
cargo nextest run --retries 3

# Run only failed tests
cargo nextest run --failed
```

### Watch Mode

Automatically run tests on file changes:

```bash
# Install cargo-watch
cargo install cargo-watch

# Watch and test on changes
cargo watch -x 'test --workspace'

# Watch specific crate
cargo watch -x 'test -p crabontree-git'

# Watch, check, and test
cargo watch -x check -x 'test --workspace'
```

---

## Writing Tests

### Test Template

**Basic Test**:
```rust
#[test]
fn test_function_name() {
    // Arrange: Set up test data
    let input = "test data";

    // Act: Perform operation
    let result = function_under_test(input);

    // Assert: Verify result
    assert_eq!(result, expected_value);
}
```

**Integration Test with Temp Repo**:
```rust
#[test]
fn test_git_operation() {
    // Create temporary test repository
    let (temp_dir, repo) = init_test_repo();
    let repo_path = temp_dir.path();

    // Create test file
    std::fs::write(repo_path.join("test.txt"), "content").unwrap();

    // Execute git operation
    repo.stage_file(Path::new("test.txt")).unwrap();

    // Verify result
    let status = repo.get_working_dir_status().unwrap();
    assert!(status.iter().any(|f| f.is_staged));

    // temp_dir automatically cleaned up when dropped
}
```

### Test Helpers

**Common Helper Functions** (`crates/git/tests/integration_test.rs`):

```rust
/// Initialize a test repository with basic config
fn init_test_repo() -> (TempDir, GitRepository) {
    let temp_dir = tempfile::tempdir().unwrap();
    let repo_path = temp_dir.path();

    // Initialize git repository
    Command::new("git")
        .args(["init"])
        .current_dir(repo_path)
        .output()
        .unwrap();

    // Set git config (required for commits)
    Command::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(repo_path)
        .output()
        .unwrap();

    Command::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(repo_path)
        .output()
        .unwrap();

    // Create initial commit (required for diffs)
    std::fs::write(repo_path.join("README.md"), "# Test").unwrap();
    Command::new("git")
        .args(["add", "."])
        .current_dir(repo_path)
        .output()
        .unwrap();
    Command::new("git")
        .args(["commit", "-m", "Initial commit"])
        .current_dir(repo_path)
        .output()
        .unwrap();

    let repo = GitRepository::open(repo_path).unwrap();
    (temp_dir, repo)
}

/// Create a test file and return its path
fn create_test_file(temp_dir: &TempDir, name: &str, content: &str) -> PathBuf {
    let path = temp_dir.path().join(name);
    std::fs::write(&path, content).unwrap();
    path
}

/// Stage a file via command line (for testing)
fn stage_file_via_cli(repo_path: &Path, file: &str) {
    Command::new("git")
        .args(["add", file])
        .current_dir(repo_path)
        .output()
        .unwrap();
}
```

### Testing Error Cases

**Test Expected Errors**:
```rust
#[test]
fn test_commit_without_staged_files() {
    let (_temp_dir, repo) = init_test_repo();

    // Try to commit with no staged files
    let result = repo.create_commit("Test message");

    // Verify it fails with expected error
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.to_string().contains("No staged changes"));
}
```

**Test Error Recovery**:
```rust
#[test]
fn test_error_recovery() {
    let mut state = AppState::new();

    // Simulate error
    let effect = reduce(&mut state, AppMessage::OperationFailed("Test error".to_string()));

    // Verify error is stored
    assert!(state.error.is_some());
    assert_eq!(state.error.as_ref().unwrap(), "Test error");

    // Verify loading is stopped
    assert!(!state.loading);
}
```

### Testing Async Code

**Async Test with Tokio**:
```rust
#[tokio::test]
async fn test_async_operation() {
    let result = execute_async_job(params).await;

    assert!(result.is_ok());
    let message = result.unwrap();
    assert!(matches!(message, AppMessage::Success { .. }));
}
```

---

## Test Coverage

### Current Coverage

**Git Layer** (19 tests):
- ✅ Repository opening
- ✅ Working directory status
- ✅ File staging (individual and batch)
- ✅ File unstaging
- ✅ Commit creation
- ✅ Author identity loading
- ✅ Commit history
- ✅ Commit diffs
- ✅ Error cases (no staged files, empty message)

**App Layer** (14 tests):
- ✅ Reducer state updates
- ✅ Effect generation
- ✅ Message handling
- ✅ Error propagation
- ✅ State consistency
- ✅ Integration workflows

**UI Layer**:
- ⚠️ Limited automated tests (egui is hard to test)
- ✅ Manual testing via usage
- ✅ Visual verification

### Coverage Goals

**Critical Paths** (100% target):
- [x] Staging operations
- [x] Commit creation
- [x] Working directory status
- [x] Commit history loading
- [x] Diff generation

**Error Handling** (80% target):
- [x] Git operation errors
- [x] Invalid repository paths
- [x] Empty commit messages
- [x] No staged files
- [ ] Corrupted repositories (future)
- [ ] Permission denied (future)
- [ ] Disk full (future)

**Edge Cases** (70% target):
- [x] Empty repositories
- [x] Large file counts
- [x] Long commit history
- [ ] Binary files (future)
- [ ] Submodules (future)
- [ ] Merge conflicts (future)

### Measuring Coverage

**Using tarpaulin** (Linux only):
```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Generate coverage report
cargo tarpaulin --workspace --out Html --output-dir coverage

# View report
open coverage/index.html
```

**Using llvm-cov** (Cross-platform):
```bash
# Install llvm-cov
cargo install cargo-llvm-cov

# Generate coverage report
cargo llvm-cov --workspace --html

# View report
open target/llvm-cov/html/index.html
```

---

## Performance Testing

### Test Repository Generation

**Create Test Repositories**:
```bash
# Generate three test repositories
./scripts/create_test_repos.sh

# Creates:
# - test-repos/large-files/     (1,000 modified files)
# - test-repos/long-history/    (500 commits)
# - test-repos/large-diff/      (50 files, ~5,000 lines)
```

### Performance Test Scenarios

**Scenario 1: Large Working Directory**
```bash
# 1. Open test-repos/large-files
# 2. Measure load time (check logs)
RUST_LOG=crabontree_git=info cargo run --release

# Expected: < 1 second load time
# Expected: Smooth scrolling through 1,000 files
```

**Scenario 2: Long Commit History**
```bash
# 1. Open test-repos/long-history
# 2. Load commit history
# 3. Scroll through commits
# 4. Search commits

# Expected: < 500ms load time for 100 commits
# Expected: Smooth scrolling
# Expected: Instant search filtering
```

**Scenario 3: Large Diff**
```bash
# 1. Open test-repos/large-diff
# 2. View commit with 50 files
# 3. Expand file sections

# Expected: < 1 second diff load
# Expected: Smooth scrolling
# Expected: Fast file expansion
```

### Performance Benchmarks

**Create Benchmark Tests** (future):
```rust
#[bench]
fn bench_stage_1000_files(b: &mut Bencher) {
    let repo = create_large_test_repo(1000);
    let paths: Vec<PathBuf> = get_all_file_paths(&repo);

    b.iter(|| {
        repo.stage_all(&paths).unwrap();
    });
}
```

**Run Benchmarks**:
```bash
# Benchmarks require nightly Rust
rustup install nightly

# Run benchmarks
cargo +nightly bench --workspace
```

### Performance Monitoring

**Enable Performance Logging**:
```rust
// Already implemented in git layer
let start = std::time::Instant::now();
let result = expensive_operation()?;
let elapsed = start.elapsed();
tracing::info!("operation completed: {} items in {:?}", count, elapsed);
```

**View Performance Logs**:
```bash
# Run with info-level logging
RUST_LOG=crabontree_git=info cargo run --release

# Look for timing messages:
# INFO get_working_dir_status completed: 1000 files in 245ms
# INFO get_commit_history completed: 100 commits in 157ms
# INFO get_commit_diff completed: 50 files in 423ms
```

---

## Manual Testing

### Pre-Release Checklist

**Basic Operations**:
- [ ] Open repository successfully
- [ ] View working directory status
- [ ] Stage individual file
- [ ] Unstage individual file
- [ ] Stage all files
- [ ] Unstage all files
- [ ] Write commit message
- [ ] Create commit successfully
- [ ] View commit in history

**Search Functionality**:
- [ ] File search filters correctly
- [ ] Commit search finds by message
- [ ] Commit search finds by author
- [ ] Commit search finds by hash
- [ ] Search clears with Esc

**Keyboard Shortcuts**:
- [ ] `1`, `2`, `3` switch panels
- [ ] `j`, `k` navigate lists
- [ ] `Space` toggles staging
- [ ] `Enter` views commit
- [ ] `a` stages all
- [ ] `u` unstages all
- [ ] `c` focuses commit message
- [ ] `Ctrl+Enter` creates commit
- [ ] `/` focuses file search
- [ ] `Ctrl+F` focuses commit search
- [ ] `r` refreshes repository
- [ ] `?` shows help

**Edge Cases**:
- [ ] Empty repository
- [ ] Repository with no changes
- [ ] Very long commit message (1000+ chars)
- [ ] Very long file name
- [ ] Binary files
- [ ] Large files (10MB+)
- [ ] Many files (1000+)
- [ ] Long history (500+ commits)

**Error Handling**:
- [ ] Try to commit with no staged files
- [ ] Try to commit with empty message
- [ ] Open invalid repository path
- [ ] Open non-git directory
- [ ] Handle permission denied

**Performance**:
- [ ] Large repository (1000+ files) loads smoothly
- [ ] Scrolling is smooth (60 FPS)
- [ ] Search is instant
- [ ] No UI freezes during operations

### Testing on Multiple Platforms

**Linux**:
- [ ] Build succeeds
- [ ] All tests pass
- [ ] Application runs
- [ ] UI renders correctly

**macOS**:
- [ ] Build succeeds
- [ ] All tests pass
- [ ] Application runs
- [ ] Keyboard shortcuts work (Cmd vs Ctrl)
- [ ] Native look/feel

**Windows**:
- [ ] Build succeeds
- [ ] All tests pass
- [ ] Application runs
- [ ] UI renders correctly
- [ ] File path handling (backslashes)

---

## CI/CD Integration

### Continuous Integration

**GitHub Actions Workflow** (example):
```yaml
name: CI

on: [push, pull_request]

jobs:
  test:
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
        rust: [stable, beta]

    steps:
      - uses: actions/checkout@v3

      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: ${{ matrix.rust }}
          override: true

      - name: Build
        run: cargo build --workspace --verbose

      - name: Run tests
        run: cargo test --workspace --verbose

      - name: Check formatting
        run: cargo fmt --all -- --check

      - name: Run clippy
        run: cargo clippy --workspace -- -D warnings
```

### Pre-Commit Hooks

**Git Hook** (`.git/hooks/pre-commit`):
```bash
#!/bin/bash

# Run tests before commit
cargo test --workspace --quiet

if [ $? -ne 0 ]; then
    echo "Tests failed! Commit aborted."
    exit 1
fi

# Check formatting
cargo fmt --all -- --check

if [ $? -ne 0 ]; then
    echo "Code not formatted! Run 'cargo fmt --all' first."
    exit 1
fi

echo "All checks passed!"
```

**Install hook**:
```bash
chmod +x .git/hooks/pre-commit
```

### Automated Performance Tests

**Weekly Performance Run** (example):
```yaml
name: Performance

on:
  schedule:
    - cron: '0 0 * * 0'  # Weekly on Sunday

jobs:
  performance:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Generate test repositories
        run: ./scripts/create_test_repos.sh

      - name: Build release
        run: cargo build --release

      - name: Run performance tests
        run: |
          RUST_LOG=info cargo run --release 2>&1 | tee performance.log

      - name: Upload results
        uses: actions/upload-artifact@v3
        with:
          name: performance-log
          path: performance.log
```

---

## Best Practices

### Test Writing

1. **One Assertion Per Test**: Makes failures clear
2. **Descriptive Names**: `test_stage_file_success` not `test1`
3. **Arrange-Act-Assert**: Clear structure in every test
4. **Use Helpers**: Reduce duplication with helper functions
5. **Clean Up**: Use TempDir for automatic cleanup
6. **Test Data**: Use realistic test data
7. **Fast Tests**: Keep unit tests fast (< 100ms)

### Test Maintenance

1. **Update Tests with Code**: Tests are documentation
2. **Fix Flaky Tests**: Don't ignore intermittent failures
3. **Refactor Tests**: DRY principle applies to tests too
4. **Remove Dead Tests**: Delete tests for removed features
5. **Review Coverage**: Check coverage reports regularly

### Common Pitfalls

**Avoid**:
- Tests that depend on external state
- Tests that require manual setup
- Tests that are slow (> 1s for unit tests)
- Tests with hidden assertions (in helpers)
- Tests that are flaky (pass/fail randomly)

**Prefer**:
- Isolated tests (TempDir, fresh state)
- Self-contained tests (all setup in test)
- Fast tests (mock expensive operations)
- Clear assertions (in test body)
- Deterministic tests (same result every time)

---

## Debugging Tests

### Failing Tests

**Get More Information**:
```bash
# Show output
cargo test test_name -- --nocapture

# Show logs
RUST_LOG=debug cargo test test_name -- --nocapture

# Run single-threaded
cargo test test_name -- --test-threads=1
```

**Use Debugger**:
```bash
# Run test under gdb (Linux)
rust-gdb --args target/debug/crabontree-git-<hash> test_name

# Run test under lldb (macOS)
rust-lldb target/debug/crabontree-git-<hash> test_name
```

### Flaky Tests

**Identify**:
```bash
# Run test multiple times
for i in {1..100}; do cargo test test_name || break; done
```

**Common Causes**:
- Race conditions (async timing)
- External state dependencies
- Timing assumptions
- Random data

**Solutions**:
- Use deterministic data
- Add proper synchronization
- Increase timeouts (if timing-sensitive)
- Isolate test from external state

---

## Future Testing Improvements

**Planned Enhancements**:
- [ ] UI component tests (if testing framework improves)
- [ ] Automated visual regression testing
- [ ] Fuzz testing for git operations
- [ ] Property-based testing for reducers
- [ ] Benchmark suite with criterion
- [ ] Code coverage targets in CI
- [ ] Automated performance regression detection

---

## Resources

### Documentation
- [Rust Testing Guide](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [cargo test Documentation](https://doc.rust-lang.org/cargo/commands/cargo-test.html)
- [Tokio Testing](https://tokio.rs/tokio/topics/testing)

### Tools
- [cargo-nextest](https://nexte.st/) - Faster test runner
- [cargo-watch](https://github.com/watchexec/cargo-watch) - Auto-run tests
- [cargo-tarpaulin](https://github.com/xd009642/tarpaulin) - Coverage tool
- [cargo-llvm-cov](https://github.com/taiki-e/cargo-llvm-cov) - Coverage tool

---

**Happy testing! 🧪🦀🌳**
