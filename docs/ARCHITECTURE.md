# CrabOnTree Architecture

This document describes the architecture, design decisions, and technical implementation of CrabOnTree.

## Table of Contents

1. [Overview](#overview)
2. [Architecture Pattern](#architecture-pattern)
3. [System Layers](#system-layers)
4. [Data Flow](#data-flow)
5. [Crate Structure](#crate-structure)
6. [Key Components](#key-components)
7. [Design Decisions](#design-decisions)
8. [Performance Considerations](#performance-considerations)
9. [Technology Choices](#technology-choices)

---

## Overview

CrabOnTree is built using a clean, functional architecture inspired by the Elm Architecture. The application is structured into distinct layers with clear responsibilities and unidirectional data flow.

### High-Level Architecture

```
┌─────────────────────────────────────┐
│         UI Layer (egui)             │
│   - Immediate mode rendering        │
│   - User interaction handling       │
│   - View functions (pure)           │
└──────────────┬──────────────────────┘
               │ Messages (up)
               │ State (down)
┌──────────────▼──────────────────────┐
│       App Layer (Pure State)        │
│   - Reducer (pure function)         │
│   - Effects (side effect specs)     │
│   - State management                │
└──────────────┬──────────────────────┘
               │ Jobs (async specs)
               │ Messages (results)
┌──────────────▼──────────────────────┐
│      Executor (Async Runtime)       │
│   - Job execution with tokio        │
│   - Message passing back to UI      │
│   - Async/await coordination        │
└──────────────┬──────────────────────┘
               │ Git API calls
               │ Results
┌──────────────▼──────────────────────┐
│      Git Layer (gix + git2)         │
│   - gix for read operations         │
│   - git2 for write operations       │
│   - Repository abstraction          │
└─────────────────────────────────────┘
```

### Core Principles

1. **Unidirectional Data Flow**: State flows down, messages flow up
2. **Pure Functions**: State updates are deterministic and testable
3. **Effects as Data**: Side effects are represented as values, not performed directly
4. **Separation of Concerns**: Each layer has a single, clear responsibility
5. **Type Safety**: Leverage Rust's type system to prevent bugs at compile time

---

## Architecture Pattern

CrabOnTree follows the **Elm Architecture**, adapted for Rust:

### The Elm Architecture

```rust
// State: The entire application state
struct AppState {
    current_repo: Option<RepoState>,
    loading: bool,
    error: Option<String>,
}

// Messages: Events that can happen
enum AppMessage {
    RepoOpened(PathBuf),
    FileStaged(PathBuf),
    CommitCreated { hash: String },
    // ... more messages
}

// Update (Reducer): Pure function that updates state
fn reduce(state: &mut AppState, message: AppMessage) -> Effect {
    match message {
        AppMessage::FileStaged(path) => {
            // Update state
            state.current_repo.as_mut()?.stage_file(path);
            // Return effect to perform
            Effect::LoadWorkingDirStatus(repo_path)
        }
        // ... handle other messages
    }
}

// Effects: Descriptions of side effects
enum Effect {
    LoadWorkingDirStatus(PathBuf),
    StageFile { repo_path: PathBuf, file_path: PathBuf },
    None,
}

// Executor: Performs effects and returns messages
async fn execute_effect(effect: Effect) -> AppMessage {
    match effect {
        Effect::StageFile { repo_path, file_path } => {
            // Perform async git operation
            let result = stage_file_async(repo_path, file_path).await;
            // Return message with result
            AppMessage::FileStageResult(result)
        }
        // ... handle other effects
    }
}
```

### Why This Pattern?

**Benefits**:
- **Testability**: Pure reducer functions are easy to test
- **Debuggability**: All state changes go through reducer (can log/replay)
- **Predictability**: Same input always produces same output
- **Maintainability**: Clear separation of concerns
- **Correctness**: Type system ensures all messages are handled

**Trade-offs**:
- **Boilerplate**: More types and code than imperative approach
- **Learning curve**: Pattern is unfamiliar to some developers
- **Indirection**: Message → Reducer → Effect → Executor adds layers

---

## System Layers

### Layer 1: Git Layer (`crates/git/`)

**Responsibility**: Interact with Git repositories

**Key Type**: `GitRepository`

**Operations**:
```rust
// Read operations (using gix - gitoxide)
pub fn get_working_dir_status(&self) -> Result<Vec<WorkingDirFile>, GitError>
pub fn get_commit_history(&self, limit: Option<usize>) -> Result<Vec<Commit>, GitError>
pub fn get_commit_diff(&self, hash: &str) -> Result<Vec<FileDiff>, GitError>

// Write operations (using git2 - libgit2)
pub fn stage_file(&self, path: &Path) -> Result<(), GitError>
pub fn unstage_file(&self, path: &Path) -> Result<(), GitError>
pub fn stage_all(&self, paths: &[PathBuf]) -> Result<(), GitError>
pub fn create_commit(&self, message: &str) -> Result<String, GitError>
```

**Design**:
- Synchronous API (blocking operations)
- Hybrid approach: gix for reads (fast), git2 for writes (reliable)
- Rich error types with context
- Performance logging for monitoring

### Layer 2: App Layer (`crates/app/`)

**Responsibility**: Application state and logic

**Key Types**:
```rust
// Complete application state
pub struct AppState { ... }

// All possible events
pub enum AppMessage { ... }

// Side effects to perform
pub enum Effect { ... }

// Async job specifications
pub enum Job { ... }
```

**Files**:
- `state.rs` - State structures
- `message.rs` - Message types
- `effect.rs` - Effect types
- `job.rs` - Job types
- `reducer.rs` - Pure update function
- `executor.rs` - Effect execution

**Design**:
- Pure reducer (no side effects)
- Effects as immutable data structures
- Jobs executed asynchronously
- Results return as messages

### Layer 3: UI Core Layer (`crates/ui_core/`)

**Responsibility**: Framework-agnostic UI primitives

**Purpose**: Shared UI logic that could work with any GUI framework

**Current State**: Minimal (most logic in ui_egui)

**Future**: Could extract common logic from egui implementation

### Layer 4: UI Layer (`crates/ui_egui/`)

**Responsibility**: User interface with egui

**Key Type**: `CrabOnTreeApp`

**Structure**:
```rust
struct CrabOnTreeApp {
    // Application state
    state: AppState,

    // UI-specific state (not in app state)
    commit_message_buffer: String,
    working_dir_search: String,
    focused_file_index: Option<usize>,
    active_panel: ActivePanel,

    // Message channel from executor
    message_rx: Receiver<AppMessage>,
}
```

**Design**:
- Immediate mode rendering (egui)
- Handle user input → send messages
- Render based on current state
- Manage UI-specific state separately

---

## Data Flow

### Complete Flow Example: Staging a File

```
1. User clicks "+" button next to file
   ↓
2. UI layer creates message: AppMessage::StageFileRequested(path)
   ↓
3. Reducer function receives message:
   - Sets state.loading = true
   - Returns Effect::StageFile { repo_path, file_path }
   ↓
4. Executor receives effect:
   - Spawns tokio task
   - Calls git layer: repo.stage_file(path)
   - Creates result message: AppMessage::FileStaged { path, result }
   ↓
5. Result message sent to UI thread via channel
   ↓
6. Reducer receives result message:
   - Sets state.loading = false
   - If success: updates state to reflect staged file
   - If error: sets state.error = Some(error_msg)
   - Returns Effect::LoadWorkingDirStatus (refresh)
   ↓
7. UI re-renders with updated state
   - File now shows as [S] (staged)
   - Button changes to "-" (unstage)
```

### Message Flow Diagram

```
User Input
    ↓
┌───────────────────┐
│   UI Event        │ (Button click, key press)
└────────┬──────────┘
         ↓
┌────────▼──────────┐
│   AppMessage      │ (StageFileRequested)
└────────┬──────────┘
         ↓
┌────────▼──────────┐
│   Reducer         │ (Pure function)
│   - Update state  │
│   - Return effect │
└────────┬──────────┘
         ↓
┌────────▼──────────┐
│   Effect          │ (StageFile spec)
└────────┬──────────┘
         ↓
┌────────▼──────────┐
│   Executor        │ (Async runtime)
│   - Execute job   │
│   - Call git      │
└────────┬──────────┘
         ↓
┌────────▼──────────┐
│   Git Layer       │ (git2 operation)
└────────┬──────────┘
         ↓
┌────────▼──────────┐
│   Result Message  │ (FileStaged)
└────────┬──────────┘
         ↓
┌────────▼──────────┐
│   Reducer         │ (Handle result)
└────────┬──────────┘
         ↓
┌────────▼──────────┐
│   UI Re-render    │ (New state)
└───────────────────┘
```

---

## Crate Structure

### Dependency Graph

```
ui_egui (binary crate)
    ├── depends on: app
    ├── depends on: git
    └── depends on: egui, tokio

app (library crate)
    ├── depends on: git
    └── depends on: tokio, anyhow, tracing

git (library crate)
    ├── depends on: gix
    ├── depends on: git2
    └── depends on: thiserror, tracing

ui_core (library crate)
    └── minimal dependencies (future use)
```

### Why This Structure?

**Benefits**:
- **Modularity**: Each crate can be developed/tested independently
- **Reusability**: Git and app layers can be used without egui
- **Compilation**: Parallel compilation of independent crates
- **Testing**: Focused unit tests per crate

**Structure Details**:

**crates/git/**:
- No app/UI dependencies
- Can be used standalone
- 19 integration tests
- Handles all git operations

**crates/app/**:
- Depends on git layer
- No UI dependencies
- 14 unit/integration tests
- Framework-agnostic state management

**crates/ui_core/**:
- Future home for shared UI logic
- Currently minimal
- Intended for multi-framework support

**crates/ui_egui/**:
- Binary crate (produces executable)
- Depends on everything
- Implements concrete UI
- Main entry point

---

## Key Components

### 1. GitRepository

**Location**: `crates/git/src/repo.rs`

**Purpose**: Abstraction over git operations

**Key Design**:
```rust
pub struct GitRepository {
    gix_repo: gix::Repository,      // For fast reads
    git2_repo: git2::Repository,     // For reliable writes
    path: PathBuf,
}
```

**Rationale**:
- gix (gitoxide): Pure Rust, very fast for reads, safe
- git2 (libgit2 bindings): Mature, reliable for writes
- Hybrid approach gets best of both worlds

### 2. Reducer

**Location**: `crates/app/src/reducer.rs`

**Purpose**: Pure state update function

**Signature**:
```rust
pub fn reduce(state: &mut AppState, message: AppMessage) -> Effect
```

**Constraints**:
- **Pure**: Same input always produces same output
- **No I/O**: Cannot do file/network operations
- **No side effects**: Cannot modify anything outside state
- **Synchronous**: Cannot await async operations

**Why Pure?**:
- **Testable**: Easy to write unit tests
- **Debuggable**: Can log all state transitions
- **Replayable**: Can replay messages to reproduce state
- **Predictable**: No hidden state or surprises

### 3. Executor

**Location**: `crates/app/src/executor.rs`

**Purpose**: Execute effects asynchronously

**Design**:
```rust
pub async fn execute_effect(effect: Effect) -> anyhow::Result<AppMessage> {
    match effect {
        Effect::StageFile { repo_path, file_path } => {
            tokio::task::spawn_blocking(move || {
                // Git operations on thread pool
                let repo = GitRepository::open(&repo_path)?;
                repo.stage_file(&file_path)?;
                Ok(AppMessage::FileStaged { ... })
            }).await?
        }
        // ... handle other effects
    }
}
```

**Key Points**:
- Uses tokio for async runtime
- `spawn_blocking` for CPU-intensive git operations
- Returns messages with results
- Errors converted to error messages

### 4. Message Channel

**Location**: `crates/ui_egui/src/main.rs`

**Purpose**: Communication between executor and UI thread

**Design**:
```rust
let (message_tx, message_rx) = tokio::sync::mpsc::unbounded_channel();

// Executor sends messages
message_tx.send(result_message).unwrap();

// UI receives messages
while let Ok(message) = self.message_rx.try_recv() {
    let effect = reduce(&mut self.state, message);
    // ... handle effect
}
```

**Why Async Channel?**:
- UI thread doesn't block waiting for results
- Multiple effects can execute concurrently
- Results processed on next frame

---

## Design Decisions

### 1. Why Elm Architecture?

**Considered Alternatives**:
- **MVC (Model-View-Controller)**: Traditional, but mutable shared state
- **MVVM (Model-View-ViewModel)**: Good for data binding, complex for Rust
- **Direct callbacks**: Simple but hard to test/maintain

**Chosen**: Elm Architecture

**Rationale**:
- **Pure reducers**: Easy to test, debug, reason about
- **Explicit state**: All state in one place
- **Type safety**: Rust enums ensure all messages handled
- **Community**: Proven pattern (Redux, Elm, etc.)

### 2. Why Hybrid Git Libraries?

**Considered**:
- **git2 only**: Mature, reliable, but C dependency
- **gix only**: Pure Rust, fast, but newer (less battle-tested for writes)

**Chosen**: Hybrid (gix reads, git2 writes)

**Rationale**:
- gix read performance is excellent
- git2 write reliability is proven
- Get best of both worlds
- Can swap implementations per-operation

### 3. Why Immediate Mode GUI (egui)?

**Considered**:
- **Native toolkit** (GTK, Qt): Platform-native, but heavy, C++ bindings
- **Retained mode** (Iced, Druid): Rust-native, but more complex state management
- **Web tech** (Tauri, Electron): Cross-platform, but heavy, JS dependency

**Chosen**: egui (immediate mode)

**Rationale**:
- **Simple mental model**: Render based on current state
- **Pure Rust**: No C/C++ dependencies
- **Performant**: 60 FPS rendering
- **Flexible**: Easy to customize
- **Lightweight**: Small binary size

### 4. Why Async with Tokio?

**Considered**:
- **Blocking operations**: Simple, but freezes UI
- **Thread pool**: Manual thread management, complex
- **async-std**: Alternative runtime

**Chosen**: Tokio async runtime

**Rationale**:
- **Non-blocking**: UI stays responsive
- **Efficient**: Work-stealing task scheduler
- **Standard**: Most popular Rust async runtime
- **Integration**: Good egui integration via channels

### 5. Why Virtual Scrolling?

**Considered**:
- **Render all items**: Simple, but slow with 1000+ items
- **Pagination**: Good for web, awkward for desktop
- **Virtual scrolling**: Complex, but performant

**Chosen**: Virtual scrolling (egui's `show_rows`)

**Rationale**:
- **Performance**: O(visible) instead of O(total)
- **Built-in**: egui provides `show_rows` helper
- **Smooth**: Handles 10,000+ items easily
- **Transparent**: Users don't notice virtualization

### 6. Why Chunked Batch Operations?

**Problem**: Staging 1000+ files in one go is slow

**Solution**: Process in chunks of 500

**Implementation**:
```rust
const CHUNK_SIZE: usize = 500;
for chunk in paths.chunks(CHUNK_SIZE) {
    stage_multiple_files(chunk)?;
}
```

**Rationale**:
- **Responsiveness**: UI updates between chunks
- **Progress**: Can show progress per chunk
- **Memory**: Don't hold all results in memory
- **Cancellation**: Can cancel between chunks

---

## Performance Considerations

### 1. Virtual Scrolling

**Where**: Working directory list, commit history list

**Benefit**: Constant rendering time regardless of item count

**How**: egui's `show_rows()` only renders visible items

### 2. Lazy Loading

**Where**: Commit diffs

**Benefit**: Don't load diffs until user selects commit

**How**: Diff only loaded on `Enter` or click, not on focus/navigation

### 3. Async Operations

**Where**: All git operations

**Benefit**: UI never freezes

**How**: tokio runtime + spawn_blocking for git calls

### 4. Collapsible Diffs

**Where**: Commit diff display

**Benefit**: Don't render large file contents until expanded

**How**: Files collapsed by default, expand on click

### 5. Smart Caching

**Where**: Working directory status, commit history

**Benefit**: Don't reload unless something changed

**How**: Only reload on explicit refresh or after operations

### 6. Local Search/Filter

**Where**: File search, commit search

**Benefit**: Instant filtering (no I/O)

**How**: Client-side substring matching on already-loaded data

### Performance Characteristics

| Operation | Time | Notes |
|-----------|------|-------|
| Load 1,000 files | < 1s | Virtual scrolling makes rendering instant |
| Load 500 commits | < 500ms | Lazy diff loading keeps it fast |
| Stage single file | < 100ms | git2 operation + refresh |
| Stage 1,000 files | < 2s | Chunked processing (500/chunk) |
| Search/filter | < 100ms | Local data, instant |
| Load large diff (50 files) | < 1s | Collapsed by default |
| Scrolling | 60 FPS | Virtual scrolling ensures smoothness |

---

## Technology Choices

### Core Technologies

**Language**: Rust 2021 Edition
- **Why**: Memory safety, performance, excellent ecosystem
- **Trade-off**: Longer compile times, steeper learning curve

**GUI Framework**: egui
- **Why**: Pure Rust, immediate mode, excellent performance
- **Trade-off**: Not native-looking, newer ecosystem

**Git Libraries**: gix + git2
- **Why**: Fast reads (gix) + reliable writes (git2)
- **Trade-off**: Two dependencies instead of one

**Async Runtime**: tokio
- **Why**: Industry standard, excellent performance, good docs
- **Trade-off**: Adds complexity, binary size

**Logging**: tracing
- **Why**: Structured logging, good async support, extensible
- **Trade-off**: More complex than simple println!

### Dependencies

**Production Dependencies**:
```toml
[dependencies]
egui = "0.28"                 # GUI framework
eframe = "0.28"               # egui application framework
tokio = { version = "1", features = ["full"] }  # Async runtime
gix = "0.66"                  # Pure Rust git (gitoxide)
git2 = "0.19"                 # libgit2 bindings
tracing = "0.1"               # Structured logging
tracing-subscriber = "0.3"    # Logging subscriber
anyhow = "1.0"                # Error handling
thiserror = "2.0"             # Error derive macros
whoami = "1.5"                # System user info (fallback author)
```

**Dev Dependencies**:
```toml
[dev-dependencies]
tempfile = "3.5"              # Temporary directories for tests
```

### Why These Choices?

**egui over native toolkits**:
- Pure Rust (no C/C++ FFI)
- Simple API
- Great performance
- Cross-platform without platform-specific code

**gix + git2 over just one**:
- gix: Faster reads, pure Rust
- git2: Proven writes, mature
- Best of both worlds

**tokio over async-std**:
- More popular (better docs, examples)
- Better ecosystem integration
- Proven at scale

**tracing over log**:
- Structured logging (key-value pairs)
- Async-aware
- Better filtering and formatting

---

## Future Considerations

### Potential Improvements

**1. Extract UI Core**:
- Move shared logic from ui_egui to ui_core
- Enable alternative UI implementations

**2. Plugin System**:
- Allow extending functionality
- Could use WebAssembly for sandboxing

**3. Performance Dashboard**:
- Real-time performance metrics
- Memory usage visualization
- Operation timing graphs

**4. Incremental Loading**:
- Load data in background
- Progressive rendering
- Better for very large repositories

**5. Advanced Caching**:
- Persist cache across sessions
- Intelligent invalidation
- Reduce repeated git operations

### Scalability Limits

**Current Limits** (tested):
- ✅ 1,000 files: Smooth
- ✅ 500 commits: Smooth
- ✅ Large diffs (50 files, 5000 lines): Smooth

**Theoretical Limits**:
- **Files**: Virtual scrolling handles 10,000+ easily
- **Commits**: Limited to 100 by default, could increase
- **Diff size**: Collapsible sections mitigate large diffs
- **Memory**: Reasonable (150-250 MB typical)

**Optimization Opportunities** (if needed):
- Pagination for extreme file counts (10,000+)
- Streaming for very large diffs
- Background indexing for search
- Commit dehydration (minimal metadata initially)

---

## Conclusion

CrabOnTree's architecture balances:
- **Simplicity**: Easy to understand and modify
- **Performance**: Handles large repositories smoothly
- **Correctness**: Type system prevents many bugs
- **Maintainability**: Clear separation of concerns

The Elm-inspired architecture provides a solid foundation for future enhancements while keeping the codebase clean and testable.

**Key Strengths**:
- Pure reducer functions (easy to test)
- Unidirectional data flow (easy to reason about)
- Hybrid git approach (fast and reliable)
- Async operations (responsive UI)
- Virtual scrolling (handles large data sets)

**Key Trade-offs**:
- More boilerplate than imperative style
- Learning curve for Elm pattern
- Two git libraries instead of one
- Async complexity

Overall, the architecture successfully delivers a fast, reliable, maintainable Git GUI.

---

**For more details, see**:
- [Contributing Guide](CONTRIBUTING.md) - How to modify the architecture
- [Testing Guide](TESTING.md) - How to test components
- [User Guide](USER_GUIDE.md) - How the application works from user perspective
