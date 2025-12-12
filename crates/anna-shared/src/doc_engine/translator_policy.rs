//! Translator policy for documentation needs (v0.0.429).
//!
//! Determines when specialists should query documentation:
//! - "What does this error mean?" -> needs docs
//! - "How do I fix this?" -> needs docs
//! - "Do I have failed services?" -> probes + maybe recipes, docs if unclear

use super::{DocQuery, DocSourceKind};

/// Doc need level for a query
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DocNeedLevel {
    /// No docs needed - pure probe/recipe question
    None,
    /// Docs optional - might help clarify
    Optional,
    /// Docs recommended - would improve answer
    Recommended,
    /// Docs required - question asks for explanation
    Required,
}

impl DocNeedLevel {
    /// Check if docs should be queried
    pub fn should_query(&self) -> bool {
        matches!(self, Self::Recommended | Self::Required)
    }

    /// Check if answer must cite docs
    pub fn must_cite(&self) -> bool {
        matches!(self, Self::Required)
    }
}

/// Analyze a question to determine doc needs
pub fn analyze_doc_need(question: &str) -> DocNeedAnalysis {
    let lower = question.to_lowercase();

    // Explanation requests always need docs
    if is_explanation_request(&lower) {
        return DocNeedAnalysis {
            level: DocNeedLevel::Required,
            reason: "User is asking for explanation".to_string(),
            suggested_sources: vec![DocSourceKind::ArchWiki, DocSourceKind::ManPage],
            suggested_topics: extract_topics(&lower),
        };
    }

    // Fix/troubleshooting requests should use docs
    if is_fix_request(&lower) {
        return DocNeedAnalysis {
            level: DocNeedLevel::Recommended,
            reason: "User needs troubleshooting guidance".to_string(),
            suggested_sources: vec![DocSourceKind::ArchWiki, DocSourceKind::ManPage],
            suggested_topics: extract_topics(&lower),
        };
    }

    // Error interpretation needs docs
    if is_error_interpretation(&lower) {
        return DocNeedAnalysis {
            level: DocNeedLevel::Required,
            reason: "User wants to understand an error".to_string(),
            suggested_sources: vec![DocSourceKind::ArchWiki, DocSourceKind::ManPage],
            suggested_topics: extract_topics(&lower),
        };
    }

    // Status checks usually don't need docs
    if is_status_check(&lower) {
        return DocNeedAnalysis {
            level: DocNeedLevel::None,
            reason: "Simple status check".to_string(),
            suggested_sources: vec![],
            suggested_topics: vec![],
        };
    }

    // Default: optional
    DocNeedAnalysis {
        level: DocNeedLevel::Optional,
        reason: "May benefit from documentation context".to_string(),
        suggested_sources: vec![DocSourceKind::ArchWiki],
        suggested_topics: extract_topics(&lower),
    }
}

/// Analysis result
#[derive(Debug, Clone)]
pub struct DocNeedAnalysis {
    /// How much docs are needed
    pub level: DocNeedLevel,
    /// Why this level was chosen
    pub reason: String,
    /// Suggested doc sources to query
    pub suggested_sources: Vec<DocSourceKind>,
    /// Topics to search for
    pub suggested_topics: Vec<String>,
}

impl DocNeedAnalysis {
    /// Convert to a doc query
    pub fn to_query(&self, base_query: &str) -> Option<DocQuery> {
        if !self.level.should_query() {
            return None;
        }

        let query_text = if self.suggested_topics.is_empty() {
            base_query.to_string()
        } else {
            self.suggested_topics.join(" ")
        };

        Some(DocQuery::new(&query_text).with_sources(self.suggested_sources.clone()))
    }
}

/// Check if question asks for explanation
fn is_explanation_request(lower: &str) -> bool {
    let patterns = [
        "what is",
        "what does",
        "what are",
        "explain",
        "why is",
        "why does",
        "how does",
        "tell me about",
        "what's the difference",
        "meaning of",
        "definition of",
    ];

    patterns.iter().any(|p| lower.contains(p))
}

/// Check if question asks for fix/help
fn is_fix_request(lower: &str) -> bool {
    let patterns = [
        "how do i fix",
        "how can i fix",
        "how to fix",
        "how do i solve",
        "how to solve",
        "how do i debug",
        "how to debug",
        "troubleshoot",
        "not working",
        "doesn't work",
        "won't start",
        "fails to",
        "error",
        "problem with",
    ];

    patterns.iter().any(|p| lower.contains(p))
}

/// Check if question asks about error meaning
fn is_error_interpretation(lower: &str) -> bool {
    let patterns = [
        "what does this error mean",
        "what does this mean",
        "why am i getting",
        "what causes",
        "this error",
        "exit code",
        "failed with",
    ];

    patterns.iter().any(|p| lower.contains(p))
}

/// Check if question is a simple status check
fn is_status_check(lower: &str) -> bool {
    let patterns = [
        "do i have",
        "is my",
        "how much",
        "how many",
        "show me",
        "list my",
        "check if",
        "is it running",
        "status of",
        "what's my",
    ];

    patterns.iter().any(|p| lower.contains(p))
}

/// Extract likely topics from question
fn extract_topics(lower: &str) -> Vec<String> {
    let mut topics = Vec::new();

    // Common system topics
    let known_topics = [
        ("systemd", "systemd"),
        ("systemctl", "systemd"),
        ("service", "systemd"),
        ("timer", "systemd.timer"),
        ("pacman", "pacman"),
        ("package", "pacman"),
        ("disk", "File_systems"),
        ("mount", "fstab"),
        ("boot", "Arch_boot_process"),
        ("grub", "GRUB"),
        ("ssd", "Solid_state_drive"),
        ("trim", "TRIM"),
        ("network", "Network_configuration"),
        ("wifi", "Wireless"),
        ("wireless", "Wireless"),
        ("audio", "PipeWire"),
        ("sound", "PipeWire"),
        ("nvidia", "NVIDIA"),
        ("amd", "AMDGPU"),
        ("intel", "Intel_graphics"),
        ("wayland", "Wayland"),
        ("xorg", "Xorg"),
        ("ssh", "SSH"),
        ("firewall", "Firewall"),
        ("memory", "Swap"),
        ("ram", "Swap"),
        ("swap", "Swap"),
        ("kernel", "Kernel"),
        ("mkinitcpio", "Mkinitcpio"),
        ("initramfs", "Mkinitcpio"),
    ];

    for (keyword, topic) in &known_topics {
        if lower.contains(keyword) {
            let topic_str = topic.to_string();
            if !topics.contains(&topic_str) {
                topics.push(topic_str);
            }
        }
    }

    topics
}

/// Guidelines for specialists on using docs
pub struct DocUsageGuidelines;

impl DocUsageGuidelines {
    /// Rule 1: Probes first, docs second
    pub const PROBES_FIRST: &'static str =
        "Always collect system state via probes before consulting docs. \
         Docs interpret results, they don't invent state.";

    /// Rule 2: Doc queries must be explicit
    pub const EXPLICIT_QUERIES: &'static str = "Send structured doc queries with specific topics. \
         Don't just search for entire questions.";

    /// Rule 3: Summarization required
    pub const SUMMARIZE: &'static str = "Don't dump raw doc content into answers. \
         Summarize the relevant part and cite the source.";

    /// Rule 4: No overreach
    pub const NO_OVERREACH: &'static str =
        "If docs are unclear or conflict with probes, prefer probes. \
         Lower confidence and mark as partial if uncertain.";

    /// Rule 5: Minimal usage
    pub const MINIMAL_USAGE: &'static str =
        "Use docs to clarify results and provide 1-2 short recommendations. \
         Don't paste entire wiki sections.";

    /// Rule 6: Never pretend
    pub const NEVER_PRETEND: &'static str =
        "If no relevant docs are found, return partial or failure. \
         Never make up 'according to Arch Wiki' statements.";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_explanation_needs_docs() {
        let analysis = analyze_doc_need("What is TRIM and why do I need it?");
        assert_eq!(analysis.level, DocNeedLevel::Required);
        assert!(
            analysis.suggested_topics.contains(&"TRIM".to_string())
                || analysis
                    .suggested_topics
                    .contains(&"Solid_state_drive".to_string())
        );
    }

    #[test]
    fn test_fix_request_recommends_docs() {
        let analysis = analyze_doc_need("How do I fix a failing systemd service?");
        assert!(
            analysis.level == DocNeedLevel::Recommended || analysis.level == DocNeedLevel::Required
        );
        assert!(analysis
            .suggested_topics
            .iter()
            .any(|t| t.contains("systemd")));
    }

    #[test]
    fn test_status_check_no_docs() {
        let analysis = analyze_doc_need("Do I have any failed services?");
        assert_eq!(analysis.level, DocNeedLevel::None);
    }

    #[test]
    fn test_error_interpretation() {
        let analysis = analyze_doc_need("What does this error mean: Exit code 1");
        assert_eq!(analysis.level, DocNeedLevel::Required);
    }

    #[test]
    fn test_topic_extraction() {
        let topics = extract_topics("how to configure systemd timer for weekly trim");
        assert!(
            topics.contains(&"systemd".to_string())
                || topics.contains(&"systemd.timer".to_string())
        );
        assert!(
            topics.contains(&"TRIM".to_string())
                || topics.contains(&"Solid_state_drive".to_string())
        );
    }

    #[test]
    fn test_to_query() {
        let analysis = analyze_doc_need("What is TRIM?");
        let query = analysis.to_query("What is TRIM?");
        assert!(query.is_some());

        let analysis = analyze_doc_need("How much RAM do I have?");
        let query = analysis.to_query("How much RAM do I have?");
        assert!(query.is_none()); // Status check doesn't need docs
    }
}
