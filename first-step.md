# Phase 0 — Project Setup & Architecture (Rust Git GUI with gitoxide)

## Goal

The goal of Phase 0 is to set up a solid technical foundation for a 100% Rust Git GUI application using:

- gitoxide (`gix`) as the Git backend
- egui and/or iced as UI frontends
- A clean layered architecture
- Async job system
- Reproducible build environment

After this phase, the project must compile, launch, and open a repository with basic metadata loaded.

This phase does NOT include full Git features yet.

---

## Tech Stack

### Core
- Rust (stable)
- Cargo workspace
- gitoxide (`gix` crate)
- tokio (async runtime)
- tracing (logging)

### UI
- egui (via eframe) — primary MVP frontend
- iced — optional secondary frontend (scaffold only)

### Serialization / Config
- serde
- toml
- directories

---

## Repository Structure

Create the following workspace layout:
git-gui/
Cargo.toml # workspace root
crates/
app/ # application state & logic
git/ # gitoxide integration
ui_core/ # themes, shortcuts, shared UI models
ui_egui/ # egui frontend (main MVP UI)
ui_iced/ # iced frontend (stub only)
assets/
fonts/
icons/
docs/
architecture.md
All crates must compile independently.

UI crates must depend only on `app` and `ui_core`, never directly on `git`.

---

## Task 1 — Initialize Workspace

### Description

Set up Cargo workspace and base crates.

### Steps

1. Create root `Cargo.toml`:

```toml
[workspace]
members = [
    "crates/app",
    "crates/git",
    "crates/ui_core",
    "crates/ui_egui",
    "crates/ui_iced"
]
resolver = "2"
Initialize each crate with:

cargo new crates/<name> --lib
cargo new crates/ui_egui --bin
cargo new crates/ui_iced --bin
Configure dependencies.

Acceptance Criteria

cargo build succeeds

All crates compile

No circular dependencies

Task 2 — Git Layer Integration
Description

Create minimal integration with gitoxide.

This layer must abstract all Git access.

Steps

Add dependency to crates/git/Cargo.toml:

gix = "0.66"
tokio = { version = "1", features = ["full"] }
anyhow = "1"


Implement basic API:

Create crates/git/src/repo.rs:

pub struct GitRepository {
    pub path: PathBuf,
    pub repo: gix::Repository,
}


Implement functions:

open_repository(path)

get_head()

get_branches()

get_status()

Example:

pub fn open_repository(path: PathBuf) -> Result<GitRepository> {
    let repo = gix::open(path)?;
    Ok(GitRepository { path, repo })
}


Add basic error handling using anyhow.

Acceptance Criteria

Can open existing repository

Can read HEAD

Can list branches

No panic on invalid repo

Task 3 — Application State Model
Description

Create central application model and message system.

This crate must be UI-agnostic.

Steps

In crates/app, define:

pub struct AppState {
    pub current_repo: Option<RepoState>,
    pub loading: bool,
    pub error: Option<String>,
}


Define RepoState:

pub struct RepoState {
    pub path: PathBuf,
    pub head: String,
    pub branches: Vec<String>,
}


Define messages/events:

pub enum AppMessage {
    OpenRepo(PathBuf),
    RepoOpened(RepoState),
    Error(String),
}


Implement reducer:

pub fn update(state: &mut AppState, msg: AppMessage) -> Effect


Implement Effect enum:

pub enum Effect {
    None,
    OpenRepo(PathBuf),
}

Acceptance Criteria

AppState can be mutated only through messages

No Git logic inside app crate

Reducer is pure (no IO)

Task 4 — Async Job System
Description

Implement background task runner for Git operations.

Steps

Create job executor in app crate:

tokio runtime

mpsc channels

Create Job enum:

pub enum Job {
    OpenRepo(PathBuf),
}


Worker loop:

Receives Job

Executes via git crate

Returns AppMessage

Cancellation support (basic):

Track job id

Drop outdated jobs

Acceptance Criteria

Git operations run off main thread

UI never blocks

Errors propagated to AppState

Task 5 — UI Core (Theme & Shared Models)
Description

Create shared UI configuration.

Steps

Create Theme struct:

pub struct Theme {
    pub background: Color,
    pub foreground: Color,
    pub accent: Color,
}


Define default themes:

Dark

Light

Define shortcuts model:

pub struct Shortcut {
    pub action: Action,
    pub key: String,
}


Expose via ui_core crate.

Acceptance Criteria

UI crates depend on ui_core

No theme logic in UI code

Task 6 — egui Frontend Bootstrap
Description

Create minimal egui application that connects to AppState.

Steps

Add dependencies:

eframe = "0.27"
egui = "0.27"
app = { path = "../app" }
ui_core = { path = "../ui_core" }


Implement main.rs:

Create App struct

Hold AppState

Process messages

Render UI

Implement minimal UI:

Top bar

"Open Repository" button

Status panel

File dialog (temporary):

Use rfd crate

Acceptance Criteria

Application window opens

Can select repository

Repo info displayed

No crashes

Task 7 — iced Frontend Stub (Optional)
Description

Create minimal iced frontend for future support.

Steps

Setup iced application

Connect to AppState

Display placeholder UI

No advanced features required.

Acceptance Criteria

Compiles

Shows window

Displays "Not implemented"

Task 8 — Logging and Diagnostics
Description

Add structured logging.

Steps

Add dependencies:

tracing
tracing-subscriber


Initialize logger in main

Add spans in:

Git open

Job execution

Errors

Acceptance Criteria

Logs visible in console

Errors include context

Task 9 — Documentation
Description

Document architecture and decisions.

Steps

Create docs/architecture.md with:

Layer diagram

Data flow

Message flow

Job system

Error handling

Acceptance Criteria

Architecture documented

New contributor can understand structure

Phase 0 Completion Criteria

Phase 0 is considered complete when:

 Workspace builds

 egui UI launches

 Repository can be opened

 Branches visible

 No blocking IO in UI thread

 Errors handled gracefully

 Logging works

 Docs written

Notes for AI Coding Agent (Claude Code)

Do NOT mix UI and Git logic

Always add error context

Prefer explicit types

Avoid global state

Prefer message-passing over shared mutability

Write minimal tests where possible

Keep APIs stable

When in doubt, prioritize architecture clarity over feature count.
