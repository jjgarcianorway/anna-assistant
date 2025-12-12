//! Knowledge item abstraction for research-first answers (v0.0.408).
//!
//! Unified representation of knowledge from various sources:
//! - Probe outputs (shell commands)
//! - Man pages
//! - --help output
//! - Local documentation (/usr/share/doc)
//! - Offline Arch Wiki mirror
//! - Anna's own docs (recipes, handbook)

use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;

/// Type of knowledge source
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KnowledgeSourceType {
    /// Output from shell probes (df -h, systemctl status, etc.)
    ProbeOutput,
    /// Man pages (man systemd.service, man pacman)
    ManPage,
    /// Help output (--help, -h)
    HelpOutput,
    /// Local docs (/usr/share/doc/**)
    LocalDoc,
    /// Offline Arch Wiki mirror (/var/lib/anna/arch_wiki)
    ArchWikiLocal,
    /// Anna's own documentation (recipes, handbook)
    AnnaDoc,
}

impl KnowledgeSourceType {
    /// Default confidence for this source type
    pub fn default_confidence(&self) -> u8 {
        match self {
            Self::ProbeOutput => 95,   // Direct observation
            Self::ManPage => 90,       // Official documentation
            Self::HelpOutput => 85,    // Tool-provided help
            Self::LocalDoc => 70,      // May be outdated
            Self::ArchWikiLocal => 85, // Curated but may lag
            Self::AnnaDoc => 80,       // Learned patterns
        }
    }

    /// Human-readable label
    pub fn label(&self) -> &'static str {
        match self {
            Self::ProbeOutput => "probe",
            Self::ManPage => "man",
            Self::HelpOutput => "help",
            Self::LocalDoc => "doc",
            Self::ArchWikiLocal => "wiki",
            Self::AnnaDoc => "anna",
        }
    }
}

impl std::fmt::Display for KnowledgeSourceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.label())
    }
}

/// A single knowledge item from any source
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeItem {
    /// Unique ID (hash of source_type + path/title)
    pub id: String,
    /// Source type
    pub source_type: KnowledgeSourceType,
    /// File path (for local files, man pages)
    pub path: Option<PathBuf>,
    /// Human-readable title
    pub title: String,
    /// Tags for matching (e.g., ["systemd", "service", "debug"])
    pub tags: Vec<String>,
    /// Content snippet (max ~500 chars)
    pub content_snippet: String,
    /// Confidence level (0-100)
    pub confidence: u8,
}

impl KnowledgeItem {
    /// Create a new knowledge item
    pub fn new(
        source_type: KnowledgeSourceType,
        title: impl Into<String>,
        content_snippet: impl Into<String>,
    ) -> Self {
        let title = title.into();
        let content_snippet = truncate_snippet(&content_snippet.into(), 500);
        let id = compute_id(&source_type, None, &title);

        Self {
            id,
            source_type,
            path: None,
            title,
            tags: vec![],
            content_snippet,
            confidence: source_type.default_confidence(),
        }
    }

    /// Create from a file path
    pub fn from_path(
        source_type: KnowledgeSourceType,
        path: PathBuf,
        title: impl Into<String>,
        content_snippet: impl Into<String>,
    ) -> Self {
        let title = title.into();
        let content_snippet = truncate_snippet(&content_snippet.into(), 500);
        let id = compute_id(&source_type, Some(&path), &title);

        Self {
            id,
            source_type,
            path: Some(path),
            title,
            tags: vec![],
            content_snippet,
            confidence: source_type.default_confidence(),
        }
    }

    /// Create from probe output
    pub fn from_probe(command: &str, output: &str) -> Self {
        let title = format!("probe: {}", truncate_command(command, 40));
        let snippet = truncate_snippet(output, 500);

        Self {
            id: compute_id(&KnowledgeSourceType::ProbeOutput, None, command),
            source_type: KnowledgeSourceType::ProbeOutput,
            path: None,
            title,
            tags: extract_tags_from_command(command),
            content_snippet: snippet,
            confidence: KnowledgeSourceType::ProbeOutput.default_confidence(),
        }
    }

    /// Set tags
    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

    /// Set confidence
    pub fn with_confidence(mut self, confidence: u8) -> Self {
        self.confidence = confidence.min(100);
        self
    }

    /// Format for display in evidence section
    pub fn format_evidence(&self) -> String {
        let source_label = self.source_type.label();
        if let Some(ref path) = self.path {
            format!("[{}] {} ({})", source_label, self.title, path.display())
        } else {
            format!("[{}] {}", source_label, self.title)
        }
    }

    /// Format with snippet for solver context
    pub fn format_for_solver(&self) -> String {
        format!(
            "--- {} [id={}] ---\n{}\n",
            self.title, self.id, self.content_snippet
        )
    }
}

/// Query parameters for knowledge search
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct KnowledgeQuery {
    /// Keywords to search for
    pub keywords: Vec<String>,
    /// Tags to filter by
    pub tags: Vec<String>,
    /// Source types to search (empty = all)
    pub source_types: Vec<KnowledgeSourceType>,
    /// Maximum items to return
    pub max_items: usize,
}

impl KnowledgeQuery {
    /// Create a new query
    pub fn new() -> Self {
        Self {
            keywords: vec![],
            tags: vec![],
            source_types: vec![],
            max_items: 10,
        }
    }

    /// Add keywords
    pub fn with_keywords(mut self, keywords: Vec<String>) -> Self {
        self.keywords = keywords;
        self
    }

    /// Add tags
    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

    /// Filter by source types
    pub fn with_source_types(mut self, types: Vec<KnowledgeSourceType>) -> Self {
        self.source_types = types;
        self
    }

    /// Set max items
    pub fn with_limit(mut self, max: usize) -> Self {
        self.max_items = max;
        self
    }

    /// Build from translator output
    pub fn from_translator(domain: &str, intent: &str, entities: &[String]) -> Self {
        let mut keywords: Vec<String> = entities.iter().cloned().collect();

        // Add domain-specific keywords
        let domain_tags = domain_to_tags(domain);
        let tags: Vec<String> = domain_tags.iter().map(|s| s.to_string()).collect();

        // Add intent as keyword
        keywords.push(intent.to_string());

        Self {
            keywords,
            tags,
            source_types: vec![], // Search all
            max_items: 10,
        }
    }
}

/// Compute deterministic ID for knowledge item
fn compute_id(source_type: &KnowledgeSourceType, path: Option<&PathBuf>, title: &str) -> String {
    let mut hasher = DefaultHasher::new();
    source_type.hash(&mut hasher);
    if let Some(p) = path {
        p.to_string_lossy().hash(&mut hasher);
    }
    title.hash(&mut hasher);
    format!("k{:016x}", hasher.finish())
}

/// Truncate snippet to max length
fn truncate_snippet(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max.saturating_sub(3)])
    }
}

/// Truncate command for display
fn truncate_command(cmd: &str, max: usize) -> String {
    let first_line = cmd.lines().next().unwrap_or(cmd);
    if first_line.len() <= max {
        first_line.to_string()
    } else {
        format!("{}...", &first_line[..max.saturating_sub(3)])
    }
}

/// Extract tags from command
fn extract_tags_from_command(cmd: &str) -> Vec<String> {
    let mut tags = vec![];
    let cmd_lower = cmd.to_lowercase();

    // Common tool patterns
    let patterns = [
        ("systemctl", "systemd"),
        ("journalctl", "systemd"),
        ("pacman", "packages"),
        ("yay", "packages"),
        ("paru", "packages"),
        ("df", "storage"),
        ("du", "storage"),
        ("lsblk", "storage"),
        ("ip ", "network"),
        ("ss ", "network"),
        ("nmcli", "network"),
        ("pactl", "audio"),
        ("wpctl", "audio"),
        ("hyprctl", "desktop"),
        ("swaymsg", "desktop"),
        ("nvidia", "gpu"),
        ("amdgpu", "gpu"),
    ];

    for (pattern, tag) in patterns {
        if cmd_lower.contains(pattern) {
            tags.push(tag.to_string());
        }
    }

    tags
}

/// Map domain to relevant tags
fn domain_to_tags(domain: &str) -> Vec<&'static str> {
    match domain.to_lowercase().as_str() {
        "services" | "systemd" => vec!["systemd", "service", "unit"],
        "packages" => vec!["pacman", "package", "aur"],
        "storage" => vec!["disk", "storage", "filesystem", "mount"],
        "network" => vec!["network", "ip", "dns", "wifi"],
        "audio" => vec!["audio", "pulseaudio", "pipewire", "alsa"],
        "display" | "graphics" => vec!["display", "gpu", "xorg", "wayland"],
        "desktop" => vec!["desktop", "wayland", "hyprland", "kde"],
        "boot" => vec!["boot", "grub", "systemd-boot", "initramfs"],
        "security" => vec!["security", "permissions", "sudo", "firewall"],
        "system" => vec!["system", "kernel", "hardware"],
        _ => vec![],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_knowledge_item_creation() {
        let item = KnowledgeItem::new(
            KnowledgeSourceType::ManPage,
            "man systemd.service",
            "Restart= takes one of...",
        );

        assert!(item.id.starts_with('k'));
        assert_eq!(item.source_type, KnowledgeSourceType::ManPage);
        assert_eq!(item.confidence, 90);
    }

    #[test]
    fn test_from_probe() {
        let item = KnowledgeItem::from_probe("systemctl status sshd", "Active: active (running)");

        assert_eq!(item.source_type, KnowledgeSourceType::ProbeOutput);
        assert!(item.tags.contains(&"systemd".to_string()));
    }

    #[test]
    fn test_truncate_snippet() {
        let short = "hello";
        assert_eq!(truncate_snippet(short, 10), short);

        let long = "a".repeat(100);
        let truncated = truncate_snippet(&long, 20);
        assert!(truncated.len() <= 20);
        assert!(truncated.ends_with("..."));
    }

    #[test]
    fn test_knowledge_query_from_translator() {
        let query = KnowledgeQuery::from_translator(
            "services",
            "diagnose",
            &["sshd".to_string(), "failed".to_string()],
        );

        assert!(query.keywords.contains(&"sshd".to_string()));
        assert!(query.tags.contains(&"systemd".to_string()));
    }

    #[test]
    fn test_format_evidence() {
        let item = KnowledgeItem::new(KnowledgeSourceType::ManPage, "man pacman", "content");

        let evidence = item.format_evidence();
        assert!(evidence.contains("[man]"));
        assert!(evidence.contains("man pacman"));
    }
}
