# CrabOnTree Codebase Documentation

Last updated: February 21, 2026

## Scope
This document summarizes the codebase inside the `crates/` workspace, focusing on architecture, crate responsibilities, core runtime flow, and operational behavior.

## Workspace Structure
The workspace is defined in `Cargo.toml` and contains four crates:

1. `crates/git` (`crabontree-git`): Git data access and repository operations.
2. `crates/app` (`crabontree-app`): Application state, reducer, effects, jobs, and config.
3. `crates/ui_core` (`crabontree-ui-core`): Framework-agnostic UI primitives.
4. `crates/ui_egui` (`crabontree` binary): Desktop UI built with `eframe/egui`.

## High-Level Architecture
The project uses a reducer/effect/job architecture:

1. UI dispatches `AppMessage` values.
2. `reduce(state, message)` updates state synchronously and returns an `Effect`.
3. UI effect handler maps `Effect` to async `Job` submissions or direct side effects.
4. `JobExecutor` worker executes Git/file I/O and returns `AppMessage` results.
5. Result messages go back through reducer.

This enforces:

- Deterministic state transitions in reducer.
- Side effects isolated in executor.
- UI mostly focused on rendering and intent dispatch.

## Crate: `crates/git`

### Responsibility
`GitRepository` is the central service for repository operations.

- Read-heavy operations use `gix`.
- Write-heavy operations use `git2`.

This hybrid model is explicitly documented in code and implemented in `GitRepository::open()` by opening both engines.

### Domain Types
Core data models include:

- `Commit`
- `FileDiff`, `DiffHunk`, `DiffLine`, `DiffLineType`
- `WorkingDirFile`, `WorkingDirStatus`
- `StatusSummary`

### Main Capabilities
Implemented operations include:

- Repo/open metadata: `open`, `get_head`, `get_branches`, `get_status`.
- Working directory: `get_working_dir_status`.
- Staging: `stage_file`, `unstage_file`, `stage_all`, `unstage_all`, batch stage/unstage.
- History and diffs: `get_commit_history`, `get_commit_details`, `get_commit_diff`, `get_file_diff`.
- Commits/push: `has_staged_changes`, `create_commit`, `amend_commit`, `push`.
- Branch workflows: `list_local_branches`, `list_remote_branches`, `list_tags`, `checkout_branch`, `create_tracking_branch`.
- Safety helpers: `has_uncommitted_changes`, `stash_changes`, `discard_all_changes`, `local_branch_exists`.
- File access: `get_file_content`, `is_binary_file`, `get_repository_tree`.

### Notable Implementation Details
- `push()` configures credential callbacks for SSH agent, credential helper, and default credentials.
- Large stage/unstage operations are chunked (`CHUNK_SIZE = 500`).
- Working directory status is derived from libgit2 status flags with staged/unstaged distinction.
- Binary file detection reads first 8KB and checks for `\0` bytes.

### Current Gaps/TODOs in Code
- Local branch upstream metadata is currently hardcoded as `None` in `list_local_branches`.
- Tag message extraction is currently `None` in `list_tags`.

## Crate: `crates/app`

### Responsibility
Owns app-level state and orchestration between UI and Git.

### Core Modules
- `state.rs`: `AppState`, `RepoState`, and pane-specific states.
- `message.rs`: all user/system events (`AppMessage`).
- `effect.rs`: reducer-emitted side effects (`Effect`).
- `job.rs`: async job types (`Job`, `JobId`).
- `reducer.rs`: pure state transition function.
- `executor.rs`: async job worker and I/O execution.
- `config.rs`: load/save TOML config via `directories`.

### State Model Highlights
`AppState` tracks:

- Current repo and loading/committing flags.
- Error state.
- Persistent config.
- Optional dialogs (checkout-with-changes and branch-conflict).

`RepoState` tracks:

- Repo metadata/head/branches/status.
- Commit list + selected commit + commit diff.
- Working directory files.
- 4-pane UI data (`branch_tree`, `changed_files`, `file_view`).

### Reducer Behavior Highlights
- Opening repo triggers batched effects: save config + load commits + load working dir + author identity.
- Commit selection supports a virtual `WORKING_DIRECTORY` entry for working-tree inspection.
- Changed-files selection supports single, ctrl-toggle, and shift-range selection.
- Multi-select can trigger `LoadMultipleFileDiffs` (working tree) or build multi-diff directly from cached commit diff (commit view).
- Commit panel supports summary/description/amend/push options and generates full message (`summary\n\ndescription`).
- Commit drafts are restored from config and preserved through changed-file refreshes.

### Executor Behavior Highlights
- Worker thread runs a Tokio multi-thread runtime.
- Blocking Git operations are wrapped in `spawn_blocking`.
- Result is always mapped to `AppMessage`, including errors.
- Branch switching flow handles:
  - uncommitted-change checks,
  - stash/discard paths,
  - remote branch local-name conflict dialog,
  - tracking-branch creation.

### Config
Config file path is resolved with `ProjectDirs` (`com/crabontree/CrabOnTree/config.toml`) and stores:

- theme
- recent repositories
- pane widths
- serialized dock layout JSON
- commit drafts per repository

## Crate: `crates/ui_core`

### Responsibility
Framework-independent UI primitives.

### Provided Types
- `Color` with `from_hex` parsing.
- `Theme` with `dark()` and `light()` presets.
- Keyboard shortcut domain (`Key`, `Modifiers`, `Action`, `Shortcut`).

This crate is intentionally lightweight and serializable.

## Crate: `crates/ui_egui`

### Responsibility
Desktop GUI implementation using `eframe`, `egui`, and `egui_dock`.

### Main App Loop
`CrabOnTreeApp` in `src/main.rs` owns:

- `AppState`
- `JobExecutor` + message receiver
- `DockState<Pane>`
- top-level UI flags (help dialog, active pane)

Per-frame `update()`:

1. Polls async messages.
2. Applies theme.
3. Renders top panel + error panel + dialogs.
4. Renders either welcome screen or docked repository view.

### Docked Pane System
Pane enum:

- `CommitHistory`
- `Branches`
- `ChangedFiles`
- `DiffViewer`

Features:

- Default 4-pane layout creation.
- Layout persistence through config (`dock_layout`).
- Pane visibility toggles with state restore support.

### Components and Panes
- Components: top toolbar, welcome view, error banner, shortcuts dialog, branch dialogs.
- Panes:
  - Commit history pane includes virtual working-directory row.
  - Branch pane supports local/remote listing and checkout actions.
  - Changed-files pane supports grouping, selection modifiers, stage/unstage, and commit UI.
  - Diff viewer pane renders file content, single diff, multi-diff, or binary-file placeholder.

### Keyboard Handling
`utils/keyboard.rs` provides pane cycling, refresh key, and shortcuts help toggle while respecting text-focus state.

## Key End-to-End Workflows

### Open Repository
1. UI emits `OpenRepoRequested`.
2. Reducer returns `Effect::OpenRepo`.
3. Executor runs `Job::OpenRepo` and sends `RepoOpened`.
4. Reducer stores repo and returns batched load effects.

### Stage/Unstage and Commit
1. Changed-files pane emits stage/unstage or commit-related messages.
2. Reducer emits relevant effects/jobs.
3. Executor runs Git operations.
4. On success, reducer refreshes status/history/changed-files and clears commit UI state.

### Branch Checkout with Uncommitted Changes
1. User requests checkout.
2. Executor checks for local changes.
3. If changes exist, dialog prompts stash/discard/cancel.
4. For remote branches, local-name conflict handling is performed before tracking branch creation.

## Tests and Current Verification
Executed on February 21, 2026:

- `cargo test -q` passed.
- Observed passing groups include app tests, git tests, and ui_core tests.
- No failing tests were reported.

Test locations:

- `crates/app/tests/integration_test.rs`
- `crates/app/tests/reducer_test.rs`
- `crates/git/tests/integration_test.rs`
- `crates/ui_core/src/*` (unit tests in module files)

## Developer Notes
- `crates/git/examples/api_spike.rs` is a feasibility spike and not part of normal runtime flow.
- The reducer/effect/job separation is the main architectural seam for adding new features.
- New UI frameworks can reuse `crates/app` and `crates/ui_core` while replacing `crates/ui_egui`.
