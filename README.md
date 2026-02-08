# CrabOnTree 🦀🌳

A fast, modern Git GUI built with Rust, powered by [gitoxide](https://github.com/Byron/gitoxide) and [egui](https://github.com/emilk/egui).

**Status**: Phase 2a Complete + 4-Pane Layout - Production Ready ✅

[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE-MIT)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org)

## ✨ Features

### Core Functionality
- 📁 **Repository Management** - Open and browse Git repositories
- 🖼️ **4-Pane Layout** - Modern horizontal layout with branches, file tree, changed files, and file viewer
- 🌿 **Branch Navigation** - Visual branch/tag navigation with checkout support
- 📂 **Full File Tree** - Browse entire repository, not just changed files
- 📝 **Smart Staging** - Stage/unstage files individually or in bulk with optimized batch operations
- 💬 **Rich Commits** - Create commits with multi-line messages and author identity
- 📜 **Commit History** - Browse commit history with detailed diffs
- 🔍 **Powerful Search** - Search files and commits instantly
- ⌨️ **Keyboard First** - Complete keyboard navigation with vim-style shortcuts
- 🎨 **Modern UI** - Clean, responsive interface with dark/light themes and layout toggle

### Performance
- 🚀 **Blazing Fast** - Handles 1,000+ files and 500+ commits smoothly
- ⚡ **Virtual Scrolling** - Smooth scrolling even with thousands of items
- 🔄 **Async Operations** - Non-blocking UI, always responsive
- 💾 **Memory Efficient** - Smart caching and lazy loading

### User Experience
- 🎯 **Search Everything** - Filter files by path, commits by message/author/hash
- 📍 **Focus Indicators** - Visual markers show keyboard focus
- ℹ️ **Help Dialog** - Press `?` for comprehensive keyboard shortcuts
- 🔔 **Real-time Updates** - Live character/line counts, staged file counts
- 📊 **Progress Tracking** - Visual feedback for long operations

## 🚀 Quick Start

### Installation

#### From Source

```bash
# Clone the repository
git clone https://github.com/yourusername/crabontree.git
cd crabontree

# Build release version
cargo build --release

# Run
./target/release/crabontree
```

#### Prerequisites
- Rust 1.70 or later
- Git

### Usage

1. **Open a repository**: Click "📂 Open Repository" or use a recent repository
2. **View changes**: See all modified files in the working directory
3. **Stage files**: Click `+` or press `Space` on focused files
4. **Write commit message**: Type in the commit message box
5. **Create commit**: Click "📝 Commit" or press `Ctrl+Enter`
6. **Browse history**: View commits and their diffs

## ⌨️ Keyboard Shortcuts

### Navigation
| Key | Action |
|-----|--------|
| `↑` / `k` | Previous item |
| `↓` / `j` | Next item |
| `Enter` | View commit details |
| `Space` | Toggle file staging |
| `Home` / `End` | Jump to edges |
| `g g` / `G` | Vim-style top/bottom |

### Search
| Key | Action |
|-----|--------|
| `/` | Focus file search |
| `Ctrl+F` | Focus commit search |
| `Esc` | Clear/blur |

### Panels (Classic Mode)
| Key | Action |
|-----|--------|
| `1` | Working directory |
| `2` | Commit message |
| `3` | Commit history |
| `Tab` | Cycle panels |

### Panels (4-Pane Mode)
| Key | Action |
|-----|--------|
| `1` | Branch tree |
| `2` | File tree |
| `3` | Changed files |
| `4` | File viewer |
| `Tab` / `Shift+Tab` | Cycle panes |
| `Shift+L` | Toggle layout mode |

### Actions
| Key | Action |
|-----|--------|
| `c` | Focus commit message |
| `Ctrl+Enter` | Create commit |
| `a` / `u` | Stage/unstage all |
| `r` | Refresh |
| `?` | Show help |

[**Full Keyboard Reference →**](docs/KEYBOARD_SHORTCUTS.md)

## 📖 Documentation

- [**User Guide**](docs/USER_GUIDE.md) - Complete usage instructions
- [**Architecture**](docs/ARCHITECTURE.md) - System design and components
- [**Contributing**](docs/CONTRIBUTING.md) - Developer guidelines
- [**Testing**](docs/TESTING.md) - Testing strategy and guides
- [**Performance**](docs/TASK_3.3_COMPLETE.md) - Performance characteristics

## 🏗️ Architecture

CrabOnTree uses a clean, Elm-inspired architecture:

```
┌─────────────────────────────────────┐
│         UI Layer (egui)             │
│   - Immediate mode rendering        │
│   - User interaction handling       │
└──────────────┬──────────────────────┘
               │ Messages
┌──────────────▼──────────────────────┐
│       App Layer (Pure State)        │
│   - Reducer (pure function)         │
│   - Effects (side effect specs)     │
└──────────────┬──────────────────────┘
               │ Jobs
┌──────────────▼──────────────────────┐
│      Executor (Async Runtime)       │
│   - Job execution with tokio        │
│   - Message passing back to UI      │
└──────────────┬──────────────────────┘
               │ Git Operations
┌──────────────▼──────────────────────┐
│      Git Layer (gix + git2)         │
│   - gix for read operations         │
│   - git2 for write operations       │
└─────────────────────────────────────┘
```

**Key Principles**:
- **Pure Reducer**: All state updates through deterministic function
- **Effect System**: Side effects as data, executed separately
- **Async Jobs**: Background Git operations with tokio
- **Hybrid Git**: gix for speed, git2 for reliability

[**Full Architecture Docs →**](docs/ARCHITECTURE.md)

## 🧪 Testing

```bash
# Run all tests (38 tests across workspace)
cargo test --workspace

# Run specific crate tests
cargo test -p crabontree-git    # Git layer (19 tests)
cargo test -p crabontree-app    # App layer (14 tests)

# Run with logging
RUST_LOG=debug cargo test -- --nocapture

# Performance tests
./scripts/create_test_repos.sh  # Create test repositories
cargo run --release              # Test with large repos
```

**Test Coverage**: 38 automated tests covering:
- Git operations (status, staging, commits, diffs)
- State management (reducers, effects, jobs)
- Integration workflows
- Error handling

## 📊 Performance

Tested and optimized for real-world use:

| Scenario | Performance |
|----------|-------------|
| 1,000 files | < 1s load, smooth scrolling |
| 500 commits | < 500ms load, smooth scrolling |
| Large diff (50 files) | < 1s load, smooth scrolling |
| Search/filter | < 100ms (instant) |
| Memory usage | 150-250 MB typical |

**Optimizations**:
- Virtual scrolling for large lists
- Chunked batch operations (500 files/chunk)
- Async git operations (non-blocking)
- Smart caching and lazy loading

[**Performance Details →**](docs/TASK_3.3_COMPLETE.md)

## 🗂️ Project Structure

```
crabontree/
├── crates/
│   ├── git/           # Git operations (gix + git2)
│   ├── app/           # State management (Elm architecture)
│   ├── ui_core/       # Framework-agnostic UI primitives
│   └── ui_egui/       # egui-based frontend (main binary)
├── docs/              # Comprehensive documentation
├── scripts/           # Utility scripts (test repos, etc.)
└── test-repos/        # Generated test repositories
```

## 🎯 Project Status

**Current Phase**: Phase 2a + 4-Pane Layout ✅ COMPLETE

### Completed (65-70 hours)

**Sprint 1**: Foundation
- ✅ Working directory status
- ✅ Full diff implementation
- ✅ Working directory UI panel

**Sprint 2**: Staging & Commits
- ✅ Staging operations (individual + batch)
- ✅ Commit creation
- ✅ Author identity management
- ✅ Performance optimizations (chunking, virtual scrolling)

**Sprint 3**: UX Enhancements
- ✅ Search and filter (files + commits)
- ✅ Enhanced keyboard shortcuts (vim-style navigation)
- ✅ Performance testing and validation

**4-Pane Layout Implementation**: Professional GUI Experience
- ✅ 4-pane horizontal layout (branches, file tree, changed files, file viewer)
- ✅ Branch and tag navigation with checkout support
- ✅ Full repository file tree browser
- ✅ Grouped changed files (staged/unstaged/untracked/conflicted)
- ✅ File content and diff viewer with syntax highlighting
- ✅ Resizable panes with drag separators
- ✅ Layout toggle (Classic ↔ 4-Pane)
- ✅ Configuration persistence

### Future Phases

**Phase 2b**: Advanced Features
- Branch creation/deletion
- Stash management
- Enhanced conflict resolution

**Phase 2c**: Remote Operations
- Fetch/pull/push
- Remote branch tracking
- Conflict resolution

**Phase 3**: Collaboration
- Merge operations
- Rebase support
- Cherry-pick

## 🛠️ Technology Stack

- **Language**: Rust 2021 Edition
- **Git**: [gitoxide](https://github.com/Byron/gitoxide) (gix) + [git2](https://github.com/rust-lang/git2-rs)
- **UI**: [egui](https://github.com/emilk/egui) - Immediate mode GUI
- **Async**: [tokio](https://tokio.rs/) - Async runtime
- **Logging**: [tracing](https://github.com/tokio-rs/tracing) - Structured logging
- **Testing**: [cargo test](https://doc.rust-lang.org/cargo/) + integration tests

## 🤝 Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](docs/CONTRIBUTING.md) for:
- Development setup
- Code style guidelines
- Testing requirements
- Pull request process

### Quick Contribution Guide

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes and add tests
4. Ensure all tests pass (`cargo test --workspace`)
5. Commit your changes (`git commit -m 'Add amazing feature'`)
6. Push to the branch (`git push origin feature/amazing-feature`)
7. Open a Pull Request

## 📝 License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## 🙏 Acknowledgments

Built with excellent open-source libraries:
- [**gitoxide**](https://github.com/Byron/gitoxide) - Byron Wasti - Fast, safe Git in Rust
- [**egui**](https://github.com/emilk/egui) - Emil Ernerfeldt - Immediate mode GUI
- [**tokio**](https://tokio.rs/) - Tokio Contributors - Async runtime
- [**git2-rs**](https://github.com/rust-lang/git2-rs) - Rust Language Team - libgit2 bindings
- [**tracing**](https://github.com/tokio-rs/tracing) - Tokio Contributors - Structured logging

## 📬 Contact

- **Issues**: [GitHub Issues](https://github.com/yourusername/crabontree/issues)
- **Discussions**: [GitHub Discussions](https://github.com/yourusername/crabontree/discussions)

---

**Made with 🦀 and ❤️ by the Rust community**
