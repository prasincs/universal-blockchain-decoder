//! Git operations helper

use anyhow::{Context, Result};
use std::process::Command;

pub struct GitHelper;

impl GitHelper {
    /// Create and checkout new branch
    pub fn create_branch(name: &str) -> Result<()> {
        let output = Command::new("git")
            .args(["checkout", "-b", name])
            .output()
            .context("Failed to create git branch")?;

        if !output.status.success() {
            anyhow::bail!(
                "Git checkout failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }

        Ok(())
    }

    /// Stage all changes
    pub fn add_all() -> Result<()> {
        let output = Command::new("git")
            .args(["add", "-A"])
            .output()
            .context("Failed to stage changes")?;

        if !output.status.success() {
            anyhow::bail!(
                "Git add failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }

        Ok(())
    }

    /// Commit changes
    pub fn commit(message: &str) -> Result<()> {
        let output = Command::new("git")
            .args(["commit", "-m", message])
            .output()
            .context("Failed to commit changes")?;

        if !output.status.success() {
            anyhow::bail!(
                "Git commit failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }

        Ok(())
    }

    /// Push branch to remote
    pub fn push(branch: &str) -> Result<()> {
        let output = Command::new("git")
            .args(["push", "-u", "origin", branch])
            .output()
            .context("Failed to push branch")?;

        if !output.status.success() {
            anyhow::bail!(
                "Git push failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }

        Ok(())
    }

    /// Run cargo fmt
    pub fn cargo_fmt() -> Result<()> {
        let output = Command::new("cargo")
            .args(["fmt", "--all"])
            .output()
            .context("Failed to run cargo fmt")?;

        if !output.status.success() {
            anyhow::bail!(
                "Cargo fmt failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }

        Ok(())
    }

    /// Run cargo clippy
    pub fn cargo_clippy() -> Result<()> {
        let output = Command::new("cargo")
            .args([
                "clippy",
                "--all",
                "--all-targets",
                "--all-features",
                "--",
                "-D",
                "warnings",
            ])
            .output()
            .context("Failed to run cargo clippy")?;

        if !output.status.success() {
            anyhow::bail!(
                "Cargo clippy failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }

        Ok(())
    }

    /// Run cargo test
    pub fn cargo_test() -> Result<()> {
        let output = Command::new("cargo")
            .args(["test", "--all"])
            .output()
            .context("Failed to run cargo test")?;

        if !output.status.success() {
            anyhow::bail!(
                "Cargo test failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }

        Ok(())
    }
}
