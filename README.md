# CrabOnTree 🦀

A modern Git GUI written in Rust, powered by [gitoxide](https://github.com/Byron/gitoxide) and [egui](https://github.com/emilk/egui).

## Features (Phase 0)

- ✅ Open Git repositories
- ✅ View current branch and HEAD
- ✅ List all branches
- ✅ Repository status summary
- ✅ Recent repositories list
- ✅ Dark and light themes
- ✅ Async Git operations (non-blocking UI)
- ✅ Structured logging with tracing
- ✅ Cross-platform support (Linux, macOS, Windows)

## Screenshots

*Coming soon - Phase 0 implementation complete*

## Architecture

CrabOnTree uses a clean, layered architecture:

- **Git Layer**: High-level wrapper around gitoxide
- **App Layer**: State management with Elm-like architecture
- **UI Core**: Framework-agnostic UI primitives (themes, colors, shortcuts)
- **UI Layer**: egui-based frontend

Key architectural features:
- **Message-passing**: All state updates go through a pure reducer function
- **Async jobs**: Git operations run in background thread with tokio
- **Effect system**: Clear separation of business logic and side effects
- **Non-blocking UI**: UI never freezes, even during heavy Git operations

See [Architecture Documentation](docs/architecture.md) for details.

## Building

### Prerequisites

- Rust 1.70 or later
- Git (for running integration tests)

### Build Commands

```bash
# Debug build
cargo build --workspace

# Release build
cargo build --release

# Run tests
cargo test --workspace

# Run the application
cargo run --bin crabontree

# Run with debug logging
RUST_LOG=debug cargo run --bin crabontree
```

## Running

### From Source

```bash
cargo run --bin crabontree
```

### Release Binary

```bash
cargo build --release
./target/release/crabontree
```

## Configuration

Configuration is stored in a platform-specific location:

- **Linux**: `~/.config/crabontree/CrabOnTree/config.toml`
- **macOS**: `~/Library/Application Support/com.crabontree.CrabOnTree/config.toml`
- **Windows**: `%APPDATA%\crabontree\CrabOnTree\config\config.toml`

### Configuration Options

```toml
theme = "dark"          # or "light"
max_recent = 10         # Maximum recent repositories

[[recent_repos]]
"/path/to/repo1"
```

## Development

See [Development Guide](docs/development.md) for:
- Project structure
- Development workflow
- How to add new features
- Testing guidelines
- Debugging tips

## Logging

CrabOnTree uses structured logging with the `tracing` crate. Control log levels with the `RUST_LOG` environment variable:

```bash
# Debug logging
RUST_LOG=debug cargo run

# Trace Git operations
RUST_LOG=crabontree_git=trace cargo run

# Info level (default)
cargo run
```

See [Logging Documentation](docs/logging.md) for details.

## Testing

```bash
# Run all tests
cargo test --workspace

# Run specific crate tests
cargo test -p crabontree-git

# Run with logging output
RUST_LOG=debug cargo test -- --nocapture
```

## Project Status

**Current Phase**: Phase 0 - Foundation ✅

### Completed
- [x] Workspace setup with 4 crates
- [x] Git layer with gitoxide integration
- [x] Application state management
- [x] Async job system with tokio
- [x] UI core (themes, colors, shortcuts)
- [x] egui frontend with full UI
- [x] Logging and diagnostics
- [x] Comprehensive documentation

### Future Phases

**Phase 1**: Commit History
- View commit log
- Commit details
- Commit graph visualization

**Phase 2**: Diffs and Staging
- File-level status
- Diff viewer
- Stage/unstage files

**Phase 3**: Commit Creation
- Commit message editor
- Staging area UI
- Create commits

**Phase 4**: Advanced Features
- Branch management
- Remote operations
- Merge/rebase support

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Acknowledgments

- [gitoxide](https://github.com/Byron/gitoxide) - Fast and safe Git implementation in Rust
- [egui](https://github.com/emilk/egui) - Immediate mode GUI framework
- [tokio](https://tokio.rs/) - Async runtime for Rust

## Contributing

Contributions are welcome! Please see the [Development Guide](docs/development.md) for guidelines.

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests
5. Submit a pull request
