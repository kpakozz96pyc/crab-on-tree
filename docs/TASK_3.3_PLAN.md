# Task 3.3: Performance Testing - Implementation Plan

**Estimated Time**: 3-4 hours
**Priority**: HIGH - Ensure production readiness
**Status**: Starting implementation

## Overview

Test the application with large repositories and demanding scenarios to ensure it performs well in real-world usage. Document performance characteristics and add optimizations if needed.

## Test Scenarios

### 1. Large Working Directory (10,000+ Files)

**Scenario**: Repository with 10,000+ changed files
- Load working directory status
- Scroll through file list
- Search/filter files
- Stage/unstage operations
- Virtual scrolling performance

**Success Criteria**:
- Initial load < 2 seconds
- Scrolling is smooth (60 FPS)
- Search/filter < 100ms
- Stage/unstage < 500ms

### 2. Long Commit History (1,000+ Commits)

**Scenario**: Repository with 1,000+ commits
- Load commit history
- Scroll through commits
- Search/filter commits
- Select commit and load diff

**Success Criteria**:
- Load 100 commits < 1 second
- Scrolling is smooth
- Search/filter < 100ms
- Commit diff load < 500ms

### 3. Large Commit Diffs

**Scenario**: Commit with 100+ files, 10,000+ line changes
- Load large diff
- Scroll through diff
- Expand/collapse file sections

**Success Criteria**:
- Diff load < 2 seconds
- Scrolling is smooth
- No UI freezing

### 4. Rapid Operations

**Scenario**: Stress test with rapid user actions
- Rapid searching
- Rapid staging/unstaging
- Rapid commit selection
- Panel switching

**Success Criteria**:
- No lag or freezing
- UI remains responsive
- Memory stable (no leaks)

## Performance Testing Approach

### Phase 1: Create Test Repositories (1 hour)

**Create Large Test Repo**:
```bash
# Script to generate test repository
mkdir test-large-repo && cd test-large-repo
git init

# Generate 10,000 files
for i in {1..10000}; do
    echo "File $i content" > "file_$i.txt"
done

# Create initial commit
git add .
git commit -m "Initial commit with 10,000 files"

# Modify all files
for i in {1..10000}; do
    echo "Modified content $i" >> "file_$i.txt"
done

# Now have 10,000 modified files in working dir
```

**Create Long History Repo**:
```bash
mkdir test-history-repo && cd test-history-repo
git init

# Generate 1,000 commits
for i in {1..1000}; do
    echo "Commit $i" > file.txt
    git add file.txt
    git commit -m "Commit $i: Some message here with details"
done
```

**Create Large Diff Repo**:
```bash
mkdir test-diff-repo && cd test-diff-repo
git init

# Create commit with 100 files, 10,000 lines total
for i in {1..100}; do
    for j in {1..100}; do
        echo "Line $j in file $i" >> "file_$i.txt"
    done
done

git add .
git commit -m "Large commit with 100 files, 10,000 lines"
```

### Phase 2: Manual Performance Testing (1 hour)

**Test Checklist**:
- [ ] Open repo with 10,000 files
- [ ] Measure initial load time
- [ ] Test scrolling smoothness
- [ ] Test search performance
- [ ] Test staging operations
- [ ] Open repo with 1,000 commits
- [ ] Measure commit load time
- [ ] Test commit search
- [ ] Test commit selection
- [ ] Load large diff
- [ ] Test diff scrolling
- [ ] Rapid operation stress test

### Phase 3: Profile & Optimize (1-2 hours)

**Profiling Tools**:
- Use `tracing` logs to measure operation times
- Add performance markers
- Identify bottlenecks

**Common Optimizations**:
- Lazy loading (if needed)
- Pagination (if needed)
- Caching (if needed)
- Debouncing (if needed)

### Phase 4: Documentation (30 min)

**Document**:
- Test results
- Performance characteristics
- Known limitations
- Recommendations for large repos

## Implementation Steps

### Step 1: Add Performance Logging

Add timing measurements to key operations:
```rust
use std::time::Instant;

let start = Instant::now();
// ... operation ...
tracing::info!("Operation took {:?}", start.elapsed());
```

**Key Operations to Measure**:
- `get_working_dir_status()`
- `get_commit_history()`
- `get_commit_diff()`
- `stage_file()` / `unstage_file()`
- UI rendering (frame time)

### Step 2: Create Test Script

Create a shell script to generate test repositories:
```bash
#!/bin/bash
# scripts/create_test_repos.sh

# Create test repositories for performance testing
```

### Step 3: Run Tests

Manually test with generated repositories and record results.

### Step 4: Add Optimizations (If Needed)

Based on test results, add optimizations:
- Commit history pagination
- Diff lazy loading
- Caching strategies
- Virtual scrolling improvements

### Step 5: Document Results

Create performance documentation with:
- Test results table
- Performance graphs (if needed)
- Recommendations
- Known limitations

## Success Criteria

- ✅ Can handle 10,000+ files without freezing
- ✅ Can handle 1,000+ commits smoothly
- ✅ Search/filter operations < 100ms
- ✅ UI remains responsive during operations
- ✅ Memory usage is reasonable
- ✅ Performance characteristics documented

## Expected Results

Based on current architecture:

**Working Directory (10,000 files)**:
- Load: ~1-2 seconds (git2 is fast)
- Scroll: Smooth (virtual scrolling)
- Search: ~10-50ms (local filtering)
- Stage: ~500ms per file

**Commit History (1,000 commits)**:
- Load 100: ~200-500ms
- Search: ~10-50ms
- Select: ~100-300ms for diff

**Large Diff**:
- Load: ~500ms-2s depending on size
- Render: Should be smooth (collapsible sections)

## Potential Bottlenecks

1. **Working Directory Status**: git2 status check on 10,000 files
2. **Commit History**: Loading all commits at once
3. **Diff Generation**: Large diffs take time
4. **UI Rendering**: egui should handle well

## Optimization Strategies (If Needed)

### If Working Dir Too Slow
- Add pagination/lazy loading
- Limit initial file count
- Add "Load More" button

### If Commit History Too Slow
- Implement pagination (already at 100 limit)
- Add "Load More" option
- Consider background loading

### If Diff Too Slow
- Lazy load collapsed sections
- Limit visible lines
- Add pagination for very large files

### If UI Lag
- Reduce frame rate during heavy ops
- Add loading indicators
- Debounce expensive operations

## Testing Environment

**Hardware Requirements**:
- Modern CPU (for reference)
- 8GB+ RAM
- SSD (for git operations)

**Test Configuration**:
- Debug build (for profiling)
- Release build (for real performance)
- Compare both

## Deliverables

1. Test repositories (scripts to generate)
2. Performance test results document
3. Performance logging in code
4. Optimization implementations (if needed)
5. Performance recommendations guide

---

**Ready to implement!**
