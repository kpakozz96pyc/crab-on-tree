# CrabOnTree Implementation Plan

## Scope
This plan contains only pending and actionable work. Completed historical phases (docking integration and layout persistence) are considered done.

## Priority 1: Stabilization
1. Remove or integrate currently unused pane wrapper functions in `crates/ui_egui/src/panes/mod.rs` to eliminate dead-code warnings.
2. Confirm keyboard navigation/focus behavior after dock rearrangements.
3. Validate config persistence robustness for malformed `dock_layout` values.

## Priority 2: Validation and QA
1. Manual test matrix:
   - open repo, load commits, select commit/file, view diffs
   - rearrange/split panes, close app, reopen, verify layout restore
   - large repo smoke test (high commit/file count)
2. Add/expand automated tests where feasible for:
   - config load/save fallback behavior (`dock_layout` absent/invalid)
   - reducer/effect flows tied to changed files and diff loading

## Priority 3: UX Improvements
1. Add user-facing layout controls:
   - reset layout to defaults
   - optional "save layout now" action
2. Document docking behavior and shortcuts in in-app help or a dedicated help section.

## Priority 4: Technical Debt
1. Review cloning in UI render path (`repo_data` extraction in `crates/ui_egui/src/main.rs`) and reduce unnecessary allocations if profiling shows impact.
2. Investigate and plan remediation for future incompatibility warning (`ashpd v0.8.1`) during dependency updates.

## Definition of Done for Next Iteration
1. No avoidable warnings in normal `cargo check`.
2. Dock layout persistence validated by tests or repeatable manual QA checklist.
3. A basic "Reset Layout" user action is available.
