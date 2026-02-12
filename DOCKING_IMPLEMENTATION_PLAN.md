# Docking Implementation Plan for CrabOnTree

## Executive Summary

This plan details the migration from the current `egui::SidePanel`/`CentralPanel` layout to an IDE-like docking system using `egui_dock`. The implementation will enable drag-and-drop pane rearrangement, split views, and tab management while preserving all existing functionality.

---

## Current State Analysis

### Architecture Overview
- **Main App Structure**: `CrabOnTreeApp` in `crates/ui_egui/src/main.rs`
- **Layout Method**: `render_four_pane_layout()` at line 208
- **Current Layout**:
  - Left `SidePanel`: Commit History (lines 246-282)
  - Right `SidePanel`: Diff Viewer (lines 284-300)
  - `CentralPanel`: Changed Files (lines 302-320)

### Existing Pane Modules
Located in `crates/ui_egui/src/panes/`:

1. **commit_history.rs**
   - Function: `render(ui, commits, selected_commit, has_working_dir_changes) -> CommitHistoryAction`
   - Returns actions that convert to `AppMessage`
   - Already well-encapsulated

2. **changed_files.rs**
   - Function: `render(ui, files) -> ChangedFilesAction`
   - Displays staged/unstaged/untracked/conflicted files
   - Already well-encapsulated

3. **diff_viewer.rs**
   - Function: `render(ui, state)`
   - Displays file content, diffs, or binary file info
   - Already well-encapsulated

4. **scrollable_pane.rs**
   - Reusable component for pane headers and scrolling
   - Will be adapted for dock tabs

### Dependencies
Current: `egui = "0.27"`, `eframe = "0.27"`
Need to add: `egui_dock` compatible with egui 0.27

---

## Implementation Phases

### Phase 0: Preparation & Decoupling (3-4 hours)

**Goal**: Create the foundation for docking without breaking existing functionality.

#### Step 0.1: Create Pane Enum
**File**: `crates/ui_egui/src/panes/mod.rs`

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Pane {
    CommitHistory,
    ChangedFiles,
    DiffViewer,
}

impl Pane {
    pub fn title(&self) -> &'static str {
        match self {
            Pane::CommitHistory => "Commit History",
            Pane::ChangedFiles => "Changed Files",
            Pane::DiffViewer => "Diff Viewer",
        }
    }
}
```

**Testing**: Compile check only, no behavioral changes.

#### Step 0.2: Extract Pane Rendering Functions
**File**: `crates/ui_egui/src/panes/mod.rs`

Add wrapper functions that include the scrollable pane logic:

```rust
pub fn render_commit_history_pane(ui: &mut egui::Ui, state: &RepoState) -> Option<AppMessage> {
    let (commits, selected_commit, has_working_dir_changes) = (
        state.commits.as_slice(),
        state.selected_commit.as_ref(),
        !state.working_dir_files.is_empty(),
    );

    let action = commit_history::render(ui, commits, selected_commit, has_working_dir_changes);
    commit_history::action_to_message(action)
}

pub fn render_changed_files_pane(ui: &mut egui::Ui, files: &Option<ChangedFilesState>) -> Option<AppMessage> {
    if let Some(files) = files {
        let action = changed_files::render(ui, files);
        changed_files::action_to_message(action)
    } else {
        ui.label("Loading changed files...");
        None
    }
}

pub fn render_diff_viewer_pane(ui: &mut egui::Ui, file_view: &FileViewState) {
    diff_viewer::render(ui, file_view);
}
```

**Testing**: Replace existing inline rendering with these functions to verify behavior is identical.

#### Step 0.3: Refactor main.rs to Use Extracted Functions
**File**: `crates/ui_egui/src/main.rs`

Modify `render_four_pane_layout()` to use the new functions while keeping SidePanel layout:

```rust
egui::SidePanel::left("commit_history_panel")
    .resizable(true)
    .default_width(300.0)
    .width_range(200.0..=600.0)
    .show_inside(ui, |ui| {
        ui.add_space(5.0);
        scrollable_pane::render(
            ui,
            &scrollable_pane::ScrollablePaneConfig::new(
                "Commit History",
                "commit_history_scroll",
            ),
            |ui| {
                if let Some(repo) = &self.state.current_repo {
                    if let Some(msg) = panes::render_commit_history_pane(ui, repo) {
                        self.handle_message(msg);
                    }
                }
            },
        );
    });
// Similar for other panes...
```

**Testing**:
- Manual testing: verify all three panes work identically
- Check file selection, commit selection, diff display
- Verify keyboard shortcuts still work

**Rollback**: Git revert if issues found

---

### Phase 1: egui_dock Integration (6-8 hours)

**Goal**: Replace SidePanel/CentralPanel with DockArea while maintaining all functionality.

#### Step 1.1: Add egui_dock Dependency
**File**: `Cargo.toml` (workspace root)

```toml
[workspace.dependencies]
egui_dock = "0.12"  # Latest version compatible with egui 0.27
```

**File**: `crates/ui_egui/Cargo.toml`

```toml
[dependencies]
egui_dock.workspace = true
```

**Testing**: Run `cargo check` to verify dependency resolution.

#### Step 1.2: Add DockState to App
**File**: `crates/ui_egui/src/main.rs`

Add import:
```rust
use egui_dock::{DockState, DockArea, NodeIndex};
```

Modify `CrabOnTreeApp` struct (around line 45):
```rust
struct CrabOnTreeApp {
    state: AppState,
    executor: JobExecutor,
    message_rx: tokio::sync::mpsc::Receiver<crabontree_app::AppMessage>,
    theme: Theme,
    active_panel: keyboard::ActivePanel,
    show_shortcuts_help: bool,
    active_pane: usize,
    dock_state: DockState<panes::Pane>,  // NEW
}
```

#### Step 1.3: Initialize Dock Layout
**File**: `crates/ui_egui/src/main.rs`

Modify `CrabOnTreeApp::new()` (around line 56):

```rust
fn new(_cc: &eframe::CreationContext<'_>) -> Self {
    // ... existing config loading code ...

    // Initialize dock state with 3-pane layout
    let mut dock_state = DockState::new(vec![panes::Pane::CommitHistory]);

    // Split right to create CommitHistory | DiffViewer
    let [commit_node, diff_node] = dock_state.main_surface_mut().split_right(
        NodeIndex::root(),
        0.70,  // DiffViewer takes 70% of the right side
        vec![panes::Pane::DiffViewer],
    );

    // Split the left node to create CommitHistory | ChangedFiles | DiffViewer
    let [_commit_final, _changed_final] = dock_state.main_surface_mut().split_right(
        commit_node,
        0.50,  // ChangedFiles takes 50% of left 70%
        vec![panes::Pane::ChangedFiles],
    );

    Self {
        state,
        executor,
        message_rx,
        theme,
        active_panel: keyboard::ActivePanel::BranchTree,
        show_shortcuts_help: false,
        active_pane: 0,
        dock_state,  // NEW
    }
}
```

**Testing**: Compile check only at this stage.

#### Step 1.4: Implement TabViewer
**File**: `crates/ui_egui/src/main.rs`

Add before `impl eframe::App for CrabOnTreeApp`:

```rust
struct PaneViewer<'a> {
    app_state: &'a RepoState,
    messages: &'a mut Vec<crabontree_app::AppMessage>,
}

impl<'a> egui_dock::TabViewer for PaneViewer<'a> {
    type Tab = panes::Pane;

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        tab.title().into()
    }

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        // Apply scrollable pane wrapper
        panes::scrollable_pane::render(
            ui,
            &panes::scrollable_pane::ScrollablePaneConfig::new(
                tab.title(),
                &format!("{:?}_dock_scroll", tab),
            ),
            |ui| {
                let msg = match tab {
                    panes::Pane::CommitHistory => {
                        panes::render_commit_history_pane(ui, self.app_state)
                    }
                    panes::Pane::ChangedFiles => {
                        panes::render_changed_files_pane(ui, &self.app_state.changed_files)
                    }
                    panes::Pane::DiffViewer => {
                        panes::render_diff_viewer_pane(ui, &self.app_state.file_view);
                        None
                    }
                };

                if let Some(msg) = msg {
                    self.messages.push(msg);
                }
            },
        );
    }

    fn closeable(&mut self, _tab: &mut Self::Tab) -> bool {
        false  // Prevent closing essential panes
    }
}
```

**Testing**: Compile check.

#### Step 1.5: Replace render_four_pane_layout with Dock
**File**: `crates/ui_egui/src/main.rs`

Replace the entire `render_four_pane_layout()` method (lines 208-321):

```rust
fn render_repository_view(&mut self, ui: &mut egui::Ui) {
    // Handle keyboard shortcuts (keep existing logic)
    let (action, new_pane, new_panel) =
        keyboard::handle_shortcuts(ui, self.active_pane, self.active_panel);

    self.active_pane = new_pane;
    self.active_panel = new_panel;

    match action {
        keyboard::KeyboardAction::ToggleHelp => {
            self.show_shortcuts_help = !self.show_shortcuts_help;
        }
        _ => {
            if let Some(msg) = keyboard::action_to_message(action) {
                self.handle_message(msg);
            }
        }
    }

    // Auto-load missing data (keep existing logic)
    let (need_branch_tree, need_changed_files) = if let Some(repo) = &self.state.current_repo {
        (repo.branch_tree.is_none(), repo.changed_files.is_none())
    } else {
        (false, false)
    };

    if need_branch_tree {
        self.handle_message(crabontree_app::AppMessage::LoadBranchTreeRequested);
    }
    if need_changed_files {
        self.handle_message(crabontree_app::AppMessage::LoadChangedFilesRequested);
    }

    // Render with DockArea
    if let Some(repo) = &self.state.current_repo {
        let mut messages = Vec::new();

        let mut viewer = PaneViewer {
            app_state: repo,
            messages: &mut messages,
        };

        DockArea::new(&mut self.dock_state)
            .show_inside(ui, &mut viewer);

        // Process collected messages
        for msg in messages {
            self.handle_message(msg);
        }
    }
}
```

**Testing**:
1. **Compile**: `cargo build`
2. **Run**: `cargo run`
3. **Manual tests**:
   - Open a repository
   - Verify all 3 panes display correctly
   - Test commit selection (left pane → diff viewer)
   - Test file selection (middle pane → diff viewer)
   - **NEW**: Drag tabs to rearrange
   - **NEW**: Drag dividers to resize panes
   - **NEW**: Right-click tab headers
   - Test keyboard shortcuts work
4. **Visual regression**: Compare with screenshots from Phase 0

**Rollback**: Feature flag or git branch for safety.

#### Step 1.6: Styling and Polish
**File**: `crates/ui_egui/src/utils/theme.rs`

Add DockArea styling to match dark theme:

```rust
pub fn apply_theme(ctx: &egui::Context, theme: &Theme) {
    // ... existing theme code ...

    // Add dock-specific styling
    let mut style = (*ctx.style()).clone();

    // Tab bar styling
    style.visuals.widgets.inactive.bg_fill = theme.background;
    style.visuals.widgets.hovered.bg_fill = theme.surface;
    style.visuals.widgets.active.bg_fill = theme.primary;

    // Separator styling
    style.visuals.widgets.noninteractive.bg_stroke = egui::Stroke::new(
        1.0,
        theme.surface,
    );

    ctx.set_style(style);
}
```

**Testing**: Verify visual consistency with existing theme.

---

### Phase 2: Layout Persistence (2-3 hours)

**Goal**: Save and restore user's custom dock layout across sessions.

#### Step 2.1: Add Serde Support
**File**: `crates/ui_egui/src/panes/mod.rs`

Add derive macros:
```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum Pane {
    CommitHistory,
    ChangedFiles,
    DiffViewer,
}
```

#### Step 2.2: Persist Layout to Config
**File**: `crates/app/src/config.rs`

Add field to `AppConfig`:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    // ... existing fields ...
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dock_layout: Option<String>,  // JSON-serialized DockState
}
```

**File**: `crates/ui_egui/src/main.rs`

Save layout on exit in `impl Drop for CrabOnTreeApp`:
```rust
impl Drop for CrabOnTreeApp {
    fn drop(&mut self) {
        if let Ok(layout_json) = serde_json::to_string(&self.dock_state) {
            self.state.config.dock_layout = Some(layout_json);
            let _ = crabontree_app::save_config(&self.state.config);
        }
    }
}
```

Load layout on startup in `new()`:
```rust
let dock_state = if let Some(layout_json) = &config.dock_layout {
    serde_json::from_str(layout_json).unwrap_or_else(|_| create_default_layout())
} else {
    create_default_layout()
};
```

**Testing**:
1. Rearrange panes
2. Close and reopen app
3. Verify layout is restored
4. Delete config file and verify default layout works

---

### Phase 3 (Optional): Floating Windows (4-6 hours)

**Goal**: Allow panes to be undocked into floating windows.

*This is deferred to a later milestone. Document requirements only:*

#### Requirements
1. Add `PaneMode` enum: `Docked` vs `Floating { pos, size }`
2. Context menu on tab headers: "Undock"
3. Remove tab from dock tree when undocked
4. Render floating panes via `egui::Window`
5. "Dock back" button in floating window
6. Persist floating window positions

#### Estimated Effort
- 2 hours: Context menu integration
- 2 hours: Window state management
- 2 hours: Persistence and testing

---

## Testing Strategy

### Unit Tests
- **Pane enum**: Verify title strings
- **Serialization**: Round-trip test for DockState persistence

### Integration Tests
- **File**: `crates/ui_egui/tests/dock_layout_test.rs`
- Test default layout creation
- Test layout save/restore
- Test invalid layout recovery

### Manual Testing Checklist

**Phase 0 Completion**:
- [ ] All panes render identically to before
- [ ] Commit selection works
- [ ] File selection works
- [ ] Diff viewer updates correctly
- [ ] Keyboard shortcuts functional

**Phase 1 Completion**:
- [ ] All Phase 0 tests pass
- [ ] Can drag tabs between panes
- [ ] Can split panes horizontally/vertically
- [ ] Can resize panes by dragging dividers
- [ ] Tab context menu appears
- [ ] Cannot close panes (closeable = false)
- [ ] Theme matches existing UI
- [ ] Performance is acceptable (no lag)

**Phase 2 Completion**:
- [ ] Layout persists across restarts
- [ ] Invalid layout falls back to default
- [ ] Config file is valid TOML

---

## Rollback Plan

### Phase 0 Rollback
- Simple `git revert` of refactoring commits
- No user-facing changes, minimal risk

### Phase 1 Rollback
**Option A: Feature Flag**
```rust
#[cfg(feature = "docking")]
fn render_repository_view(&mut self, ui: &mut egui::Ui) {
    self.render_dock_layout(ui);
}

#[cfg(not(feature = "docking"))]
fn render_repository_view(&mut self, ui: &mut egui::Ui) {
    self.render_four_pane_layout(ui);
}
```

**Option B: Git Branch**
- Keep old layout in `main` until Phase 1 is verified
- Merge `feature/docking` after 1 week of testing

---

## Risk Assessment

### High Risk
- **Message handling in TabViewer**: Messages must be collected and processed outside the TabViewer callback to avoid borrow checker issues
  - *Mitigation*: Use `Vec<AppMessage>` buffer pattern (shown in Step 1.4)

### Medium Risk
- **Performance with large commit histories**: Rendering all panes on every frame
  - *Mitigation*: Profile with 1000+ commits, add caching if needed

- **Keyboard focus**: Dock tabs may interfere with existing focus handling
  - *Mitigation*: Test thoroughly, may need to track active tab

### Low Risk
- **Theme inconsistency**: Dock UI may not match app theme initially
  - *Mitigation*: Apply custom styling in Step 1.6

- **Layout serialization failure**: Invalid JSON in config
  - *Mitigation*: Fall back to default layout with error log

---

## Success Metrics

### Phase 0
- ✅ All tests pass
- ✅ Code compiles without warnings
- ✅ No behavioral changes observed

### Phase 1
- ✅ User can rearrange panes freely
- ✅ Layout feels responsive (no lag)
- ✅ No loss of existing functionality
- ✅ Visual consistency with existing theme

### Phase 2
- ✅ Layout persists across sessions
- ✅ Config file remains human-readable

---

## Timeline Estimate

| Phase | Estimated Time | Complexity |
|-------|---------------|------------|
| Phase 0 | 3-4 hours | Low |
| Phase 1 | 6-8 hours | Medium-High |
| Phase 2 | 2-3 hours | Low |
| Testing | 2-3 hours | Medium |
| **Total** | **13-18 hours** | |

Add 20% buffer for unexpected issues: **16-22 hours**

---

## Next Steps

1. **Review this plan** with stakeholders
2. **Create git branch**: `feature/egui-dock-integration`
3. **Start Phase 0**: Create Pane enum and extraction functions
4. **Checkpoint after each phase**: Manual testing + git commit
5. **Document findings**: Update this plan with actual time spent and issues encountered

---

## References

- [egui_dock GitHub](https://github.com/Adanos020/egui_dock)
- [egui_dock Examples](https://github.com/Adanos020/egui_dock/tree/main/examples)
- [DOCKING_GUIDE.md](./DOCKING_GUIDE.md) - Original specification
- Current codebase:
  - `crates/ui_egui/src/main.rs` - Main app and layout
  - `crates/ui_egui/src/panes/*` - Pane rendering modules
  - `crates/app/src/state.rs` - Application state
