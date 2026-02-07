# Hybrid Approach: gix + git2

## Strategy

**Read with gix**: Fast, safe, pure Rust
- ✅ Commit history
- ✅ File-level diffs
- ✅ Tree walking
- ✅ Config reading
- ✅ Branch listing

**Write with git2**: Mature, proven, well-documented
- ✅ Staging/unstaging files
- ✅ Creating commits
- ✅ Updating references
- ✅ Working directory status

## Why This Works

### Proven Pattern
- Used by: GitUI, gitoxide (during transition), many others
- Separates concerns cleanly
- Reduces risk significantly

### Performance
- gix: Pure Rust, zero-copy, very fast for reading
- git2: libgit2 is optimized, battle-tested for writing
- Best of both worlds

### Future-Proof
- Can migrate to pure gix later if/when APIs mature
- No architectural changes needed
- Incremental improvement path

## Implementation

### Dependencies
```toml
[workspace.dependencies]
gix = { version = "0.66", features = ["blocking-network-client", "worktree-mutation"] }
git2 = "0.19"  # NEW: Add libgit2 bindings
```

### Crate Structure
```
crates/git/
├── src/
│   ├── lib.rs          # Public API (unchanged)
│   ├── error.rs        # Unified error type
│   ├── repo.rs         # Main repository wrapper
│   ├── read.rs         # NEW: gix-based read operations
│   └── write.rs        # NEW: git2-based write operations
```

### API Design (Unchanged)
```rust
// Public API remains the same
impl GitRepository {
    // Read operations (gix)
    pub fn get_commit_history(&self) -> Result<Vec<Commit>>;
    pub fn get_commit_diff(&self) -> Result<FileDiff>;
    pub fn get_working_dir_status(&self) -> Result<Vec<WorkingDirFile>>;

    // Write operations (git2)
    pub fn stage_file(&self, path: &Path) -> Result<()>;
    pub fn unstage_file(&self, path: &Path) -> Result<()>;
    pub fn create_commit(&self, message: &str) -> Result<String>;
}
```

### Internal Implementation
```rust
pub struct GitRepository {
    path: PathBuf,
    gix_repo: gix::Repository,     // For reading
    git2_repo: git2::Repository,   // For writing
}

impl GitRepository {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        let gix_repo = gix::discover(path)?;
        let git2_repo = git2::Repository::open(path)?;

        Ok(Self {
            path: path.to_path_buf(),
            gix_repo,
            git2_repo,
        })
    }
}
```

## Error Handling

### Unified Error Type
```rust
#[derive(Debug, thiserror::Error)]
pub enum GitError {
    // Existing variants...

    #[error("git2 error: {0}")]
    Git2Error(#[from] git2::Error),
}
```

## Benefits

### ✅ Immediate Advantages
1. **Known APIs**: git2 has excellent docs and examples
2. **Reduced Risk**: Both libraries are proven
3. **Faster Implementation**: Less API hunting
4. **Better Testing**: Can compare against git CLI

### ✅ Long-term Advantages
1. **Migration Path**: Can move to pure gix incrementally
2. **Fallback Option**: If gix has issues, git2 is there
3. **Community Knowledge**: Many examples of git2 usage
4. **Stability**: libgit2 is rock-solid

## Time Savings

### Original Estimate (Pure gix)
- API research: 4-6 hours
- Implementation challenges: 10-15 hours buffer
- **Total risk**: 14-21 hours

### Hybrid Approach
- Setup: 2-3 hours
- No API research needed (git2 docs are excellent)
- Known patterns
- **Risk reduced**: 8-10 hours saved

### Revised Phase 2a Estimate
- **Original**: 50-70 hours
- **With hybrid**: 45-60 hours (10-15% faster)
- **Confidence**: Much higher

## Migration Strategy (Future)

When gix APIs mature:
1. Move one operation at a time
2. Test thoroughly
3. Keep git2 as fallback
4. No changes to public API
5. Internal implementation detail only

## Examples in the Wild

### Projects Using Hybrid Approach
- **GitUI**: Uses git2 for most operations, gix for some
- **gitoxide**: Byron (author) uses both during development
- **Many Git tools**: Proven pattern

## Next Steps

1. ✅ Add git2 to workspace dependencies
2. ✅ Update GitRepository to hold both repos
3. ✅ Implement status with git2
4. ✅ Implement staging with git2
5. ✅ Implement commits with git2
6. ✅ Keep existing gix-based reading

---

**Status**: Ready to implement
**Time to setup**: 2-3 hours
**Risk level**: 🟢 LOW
**Confidence**: 🟢 HIGH
