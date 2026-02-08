#!/bin/bash

# Script to create test repositories for performance testing
# Usage: ./scripts/create_test_repos.sh

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TEST_DIR="$SCRIPT_DIR/../test-repos"

echo "Creating test repositories in: $TEST_DIR"
mkdir -p "$TEST_DIR"
cd "$TEST_DIR"

# Clean up existing test repos
rm -rf large-files long-history large-diff 2>/dev/null || true

echo ""
echo "=== Creating Test Repository 1: Large Working Directory ==="
echo "Generating repository with 1,000 modified files..."

mkdir -p large-files
cd large-files
git init -q
git config user.name "Test User"
git config user.email "test@example.com"

# Create initial commit with 1,000 files
echo "Creating initial files..."
for i in $(seq 1 1000); do
    dir="dir_$((i / 100))"
    mkdir -p "$dir"
    echo "Initial content for file $i" > "$dir/file_$i.txt"
done

git add . >/dev/null 2>&1
git commit -q -m "Initial commit with 1,000 files"

# Modify all files to create working directory changes
echo "Modifying all files..."
for i in $(seq 1 1000); do
    dir="dir_$((i / 100))"
    echo "Modified content $i at $(date)" >> "$dir/file_$i.txt"
done

echo "✓ Large working directory created: $(pwd)"
echo "  - 1,000 modified files (can test up to 10,000 by increasing loop)"
cd ..

echo ""
echo "=== Creating Test Repository 2: Long Commit History ==="
echo "Generating repository with 500 commits..."

mkdir -p long-history
cd long-history
git init -q
git config user.name "Test User"
git config user.email "test@example.com"

# Create initial file
echo "Initial content" > file.txt
git add file.txt
git commit -q -m "Initial commit"

# Generate 500 commits (can increase for more)
echo "Creating commits (this may take a minute)..."
for i in $(seq 1 500); do
    echo "Commit $i content at $(date +%s)" >> file.txt
    if [ $((i % 10)) -eq 0 ]; then
        # Add a new file every 10 commits
        echo "Extra file $i" > "extra_$i.txt"
        git add "extra_$i.txt"
    fi
    git add file.txt
    git commit -q -m "Commit $i: Add feature and improve performance

This is a longer commit message body that includes
multiple lines to test commit message search and
display performance.

Related to issue #$i"

    if [ $((i % 50)) -eq 0 ]; then
        echo "  ... $i commits created"
    fi
done

echo "✓ Long commit history created: $(pwd)"
echo "  - 500 commits (can test up to 1,000+ by increasing loop)"
cd ..

echo ""
echo "=== Creating Test Repository 3: Large Diff ==="
echo "Generating repository with large commit diff..."

mkdir -p large-diff
cd large-diff
git init -q
git config user.name "Test User"
git config user.email "test@example.com"

# Create initial small commit
echo "Initial" > README.md
git add README.md
git commit -q -m "Initial commit"

# Create a commit with 50 files, 100 lines each
echo "Creating large diff..."
for i in $(seq 1 50); do
    {
        echo "// File $i"
        echo "// This file has many lines to create a large diff"
        echo ""
        for j in $(seq 1 100); do
            echo "function test_$i_$j() {"
            echo "    console.log('Test function $i line $j');"
            echo "    return true;"
            echo "}"
            echo ""
        done
    } > "src_$i.js"
done

git add . >/dev/null 2>&1
git commit -q -m "Add 50 source files with 100 lines each

This commit adds a large number of files and lines
to test diff rendering performance."

echo "✓ Large diff created: $(pwd)"
echo "  - 1 commit with 50 files, ~5,000 lines total"
cd ..

echo ""
echo "=== Test Repositories Summary ==="
echo ""
echo "1. Large Working Directory: $TEST_DIR/large-files"
echo "   - 1,000 modified files in working directory"
echo "   - Use to test file list performance, staging, search"
echo ""
echo "2. Long Commit History: $TEST_DIR/long-history"
echo "   - 500 commits with detailed messages"
echo "   - Use to test commit history, scrolling, search"
echo ""
echo "3. Large Diff: $TEST_DIR/large-diff"
echo "   - 1 commit with 50 files, ~5,000 lines"
echo "   - Use to test diff loading and rendering"
echo ""
echo "✓ All test repositories created successfully!"
echo ""
echo "To use:"
echo "  1. Run CrabOnTree: cargo run"
echo "  2. Open test repository (File -> Open)"
echo "  3. Observe performance and responsiveness"
echo ""
echo "To increase test data size:"
echo "  - Edit loop limits in this script"
echo "  - large-files: increase from 1,000 to 10,000"
echo "  - long-history: increase from 500 to 1,000+"
echo "  - large-diff: increase file count or lines"
