# Goal: IDE-like docking (tabs, split panes, drag-to-rearrange) in egui

## Recommended approach
Use a docking library instead of `egui::SidePanel / CentralPanel`:
- Prefer **egui_dock** for a classic "IDE docking" experience (split + tab containers).
- Optionally add "float/undock" via `egui::Window` (Phase 2).

This instruction assumes `egui_dock`.

---

## Phase 0 — Prep: decouple panel rendering
1) Create an enum representing panes:
```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pane {
    CommitHistory,
    ChangedFiles,
    DiffViewer,
}
```
Extract each panel UI into a function that renders into &mut egui::Ui:

```rust
fn ui_commit_history(ui: &mut egui::Ui, app: &mut AppState) -> PaneAction

fn ui_changed_files(ui: &mut egui::Ui, app: &mut AppState) -> PaneAction

fn ui_diff_viewer(ui: &mut egui::Ui, app: &mut AppState) -> PaneAction
```

Keep logic/actions the same; only change the container.

## Phase 1 — Integrate egui_dock
Add dependency:

```toml
[dependencies]
egui_dock = "0.##"   # use latest compatible with your egui/eframe version
```

Add docking state to your app struct:

```rust
use egui_dock::{DockState, DockArea};

pub struct App {
    dock_state: DockState<Pane>,
    // existing fields...
}
```

Initialize a 3-pane layout (left / middle / right):

```rust
let mut dock_state = DockState::new(vec![Pane::CommitHistory]);
let [a, b] = dock_state.main_surface_mut().split_right(
    egui_dock::NodeIndex::root(),
    0.70,
    vec![Pane::DiffViewer],
);
dock_state.main_surface_mut().split_left(a, 0.35, vec![Pane::ChangedFiles]);
self.dock_state = dock_state;
```
Adjust split ratios to match your current 3-column design.

Implement a tab viewer to render each Pane:

```rust
struct PaneViewer<'a> {
    app: &'a mut AppState,
}

impl egui_dock::TabViewer for PaneViewer<'_> {
    type Tab = Pane;

    fn title(&mut self, tab: &mut Pane) -> egui::WidgetText {
        match tab {
            Pane::CommitHistory => "Commit History".into(),
            Pane::ChangedFiles  => "Changed Files".into(),
            Pane::DiffViewer    => "Diff Viewer".into(),
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Pane) {
        match tab {
            Pane::CommitHistory => { ui_commit_history(ui, self.app); }
            Pane::ChangedFiles  => { ui_changed_files(ui, self.app); }
            Pane::DiffViewer    => { ui_diff_viewer(ui, self.app); }
        }
    }
}
```

Replace your old panels with the dock area inside a single CentralPanel:

```rust
egui::CentralPanel::default().show(ctx, |ui| {
    let mut viewer = PaneViewer { app: &mut self.state };
    DockArea::new(&mut self.dock_state)
        .show_inside(ui, &mut viewer);
});
```

Now you have IDE-like split panes + tabs + drag rearrangement.

## Phase 2 — "Undock to floating window" (optional, later)
egui_dock focuses on docked layout. For true "tear-off" floating panes:

1. Add per-pane mode state:

```rust
enum PaneMode { Docked, Floating { open: bool, pos: egui::Pos2, size: egui::Vec2 } }
```

2. Add a context menu / button on tab header "Undock".

3. On undock: remove that tab from dock tree, set mode = Floating.

4. Render floating panes via egui::Window calling the same ui_* function.

5. Add "Dock back" button that inserts the tab back into the dock tree.

(Keep Phase 2 separate to avoid destabilizing MVP.)

## UX polish checklist
- Persist layout: serialize DockState<Pane> (or store split ratios + tab positions) and restore on startup.

- Keyboard focus: ensure your panels use ui.allocate_response / focusable widgets properly.

- Theming: style DockArea separators/tab bar to match your dark UI.

## Definition of Done (Phase 1)
- All 3 panes render through DockArea.

- User can drag tabs between columns, split/merge, and reorder tabs.

- No panel logic duplication: only container changed.
