//! Evidence Engine - Anna's real knowledge layer (v0.0.410).
//!
//! The evidence engine is Anna's "brain" for gathering facts:
//! 1. Runs targeted probes based on domain/intent/tags
//! 2. Fetches relevant docs (man pages, Arch wiki, help output)
//! 3. Produces a compact EvidenceBundle for specialists
//!
//! Key principle: LLMs interpret evidence, they don't invent it.

use serde::{Deserialize, Serialize};
use std::time::SystemTime;

/// Request for evidence gathering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceRequest {
    /// Ticket ID for tracking
    pub ticket_id: String,
    /// Domain classification
    pub domain: EvidenceDomain,
    /// Intent classification
    pub intent: EvidenceIntent,
    /// Original user question
    pub question: String,
    /// Tags extracted by translator (e.g., ["vim", "syntax", "editor"])
    pub tags: Vec<String>,
}

/// Evidence domain classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceDomain {
    Desktop,
    Network,
    Storage,
    Services,
    Performance,
    Hardware,
    Security,
    Packages,
    Audio,
    Display,
    Boot,
    System,
}

impl EvidenceDomain {
    /// Convert from string (case-insensitive)
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "desktop" => Some(Self::Desktop),
            "network" => Some(Self::Network),
            "storage" => Some(Self::Storage),
            "services" | "systemd" => Some(Self::Services),
            "performance" => Some(Self::Performance),
            "hardware" => Some(Self::Hardware),
            "security" => Some(Self::Security),
            "packages" => Some(Self::Packages),
            "audio" => Some(Self::Audio),
            "display" | "graphics" => Some(Self::Display),
            "boot" => Some(Self::Boot),
            "system" => Some(Self::System),
            _ => None,
        }
    }

    /// Get related domains (for broader searches)
    pub fn related(&self) -> Vec<Self> {
        match self {
            Self::Desktop => vec![Self::Display, Self::Audio],
            Self::Performance => vec![Self::System, Self::Hardware],
            Self::Services => vec![Self::System, Self::Boot],
            Self::Storage => vec![Self::System],
            Self::Network => vec![Self::Security],
            _ => vec![],
        }
    }
}

impl std::fmt::Display for EvidenceDomain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Desktop => write!(f, "desktop"),
            Self::Network => write!(f, "network"),
            Self::Storage => write!(f, "storage"),
            Self::Services => write!(f, "services"),
            Self::Performance => write!(f, "performance"),
            Self::Hardware => write!(f, "hardware"),
            Self::Security => write!(f, "security"),
            Self::Packages => write!(f, "packages"),
            Self::Audio => write!(f, "audio"),
            Self::Display => write!(f, "display"),
            Self::Boot => write!(f, "boot"),
            Self::System => write!(f, "system"),
        }
    }
}

/// Intent classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceIntent {
    /// User wants to understand something
    Diagnose,
    /// User wants an explanation
    Explain,
    /// User wants to change configuration
    Configure,
    /// User wants to inspect current state
    Inspect,
    /// User wants statistics/metrics
    Stats,
    /// User wants to fix a problem
    Fix,
}

impl EvidenceIntent {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "diagnose" | "debug" | "troubleshoot" => Some(Self::Diagnose),
            "explain" | "what" | "why" => Some(Self::Explain),
            "configure" | "setup" | "enable" | "disable" => Some(Self::Configure),
            "inspect" | "check" | "show" | "list" => Some(Self::Inspect),
            "stats" | "metrics" | "usage" | "count" => Some(Self::Stats),
            "fix" | "repair" | "solve" => Some(Self::Fix),
            _ => None,
        }
    }
}

impl std::fmt::Display for EvidenceIntent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Diagnose => write!(f, "diagnose"),
            Self::Explain => write!(f, "explain"),
            Self::Configure => write!(f, "configure"),
            Self::Inspect => write!(f, "inspect"),
            Self::Stats => write!(f, "stats"),
            Self::Fix => write!(f, "fix"),
        }
    }
}

/// Complete evidence bundle for specialist consumption
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EvidenceBundle {
    /// Probe results (system state facts)
    pub probes: Vec<ProbeEvidence>,
    /// Documentation snippets (authoritative text)
    pub docs: Vec<DocSnippet>,
    /// Matching recipe candidates (learned patterns)
    pub recipes: Vec<RecipeMatch>,
    /// Bundle metadata
    pub metadata: BundleMetadata,
}

impl EvidenceBundle {
    pub fn new(ticket_id: &str) -> Self {
        Self {
            probes: vec![],
            docs: vec![],
            recipes: vec![],
            metadata: BundleMetadata::new(ticket_id),
        }
    }

    /// Check if bundle has any useful evidence
    pub fn has_evidence(&self) -> bool {
        !self.probes.is_empty() || !self.docs.is_empty()
    }

    /// Get total evidence count
    pub fn evidence_count(&self) -> usize {
        self.probes.len() + self.docs.len() + self.recipes.len()
    }

    /// Add a probe result
    pub fn add_probe(&mut self, probe: ProbeEvidence) {
        self.probes.push(probe);
    }

    /// Add a doc snippet
    pub fn add_doc(&mut self, doc: DocSnippet) {
        self.docs.push(doc);
    }

    /// Add a recipe match
    pub fn add_recipe(&mut self, recipe: RecipeMatch) {
        self.recipes.push(recipe);
    }

    /// Get all evidence IDs for citation
    pub fn all_evidence_ids(&self) -> Vec<String> {
        let mut ids: Vec<String> = self.probes.iter().map(|p| p.id.clone()).collect();
        ids.extend(self.docs.iter().map(|d| d.id.clone()));
        ids.extend(self.recipes.iter().map(|r| r.id.clone()));
        ids
    }

    /// Format for specialist context (concise)
    pub fn format_for_specialist(&self) -> String {
        let mut output = String::new();

        if !self.probes.is_empty() {
            output.push_str("=== PROBE EVIDENCE ===\n");
            for probe in &self.probes {
                output.push_str(&format!(
                    "[{}] {}\n{}\n\n",
                    probe.id, probe.summary, probe.excerpt
                ));
            }
        }

        if !self.docs.is_empty() {
            output.push_str("=== DOCUMENTATION ===\n");
            for doc in &self.docs {
                output.push_str(&format!(
                    "[{}] {} ({})\n{}\n\n",
                    doc.id, doc.title, doc.source, doc.snippet
                ));
            }
        }

        if !self.recipes.is_empty() {
            output.push_str("=== MATCHING RECIPES ===\n");
            for recipe in &self.recipes {
                output.push_str(&format!(
                    "[{}] {} (confidence: {}%)\n{}\n\n",
                    recipe.id, recipe.title, recipe.confidence, recipe.summary
                ));
            }
        }

        if output.is_empty() {
            output.push_str("No evidence gathered.\n");
        }

        output
    }
}

/// Evidence from a probe (system command)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProbeEvidence {
    /// Unique ID (e.g., "probe:df_root")
    pub id: String,
    /// Short human summary
    pub summary: String,
    /// Relevant excerpt (compact)
    pub excerpt: String,
    /// Reference to raw output if needed
    pub raw_ref: Option<String>,
    /// Command that was run
    pub command: String,
    /// Exit code
    pub exit_code: i32,
    /// Timestamp
    pub timestamp: u64,
}

impl ProbeEvidence {
    pub fn new(id: &str, command: &str, summary: &str, excerpt: &str) -> Self {
        Self {
            id: id.to_string(),
            summary: summary.to_string(),
            excerpt: excerpt.to_string(),
            raw_ref: None,
            command: command.to_string(),
            exit_code: 0,
            timestamp: current_millis(),
        }
    }

    pub fn with_exit_code(mut self, code: i32) -> Self {
        self.exit_code = code;
        self
    }
}

/// Documentation snippet from authoritative source
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocSnippet {
    /// Unique ID (e.g., "doc:arch:fancontrol", "man:systemd.service")
    pub id: String,
    /// Human-readable title
    pub title: String,
    /// Source type
    pub source: DocSource,
    /// Relevant text snippet
    pub snippet: String,
    /// Location reference (URL, man section, file path)
    pub location: String,
    /// Relevance score (0-100)
    pub relevance: u8,
}

/// Documentation source type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DocSource {
    ArchWiki,
    ManPage,
    HelpOutput,
    LocalDoc,
    ConfigFile,
}

impl std::fmt::Display for DocSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ArchWiki => write!(f, "arch_wiki"),
            Self::ManPage => write!(f, "man"),
            Self::HelpOutput => write!(f, "help"),
            Self::LocalDoc => write!(f, "doc"),
            Self::ConfigFile => write!(f, "config"),
        }
    }
}

impl DocSnippet {
    pub fn new(source: DocSource, title: &str, snippet: &str, location: &str) -> Self {
        let id = format!("{}:{}", source, title.to_lowercase().replace(' ', "_"));
        Self {
            id,
            title: title.to_string(),
            source,
            snippet: truncate_snippet(snippet, 500),
            location: location.to_string(),
            relevance: 50,
        }
    }

    pub fn with_relevance(mut self, relevance: u8) -> Self {
        self.relevance = relevance.min(100);
        self
    }
}

/// A matching recipe candidate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeMatch {
    /// Recipe ID
    pub id: String,
    /// Recipe title
    pub title: String,
    /// Short summary
    pub summary: String,
    /// Confidence percentage
    pub confidence: u8,
    /// Required actions
    pub actions: Vec<String>,
}

/// Bundle metadata
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BundleMetadata {
    /// Ticket ID
    pub ticket_id: String,
    /// Creation timestamp
    pub created_at: u64,
    /// Time spent gathering evidence (ms)
    pub gather_time_ms: u64,
    /// Probes that were run
    pub probes_run: Vec<String>,
    /// Doc sources searched
    pub docs_searched: Vec<String>,
}

impl BundleMetadata {
    pub fn new(ticket_id: &str) -> Self {
        Self {
            ticket_id: ticket_id.to_string(),
            created_at: current_millis(),
            gather_time_ms: 0,
            probes_run: vec![],
            docs_searched: vec![],
        }
    }
}

/// Truncate to max length with ellipsis
fn truncate_snippet(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max.saturating_sub(3)])
    }
}

fn current_millis() -> u64 {
    SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_evidence_domain_from_str() {
        assert_eq!(
            EvidenceDomain::from_str("services"),
            Some(EvidenceDomain::Services)
        );
        assert_eq!(
            EvidenceDomain::from_str("STORAGE"),
            Some(EvidenceDomain::Storage)
        );
        assert_eq!(EvidenceDomain::from_str("unknown"), None);
    }

    #[test]
    fn test_evidence_bundle() {
        let mut bundle = EvidenceBundle::new("TEST-001");
        assert!(!bundle.has_evidence());

        bundle.add_probe(ProbeEvidence::new(
            "probe:df_root",
            "df -h /",
            "Root filesystem 75% full",
            "/dev/sda1 100G 75G 25G 75% /",
        ));

        assert!(bundle.has_evidence());
        assert_eq!(bundle.evidence_count(), 1);
    }

    #[test]
    fn test_doc_snippet() {
        let doc = DocSnippet::new(
            DocSource::ManPage,
            "systemd.service",
            "A service unit file...",
            "man systemd.service(5)",
        );

        assert!(doc.id.starts_with("man:"));
        assert_eq!(doc.source, DocSource::ManPage);
    }
}
