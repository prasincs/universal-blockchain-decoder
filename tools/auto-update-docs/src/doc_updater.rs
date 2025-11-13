use anyhow::{Context, Result};
use log::{debug, info};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use crate::analyzer::{CodebaseAnalysis, ModuleType};

#[derive(Debug, Clone)]
pub struct DocUpdate {
    pub doc_path: PathBuf,
    pub reason: String,
    pub current_content: String,
    pub priority: UpdatePriority,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum UpdatePriority {
    Low,
    Medium,
    High,
}

/// Determine which documentation files need updating
pub fn determine_updates(repo_root: &Path, analysis: &CodebaseAnalysis) -> Result<Vec<DocUpdate>> {
    info!("Determining documentation updates needed");

    let mut updates = Vec::new();

    // Check key documentation files
    let doc_files = find_documentation_files(repo_root)?;

    for doc_path in doc_files {
        if let Some(update) = check_doc_needs_update(repo_root, &doc_path, analysis)? {
            updates.push(update);
        }
    }

    // Sort by priority
    updates.sort_by(|a, b| b.priority.cmp(&a.priority));

    Ok(updates)
}

/// Find all documentation files in the repository
fn find_documentation_files(repo_root: &Path) -> Result<Vec<PathBuf>> {
    let mut doc_files = Vec::new();

    // Key documentation files to always check
    let key_docs = vec![
        "README.md",
        "ARCHITECTURE.md",
        "ROADMAP.md",
        "CLAUDE.md",
        "docs/ARCHITECTURE_REFACTORING.md",
        "docs/TRAIT_BASED_ARCHITECTURE.md",
        "docs/TESTING_STRATEGY.md",
    ];

    for doc in key_docs {
        let path = repo_root.join(doc);
        if path.exists() {
            doc_files.push(path);
        }
    }

    // Find other markdown files in docs/
    let docs_dir = repo_root.join("docs");
    if docs_dir.exists() {
        for entry in WalkDir::new(&docs_dir)
            .max_depth(2)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("md")
                && !doc_files.contains(&path.to_path_buf())
            {
                doc_files.push(path.to_path_buf());
            }
        }
    }

    Ok(doc_files)
}

/// Check if a documentation file needs updating
fn check_doc_needs_update(
    _repo_root: &Path,
    doc_path: &Path,
    analysis: &CodebaseAnalysis,
) -> Result<Option<DocUpdate>> {
    let content =
        fs::read_to_string(doc_path).with_context(|| format!("Failed to read {:?}", doc_path))?;

    // Check various indicators that doc might be outdated
    let mut reasons = Vec::new();
    let mut priority = UpdatePriority::Low;

    // 1. Check if doc mentions module counts
    if let Some(reason) = check_module_counts(&content, analysis) {
        reasons.push(reason);
        priority = UpdatePriority::High;
    }

    // 2. Check if doc has outdated diagrams
    if content.contains("```mermaid") && !content.contains("Auto-generated") {
        reasons.push("Contains Mermaid diagram that may need updating".to_string());
        priority = std::cmp::max(priority, UpdatePriority::Medium);
    }

    // 3. Check if doc mentions specific modules that changed
    if let Some(reason) = check_mentions_changed_modules(&content, analysis) {
        reasons.push(reason);
        priority = std::cmp::max(priority, UpdatePriority::Medium);
    }

    // 4. Check if doc is a roadmap/status document
    if doc_path.to_string_lossy().contains("ROADMAP")
        || doc_path.to_string_lossy().contains("STATUS")
        || doc_path.to_string_lossy().contains("PLAN")
    {
        reasons.push("Roadmap/status document should be kept current".to_string());
        priority = std::cmp::max(priority, UpdatePriority::High);
    }

    // 5. Check if doc is very outdated (mentions old phases/versions)
    if content.contains("Phase 0") || content.contains("v0.0.") {
        reasons.push("Document references old phases/versions".to_string());
        priority = UpdatePriority::Medium;
    }

    // 6. Check if architecture doc and structure changed significantly
    if doc_path.to_string_lossy().contains("ARCHITECTURE") && !analysis.changes.is_empty() {
        reasons.push("Architecture document with recent code changes".to_string());
        priority = std::cmp::max(priority, UpdatePriority::High);
    }

    if reasons.is_empty() {
        return Ok(None);
    }

    debug!("Doc needs update: {:?} - {}", doc_path, reasons.join("; "));

    Ok(Some(DocUpdate {
        doc_path: doc_path.to_path_buf(),
        reason: reasons.join("; "),
        current_content: content,
        priority,
    }))
}

/// Check if document mentions module counts that might be outdated
fn check_module_counts(content: &str, analysis: &CodebaseAnalysis) -> Option<String> {
    // Look for patterns like "X modules", "X decoders", "X crates"
    let re = regex::Regex::new(r"(\d+)\s+(modules|decoders|crates)").ok()?;

    for cap in re.captures_iter(content) {
        if let Ok(count) = cap[1].parse::<usize>() {
            let actual_count = match &cap[2] {
                "modules" | "crates" => analysis.module_count,
                "decoders" => analysis
                    .modules
                    .iter()
                    .filter(|m| m.module_type == ModuleType::Decoder)
                    .count(),
                _ => continue,
            };

            if count != actual_count {
                return Some(format!(
                    "Document states {} {} but actual count is {}",
                    count, &cap[2], actual_count
                ));
            }
        }
    }

    None
}

/// Check if document mentions modules that recently changed
fn check_mentions_changed_modules(content: &str, analysis: &CodebaseAnalysis) -> Option<String> {
    let changed_modules: HashSet<String> = analysis
        .changes
        .iter()
        .filter_map(|change| {
            change
                .file_path
                .components()
                .nth(1) // Get crate name (crates/<name>/...)
                .and_then(|c| c.as_os_str().to_str())
                .map(|s| s.to_string())
        })
        .collect();

    if changed_modules.is_empty() {
        return None;
    }

    // Check if document mentions any of the changed modules
    for module in &changed_modules {
        if content.contains(module) {
            return Some(format!(
                "Document mentions recently changed module: {}",
                module
            ));
        }
    }

    None
}

/// Write updated documentation to file
pub fn write_doc(doc_path: &Path, content: &str) -> Result<()> {
    // Ensure parent directory exists
    if let Some(parent) = doc_path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory {:?}", parent))?;
    }

    fs::write(doc_path, content)
        .with_context(|| format!("Failed to write documentation to {:?}", doc_path))?;

    Ok(())
}

/// Generate a summary of documentation updates
#[allow(dead_code)]
pub fn generate_update_summary(updates: &[DocUpdate]) -> String {
    let mut summary = String::new();

    summary.push_str("# Documentation Update Summary\n\n");
    summary.push_str(&format!("Total files to update: {}\n\n", updates.len()));

    let high_priority: Vec<_> = updates
        .iter()
        .filter(|u| u.priority == UpdatePriority::High)
        .collect();

    let medium_priority: Vec<_> = updates
        .iter()
        .filter(|u| u.priority == UpdatePriority::Medium)
        .collect();

    let low_priority: Vec<_> = updates
        .iter()
        .filter(|u| u.priority == UpdatePriority::Low)
        .collect();

    summary.push_str(&format!("- High priority: {}\n", high_priority.len()));
    summary.push_str(&format!("- Medium priority: {}\n", medium_priority.len()));
    summary.push_str(&format!("- Low priority: {}\n\n", low_priority.len()));

    if !high_priority.is_empty() {
        summary.push_str("## High Priority\n\n");
        for update in high_priority {
            summary.push_str(&format!("- `{}`\n", update.doc_path.display()));
            summary.push_str(&format!("  Reason: {}\n\n", update.reason));
        }
    }

    if !medium_priority.is_empty() {
        summary.push_str("## Medium Priority\n\n");
        for update in medium_priority {
            summary.push_str(&format!("- `{}`\n", update.doc_path.display()));
            summary.push_str(&format!("  Reason: {}\n\n", update.reason));
        }
    }

    if !low_priority.is_empty() {
        summary.push_str("## Low Priority\n\n");
        for update in low_priority {
            summary.push_str(&format!("- `{}`\n", update.doc_path.display()));
            summary.push_str(&format!("  Reason: {}\n\n", update.reason));
        }
    }

    summary
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_module_counts() {
        let content = "The project has 25 modules and 15 decoders.";

        // Create 15 decoder modules to match the content
        let decoder_modules: Vec<_> = (0..15)
            .map(|i| crate::analyzer::ModuleInfo {
                name: format!("decoder-{}", i),
                path: PathBuf::from(format!("crates/decoder-{}", i)),
                module_type: crate::analyzer::ModuleType::Decoder,
                loc: 100,
                dependencies: vec![],
                description: String::new(),
                exports: vec![],
            })
            .collect();

        let mut analysis = CodebaseAnalysis {
            module_count: 25,
            total_loc: 1000,
            modules: decoder_modules,
            changes: vec![],
            dependencies: Default::default(),
            architecture_summary: String::new(),
        };

        // Should return None when counts match
        assert!(check_module_counts(content, &analysis).is_none());

        // Should return Some when counts don't match
        analysis.module_count = 30;
        assert!(check_module_counts(content, &analysis).is_some());
    }

    #[test]
    fn test_update_priority_ordering() {
        assert!(UpdatePriority::High > UpdatePriority::Medium);
        assert!(UpdatePriority::Medium > UpdatePriority::Low);
    }
}
