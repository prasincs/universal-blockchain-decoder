use anyhow::{Context, Result};
use git2::{Repository, Signature};
use log::{debug, info};
use std::path::Path;
use std::process::Command;

/// Commit changes to git
pub fn commit_changes(repo_root: &Path, message: &str) -> Result<()> {
    info!("Committing changes to git");

    let repo = Repository::open(repo_root).context("Failed to open git repository")?;

    // Get the current index
    let mut index = repo.index()?;

    // Add all changes in docs/ directory
    index.add_all(["docs/*"].iter(), git2::IndexAddOption::DEFAULT, None)?;
    index.add_all(["*.md"].iter(), git2::IndexAddOption::DEFAULT, None)?;

    // Write the index
    let oid = index.write_tree()?;
    let tree = repo.find_tree(oid)?;

    // Get the parent commit (HEAD)
    let parent_commit = match repo.head() {
        Ok(head) => Some(head.peel_to_commit()?),
        Err(_) => None,
    };

    // Create signature
    let signature = create_signature(&repo)?;

    // Create the commit
    let parents: Vec<&git2::Commit> = if let Some(ref parent) = parent_commit {
        vec![parent]
    } else {
        vec![]
    };

    repo.commit(
        Some("HEAD"),
        &signature,
        &signature,
        message,
        &tree,
        &parents,
    )?;

    info!("Successfully committed changes");
    Ok(())
}

/// Push changes to remote
pub fn push_changes(repo_root: &Path) -> Result<()> {
    info!("Pushing changes to remote");

    // Use git command for pushing (simpler than libgit2 for this)
    let output = Command::new("git")
        .current_dir(repo_root)
        .args(["push", "-u", "origin", "HEAD"])
        .output()
        .context("Failed to execute git push")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Git push failed: {}", stderr);
    }

    info!("Successfully pushed changes");
    Ok(())
}

/// Create a pull request using gh CLI
pub fn create_pull_request(repo_root: &Path, commit_msg: &str) -> Result<()> {
    info!("Creating pull request");

    // Extract title from commit message (first line)
    let title = commit_msg
        .lines()
        .next()
        .unwrap_or("Auto-update documentation");

    // Use commit message as PR body
    let body = commit_msg;

    // Create PR using gh CLI
    let output = Command::new("gh")
        .current_dir(repo_root)
        .args([
            "pr",
            "create",
            "--title",
            title,
            "--body",
            body,
            "--label",
            "documentation",
            "--label",
            "auto-generated",
        ])
        .output()
        .context("Failed to execute gh pr create")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Failed to create PR: {}", stderr);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    info!("Pull request created: {}", stdout.trim());

    Ok(())
}

/// Get current git branch name
#[allow(dead_code)]
pub fn get_current_branch(repo_root: &Path) -> Result<String> {
    let repo = Repository::open(repo_root)?;
    let head = repo.head()?;

    if let Some(name) = head.shorthand() {
        Ok(name.to_string())
    } else {
        anyhow::bail!("Unable to determine current branch")
    }
}

/// Create a signature for commits
fn create_signature(repo: &Repository) -> Result<Signature<'static>> {
    // Try to use configured git user
    let config = repo.config()?;

    let name = config
        .get_string("user.name")
        .unwrap_or_else(|_| "Auto Update Docs Bot".to_string());

    let email = config
        .get_string("user.email")
        .unwrap_or_else(|_| "auto-update-docs@example.com".to_string());

    Signature::now(&name, &email).context("Failed to create git signature")
}

/// Check if there are uncommitted changes
#[allow(dead_code)]
pub fn has_uncommitted_changes(repo_root: &Path) -> Result<bool> {
    let repo = Repository::open(repo_root)?;

    let statuses = repo.statuses(None)?;
    Ok(!statuses.is_empty())
}

/// Create a new branch for documentation updates
#[allow(dead_code)]
pub fn create_branch(repo_root: &Path, branch_name: &str) -> Result<()> {
    debug!("Creating branch: {}", branch_name);

    let repo = Repository::open(repo_root)?;

    // Get current commit
    let head = repo.head()?;
    let head_commit = head.peel_to_commit()?;

    // Create new branch
    repo.branch(branch_name, &head_commit, false)?;

    // Checkout the branch
    let obj = repo.revparse_single(&format!("refs/heads/{}", branch_name))?;
    repo.checkout_tree(&obj, None)?;
    repo.set_head(&format!("refs/heads/{}", branch_name))?;

    info!("Created and checked out branch: {}", branch_name);
    Ok(())
}

/// Get the default branch name (usually main or master)
#[allow(dead_code)]
pub fn get_default_branch(repo_root: &Path) -> Result<String> {
    let output = Command::new("git")
        .current_dir(repo_root)
        .args(["remote", "show", "origin"])
        .output()
        .context("Failed to get default branch")?;

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Look for "HEAD branch: <branch-name>"
    for line in stdout.lines() {
        if line.contains("HEAD branch:") {
            if let Some(branch) = line.split(':').nth(1) {
                return Ok(branch.trim().to_string());
            }
        }
    }

    // Fallback
    Ok("main".to_string())
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_create_signature() {
        // This would require a git repo setup
        // Skipping for now
    }
}
