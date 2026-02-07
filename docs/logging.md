# Logging and Diagnostics

CrabOnTree uses the `tracing` crate for structured logging. This provides detailed insights into the application's behavior, which is invaluable for debugging and understanding the flow of data through the system.

## Enabling Logging

Logging is controlled by the `RUST_LOG` environment variable. By default, the application logs at the `INFO` level for most crates and `DEBUG` level for CrabOnTree-specific crates.

### Log Levels

- **TRACE**: Very detailed information, including gitoxide internals
- **DEBUG**: Detailed information useful for debugging
- **INFO**: General informational messages about application state
- **WARN**: Warning messages for recoverable issues
- **ERROR**: Error messages for failures

## Usage Examples

### Default Logging (INFO level)

```bash
cargo run --bin crabontree
```

### Debug Logging

```bash
RUST_LOG=debug cargo run --bin crabontree
```

### Trace Logging for Git Operations

```bash
RUST_LOG=crabontree_git=trace cargo run --bin crabontree
```

### Specific Crate Logging

```bash
# Only show Git layer debug messages
RUST_LOG=crabontree_git=debug cargo run

# Show both Git and App layer debug messages
RUST_LOG=crabontree_git=debug,crabontree_app=debug cargo run
```

### Production Logging

```bash
# Only show warnings and errors
RUST_LOG=warn cargo run --release
```

## What Gets Logged

### INFO Level

- Application startup
- Repository opened/closed
- Configuration loaded/saved
- Major state transitions

### DEBUG Level

- Job submissions and completions
- Message processing
- Branch counts and HEAD information
- Configuration operations

### TRACE Level

- Detailed gitoxide operations
- File system operations
- Git reference resolution

## Log Output Format

Each log line includes:

- **Timestamp**: When the event occurred
- **Level**: Log level (INFO, DEBUG, etc.)
- **Target**: The crate and module that generated the log
- **Thread ID**: The thread that generated the log
- **Line Number**: The source location
- **Message**: The log message

Example:
```
2024-02-05T12:34:56.789Z INFO crabontree::main [12345] main.rs:23: Starting CrabOnTree
2024-02-05T12:34:56.890Z DEBUG crabontree_git::repo [67890] repo.rs:45: Opened repository at /path/to/repo
```

## Adding Logging to New Code

### Instrument Functions

Use the `#[instrument]` attribute to automatically log function entry/exit:

```rust
use tracing::instrument;

#[instrument(skip(self))]
pub fn get_branches(&self) -> Result<Vec<String>, GitError> {
    // Function body
}
```

### Manual Logging

```rust
tracing::info!("Repository opened at {}", path.display());
tracing::debug!("Found {} branches", branches.len());
tracing::warn!("Failed to load config: {}, using defaults", error);
tracing::error!("Git operation failed: {}", error);
```

### Logging with Context

```rust
tracing::debug!(
    repo_path = %path.display(),
    branch_count = branches.len(),
    "Branches loaded successfully"
);
```

## Performance Considerations

- Logging at DEBUG and TRACE levels can impact performance
- In production, use INFO or WARN level
- The `#[instrument]` macro has minimal overhead when the level is disabled
- Structured logging is more efficient than formatted strings

## Troubleshooting

### No Logs Appearing

- Check that `RUST_LOG` is set correctly
- Ensure the application is compiled with tracing support
- Verify the log level is appropriate for what you want to see

### Too Much Output

- Increase the log level (e.g., from DEBUG to INFO)
- Filter to specific crates: `RUST_LOG=crabontree_app=info`

### Missing Specific Information

- Lower the log level (e.g., from INFO to DEBUG)
- Check if the code you're interested in has logging enabled
- Use TRACE level for very detailed output
