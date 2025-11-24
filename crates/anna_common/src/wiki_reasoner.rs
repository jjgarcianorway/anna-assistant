//! Wiki Reasoning Engine - v6.23.0
//!
//! Transforms Arch Wiki into a first-class reasoning source for system administration.
//! Given a user question, identifies the topic, fetches relevant wiki content,
//! combines it with telemetry and system knowledge, and produces tailored advice
//! with concrete steps, commands, and citations.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Wiki topic taxonomy - extensible set of system administration domains
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WikiTopic {
    /// Power management (TLP, powertop, battery optimization)
    PowerManagement,
    /// Disk space and filesystem maintenance
    DiskSpace,
    /// Boot performance and startup optimization
    BootPerformance,
    /// Networking (WiFi, DNS, connectivity)
    Networking,
    /// GPU stack (NVIDIA, AMD, Intel graphics)
    GpuStack,
    // Future: Printing, Sound, Containers, Virtualization, etc.
}

impl fmt::Display for WikiTopic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WikiTopic::PowerManagement => write!(f, "Power Management"),
            WikiTopic::DiskSpace => write!(f, "Disk Space"),
            WikiTopic::BootPerformance => write!(f, "Boot Performance"),
            WikiTopic::Networking => write!(f, "Networking"),
            WikiTopic::GpuStack => write!(f, "GPU Stack"),
        }
    }
}

/// User intent when asking about a wiki topic
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WikiIntent {
    /// Diagnose and fix issues
    Troubleshoot,
    /// Install new software or drivers
    Install,
    /// Configure existing system
    Configure,
    /// Understand concepts
    ExplainConcept,
}

impl fmt::Display for WikiIntent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WikiIntent::Troubleshoot => write!(f, "troubleshoot"),
            WikiIntent::Install => write!(f, "install"),
            WikiIntent::Configure => write!(f, "configure"),
            WikiIntent::ExplainConcept => write!(f, "explain"),
        }
    }
}

/// Arch Wiki citation with URL and optional section
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WikiCitation {
    /// Full Arch Wiki URL
    pub url: String,
    /// Optional section title or anchor
    pub section: Option<String>,
    /// Optional short label for citation
    pub note: Option<String>,
}

/// Single actionable step in wiki-based advice
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WikiStep {
    /// Step title (brief, actionable)
    pub title: String,
    /// Detailed description of what and why
    pub description: String,
    /// Shell commands to execute (safe by design)
    pub commands: Vec<String>,
    /// Optional warning or caution
    pub caution: Option<String>,
}

/// Complete wiki-based advice for a user question
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WikiAdvice {
    /// 2-5 line plain language summary
    pub summary: String,
    /// Ordered steps to follow
    pub steps: Vec<WikiStep>,
    /// Extra hints and tips
    pub notes: Vec<String>,
    /// Arch Wiki citations
    pub citations: Vec<WikiCitation>,
}

/// Configuration for wiki reasoning engine
#[derive(Debug, Clone)]
pub struct WikiReasonerConfig {
    /// Maximum characters to extract from wiki snippets
    pub max_snippet_chars: usize,
    /// Timeout for wiki fetches in milliseconds
    pub wiki_timeout_ms: u64,
    /// Prefer local corpus over network fetch
    pub prefer_local_corpus: bool,
}

impl Default for WikiReasonerConfig {
    fn default() -> Self {
        Self {
            max_snippet_chars: 8000,
            wiki_timeout_ms: 5000,
            prefer_local_corpus: false,
        }
    }
}

/// Errors that can occur during wiki reasoning
#[derive(Debug, thiserror::Error)]
pub enum WikiError {
    #[error("Network failure: {0}")]
    NetworkFailure(String),

    #[error("No relevant wiki topic found for question")]
    NoRelevantTopic,

    #[error("LLM reasoning failed: {0}")]
    LlmFailure(String),

    #[error("Failed to parse LLM response: {0}")]
    ParsingFailure(String),

    #[error("Wiki client error: {0}")]
    ClientError(String),
}

/// Main reasoning pipeline - combines classification, wiki fetch, and LLM reasoning
///
/// This function:
/// 1. Classifies the question into a topic + intent
/// 2. Builds system context from telemetry
/// 3. Fetches relevant wiki content (TODO: actual fetching, using metadata for now)
/// 4. Calls LLM to generate WikiAdvice
/// 5. Returns structured advice with steps and citations
pub async fn reason_with_wiki(
    question: &str,
    telemetry: &crate::telemetry::SystemTelemetry,
    _knowledge: &crate::system_knowledge::SystemKnowledgeBase,
    cfg: &WikiReasonerConfig,
) -> Result<WikiAdvice, WikiError> {
    use crate::wiki_topics;

    // Step 1: Classify topic + intent
    let topic_match = wiki_topics::classify_wiki_topic(question)
        .ok_or(WikiError::NoRelevantTopic)?;

    if topic_match.confidence < 0.4 {
        return Err(WikiError::NoRelevantTopic);
    }

    // Step 2: Build system context summary
    let system_context = build_system_context(telemetry);

    // Step 3: Get wiki metadata (TODO: fetch actual content via WikiClient)
    let metadata = wiki_topics::get_topic_metadata(topic_match.topic);

    // For now, use metadata as wiki snippet
    // TODO: Integrate WikiClient to fetch actual page content
    let wiki_snippet = format!(
        "Arch Wiki Page: {}\nKey Sections: {}\n\nTopic: {}\nIntent: {}",
        metadata.page_url,
        metadata.key_sections.join(", "),
        topic_match.topic,
        topic_match.intent
    );

    // Truncate to max length
    let wiki_snippet = if wiki_snippet.len() > cfg.max_snippet_chars {
        &wiki_snippet[..cfg.max_snippet_chars]
    } else {
        &wiki_snippet
    };

    // Step 4: Call LLM for reasoning
    let advice = crate::wiki_llm::reason_with_llm(question, &system_context, wiki_snippet)
        .await?;

    Ok(advice)
}

/// Build system context summary from telemetry
fn build_system_context(telemetry: &crate::telemetry::SystemTelemetry) -> String {
    let desktop_str = telemetry
        .desktop
        .as_ref()
        .and_then(|d| d.display_server.clone())
        .unwrap_or_else(|| "unknown".to_string());

    let disk_usage = telemetry
        .disks
        .iter()
        .find(|d| d.mount_point == "/")
        .map(|d| d.usage_percent)
        .unwrap_or(0.0);

    format!(
        "Desktop/WM: {}\nCPU: {} ({} cores)\nMemory: {:.1} GB total, {:.1} GB used\nDisk: Root at {:.0}%",
        desktop_str,
        telemetry.hardware.cpu_model,
        telemetry.cpu.cores,
        telemetry.memory.total_mb as f64 / 1024.0,
        telemetry.memory.used_mb as f64 / 1024.0,
        disk_usage
    )
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wiki_topic_display() {
        assert_eq!(WikiTopic::PowerManagement.to_string(), "Power Management");
        assert_eq!(WikiTopic::Networking.to_string(), "Networking");
    }

    #[test]
    fn test_wiki_intent_display() {
        assert_eq!(WikiIntent::Troubleshoot.to_string(), "troubleshoot");
        assert_eq!(WikiIntent::Configure.to_string(), "configure");
    }

    #[test]
    fn test_wiki_reasoner_config_default() {
        let cfg = WikiReasonerConfig::default();
        assert_eq!(cfg.max_snippet_chars, 8000);
        assert_eq!(cfg.wiki_timeout_ms, 5000);
        assert!(!cfg.prefer_local_corpus);
    }
}
