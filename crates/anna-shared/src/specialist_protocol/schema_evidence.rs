//! Evidence and metrics types for specialist protocol.

use serde::{Deserialize, Serialize};

/// Evidence backing up claims
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ResponseEvidence {
    /// Probes used and their results
    #[serde(default)]
    pub probes_used: Vec<ProbeEvidence>,

    /// Arch Wiki pages referenced
    #[serde(default)]
    pub arch_wiki_pages: Vec<String>,

    /// Man pages referenced
    #[serde(default)]
    pub man_pages: Vec<String>,

    /// Help commands referenced
    #[serde(default)]
    pub help_commands: Vec<String>,
}

/// Evidence from a probe
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProbeEvidence {
    /// Probe ID
    pub id: String,

    /// Short summary of what was found
    pub summary: String,

    /// Reference to raw probe output (hash or ID)
    #[serde(default)]
    pub raw_reference: Option<String>,
}

/// Performance metrics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ResponseMetrics {
    /// Total latency in milliseconds
    #[serde(default)]
    pub latency_ms: u64,

    /// Input tokens (if LLM used)
    #[serde(default)]
    pub tokens_in: u32,

    /// Output tokens (if LLM used)
    #[serde(default)]
    pub tokens_out: u32,
}

/// Response metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseMeta {
    /// Who handled this (e.g., "Sofia (Desktop Administrator)")
    pub handled_by: String,

    /// Ticket ID
    pub ticket_id: String,

    /// Schema version
    #[serde(default = "default_version")]
    pub version: u32,
}

fn default_version() -> u32 {
    1
}

impl Default for ResponseMeta {
    fn default() -> Self {
        Self {
            handled_by: "System".to_string(),
            ticket_id: String::new(),
            version: 1,
        }
    }
}
