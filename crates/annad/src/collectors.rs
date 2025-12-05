//! Knowledge collectors - gather system facts into KnowledgeDocs (v0.0.32R).
//!
//! Collectors run on-demand or idle to populate the knowledge store.
//! Each collector produces deterministic documents with full provenance.

use anna_shared::knowledge::{KnowledgeDoc, KnowledgeSource, Provenance};
use std::process::Command;
use tracing::{info, warn};

/// Result from a collector run
#[derive(Debug)]
pub struct CollectorResult {
    pub docs: Vec<KnowledgeDoc>,
    pub errors: Vec<String>,
}

impl CollectorResult {
    pub fn new() -> Self {
        Self {
            docs: vec![],
            errors: vec![],
        }
    }

    pub fn add_doc(&mut self, doc: KnowledgeDoc) {
        self.docs.push(doc);
    }

    pub fn add_error(&mut self, err: impl Into<String>) {
        self.errors.push(err.into());
    }
}

impl Default for CollectorResult {
    fn default() -> Self {
        Self::new()
    }
}

/// Collect boot time information from systemd-analyze
pub fn collect_boot_time() -> CollectorResult {
    let mut result = CollectorResult::new();

    let output = Command::new("systemd-analyze").output();

    match output {
        Ok(o) if o.status.success() => {
            let stdout = String::from_utf8_lossy(&o.stdout);
            if let Some(doc) = parse_boot_time(&stdout) {
                info!("Collected boot time: {}", doc.title);
                result.add_doc(doc);
            }
        }
        Ok(o) => {
            let stderr = String::from_utf8_lossy(&o.stderr);
            warn!("systemd-analyze failed: {}", stderr);
            result.add_error(format!("systemd-analyze failed: {}", stderr));
        }
        Err(e) => {
            warn!("Failed to run systemd-analyze: {}", e);
            result.add_error(format!("Failed to run systemd-analyze: {}", e));
        }
    }

    result
}

/// Parse systemd-analyze output into KnowledgeDoc
fn parse_boot_time(output: &str) -> Option<KnowledgeDoc> {
    // Format: "Startup finished in X.XXXs (kernel) + X.XXXs (userspace) = X.XXXs"
    let line = output.lines().next()?;

    // Extract total time
    let total_time = if let Some(idx) = line.rfind('=') {
        line[idx + 1..].trim().to_string()
    } else {
        line.to_string()
    };

    let body = format!(
        "Boot time: {}\nFull output: {}",
        total_time,
        line
    );

    Some(KnowledgeDoc::new(
        KnowledgeSource::SystemFact,
        format!("Boot Time: {}", total_time),
        body,
        vec!["boot".to_string(), "startup".to_string(), "performance".to_string()],
        Provenance::from_command("annad:collector", "systemd-analyze", 100),
    ).with_ttl(1)) // 1 day TTL - boot time changes on reboot
}

/// Collect installed packages from pacman
pub fn collect_packages() -> CollectorResult {
    let mut result = CollectorResult::new();

    let output = Command::new("pacman").args(["-Q"]).output();

    match output {
        Ok(o) if o.status.success() => {
            let stdout = String::from_utf8_lossy(&o.stdout);
            let doc = parse_packages(&stdout);
            info!("Collected {} packages", doc.tags.len());
            result.add_doc(doc);
        }
        Ok(o) => {
            let stderr = String::from_utf8_lossy(&o.stderr);
            warn!("pacman -Q failed: {}", stderr);
            result.add_error(format!("pacman -Q failed: {}", stderr));
        }
        Err(e) => {
            // Not on Arch, try dpkg
            if let Some(doc) = try_dpkg_packages() {
                result.add_doc(doc);
            } else {
                result.add_error(format!("No package manager found: {}", e));
            }
        }
    }

    result
}

/// Parse pacman -Q output into KnowledgeDoc
fn parse_packages(output: &str) -> KnowledgeDoc {
    let lines: Vec<&str> = output.lines().collect();
    let count = lines.len();

    // Extract package names as tags (first 50 for indexing)
    let tags: Vec<String> = lines
        .iter()
        .take(50)
        .filter_map(|line| line.split_whitespace().next())
        .map(String::from)
        .collect();

    // Build summary body
    let body = format!(
        "Total packages installed: {}\n\nSample packages:\n{}",
        count,
        lines.iter().take(20).cloned().collect::<Vec<_>>().join("\n")
    );

    KnowledgeDoc::new(
        KnowledgeSource::PackageFact,
        format!("Installed Packages: {} total", count),
        body,
        tags,
        Provenance::from_command("annad:collector", "pacman -Q", 100),
    ).with_ttl(7) // 7 day TTL
}

/// Try dpkg for Debian-based systems
fn try_dpkg_packages() -> Option<KnowledgeDoc> {
    let output = Command::new("dpkg").args(["--get-selections"]).output().ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = stdout.lines().collect();
    let count = lines.len();

    let tags: Vec<String> = lines
        .iter()
        .take(50)
        .filter_map(|line| line.split_whitespace().next())
        .map(String::from)
        .collect();

    let body = format!(
        "Total packages installed: {}\n\nSample packages:\n{}",
        count,
        lines.iter().take(20).cloned().collect::<Vec<_>>().join("\n")
    );

    Some(KnowledgeDoc::new(
        KnowledgeSource::PackageFact,
        format!("Installed Packages: {} total", count),
        body,
        tags,
        Provenance::from_command("annad:collector", "dpkg --get-selections", 100),
    ).with_ttl(7))
}

/// Collect recent journal errors
pub fn collect_journal_errors() -> CollectorResult {
    let mut result = CollectorResult::new();

    let output = Command::new("journalctl")
        .args(["-p", "3", "-b", "--no-pager", "-n", "100"])
        .output();

    match output {
        Ok(o) if o.status.success() => {
            let stdout = String::from_utf8_lossy(&o.stdout);
            if let Some(doc) = parse_journal_errors(&stdout) {
                info!("Collected journal errors");
                result.add_doc(doc);
            }
        }
        Ok(o) => {
            let stderr = String::from_utf8_lossy(&o.stderr);
            warn!("journalctl failed: {}", stderr);
            result.add_error(format!("journalctl failed: {}", stderr));
        }
        Err(e) => {
            warn!("Failed to run journalctl: {}", e);
            result.add_error(format!("Failed to run journalctl: {}", e));
        }
    }

    result
}

/// Parse journalctl output into KnowledgeDoc
fn parse_journal_errors(output: &str) -> Option<KnowledgeDoc> {
    let lines: Vec<&str> = output.lines().collect();

    if lines.is_empty() {
        return None;
    }

    // Extract unique error keywords as tags
    let mut tags: Vec<String> = vec!["error".to_string(), "journal".to_string()];

    // Look for common error patterns
    let error_patterns = ["failed", "error", "timeout", "warning", "critical"];
    for pattern in error_patterns {
        if output.to_lowercase().contains(pattern) {
            tags.push(pattern.to_string());
        }
    }

    let count = lines.len();
    let body = format!(
        "Recent errors this boot: {} entries\n\n{}",
        count,
        lines.iter().take(30).cloned().collect::<Vec<_>>().join("\n")
    );

    Some(KnowledgeDoc::new(
        KnowledgeSource::Journal,
        format!("Journal Errors: {} entries this boot", count),
        body,
        tags,
        Provenance::from_command("annad:collector", "journalctl -p 3 -b", 90),
    ).with_ttl(1)) // 1 day TTL - changes on reboot or new errors
}

/// Run all collectors and return combined results
pub fn collect_all() -> CollectorResult {
    let mut result = CollectorResult::new();

    // Boot time
    let boot = collect_boot_time();
    result.docs.extend(boot.docs);
    result.errors.extend(boot.errors);

    // Packages
    let pkgs = collect_packages();
    result.docs.extend(pkgs.docs);
    result.errors.extend(pkgs.errors);

    // Journal errors
    let journal = collect_journal_errors();
    result.docs.extend(journal.docs);
    result.errors.extend(journal.errors);

    info!("Collected {} documents with {} errors", result.docs.len(), result.errors.len());
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_boot_time() {
        let output = "Startup finished in 2.500s (kernel) + 5.123s (userspace) = 7.623s";
        let doc = parse_boot_time(output).unwrap();

        assert!(doc.title.contains("7.623s"));
        assert!(doc.tags.contains(&"boot".to_string()));
        assert_eq!(doc.source, KnowledgeSource::SystemFact);
    }

    #[test]
    fn test_parse_packages() {
        let output = "vim 9.0-1\nneovim 0.9.0-1\nbash 5.1-1";
        let doc = parse_packages(output);

        assert!(doc.title.contains("3 total"));
        assert!(doc.tags.contains(&"vim".to_string()));
        assert_eq!(doc.source, KnowledgeSource::PackageFact);
    }

    #[test]
    fn test_parse_journal_errors() {
        let output = "Dec 05 10:00:00 host systemd[1]: Failed to start some.service\nDec 05 10:01:00 host kernel: error in something";
        let doc = parse_journal_errors(output).unwrap();

        assert!(doc.title.contains("2 entries"));
        assert!(doc.tags.contains(&"error".to_string()));
        assert!(doc.tags.contains(&"failed".to_string()));
    }

    #[test]
    fn test_empty_journal() {
        let output = "";
        let doc = parse_journal_errors(output);
        assert!(doc.is_none());
    }
}
