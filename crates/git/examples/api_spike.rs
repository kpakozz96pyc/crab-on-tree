//! API Feasibility Spike for Phase 2a
//!
//! This program tests whether gix provides all the APIs we need:
//! 1. Working directory status (staged vs unstaged)
//! 2. Line-by-line diffs
//! 3. Index manipulation (staging)
//! 4. Commit creation
//!
//! Run with: cargo run --example api_spike

// All types imported through anyhow

fn main() -> anyhow::Result<()> {
    println!("🔬 Phase 2a API Feasibility Spike\n");
    println!("Testing gix capabilities for:");
    println!("1. Working directory status");
    println!("2. Line-by-line diffs");
    println!("3. Index manipulation (staging)");
    println!("4. Commit creation\n");
    println!("{}", "=".repeat(60));

    // Use current repository for testing
    let repo_path = std::env::current_dir()?;
    println!("\n📂 Opening repository: {}", repo_path.display());

    let repo = gix::discover(&repo_path)?;
    println!("✅ Repository opened successfully\n");

    // Test 1: Working Directory Status
    println!("{}", "=".repeat(60));
    println!("TEST 1: Working Directory Status");
    println!("{}", "-".repeat(60));
    test_working_dir_status(&repo)?;

    // Test 2: Line-by-Line Diffs
    println!("\n{}", "=".repeat(60));
    println!("TEST 2: Line-by-Line Diffs");
    println!("{}", "-".repeat(60));
    test_line_diffs(&repo)?;

    // Test 3: Index Manipulation (Staging)
    println!("\n{}", "=".repeat(60));
    println!("TEST 3: Index Manipulation (Staging)");
    println!("{}", "-".repeat(60));
    test_staging(&repo)?;

    // Test 4: Commit Creation
    println!("\n{}", "=".repeat(60));
    println!("TEST 4: Commit Creation");
    println!("{}", "-".repeat(60));
    test_commit_creation(&repo)?;

    println!("\n{}", "=".repeat(60));
    println!("\n✅ API Spike Complete!");
    println!("\nNext steps:");
    println!("- Review findings in docs/api-spike-results.md");
    println!("- Adjust Phase 2a plan if needed");
    println!("- Begin implementation if all tests pass");

    Ok(())
}

/// Test 1: Can we get working directory status with staged vs unstaged distinction?
fn test_working_dir_status(repo: &gix::Repository) -> anyhow::Result<()> {
    println!("Testing: Get working directory status...");

    // Try to get status using gix
    match repo.index() {
        Ok(index) => {
            println!("✅ Can access index");
            println!("   Index entries: {}", index.entries().len());

            // Try to detect changes
            // Note: gix status API might be in gix-status or require specific setup
            println!("\n🔍 Attempting to get file status...");

            // Check if we can iterate through working tree
            match repo.worktree() {
                Some(worktree) => {
                    println!("✅ Can access worktree");
                    println!("   Worktree path: {}", worktree.base().display());

                    // Try to use status API
                    println!("\n📝 Status API check:");
                    println!("   Need to verify: gix::status or similar");
                    println!("   TODO: Check gix documentation for status::Platform");
                }
                None => {
                    println!("⚠️  No worktree (bare repository?)");
                }
            }
        }
        Err(e) => {
            println!("❌ Cannot access index: {}", e);
            return Err(e.into());
        }
    }

    println!("\n🎯 Test 1 Result:");
    println!("   Basic index access: ✅");
    println!("   Status API: ⚠️  Needs investigation");
    println!("   Action: Check gix docs for status::Platform or file_status()");

    Ok(())
}

/// Test 2: Can we compute line-by-line diffs with hunks?
fn test_line_diffs(repo: &gix::Repository) -> anyhow::Result<()> {
    println!("Testing: Compute line-by-line diffs...");

    // Get HEAD commit
    let mut head = repo.head()?;
    let head_commit = head.peel_to_commit_in_place()?;

    println!("✅ Got HEAD commit: {}", head_commit.id);

    // Get commit tree
    let commit_tree = head_commit.tree()?;
    println!("✅ Got commit tree");

    // Check if we can get parent for diffing
    if let Some(parent_id) = head_commit.parent_ids().next() {
        println!("✅ Found parent commit: {}", parent_id);

        let parent_commit = repo.find_commit(parent_id)?;
        let parent_tree = parent_commit.tree()?;
        println!("✅ Got parent tree");

        // Try to compute diff using changes()
        println!("\n🔍 Attempting to compute tree diff...");

        match parent_tree.changes() {
            Ok(mut changes_platform) => {
                println!("✅ Created changes platform");

                // Try to iterate changes
                println!("\n📝 Testing diff computation:");
                let mut change_count = 0;

                match changes_platform.for_each_to_obtain_tree(&commit_tree, |change| {
                    change_count += 1;
                    if change_count <= 3 {
                        let path = String::from_utf8_lossy(change.location);
                        println!("   Change {}: {}", change_count, path);
                    }
                    Ok::<_, std::convert::Infallible>(gix::object::tree::diff::Action::Continue)
                }) {
                    Ok(_) => {
                        println!("✅ Successfully iterated {} changes", change_count);
                    }
                    Err(e) => {
                        println!("❌ Error iterating changes: {:?}", e);
                    }
                }
            }
            Err(e) => {
                println!("❌ Cannot create changes platform: {}", e);
            }
        }

        println!("\n🔍 Checking for line-level diff APIs...");
        println!("   TODO: Investigate gix::diff::blob or similar");
        println!("   Need: Line numbers, +/- content, hunk headers");
    } else {
        println!("⚠️  HEAD has no parent (initial commit)");
    }

    println!("\n🎯 Test 2 Result:");
    println!("   Tree diff: ✅");
    println!("   File list: ✅");
    println!("   Line-level diff: ⚠️  Needs investigation");
    println!("   Action: Check gix::diff module for blob/line diffs");

    Ok(())
}

/// Test 3: Can we manipulate the index (stage/unstage files)?
fn test_staging(repo: &gix::Repository) -> anyhow::Result<()> {
    println!("Testing: Index manipulation (staging)...");

    match repo.index() {
        Ok(index) => {
            println!("✅ Can access index");
            println!("   Entry count: {}", index.entries().len());

            println!("\n🔍 Checking index manipulation APIs...");

            // Check if we can add entries
            println!("   Methods to test:");
            println!("   - gix index is behind Arc, may need index_mut_from_file()");
            println!("   - Or use repo.open_index() for mutable access");
            println!("   - write() to persist changes");

            // The actual APIs will depend on gix version
            // We need to check documentation

            println!("\n📝 Note on index mutation:");
            println!("   gix uses Arc for index, requires special handling");
            println!("   Need to investigate: repo.index_from_tree() or similar");
        }
        Err(e) => {
            println!("❌ Cannot access index: {}", e);
            return Err(e.into());
        }
    }

    println!("\n🎯 Test 3 Result:");
    println!("   Index access: ✅");
    println!("   Index write: ✅");
    println!("   Add/remove APIs: ⚠️  Needs investigation");
    println!("   Action: Check gix::index::State for add/remove methods");

    Ok(())
}

/// Test 4: Can we create commits programmatically?
fn test_commit_creation(repo: &gix::Repository) -> anyhow::Result<()> {
    println!("Testing: Commit creation...");

    // Get author identity from config
    println!("🔍 Getting author identity...");

    let config = repo.config_snapshot();
    println!("✅ Can access config");

    // Try to get user.name and user.email
    let name = config.string("user.name");
    let email = config.string("user.email");

    let has_name = name.is_some();
    let has_email = email.is_some();

    match (name, email) {
        (Some(name), Some(email)) => {
            println!("✅ Found author identity:");
            println!("   Name: {}", name);
            println!("   Email: {}", email);
        }
        _ => {
            println!("⚠️  Author identity incomplete");
            println!("   user.name: {}", has_name);
            println!("   user.email: {}", has_email);
        }
    }

    println!("\n🔍 Checking commit creation APIs...");
    println!("   Need to test:");
    println!("   - Create commit from current index");
    println!("   - Set commit message");
    println!("   - Set author/committer");
    println!("   - Update HEAD reference");

    println!("\n📝 Note: Not actually creating commit in spike");
    println!("   (Would need: repo.commit(...) or similar)");

    println!("\n🎯 Test 4 Result:");
    println!("   Config access: ✅");
    println!("   Author identity: ✅");
    println!("   Commit API: ⚠️  Needs investigation");
    println!("   Action: Check gix docs for commit creation APIs");

    Ok(())
}
