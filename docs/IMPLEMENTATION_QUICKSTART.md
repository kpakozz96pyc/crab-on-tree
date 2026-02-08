# 4-Pane Layout - Implementation Quick Start

This guide helps you get started implementing the 4-pane layout.

## Prerequisites

- Read: `DESIGN_4_PANE_LAYOUT.md` (full design document)
- Read: `DESIGN_COMPARISON.md` (current vs new comparison)
- Understand: Current Elm Architecture (see `ARCHITECTURE.md`)

## Phase 1: Data Model Setup (Start Here)

### Step 1.1: Define Branch Tree State

**File**: `crates/app/src/state.rs`

Add these types after the existing state definitions:

```rust
/// Branch information for tree display.
#[derive(Debug, Clone)]
pub struct BranchInfo {
    pub name: String,
    pub commit_hash: String,
    pub is_current: bool,
    pub upstream: Option<String>,
}

/// Tag information.
#[derive(Debug, Clone)]
pub struct TagInfo {
    pub name: String,
    pub commit_hash: String,
    pub message: Option<String>, // For annotated tags
}

/// State of the branch/tag tree navigator.
#[derive(Debug, Clone)]
pub struct BranchTreeState {
    pub local_branches: Vec<BranchInfo>,
    pub remote_branches: std::collections::HashMap<String, Vec<BranchInfo>>,
    pub tags: Vec<TagInfo>,
    pub current_branch: String,
    pub expanded_sections: std::collections::HashSet<String>,
}
```

### Step 1.2: Define File Tree State

**File**: `crates/app/src/state.rs`

```rust
/// A node in the file tree.
#[derive(Debug, Clone)]
pub enum FileTreeNode {
    Directory {
        path: PathBuf,
        name: String,
        children: Vec<FileTreeNode>,
        is_expanded: bool,
        has_changes: bool,
    },
    File {
        path: PathBuf,
        name: String,
        status: Option<crabontree_git::WorkingDirStatus>,
        size: u64,
    },
}

/// State of the file tree navigator.
#[derive(Debug, Clone)]
pub struct FileTreeState {
    pub root: FileTreeNode,
    pub expanded_paths: std::collections::HashSet<PathBuf>,
    pub selected_path: Option<PathBuf>,
}
```

### Step 1.3: Define File Viewer State

**File**: `crates/app/src/state.rs`

```rust
/// Diff view mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiffViewMode {
    Unified,
    SideBySide,
    ContentOnly,
}

/// State of the file content/diff viewer.
#[derive(Debug, Clone)]
pub enum FileViewState {
    None,
    Content {
        path: PathBuf,
        content: String,
        language: Option<String>,
    },
    Diff {
        path: PathBuf,
        hunks: Vec<crabontree_git::DiffHunk>,
        view_mode: DiffViewMode,
    },
    Binary {
        path: PathBuf,
        size: u64,
    },
}

impl Default for FileViewState {
    fn default() -> Self {
        Self::None
    }
}
```

### Step 1.4: Update AppState

**File**: `crates/app/src/state.rs`

Modify the `RepoState` struct to include new state:

```rust
/// State of an open repository.
#[derive(Debug, Clone)]
pub struct RepoState {
    pub path: PathBuf,
    pub head: String,
    pub branches: Vec<String>,
    pub status_summary: StatusSummary,
    pub commits: Vec<Commit>,
    pub selected_commit: Option<String>,
    pub commit_diff: Option<Vec<FileDiff>>,
    pub working_dir_files: Vec<WorkingDirFile>,
    pub commit_message: String,
    pub author_name: String,
    pub author_email: String,

    // NEW: 4-pane layout state
    pub branch_tree: BranchTreeState,
    pub file_tree: FileTreeState,
    pub file_view: FileViewState,
}
```

### Step 1.5: Add UI State for Panes

**File**: `crates/ui_egui/src/main.rs`

Add to `CrabOnTreeApp` struct:

```rust
struct CrabOnTreeApp {
    // ... existing fields ...

    // NEW: Pane configuration
    pane_widths: [f32; 4],  // Relative widths [0.15, 0.25, 0.20, 0.40]
    active_pane: usize,     // 0-3
}
```

Update `CrabOnTreeApp::new()`:

```rust
fn new(_cc: &eframe::CreationContext<'_>) -> Self {
    // ... existing code ...

    Self {
        // ... existing fields ...
        pane_widths: [0.15, 0.25, 0.20, 0.40],
        active_pane: 0,
    }
}
```

## Phase 2: Add Messages

**File**: `crates/app/src/message.rs`

Add new message variants:

```rust
#[derive(Debug, Clone)]
pub enum AppMessage {
    // ... existing messages ...

    // Branch tree messages
    BranchTreeLoaded(BranchTreeState),
    BranchSelected(String),
    BranchCheckoutRequested(String),
    BranchCheckedOut { branch: String, result: Result<(), String> },

    // File tree messages
    FileTreeLoaded(FileTreeState),
    FileTreeNodeToggled(PathBuf),

    // File viewer messages
    FileSelectedInTree(PathBuf),
    FileSelectedInChanges(PathBuf),
    FileContentLoaded { path: PathBuf, content: String },
    FileDiffLoaded { path: PathBuf, hunks: Vec<crabontree_git::DiffHunk> },
    BinaryFileSelected { path: PathBuf, size: u64 },
}
```

## Phase 3: Add Effects & Jobs

**File**: `crates/app/src/effect.rs`

```rust
#[derive(Debug, Clone)]
pub enum Effect {
    // ... existing effects ...

    LoadBranchTree(PathBuf),
    CheckoutBranch { repo_path: PathBuf, branch: String },
    LoadFileTree(PathBuf),
    LoadFileContent { repo_path: PathBuf, file_path: PathBuf },
    LoadFileDiff { repo_path: PathBuf, file_path: PathBuf },
}
```

**File**: `crates/app/src/job.rs`

```rust
#[derive(Debug, Clone)]
pub enum Job {
    // ... existing jobs ...

    LoadBranchTree(PathBuf),
    CheckoutBranch { repo_path: PathBuf, branch: String },
    LoadFileTree(PathBuf),
    LoadFileContent { repo_path: PathBuf, file_path: PathBuf },
    LoadFileDiff { repo_path: PathBuf, file_path: PathBuf },
}
```

## Phase 4: Implement Git Operations

**File**: `crates/git/src/repo.rs`

Add stub implementations (will be completed later):

```rust
impl GitRepository {
    /// List all local branches.
    pub fn list_local_branches(&self) -> Result<Vec<BranchInfo>, GitError> {
        // TODO: Implement using gix
        unimplemented!("list_local_branches")
    }

    /// List all remote branches grouped by remote name.
    pub fn list_remote_branches(&self) -> Result<HashMap<String, Vec<BranchInfo>>, GitError> {
        // TODO: Implement using gix
        unimplemented!("list_remote_branches")
    }

    /// List all tags.
    pub fn list_tags(&self) -> Result<Vec<TagInfo>, GitError> {
        // TODO: Implement using gix
        unimplemented!("list_tags")
    }

    /// Checkout a branch.
    pub fn checkout_branch(&self, branch_name: &str) -> Result<(), GitError> {
        // TODO: Implement using git2
        unimplemented!("checkout_branch")
    }

    /// Get repository file tree.
    pub fn get_repository_tree(&self) -> Result<FileTreeNode, GitError> {
        // TODO: Implement
        unimplemented!("get_repository_tree")
    }

    /// Get file content at working tree.
    pub fn get_file_content(&self, path: &Path) -> Result<Vec<u8>, GitError> {
        // TODO: Implement
        unimplemented!("get_file_content")
    }

    /// Get diff for a specific file in working directory.
    pub fn get_file_diff(&self, path: &Path) -> Result<Vec<DiffHunk>, GitError> {
        // TODO: Implement using gix
        unimplemented!("get_file_diff")
    }

    /// Check if file is binary.
    pub fn is_binary_file(&self, path: &Path) -> Result<bool, GitError> {
        // TODO: Implement
        unimplemented!("is_binary_file")
    }
}
```

**Note**: Add necessary imports and type definitions as needed.

## Phase 5: Basic 4-Pane Layout

**File**: `crates/ui_egui/src/main.rs`

Replace `render_repository_view` with a 4-pane layout:

```rust
fn render_repository_view(&mut self, ui: &mut egui::Ui) {
    // Clone data to avoid borrow issues
    let repo_data = self.state.current_repo.as_ref().map(|repo| {
        (
            repo.branch_tree.clone(),
            repo.file_tree.clone(),
            repo.working_dir_files.clone(),
            repo.file_view.clone(),
        )
    });

    if let Some((branch_tree, file_tree, changed_files, file_view)) = repo_data {
        ui.horizontal(|ui| {
            let available_width = ui.available_width();
            let available_height = ui.available_height();

            // Pane 1: Branch Tree (15%)
            let pane1_width = available_width * self.pane_widths[0];
            ui.allocate_ui_with_layout(
                egui::vec2(pane1_width, available_height),
                egui::Layout::top_down(egui::Align::LEFT),
                |ui| self.render_branch_tree_pane(ui, &branch_tree),
            );

            // Vertical separator
            self.render_vertical_separator(ui, 0);

            // Pane 2: File Tree (25%)
            let pane2_width = available_width * self.pane_widths[1];
            ui.allocate_ui_with_layout(
                egui::vec2(pane2_width, available_height),
                egui::Layout::top_down(egui::Align::LEFT),
                |ui| self.render_file_tree_pane(ui, &file_tree),
            );

            // Vertical separator
            self.render_vertical_separator(ui, 1);

            // Pane 3: Changed Files (20%)
            let pane3_width = available_width * self.pane_widths[2];
            ui.allocate_ui_with_layout(
                egui::vec2(pane3_width, available_height),
                egui::Layout::top_down(egui::Align::LEFT),
                |ui| self.render_changed_files_pane(ui, &changed_files),
            );

            // Vertical separator
            self.render_vertical_separator(ui, 2);

            // Pane 4: File Viewer (40%)
            ui.vertical(|ui| {
                self.render_file_viewer_pane(ui, &file_view);
            });
        });
    }
}

fn render_vertical_separator(&mut self, ui: &mut egui::Ui, separator_index: usize) {
    let separator_id = ui.id().with(format!("vsep_{}", separator_index));
    let separator_rect = egui::Rect::from_min_size(
        ui.cursor().left_top(),
        egui::vec2(4.0, ui.available_height()),
    );

    let response = ui.interact(separator_rect, separator_id, egui::Sense::drag());

    if response.dragged() {
        let available_width = ui.available_width();
        let delta_ratio = response.drag_delta().x / available_width;

        // Update pane widths
        self.pane_widths[separator_index] =
            (self.pane_widths[separator_index] + delta_ratio)
            .max(0.1)
            .min(0.5);

        // Normalize to ensure sum = 1.0
        let sum: f32 = self.pane_widths.iter().sum();
        for width in &mut self.pane_widths {
            *width /= sum;
        }
    }

    // Visual feedback
    let color = if response.hovered() || response.dragged() {
        egui::Color32::from_rgb(100, 150, 200)
    } else {
        egui::Color32::from_rgb(60, 60, 60)
    };

    ui.painter().rect_filled(separator_rect, 0.0, color);

    if response.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeHorizontal);
    }
}
```

## Phase 6: Stub Pane Renderers

Add placeholder rendering functions:

```rust
fn render_branch_tree_pane(&mut self, ui: &mut egui::Ui, _state: &BranchTreeState) {
    ui.heading("Branches & Tags");
    ui.separator();
    ui.label("(Branch tree will be implemented here)");
    // TODO: Implement branch tree rendering
}

fn render_file_tree_pane(&mut self, ui: &mut egui::Ui, _state: &FileTreeState) {
    ui.heading("File Tree");
    ui.separator();
    ui.label("(File tree will be implemented here)");
    // TODO: Implement file tree rendering
}

fn render_changed_files_pane(&mut self, ui: &mut egui::Ui, files: &[WorkingDirFile]) {
    ui.heading("Changed Files");
    ui.separator();

    // Reuse existing working directory rendering for now
    self.render_working_directory(ui, files);
}

fn render_file_viewer_pane(&mut self, ui: &mut egui::Ui, state: &FileViewState) {
    ui.heading("File Viewer");
    ui.separator();

    match state {
        FileViewState::None => {
            ui.vertical_centered(|ui| {
                ui.add_space(100.0);
                ui.label("Select a file to view");
            });
        }
        FileViewState::Content { path, content, .. } => {
            ui.label(format!("File: {}", path.display()));
            ui.separator();
            ui.label("(Content viewer will be implemented here)");
            ui.label(format!("{} lines", content.lines().count()));
        }
        FileViewState::Diff { path, hunks, .. } => {
            ui.label(format!("Diff: {}", path.display()));
            ui.separator();
            ui.label("(Diff viewer will be implemented here)");
            ui.label(format!("{} hunks", hunks.len()));
        }
        FileViewState::Binary { path, size } => {
            ui.label(format!("Binary file: {} ({} bytes)", path.display(), size));
        }
    }
}
```

## Testing Phase 1

### Compile Check

```bash
cargo build --workspace
```

**Expected**: Should compile with stub implementations.

### Visual Check

```bash
cargo run --bin crabontree
```

**Expected**:
- Opens a repository
- See 4 vertical panes
- Can resize panes by dragging separators
- Pane 3 shows working directory (existing)
- Other panes show placeholder text

### What You Should See

```
┌──────────────┬──────────────┬──────────────┬──────────────┐
│ Branches &   │ File Tree    │ Changed Files│ File Viewer  │
│ Tags         │              │              │              │
│              │              │ (Existing WD │              │
│ (Placeholder)│ (Placeholder)│  list view)  │ (Placeholder)│
│              │              │              │              │
└──────────────┴──────────────┴──────────────┴──────────────┘
```

## Next Steps

After Phase 1 is complete and working:

1. **Phase 2**: Implement git operations for branches/tags
2. **Phase 3**: Implement branch tree rendering (Pane 1)
3. **Phase 4**: Implement file tree rendering (Pane 2)
4. **Phase 5**: Implement file viewer (Pane 4)
5. **Phase 6**: Wire up interactions between panes

## Troubleshooting

### Compilation Errors

**Issue**: Missing imports
- **Fix**: Add necessary `use` statements

**Issue**: Type mismatches
- **Fix**: Ensure all types are defined in the right crates

### Runtime Issues

**Issue**: Panes don't resize
- **Fix**: Check `render_vertical_separator` drag logic

**Issue**: Layout looks wrong
- **Fix**: Verify `pane_widths` sum to 1.0

## Resources

- Full design: `DESIGN_4_PANE_LAYOUT.md`
- Comparison: `DESIGN_COMPARISON.md`
- Architecture: `ARCHITECTURE.md`
- Git docs: `crates/git/src/repo.rs`

## Getting Help

If stuck:
1. Check existing code patterns
2. Review architecture document
3. Look at similar functionality (e.g., commit history panel)
4. Ask for help with specific error messages
