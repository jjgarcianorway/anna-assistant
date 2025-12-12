//! Source Providers (Part 1a) - v0.0.443.
//!
//! First-class source providers for evidence-based answers:
//! - ManProvider: man pages (`man -P cat <cmd>`)
//! - HelpProvider: command --help output
//! - ArchWikiProvider: Arch Wiki (offline-first)
//! - LocalConfigProvider: /etc, ~/.config files
//! - SystemProbeProvider: existing probes
//!
//! LLMs are for orchestration, not truth generation.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Source provider type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceType {
    /// Man pages.
    Man,
    /// Command --help output.
    Help,
    /// Arch Wiki.
    ArchWiki,
    /// Local config files.
    LocalConfig,
    /// System probes (commands).
    Probe,
}

impl SourceType {
    /// Get display label.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Man => "man",
            Self::Help => "help",
            Self::ArchWiki => "archwiki",
            Self::LocalConfig => "config",
            Self::Probe => "probe",
        }
    }

    /// Is this a documentation source (vs evidence)?
    pub fn is_documentation(&self) -> bool {
        matches!(self, Self::Man | Self::Help | Self::ArchWiki)
    }

    /// Is this an evidence source (from this machine)?
    pub fn is_evidence(&self) -> bool {
        matches!(self, Self::LocalConfig | Self::Probe)
    }
}

/// A source request in a research plan.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceRequest {
    /// Source type.
    #[serde(rename = "type")]
    pub source_type: SourceType,
    /// Source identifier (e.g., "pacman(8)", "Pacman").
    pub id: String,
    /// Query/section within source.
    pub query: String,
    /// Whether this source is required.
    pub required: bool,
}

impl SourceRequest {
    /// Create man page source request.
    pub fn man(page: &str, section: &str) -> Self {
        Self {
            source_type: SourceType::Man,
            id: page.to_string(),
            query: section.to_string(),
            required: true,
        }
    }

    /// Create help source request.
    pub fn help(command: &str, flag: &str) -> Self {
        Self {
            source_type: SourceType::Help,
            id: format!("{} --help", command),
            query: flag.to_string(),
            required: true,
        }
    }

    /// Create Arch Wiki source request.
    pub fn arch_wiki(page: &str, section: &str) -> Self {
        Self {
            source_type: SourceType::ArchWiki,
            id: page.to_string(),
            query: section.to_string(),
            required: false, // Wiki is optional (offline may not have it)
        }
    }

    /// Create optional version.
    pub fn optional(mut self) -> Self {
        self.required = false;
        self
    }
}

/// Fetched source content.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceContent {
    /// Source request that produced this.
    pub request: SourceRequest,
    /// Whether fetch succeeded.
    pub success: bool,
    /// Content (if successful).
    pub content: Option<String>,
    /// Error message (if failed).
    pub error: Option<String>,
    /// Relevant excerpt (extracted section).
    pub excerpt: Option<String>,
}

impl SourceContent {
    /// Create successful content.
    pub fn success(request: SourceRequest, content: &str, excerpt: Option<&str>) -> Self {
        Self {
            request,
            success: true,
            content: Some(content.to_string()),
            error: None,
            excerpt: excerpt.map(String::from),
        }
    }

    /// Create failed content.
    pub fn failed(request: SourceRequest, error: &str) -> Self {
        Self {
            request,
            success: false,
            content: None,
            error: Some(error.to_string()),
            excerpt: None,
        }
    }

    /// Create unavailable (offline) content.
    pub fn unavailable(request: SourceRequest) -> Self {
        Self {
            request: request.clone(),
            success: false,
            content: None,
            error: Some(format!(
                "{} source '{}' not available offline",
                request.source_type.label(),
                request.id
            )),
            excerpt: None,
        }
    }
}

/// Man page provider.
pub struct ManProvider;

impl ManProvider {
    /// Fetch man page content.
    pub fn fetch(page: &str) -> Result<String, String> {
        // Use man -P cat to get plain text
        let output = std::process::Command::new("man")
            .args(["-P", "cat", page])
            .output()
            .map_err(|e| format!("Failed to run man: {}", e))?;

        if output.status.success() {
            let content = String::from_utf8_lossy(&output.stdout).to_string();
            // Clean up formatting
            Ok(Self::clean_man_output(&content))
        } else {
            Err(format!("man {} not found", page))
        }
    }

    /// Clean man page output.
    fn clean_man_output(content: &str) -> String {
        // Remove backspace sequences used for bold/underline
        let mut result = String::new();
        let mut chars = content.chars().peekable();

        while let Some(c) = chars.next() {
            if c == '\x08' {
                // Backspace - skip previous and next char
                result.pop();
                chars.next();
            } else {
                result.push(c);
            }
        }

        result
    }

    /// Extract section from man page.
    pub fn extract_section(content: &str, section: &str) -> Option<String> {
        let section_upper = section.to_uppercase();
        let mut in_section = false;
        let mut result = Vec::new();

        for line in content.lines() {
            let trimmed = line.trim();

            // Check for section header (all caps, at start of line)
            if trimmed.chars().all(|c| c.is_ascii_uppercase() || c.is_whitespace()) && !trimmed.is_empty() {
                if trimmed.contains(&section_upper) {
                    in_section = true;
                    result.push(line.to_string());
                } else if in_section {
                    // New section started, stop
                    break;
                }
            } else if in_section {
                result.push(line.to_string());
            }
        }

        if result.is_empty() {
            None
        } else {
            Some(result.join("\n"))
        }
    }
}

/// Help provider (--help output).
pub struct HelpProvider;

impl HelpProvider {
    /// Fetch command help.
    pub fn fetch(command: &str) -> Result<String, String> {
        // Try --help first, then -h
        let output = std::process::Command::new(command)
            .arg("--help")
            .output();

        match output {
            Ok(out) if out.status.success() || !out.stdout.is_empty() => {
                Ok(String::from_utf8_lossy(&out.stdout).to_string())
            }
            _ => {
                // Try -h
                let output = std::process::Command::new(command)
                    .arg("-h")
                    .output()
                    .map_err(|e| format!("Failed to run {} -h: {}", command, e))?;

                if output.status.success() || !output.stdout.is_empty() {
                    Ok(String::from_utf8_lossy(&output.stdout).to_string())
                } else {
                    Err(format!("No help available for {}", command))
                }
            }
        }
    }

    /// Extract relevant lines matching a query.
    pub fn extract_relevant(content: &str, query: &str) -> Option<String> {
        let query_lower = query.to_lowercase();
        let mut relevant = Vec::new();

        for line in content.lines() {
            if line.to_lowercase().contains(&query_lower) {
                relevant.push(line.to_string());
            }
        }

        if relevant.is_empty() {
            // Return first 20 lines as fallback
            Some(content.lines().take(20).collect::<Vec<_>>().join("\n"))
        } else {
            Some(relevant.join("\n"))
        }
    }
}

/// Arch Wiki provider (offline-first).
pub struct ArchWikiProvider {
    /// Cache directory.
    cache_dir: String,
}

impl ArchWikiProvider {
    /// Default cache directory.
    pub const DEFAULT_CACHE_DIR: &'static str = "/var/lib/anna/sources/archwiki";

    /// Create new provider.
    pub fn new() -> Self {
        Self {
            cache_dir: Self::DEFAULT_CACHE_DIR.to_string(),
        }
    }

    /// Create with custom cache dir.
    pub fn with_cache_dir(dir: &str) -> Self {
        Self {
            cache_dir: dir.to_string(),
        }
    }

    /// Fetch wiki page (offline only).
    pub fn fetch(&self, page: &str) -> Result<String, String> {
        let page_file = format!("{}/{}.txt", self.cache_dir, Self::normalize_page_name(page));

        std::fs::read_to_string(&page_file)
            .map_err(|_| format!("Arch Wiki page '{}' not available offline", page))
    }

    /// Normalize page name for filesystem.
    fn normalize_page_name(page: &str) -> String {
        page.replace(' ', "_").replace('/', "_")
    }

    /// Extract section from wiki page.
    pub fn extract_section(content: &str, section: &str) -> Option<String> {
        let section_lower = section.to_lowercase();
        let mut in_section = false;
        let mut section_level = 0;
        let mut result = Vec::new();

        for line in content.lines() {
            // Check for heading (== Heading ==)
            if line.starts_with('=') && line.ends_with('=') {
                let level = line.chars().take_while(|&c| c == '=').count();
                let heading = line.trim_matches('=').trim().to_lowercase();

                if heading.contains(&section_lower) {
                    in_section = true;
                    section_level = level;
                    result.push(line.to_string());
                } else if in_section && level <= section_level {
                    // New section at same or higher level, stop
                    break;
                }
            } else if in_section {
                result.push(line.to_string());
            }
        }

        if result.is_empty() {
            None
        } else {
            Some(result.join("\n"))
        }
    }

    /// Check if page is cached.
    pub fn is_cached(&self, page: &str) -> bool {
        let page_file = format!("{}/{}.txt", self.cache_dir, Self::normalize_page_name(page));
        std::path::Path::new(&page_file).exists()
    }
}

impl Default for ArchWikiProvider {
    fn default() -> Self {
        Self::new()
    }
}

/// Local config provider.
pub struct LocalConfigProvider;

impl LocalConfigProvider {
    /// Common config paths.
    pub fn common_paths(name: &str) -> Vec<String> {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/root".to_string());

        vec![
            format!("/etc/{}", name),
            format!("{}/.config/{}", home, name),
            format!("{}/.{}", home, name),
            format!("/etc/{}.conf", name),
            format!("{}/.{}rc", home, name),
        ]
    }

    /// Find config file.
    pub fn find(name: &str) -> Option<String> {
        for path in Self::common_paths(name) {
            if std::path::Path::new(&path).exists() {
                return Some(path);
            }
        }
        None
    }

    /// Read config file.
    pub fn read(path: &str) -> Result<String, String> {
        std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read {}: {}", path, e))
    }
}

/// Intent to canonical commands mapping.
#[derive(Debug, Clone)]
pub struct IntentCommands {
    /// Intent name.
    pub intent: String,
    /// Canonical commands for this intent.
    pub commands: Vec<String>,
    /// Recommended wiki pages.
    pub wiki_pages: Vec<String>,
}

/// Get canonical commands for an intent.
pub fn commands_for_intent(intent: &str) -> Option<IntentCommands> {
    let (commands, wiki) = match intent {
        "packages.update_system" | "system_update" => (
            vec!["pacman", "checkupdates"],
            vec!["Pacman", "System_maintenance"],
        ),
        "packages.install" | "package_install" => (
            vec!["pacman"],
            vec!["Pacman"],
        ),
        "services.failed_services" | "services_failed" => (
            vec!["systemctl"],
            vec!["Systemd"],
        ),
        "boot.boot_time" | "boot_time" => (
            vec!["systemd-analyze"],
            vec!["Improving_performance/Boot_process"],
        ),
        "network.dns_check" | "dns_check" => (
            vec!["dig", "nslookup", "resolvectl"],
            vec!["Domain_name_resolution"],
        ),
        "security.firewall_status" | "firewall_status" => (
            vec!["firewall-cmd", "ufw", "iptables"],
            vec!["Firewalld", "Uncomplicated_Firewall"],
        ),
        "memory.status" | "memory_free" => (
            vec!["free"],
            vec!["Swap"],
        ),
        "disk.usage" | "disk_free" => (
            vec!["df", "du"],
            vec!["File_systems"],
        ),
        _ => return None,
    };

    Some(IntentCommands {
        intent: intent.to_string(),
        commands: commands.into_iter().map(String::from).collect(),
        wiki_pages: wiki.into_iter().map(String::from).collect(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_source_type() {
        assert!(SourceType::Man.is_documentation());
        assert!(!SourceType::Man.is_evidence());
        assert!(SourceType::Probe.is_evidence());
    }

    #[test]
    fn test_source_request() {
        let man = SourceRequest::man("pacman(8)", "SYSTEM UPGRADE");
        assert_eq!(man.source_type, SourceType::Man);
        assert!(man.required);

        let wiki = SourceRequest::arch_wiki("Pacman", "Upgrading packages");
        assert!(!wiki.required); // Wiki is optional by default
    }

    #[test]
    fn test_commands_for_intent() {
        let cmds = commands_for_intent("packages.update_system").unwrap();
        assert!(cmds.commands.contains(&"pacman".to_string()));
        assert!(cmds.wiki_pages.contains(&"Pacman".to_string()));
    }

    #[test]
    fn test_man_section_extraction() {
        let content = "NAME\n       pacman - package manager\n\nSYNOPSIS\n       pacman <operation>\n\nDESCRIPTION\n       pacman is a package manager.";
        let section = ManProvider::extract_section(content, "SYNOPSIS");
        assert!(section.is_some());
        assert!(section.unwrap().contains("operation"));
    }
}
