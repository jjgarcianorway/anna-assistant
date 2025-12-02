//! Local Documentation Resolver v7.26.0
//!
//! Resolves configuration and documentation from local sources:
//! - man pages (man -w, man -f)
//! - /usr/share/doc
//! - pacman -Ql (package file lists)
//!
//! No network calls - pure local discovery.

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Result of resolving local documentation
#[derive(Debug, Clone)]
pub struct LocalDocResult {
    /// Source of the documentation (man, doc, pacman)
    pub source: String,
    /// Path to the documentation file
    pub path: PathBuf,
    /// Brief description (from man -f or filename)
    pub description: Option<String>,
}

/// Check if man page exists for a command
pub fn has_man_page(cmd: &str) -> bool {
    Command::new("man")
        .args(["-w", cmd])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Get man page path for a command
pub fn get_man_path(cmd: &str) -> Option<PathBuf> {
    let output = Command::new("man")
        .args(["-w", cmd])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let path = String::from_utf8_lossy(&output.stdout)
        .trim()
        .to_string();

    if path.is_empty() {
        None
    } else {
        Some(PathBuf::from(path))
    }
}

/// Get short description from man -f (whatis)
pub fn get_man_description(cmd: &str) -> Option<String> {
    let output = Command::new("man")
        .args(["-f", cmd])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Format: "cmd (N) - description"
    // Extract just the description part
    for line in stdout.lines() {
        if let Some(idx) = line.find(" - ") {
            return Some(line[idx + 3..].trim().to_string());
        }
    }

    None
}

/// Get documentation paths from /usr/share/doc for a package
pub fn get_doc_paths(package: &str) -> Vec<PathBuf> {
    let doc_dir = Path::new("/usr/share/doc").join(package);

    if !doc_dir.exists() {
        return Vec::new();
    }

    let mut paths = Vec::new();

    // Common doc file names
    let doc_files = ["README", "README.md", "README.txt", "INSTALL", "USAGE", "EXAMPLES"];

    for name in &doc_files {
        let path = doc_dir.join(name);
        if path.exists() {
            paths.push(path);
        }
    }

    // Also check for any .txt, .md, .conf files
    if let Ok(entries) = std::fs::read_dir(&doc_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Some(ext) = path.extension() {
                    let ext = ext.to_string_lossy();
                    if ext == "txt" || ext == "md" || ext == "conf" || ext == "example" {
                        if !paths.contains(&path) {
                            paths.push(path);
                        }
                    }
                }
            }
        }
    }

    paths
}

/// Get config file paths from pacman -Ql for a package
pub fn get_config_paths_from_pacman(package: &str) -> Vec<PathBuf> {
    let output = match Command::new("pacman")
        .args(["-Ql", package])
        .output() {
        Ok(o) => o,
        Err(_) => return Vec::new(),
    };

    if !output.status.success() {
        return Vec::new();
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut paths = Vec::new();

    for line in stdout.lines() {
        // Format: "package /path/to/file"
        let parts: Vec<&str> = line.splitn(2, ' ').collect();
        if parts.len() < 2 {
            continue;
        }

        let path = parts[1];

        // Look for config files
        if path.contains("/etc/") || path.ends_with(".conf") || path.contains(".d/") {
            let path_buf = PathBuf::from(path);
            if path_buf.is_file() {
                paths.push(path_buf);
            }
        }
    }

    paths
}

/// Get sample config files from pacman -Ql
pub fn get_sample_configs_from_pacman(package: &str) -> Vec<PathBuf> {
    let output = match Command::new("pacman")
        .args(["-Ql", package])
        .output() {
        Ok(o) => o,
        Err(_) => return Vec::new(),
    };

    if !output.status.success() {
        return Vec::new();
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut paths = Vec::new();

    for line in stdout.lines() {
        let parts: Vec<&str> = line.splitn(2, ' ').collect();
        if parts.len() < 2 {
            continue;
        }

        let path = parts[1];

        // Look for sample/example configs
        if path.contains(".sample") || path.contains(".example") || path.contains(".default") {
            let path_buf = PathBuf::from(path);
            if path_buf.is_file() {
                paths.push(path_buf);
            }
        }
    }

    paths
}

/// Resolve all local documentation for a command/package
pub fn resolve_local_docs(name: &str) -> Vec<LocalDocResult> {
    let mut results = Vec::new();

    // 1. Check man page
    if let Some(path) = get_man_path(name) {
        let desc = get_man_description(name);
        results.push(LocalDocResult {
            source: "man".to_string(),
            path,
            description: desc,
        });
    }

    // 2. Check /usr/share/doc
    for path in get_doc_paths(name) {
        results.push(LocalDocResult {
            source: "doc".to_string(),
            path: path.clone(),
            description: Some(path.file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default()),
        });
    }

    // 3. Check pacman -Ql for config files
    for path in get_config_paths_from_pacman(name) {
        results.push(LocalDocResult {
            source: "pacman".to_string(),
            path: path.clone(),
            description: Some("config file".to_string()),
        });
    }

    results
}

/// Get all packages that have man pages for a given section
pub fn get_packages_with_man_section(section: u8) -> Vec<String> {
    let man_dir = format!("/usr/share/man/man{}", section);
    let path = Path::new(&man_dir);

    if !path.exists() {
        return Vec::new();
    }

    let mut packages = HashSet::new();

    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            let file_name = entry.file_name();
            let name = file_name.to_string_lossy();
            // Extract package name from "name.N.gz"
            if let Some(idx) = name.find('.') {
                packages.insert(name[..idx].to_string());
            }
        }
    }

    packages.into_iter().collect()
}

/// Summary of local documentation availability
#[derive(Debug, Clone, Default)]
pub struct LocalDocsSummary {
    /// Number of packages with man pages
    pub man_packages: usize,
    /// Has arch-wiki-lite installed
    pub has_arch_wiki_lite: bool,
    /// Has arch-wiki-docs installed
    pub has_arch_wiki_docs: bool,
    /// /usr/share/doc exists
    pub has_share_doc: bool,
}

/// Get a summary of local documentation availability
pub fn get_local_docs_summary() -> LocalDocsSummary {
    LocalDocsSummary {
        man_packages: get_packages_with_man_section(1).len()
            + get_packages_with_man_section(5).len()
            + get_packages_with_man_section(8).len(),
        has_arch_wiki_lite: Path::new("/usr/share/doc/arch-wiki/text").exists(),
        has_arch_wiki_docs: Path::new("/usr/share/doc/arch-wiki/html").exists(),
        has_share_doc: Path::new("/usr/share/doc").exists(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_has_man_page_common() {
        // These should exist on most Linux systems
        assert!(has_man_page("ls"));
        assert!(has_man_page("cat"));
    }

    #[test]
    fn test_has_man_page_nonexistent() {
        assert!(!has_man_page("definitely-not-a-real-command-xyz123"));
    }

    #[test]
    fn test_get_man_description() {
        // ls should have a description
        let desc = get_man_description("ls");
        assert!(desc.is_some());
    }

    #[test]
    fn test_resolve_local_docs() {
        let results = resolve_local_docs("ls");
        // Should at least find the man page
        assert!(results.iter().any(|r| r.source == "man"));
    }

    #[test]
    fn test_local_docs_summary() {
        let summary = get_local_docs_summary();
        // Should find at least some man pages on any Linux system
        assert!(summary.man_packages > 0);
        assert!(summary.has_share_doc);
    }
}
