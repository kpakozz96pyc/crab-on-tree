# CrabOnTree Description

## Overview
CrabOnTree is a Rust desktop Git GUI built with `eframe/egui`, organized as a Cargo workspace:
- `crates/git`: Git data access and repository operations
- `crates/app`: application state, reducer, messages, effects, job orchestration, config
- `crates/ui_core`: shared UI theme and shortcut primitives
- `crates/ui_egui`: desktop UI implementation

## Current Architecture
- State flow is reducer-driven (`crates/app/src/reducer.rs`) with async jobs submitted through `JobExecutor`.
- UI is rendered in `crates/ui_egui/src/main.rs` and dispatches `AppMessage` events to the reducer.
- Repository view uses a 3-pane dock:
  - Commit History
  - Changed Files
  - Diff Viewer

## Implemented Features
- IDE-like docking via `egui_dock`:
  - Tab-based pane containers
  - Drag-to-rearrange
  - Split and resize behavior
- Persistent dock layout:
  - `DockState<Pane>` serialized to JSON
  - Saved in `AppConfig.dock_layout`
  - Restored on startup with fallback to default layout when invalid
- Existing Git workflow support retained:
  - repo open/refresh
  - commit and working-tree views
  - changed files and diffs
  - staging/unstaging and commit actions

## Build Status (Current)
- `cargo check` passes successfully.
- Active warnings: 3 dead-code warnings in `crates/ui_egui/src/panes/mod.rs` for unused wrapper functions:
  - `render_commit_history_pane`
  - `render_changed_files_pane`
  - `render_diff_viewer_pane`

## Notes
- Legacy markdown status files for completed phases were consolidated into this file and the implementation plan.
