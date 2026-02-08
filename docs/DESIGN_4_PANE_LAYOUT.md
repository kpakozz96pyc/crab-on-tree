# 4-Pane Layout Design & Implementation Plan

## Overview

This document outlines the design and implementation plan for transforming CrabOnTree into a 4-pane vertical layout similar to modern Git GUIs like TortoiseGit.

## New Design Concept

### Visual Layout

```
┌────────────┬──────────────┬───────────────┬─────────────────────┐
│   Pane 1   │   Pane 2     │   Pane 3      │      Pane 4         │
│            │              │               │                     │
│  Branches  │ Working Dir  │ Changed Files │   File Content      │
│   & Tags   │  File Tree   │    List       │   / Diff Viewer     │
│            │              │               │                     │
│  LOCAL     │  📁 .git     │ STAGED (2)    │  src/Button.jsx     │
│  • main    │  📁 src      │ ~ Button.jsx  │  ---------------    │
│  • develop │    📁 comp   │ + Header.jsx  │  12  const Button = │
│            │      Button  │               │  13  const base =   │
│  REMOTE    │      Header  │ UNSTAGED (1)  │  14  ...            │
│  • origin  │    📁 utils  │ ~ index.js    │                     │
│            │  📁 public   │               │  [Syntax highlight] │
│  TAGS      │  📄 README   │ DELETED       │  [Diff colors]      │
│  • v1.0.0  │              │               │  [Line numbers]     │
│            │              │               │                     │
└────────────┴──────────────┴───────────────┴─────────────────────┘
    15%           25%             20%                40%
```

## Detailed Pane Specifications

### Pane 1: Branches & Tags Navigator

**Purpose**: Navigate between branches, remotes, and tags

**Features**:
- Collapsible tree structure
- Sections: LOCAL, REMOTE (origin, upstream, etc.), TAGS
- Current branch highlighted
- Click to checkout branch
- Right-click context menu (future):
  - Create branch
  - Delete branch
  - Merge
  - Rebase

**Data Required**:
```rust
struct BranchTreeState {
    local_branches: Vec<BranchInfo>,
    remote_branches: HashMap<String, Vec<BranchInfo>>, // remote name -> branches
    tags: Vec<TagInfo>,
    current_branch: String,
    expanded_sections: HashSet<String>, // "LOCAL", "REMOTE/origin", "TAGS"
}

struct BranchInfo {
    name: String,
    commit_hash: String,
    is_current: bool,
    upstream: Option<String>,
}

struct TagInfo {
    name: String,
    commit_hash: String,
    message: Option<String>, // For annotated tags
}
```

**Git Operations Needed**:
```rust
// New methods in GitRepository
fn list_local_branches(&self) -> Result<Vec<BranchInfo>>;
fn list_remote_branches(&self) -> Result<HashMap<String, Vec<BranchInfo>>>;
fn list_tags(&self) -> Result<Vec<TagInfo>>;
fn get_current_branch(&self) -> Result<String>;
fn checkout_branch(&self, name: &str) -> Result<()>;
```

### Pane 2: Working Directory File Tree

**Purpose**: Browse all files in the repository (not just changed files)

**Features**:
- Tree view of entire repository
- Folders collapsible/expandable
- Files clickable to view content
- Show icons for file types
- Visual indicators for changed files (colors/badges)
- Search/filter functionality
- Show/hide hidden files option

**Data Required**:
```rust
struct FileTreeState {
    root: FileTreeNode,
    expanded_paths: HashSet<PathBuf>,
    selected_path: Option<PathBuf>,
    filter: String,
}

enum FileTreeNode {
    Directory {
        path: PathBuf,
        name: String,
        children: Vec<FileTreeNode>,
        is_expanded: bool,
        has_changes: bool, // If any child has changes
    },
    File {
        path: PathBuf,
        name: String,
        status: Option<WorkingDirStatus>, // None if unchanged
        size: u64,
    },
}
```

**Git Operations Needed**:
```rust
// New methods in GitRepository
fn get_repository_tree(&self) -> Result<FileTreeNode>;
fn get_file_status(&self, path: &Path) -> Result<Option<WorkingDirStatus>>;
```

### Pane 3: Changed Files List

**Purpose**: Show only files with changes (staged/unstaged)

**Features**:
- Grouped by status: STAGED CHANGES, UNSTAGED CHANGES, UNTRACKED, CONFLICTED
- Show file path, status icon, and line stats (+24/-12)
- Click to view diff in Pane 4
- Stage/unstage buttons per file
- "Stage All" / "Unstage All" buttons per section
- Search/filter within changed files

**Data Required**:
```rust
struct ChangedFilesState {
    staged: Vec<ChangedFileInfo>,
    unstaged: Vec<ChangedFileInfo>,
    untracked: Vec<ChangedFileInfo>,
    conflicted: Vec<ChangedFileInfo>,
    selected_file: Option<PathBuf>,
    filter: String,
    collapsed_sections: HashSet<String>, // "STAGED", "UNSTAGED", etc.
}

struct ChangedFileInfo {
    path: PathBuf,
    status: WorkingDirStatus,
    is_staged: bool,
    additions: usize,
    deletions: usize,
    changes: usize, // Total changes
}
```

**Git Operations Needed**:
```rust
// Enhance existing methods
fn get_working_dir_status_detailed(&self) -> Result<ChangedFilesState>;
fn get_file_stats(&self, path: &Path) -> Result<(usize, usize)>; // (additions, deletions)
```

### Pane 4: File Content / Diff Viewer

**Purpose**: Display file content or diff for selected file

**Features**:
- **For unchanged files**: Show current content with syntax highlighting
- **For changed files**: Show side-by-side or unified diff
- Line numbers
- Diff highlighting (green for additions, red for deletions)
- Syntax highlighting based on file extension
- Code folding for large files
- Search within file
- View modes: Unified diff, Side-by-side, Content only

**Data Required**:
```rust
enum FileViewState {
    Content {
        path: PathBuf,
        content: String,
        language: Option<String>, // For syntax highlighting
    },
    Diff {
        path: PathBuf,
        hunks: Vec<DiffHunk>,
        view_mode: DiffViewMode,
    },
    Binary {
        path: PathBuf,
        size: u64,
    },
    None,
}

enum DiffViewMode {
    Unified,
    SideBySide,
    ContentOnly,
}
```

**Git Operations Needed**:
```rust
// New methods in GitRepository
fn get_file_content(&self, path: &Path) -> Result<Vec<u8>>;
fn get_file_diff(&self, path: &Path) -> Result<Vec<DiffHunk>>;
fn is_binary_file(&self, path: &Path) -> Result<bool>;
```

## Implementation Plan

### Phase 1: Data Model & State (Week 1)

**Tasks**:
1. **Define new state structures** in `crates/app/src/state.rs`:
   ```rust
   pub struct AppState {
       // ... existing fields ...
       pub branch_tree: Option<BranchTreeState>,
       pub file_tree: Option<FileTreeState>,
       pub changed_files: Option<ChangedFilesState>,
       pub file_view: FileViewState,
       pub pane_widths: [f32; 4], // Relative widths (0.15, 0.25, 0.20, 0.40)
   }
   ```

2. **Add new messages** in `crates/app/src/message.rs`:
   ```rust
   pub enum AppMessage {
       // ... existing messages ...

       // Branch navigation
       BranchTreeLoaded(BranchTreeState),
       BranchSelected(String),
       BranchCheckoutRequested(String),
       TagSelected(String),

       // File tree
       FileTreeLoaded(FileTreeState),
       FileTreeNodeExpanded(PathBuf),
       FileTreeNodeCollapsed(PathBuf),
       FileSelected(PathBuf),

       // File view
       FileContentLoaded { path: PathBuf, content: String },
       FileDiffLoaded { path: PathBuf, hunks: Vec<DiffHunk> },
   }
   ```

3. **Add new effects** in `crates/app/src/effect.rs`:
   ```rust
   pub enum Effect {
       // ... existing effects ...
       LoadBranchTree(PathBuf),
       CheckoutBranch { repo_path: PathBuf, branch: String },
       LoadFileTree(PathBuf),
       LoadFileContent { repo_path: PathBuf, file_path: PathBuf },
       LoadFileDiff { repo_path: PathBuf, file_path: PathBuf },
   }
   ```

4. **Add new jobs** in `crates/app/src/job.rs`:
   ```rust
   pub enum Job {
       // ... existing jobs ...
       LoadBranchTree(PathBuf),
       CheckoutBranch { repo_path: PathBuf, branch: String },
       LoadFileTree(PathBuf),
       LoadFileContent { repo_path: PathBuf, file_path: PathBuf },
       LoadFileDiff { repo_path: PathBuf, file_path: PathBuf },
   }
   ```

**Deliverables**:
- ✅ State structures defined
- ✅ Message types added
- ✅ Effect types added
- ✅ Job types added
- ✅ Unit tests for state structures

### Phase 2: Git Layer Enhancements (Week 1-2)

**Tasks**:
1. **Implement branch/tag operations** in `crates/git/src/repo.rs`:
   ```rust
   impl GitRepository {
       pub fn list_local_branches(&self) -> Result<Vec<BranchInfo>, GitError>;
       pub fn list_remote_branches(&self) -> Result<HashMap<String, Vec<BranchInfo>>, GitError>;
       pub fn list_tags(&self) -> Result<Vec<TagInfo>, GitError>;
       pub fn get_current_branch(&self) -> Result<String, GitError>;
       pub fn checkout_branch(&self, name: &str) -> Result<(), GitError>;
   }
   ```

2. **Implement file tree operations**:
   ```rust
   impl GitRepository {
       pub fn get_repository_tree(&self) -> Result<FileTreeNode, GitError>;
       pub fn get_file_content(&self, path: &Path) -> Result<Vec<u8>, GitError>;
       pub fn get_file_diff(&self, path: &Path) -> Result<Vec<DiffHunk>, GitError>;
       pub fn is_binary_file(&self, path: &Path) -> Result<bool, GitError>;
   }
   ```

3. **Enhance status operations**:
   ```rust
   impl GitRepository {
       pub fn get_file_stats(&self, path: &Path) -> Result<(usize, usize), GitError>;
       pub fn get_working_dir_status_detailed(&self) -> Result<ChangedFilesState, GitError>;
   }
   ```

**Implementation Notes**:
- Use `gix` for read operations (branches, tags, file content, tree)
- Use `git2` for write operations (checkout, stage, unstage)
- Add comprehensive error handling
- Add performance logging with `tracing`

**Deliverables**:
- ✅ All git operations implemented
- ✅ Integration tests for each operation
- ✅ Error handling tested
- ✅ Performance benchmarks

### Phase 3: Reducer & Executor Updates (Week 2)

**Tasks**:
1. **Update reducer** in `crates/app/src/reducer.rs`:
   - Handle all new message types
   - Update state correctly for each message
   - Return appropriate effects

2. **Update executor** in `crates/app/src/executor.rs`:
   - Implement job execution for all new job types
   - Call git layer operations
   - Convert results to messages
   - Handle errors gracefully

3. **Add state initialization logic**:
   - When repo is opened, load branch tree, file tree, and changed files
   - Cache results appropriately

**Deliverables**:
- ✅ Reducer handles all new messages
- ✅ Executor executes all new jobs
- ✅ Unit tests for reducer logic
- ✅ Integration tests for executor

### Phase 4: UI Layout Redesign (Week 3)

**Tasks**:
1. **Redesign main layout** in `crates/ui_egui/src/main.rs`:
   ```rust
   fn render_repository_view(&mut self, ui: &mut egui::Ui) {
       ui.horizontal(|ui| {
           // Pane 1: Branch Tree (15%)
           let pane1_width = ui.available_width() * self.state.pane_widths[0];
           ui.allocate_ui_with_layout(
               egui::vec2(pane1_width, ui.available_height()),
               egui::Layout::top_down(egui::Align::LEFT),
               |ui| self.render_branch_tree_pane(ui),
           );

           // Vertical separator (resizable)
           self.render_vertical_separator(ui, 0);

           // Pane 2: File Tree (25%)
           // ... similar pattern

           // Pane 3: Changed Files (20%)
           // ... similar pattern

           // Pane 4: File Viewer (40%)
           // ... similar pattern
       });
   }
   ```

2. **Implement resizable separators**:
   ```rust
   fn render_vertical_separator(&mut self, ui: &mut egui::Ui, index: usize) {
       let separator_id = ui.id().with(format!("vsep_{}", index));
       let separator_rect = egui::Rect::from_min_size(
           ui.cursor().left_top(),
           egui::vec2(4.0, ui.available_height()),
       );

       let response = ui.interact(separator_rect, separator_id, egui::Sense::drag());
       if response.dragged() {
           // Update pane_widths based on drag
       }

       // Visual feedback
       ui.painter().rect_filled(separator_rect, 0.0, separator_color);
   }
   ```

**Deliverables**:
- ✅ 4-pane horizontal layout
- ✅ Resizable pane separators
- ✅ Responsive to window resize
- ✅ Minimum/maximum pane widths enforced

### Phase 5: Pane 1 - Branch Tree Implementation (Week 3)

**Tasks**:
1. **Implement branch tree rendering**:
   ```rust
   fn render_branch_tree_pane(&mut self, ui: &mut egui::Ui) {
       ui.heading("Branches & Tags");
       ui.separator();

       egui::ScrollArea::vertical().show(ui, |ui| {
           // LOCAL section
           egui::CollapsingHeader::new("LOCAL")
               .default_open(true)
               .show(ui, |ui| {
                   for branch in &local_branches {
                       self.render_branch_item(ui, branch);
                   }
               });

           // REMOTE section
           egui::CollapsingHeader::new("REMOTE")
               .default_open(true)
               .show(ui, |ui| {
                   for (remote, branches) in &remote_branches {
                       egui::CollapsingHeader::new(remote)
                           .show(ui, |ui| {
                               for branch in branches {
                                   self.render_branch_item(ui, branch);
                               }
                           });
                   }
               });

           // TAGS section
           egui::CollapsingHeader::new("TAGS")
               .show(ui, |ui| {
                   for tag in &tags {
                       self.render_tag_item(ui, tag);
                   }
               });
       });
   }
   ```

2. **Add branch interaction**:
   - Click to view branch (update working dir)
   - Double-click or context menu to checkout
   - Visual indication of current branch

**Deliverables**:
- ✅ Branch tree renders correctly
- ✅ Collapsible sections work
- ✅ Branch selection works
- ✅ Current branch highlighted

### Phase 6: Pane 2 - File Tree Implementation (Week 4)

**Tasks**:
1. **Implement file tree rendering**:
   ```rust
   fn render_file_tree_pane(&mut self, ui: &mut egui::Ui) {
       ui.heading("Working Directory");
       ui.separator();

       // Search box
       ui.horizontal(|ui| {
           ui.label("🔍");
           ui.text_edit_singleline(&mut self.file_tree_search);
       });

       egui::ScrollArea::vertical().show(ui, |ui| {
           self.render_file_tree_node(ui, &root_node, 0);
       });
   }

   fn render_file_tree_node(&mut self, ui: &mut egui::Ui, node: &FileTreeNode, depth: usize) {
       match node {
           FileTreeNode::Directory { name, children, is_expanded, has_changes, .. } => {
               ui.horizontal(|ui| {
                   ui.add_space(depth as f32 * 16.0);

                   let icon = if *is_expanded { "📂" } else { "📁" };
                   if ui.small_button(icon).clicked() {
                       // Toggle expansion
                   }

                   let text = if *has_changes {
                       egui::RichText::new(name).color(self.theme.git_modified)
                   } else {
                       egui::RichText::new(name)
                   };
                   ui.label(text);
               });

               if *is_expanded {
                   for child in children {
                       self.render_file_tree_node(ui, child, depth + 1);
                   }
               }
           }
           FileTreeNode::File { name, status, .. } => {
               ui.horizontal(|ui| {
                   ui.add_space(depth as f32 * 16.0);
                   ui.label("📄");

                   let (text, color) = match status {
                       Some(WorkingDirStatus::Modified) => (name, self.theme.git_modified),
                       Some(WorkingDirStatus::Untracked) => (name, self.theme.git_added),
                       Some(WorkingDirStatus::Deleted) => (name, self.theme.git_deleted),
                       None => (name, self.theme.fg_primary),
                       _ => (name, self.theme.fg_primary),
                   };

                   if ui.selectable_label(false, egui::RichText::new(text).color(color)).clicked() {
                       // Load file content in Pane 4
                       self.handle_message(AppMessage::FileSelected(path.clone()));
                   }
               });
           }
       }
   }
   ```

2. **Add file tree navigation**:
   - Click to expand/collapse folders
   - Click file to view in Pane 4
   - Keyboard navigation (arrow keys)
   - Search/filter

**Deliverables**:
- ✅ File tree renders correctly
- ✅ Expand/collapse works
- ✅ File selection works
- ✅ Changed files highlighted
- ✅ Search/filter functional

### Phase 7: Pane 3 - Changed Files List (Week 4)

**Tasks**:
1. **Implement changed files rendering** (similar to current working dir view but grouped):
   ```rust
   fn render_changed_files_pane(&mut self, ui: &mut egui::Ui) {
       ui.heading("Changed Files");
       ui.separator();

       // STAGED CHANGES section
       egui::CollapsingHeader::new(format!("STAGED CHANGES ({})", staged.len()))
           .default_open(true)
           .show(ui, |ui| {
               ui.horizontal(|ui| {
                   if ui.button("Unstage All").clicked() {
                       // Unstage all
                   }
               });

               for file in staged {
                   self.render_changed_file_item(ui, file, true);
               }
           });

       // UNSTAGED CHANGES section
       // ... similar pattern

       // UNTRACKED FILES section
       // ... similar pattern
   }

   fn render_changed_file_item(&mut self, ui: &mut egui::Ui, file: &ChangedFileInfo, is_staged: bool) {
       ui.horizontal(|ui| {
           // Stage/unstage button
           let button = if is_staged { "−" } else { "+" };
           if ui.small_button(button).clicked() {
               // Toggle staging
           }

           // Status icon
           let (icon, color) = match file.status {
               WorkingDirStatus::Modified => ("~", self.theme.git_modified),
               WorkingDirStatus::Untracked => ("+", self.theme.git_added),
               WorkingDirStatus::Deleted => ("-", self.theme.git_deleted),
               _ => ("*", self.theme.fg_primary),
           };
           ui.colored_label(color, icon);

           // File path (clickable)
           if ui.selectable_label(false, &file.path.display().to_string()).clicked() {
               self.handle_message(AppMessage::FileSelected(file.path.clone()));
           }

           // Line stats
           ui.label(format!("+{} -{}", file.additions, file.deletions));
       });
   }
   ```

**Deliverables**:
- ✅ Changed files grouped by status
- ✅ File stats displayed
- ✅ Stage/unstage works
- ✅ File selection updates Pane 4

### Phase 8: Pane 4 - File Content/Diff Viewer (Week 5)

**Tasks**:
1. **Implement file viewer**:
   ```rust
   fn render_file_viewer_pane(&mut self, ui: &mut egui::Ui) {
       match &self.state.file_view {
           FileViewState::None => {
               ui.vertical_centered(|ui| {
                   ui.add_space(100.0);
                   ui.label("Select a file to view");
               });
           }
           FileViewState::Content { path, content, .. } => {
               self.render_file_content(ui, path, content);
           }
           FileViewState::Diff { path, hunks, view_mode } => {
               self.render_file_diff(ui, path, hunks, *view_mode);
           }
           FileViewState::Binary { path, size } => {
               ui.label(format!("Binary file: {} ({} bytes)", path.display(), size));
           }
       }
   }

   fn render_file_content(&self, ui: &mut egui::Ui, path: &Path, content: &str) {
       ui.heading(path.display().to_string());
       ui.separator();

       egui::ScrollArea::both().show(ui, |ui| {
           for (i, line) in content.lines().enumerate() {
               ui.horizontal(|ui| {
                   ui.label(egui::RichText::new(format!("{:4}", i + 1))
                       .monospace()
                       .color(self.theme.fg_tertiary));
                   ui.label(egui::RichText::new(line).monospace());
               });
           }
       });
   }

   fn render_file_diff(&self, ui: &mut egui::Ui, path: &Path, hunks: &[DiffHunk], mode: DiffViewMode) {
       ui.heading(format!("Diff: {}", path.display()));
       ui.separator();

       // View mode selector
       ui.horizontal(|ui| {
           if ui.selectable_label(mode == DiffViewMode::Unified, "Unified").clicked() {
               // Change mode
           }
           if ui.selectable_label(mode == DiffViewMode::SideBySide, "Side-by-Side").clicked() {
               // Change mode
           }
       });

       egui::ScrollArea::both().show(ui, |ui| {
           match mode {
               DiffViewMode::Unified => self.render_unified_diff(ui, hunks),
               DiffViewMode::SideBySide => self.render_side_by_side_diff(ui, hunks),
               DiffViewMode::ContentOnly => self.render_content_only(ui, hunks),
           }
       });
   }
   ```

2. **Add syntax highlighting** (future enhancement):
   - Use `syntect` crate for syntax highlighting
   - Detect language from file extension
   - Apply theme-based highlighting

**Deliverables**:
- ✅ File content renders correctly
- ✅ Diff renders with line numbers
- ✅ Diff highlighting (additions/deletions)
- ✅ View mode switching
- ✅ Scrolling works smoothly

### Phase 9: Keyboard Navigation & Shortcuts (Week 5)

**Tasks**:
1. **Update keyboard shortcuts**:
   - `1`, `2`, `3`, `4` - Focus pane 1, 2, 3, 4
   - `Tab` / `Shift+Tab` - Cycle through panes
   - Arrow keys - Navigate within active pane
   - `Space` - Stage/unstage (when in Pane 3)
   - `Enter` - Expand/collapse or select
   - `/` - Focus search in active pane

2. **Add focus management**:
   - Visual indicator for active pane
   - Maintain focus state
   - Keyboard navigation within trees

**Deliverables**:
- ✅ All keyboard shortcuts work
- ✅ Focus management correct
- ✅ Visual focus indicators
- ✅ Updated keyboard shortcuts help

### Phase 10: Testing & Polish (Week 6)

**Tasks**:
1. **Performance testing**:
   - Test with large repositories (10,000+ files)
   - Optimize tree rendering with virtual scrolling
   - Lazy load file content
   - Cache results appropriately

2. **UI polish**:
   - Consistent spacing and alignment
   - Smooth transitions
   - Loading indicators
   - Error states

3. **Integration testing**:
   - Test all workflows end-to-end
   - Test with various repository states
   - Test error handling

4. **Documentation**:
   - Update user guide
   - Update architecture docs
   - Add inline code comments

**Deliverables**:
- ✅ Performance benchmarks met
- ✅ All integration tests pass
- ✅ UI polish complete
- ✅ Documentation updated

## Technical Considerations

### Performance Optimizations

1. **Virtual Scrolling**:
   - Use egui's `show_rows()` for large lists
   - Only render visible tree nodes

2. **Lazy Loading**:
   - Load file content only when selected
   - Load file tree on demand (expandable nodes)
   - Cache results with TTL

3. **Incremental Updates**:
   - Only reload changed portions of the tree
   - Diff updates instead of full reloads

4. **Efficient State Management**:
   - Use `Rc` / `Arc` for shared data
   - Minimize cloning large structures
   - Cache computed values

### Backward Compatibility

- Keep existing functionality working during transition
- Allow toggling between old and new layouts (config option)
- Migrate saved state/preferences

### Future Enhancements

1. **Pane 1 Enhancements**:
   - Create/delete branches
   - Merge/rebase from UI
   - Remote management
   - Stash management

2. **Pane 2 Enhancements**:
   - File icons by type
   - Git ignore filtering
   - External file manager integration

3. **Pane 3 Enhancements**:
   - Partial staging (stage hunks)
   - Conflict resolution UI
   - File history view

4. **Pane 4 Enhancements**:
   - Full syntax highlighting
   - Code folding
   - Inline editing
   - Split view

## Risk Assessment

### High Risk
- **UI complexity**: 4 interactive panes with resizing
  - **Mitigation**: Build incrementally, test frequently

### Medium Risk
- **Performance with large repos**: 10,000+ files
  - **Mitigation**: Virtual scrolling, lazy loading, caching

### Low Risk
- **Git operations**: Well-tested libraries (gix, git2)
  - **Mitigation**: Comprehensive error handling

## Success Criteria

1. ✅ All 4 panes render correctly
2. ✅ Panes are resizable and responsive
3. ✅ File selection works across panes
4. ✅ Keyboard navigation works smoothly
5. ✅ Performance: < 1s to load 1000 file tree
6. ✅ Performance: < 100ms to switch files
7. ✅ All existing features still work
8. ✅ User feedback is positive

## Timeline Summary

- **Week 1**: Data model & Git layer foundations
- **Week 2**: Reducer/executor & basic layout
- **Week 3**: Panes 1 & layout system
- **Week 4**: Panes 2 & 3
- **Week 5**: Pane 4 & keyboard navigation
- **Week 6**: Testing, polish, documentation

**Total**: 6 weeks for complete implementation

## Next Steps

1. Review and approve this design document
2. Create GitHub issues for each phase
3. Set up development branch
4. Begin Phase 1 implementation
