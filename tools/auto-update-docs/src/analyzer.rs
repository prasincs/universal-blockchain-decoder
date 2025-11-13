use anyhow::{Context, Result};
use git2::{DiffOptions, Repository};
use log::{debug, info};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use toml::Value;
use walkdir::WalkDir;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodebaseAnalysis {
    pub module_count: usize,
    pub total_loc: usize,
    pub modules: Vec<ModuleInfo>,
    pub changes: Vec<CodeChange>,
    pub dependencies: HashMap<String, Vec<String>>,
    pub architecture_summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleInfo {
    pub name: String,
    pub path: PathBuf,
    pub module_type: ModuleType,
    pub loc: usize,
    pub dependencies: Vec<String>,
    pub description: String,
    pub exports: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ModuleType {
    Core,
    Decoder,
    Tool,
    Test,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeChange {
    pub file_path: PathBuf,
    pub change_type: ChangeType,
    pub lines_added: usize,
    pub lines_removed: usize,
    pub summary: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ChangeType {
    Added,
    Modified,
    Deleted,
    Renamed,
}

/// Analyze the codebase structure and recent changes
pub fn analyze_codebase(repo_root: &Path, since: Option<&str>) -> Result<CodebaseAnalysis> {
    info!("Starting codebase analysis at {:?}", repo_root);

    // Discover all Rust crates
    let modules = discover_modules(repo_root)?;
    debug!("Discovered {} modules", modules.len());

    // Analyze dependencies
    let dependencies = analyze_dependencies(repo_root)?;
    debug!("Analyzed dependencies for {} crates", dependencies.len());

    // Get recent code changes
    let changes = if let Some(ref_name) = since {
        analyze_changes_since(repo_root, ref_name)?
    } else {
        // Default: last 7 days
        analyze_recent_changes(repo_root, 7)?
    };
    debug!("Found {} recent changes", changes.len());

    // Generate architecture summary
    let architecture_summary = generate_architecture_summary(&modules);

    // Count total LOC
    let total_loc = modules.iter().map(|m| m.loc).sum();

    Ok(CodebaseAnalysis {
        module_count: modules.len(),
        total_loc,
        modules,
        changes,
        dependencies,
        architecture_summary,
    })
}

/// Discover all modules (crates) in the repository
fn discover_modules(repo_root: &Path) -> Result<Vec<ModuleInfo>> {
    let mut modules = Vec::new();

    // Find all Cargo.toml files
    for entry in WalkDir::new(repo_root)
        .max_depth(4)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.file_name() == Some(std::ffi::OsStr::new("Cargo.toml")) {
            if let Some(_parent) = path.parent() {
                if let Ok(module) = analyze_module(_parent) {
                    modules.push(module);
                }
            }
        }
    }

    modules.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(modules)
}

/// Analyze a single module
fn analyze_module(module_path: &Path) -> Result<ModuleInfo> {
    let cargo_toml_path = module_path.join("Cargo.toml");
    let cargo_toml = fs::read_to_string(&cargo_toml_path)
        .with_context(|| format!("Failed to read {:?}", cargo_toml_path))?;

    let toml: Value = toml::from_str(&cargo_toml)
        .with_context(|| format!("Failed to parse {:?}", cargo_toml_path))?;

    // Extract package name
    let name = toml
        .get("package")
        .and_then(|p| p.get("name"))
        .and_then(|n| n.as_str())
        .unwrap_or("unknown")
        .to_string();

    // Extract description
    let description = toml
        .get("package")
        .and_then(|p| p.get("description"))
        .and_then(|d| d.as_str())
        .unwrap_or("")
        .to_string();

    // Determine module type
    let module_type = determine_module_type(&name, module_path);

    // Extract dependencies
    let dependencies = extract_dependencies(&toml);

    // Count lines of code
    let loc = count_loc(module_path)?;

    // Extract public exports (simplified - would need proper parsing for real impl)
    let exports = extract_exports(module_path)?;

    Ok(ModuleInfo {
        name,
        path: module_path.to_path_buf(),
        module_type,
        loc,
        dependencies,
        description,
        exports,
    })
}

/// Determine the type of a module
fn determine_module_type(name: &str, path: &Path) -> ModuleType {
    if name.contains("core") {
        ModuleType::Core
    } else if name.starts_with("decoder-") {
        ModuleType::Decoder
    } else if path.starts_with("tools") {
        ModuleType::Tool
    } else if path.starts_with("tests") {
        ModuleType::Test
    } else {
        ModuleType::Other
    }
}

/// Extract dependencies from Cargo.toml
fn extract_dependencies(toml: &Value) -> Vec<String> {
    let mut deps = Vec::new();

    if let Some(dependencies) = toml.get("dependencies").and_then(|d| d.as_table()) {
        for (name, _) in dependencies {
            deps.push(name.clone());
        }
    }

    deps.sort();
    deps
}

/// Count lines of code in a module
fn count_loc(module_path: &Path) -> Result<usize> {
    let mut total = 0;
    let src_path = module_path.join("src");

    if !src_path.exists() {
        return Ok(0);
    }

    for entry in WalkDir::new(src_path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("rs"))
    {
        if let Ok(content) = fs::read_to_string(entry.path()) {
            total += content.lines().count();
        }
    }

    Ok(total)
}

/// Extract public exports from a module (simplified)
fn extract_exports(module_path: &Path) -> Result<Vec<String>> {
    let lib_rs = module_path.join("src/lib.rs");
    let mut exports = Vec::new();

    if lib_rs.exists() {
        let content = fs::read_to_string(&lib_rs)?;
        for line in content.lines() {
            let line = line.trim();
            if line.starts_with("pub fn ")
                || line.starts_with("pub struct ")
                || line.starts_with("pub enum ")
            {
                if let Some(name) = line.split_whitespace().nth(2) {
                    exports.push(name.trim_end_matches(['(', '<', '{']).to_string());
                }
            }
        }
    }

    Ok(exports)
}

/// Analyze dependencies between modules
fn analyze_dependencies(repo_root: &Path) -> Result<HashMap<String, Vec<String>>> {
    let mut deps = HashMap::new();

    for entry in WalkDir::new(repo_root)
        .max_depth(4)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.file_name() == Some(std::ffi::OsStr::new("Cargo.toml")) {
            if let Some(_parent) = path.parent() {
                if let Ok(cargo_toml) = fs::read_to_string(path) {
                    if let Ok(toml) = toml::from_str::<Value>(&cargo_toml) {
                        if let Some(name) = toml
                            .get("package")
                            .and_then(|p| p.get("name"))
                            .and_then(|n| n.as_str())
                        {
                            let module_deps = extract_dependencies(&toml);
                            deps.insert(name.to_string(), module_deps);
                        }
                    }
                }
            }
        }
    }

    Ok(deps)
}

/// Analyze changes since a specific git ref
fn analyze_changes_since(repo_root: &Path, ref_name: &str) -> Result<Vec<CodeChange>> {
    let repo = Repository::open(repo_root)?;
    let mut changes = Vec::new();

    // Get the ref commit
    let ref_object = repo.revparse_single(ref_name)?;
    let ref_commit = ref_object.peel_to_commit()?;

    // Get HEAD commit
    let head = repo.head()?;
    let head_commit = head.peel_to_commit()?;

    // Get diff
    let ref_tree = ref_commit.tree()?;
    let head_tree = head_commit.tree()?;

    let mut diff_opts = DiffOptions::new();
    let diff = repo.diff_tree_to_tree(Some(&ref_tree), Some(&head_tree), Some(&mut diff_opts))?;

    // Process diff
    diff.foreach(
        &mut |delta, _progress| {
            if let Some(path) = delta.new_file().path() {
                let change_type = match delta.status() {
                    git2::Delta::Added => ChangeType::Added,
                    git2::Delta::Modified => ChangeType::Modified,
                    git2::Delta::Deleted => ChangeType::Deleted,
                    git2::Delta::Renamed => ChangeType::Renamed,
                    _ => return true,
                };

                changes.push(CodeChange {
                    file_path: path.to_path_buf(),
                    change_type,
                    lines_added: 0, // Would need diff parsing for accurate counts
                    lines_removed: 0,
                    summary: format!("{:?} {}", change_type, path.display()),
                });
            }
            true
        },
        None,
        None,
        None,
    )?;

    Ok(changes)
}

/// Analyze recent changes (last N days)
fn analyze_recent_changes(repo_root: &Path, days: i64) -> Result<Vec<CodeChange>> {
    let repo = Repository::open(repo_root)?;
    let mut changes = Vec::new();

    let _head = repo.head()?;
    let _head_commit = _head.peel_to_commit()?;

    // Calculate cutoff time
    let cutoff = chrono::Utc::now() - chrono::Duration::days(days);
    let cutoff_timestamp = cutoff.timestamp();

    // Walk commits
    let mut revwalk = repo.revwalk()?;
    revwalk.push_head()?;

    for oid in revwalk {
        let oid = oid?;
        let commit = repo.find_commit(oid)?;

        if commit.time().seconds() < cutoff_timestamp {
            break;
        }

        // Get commit message
        let message = commit.message().unwrap_or("").to_string();

        // Get parent commit
        if commit.parent_count() > 0 {
            let parent = commit.parent(0)?;
            let parent_tree = parent.tree()?;
            let commit_tree = commit.tree()?;

            let mut diff_opts = DiffOptions::new();
            let diff = repo.diff_tree_to_tree(
                Some(&parent_tree),
                Some(&commit_tree),
                Some(&mut diff_opts),
            )?;

            diff.foreach(
                &mut |delta, _progress| {
                    if let Some(path) = delta.new_file().path() {
                        let change_type = match delta.status() {
                            git2::Delta::Added => ChangeType::Added,
                            git2::Delta::Modified => ChangeType::Modified,
                            git2::Delta::Deleted => ChangeType::Deleted,
                            git2::Delta::Renamed => ChangeType::Renamed,
                            _ => return true,
                        };

                        changes.push(CodeChange {
                            file_path: path.to_path_buf(),
                            change_type,
                            lines_added: 0,
                            lines_removed: 0,
                            summary: message.lines().next().unwrap_or("").to_string(),
                        });
                    }
                    true
                },
                None,
                None,
                None,
            )?;
        }
    }

    // Deduplicate changes by file path
    changes.sort_by(|a, b| a.file_path.cmp(&b.file_path));
    changes.dedup_by(|a, b| a.file_path == b.file_path);

    Ok(changes)
}

/// Generate a high-level architecture summary
fn generate_architecture_summary(modules: &[ModuleInfo]) -> String {
    let core_modules: Vec<_> = modules
        .iter()
        .filter(|m| m.module_type == ModuleType::Core)
        .collect();

    let decoder_modules: Vec<_> = modules
        .iter()
        .filter(|m| m.module_type == ModuleType::Decoder)
        .collect();

    let tool_modules: Vec<_> = modules
        .iter()
        .filter(|m| m.module_type == ModuleType::Tool)
        .collect();

    format!(
        "Universal Blockchain Decoder Architecture:\n\
         - Core Libraries: {} modules ({} LOC)\n\
         - Blockchain Decoders: {} modules ({} LOC)\n\
         - Development Tools: {} modules ({} LOC)\n\
         Total: {} modules, {} LOC",
        core_modules.len(),
        core_modules.iter().map(|m| m.loc).sum::<usize>(),
        decoder_modules.len(),
        decoder_modules.iter().map(|m| m.loc).sum::<usize>(),
        tool_modules.len(),
        tool_modules.iter().map(|m| m.loc).sum::<usize>(),
        modules.len(),
        modules.iter().map(|m| m.loc).sum::<usize>(),
    )
}
