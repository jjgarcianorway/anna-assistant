//! Transcript Events v0.0.72 - Unified Event Types
//!
//! Single source of truth event types. Both human and debug renderers
//! consume these events, ensuring they cannot diverge.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Unified transcript event - source of truth for both modes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptEventV72 {
    /// Timestamp
    pub ts: DateTime<Utc>,
    /// Event data
    pub data: EventDataV72,
}

/// Event data variants - each contains both human and debug info
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum EventDataV72 {
    /// User message
    UserMessage { text: String },

    /// Staff message (service desk, department, anna)
    StaffMessage {
        role: RoleV72,
        tone: ToneV72,
        /// Human-friendly content (shown in human mode)
        content_human: String,
        /// Debug content with internals (shown in debug mode)
        content_debug: Option<String>,
    },

    /// Evidence gathered from tools
    Evidence {
        /// Internal evidence ID (E1, E2, etc.) - debug only
        evidence_id: String,
        /// Internal tool name (hw_snapshot_summary) - debug only
        tool_name: String,
        /// Human-friendly label (hardware inventory) - human mode
        human_label: String,
        /// Human-friendly summary - human mode
        summary_human: String,
        /// Debug summary with raw data - debug mode
        summary_debug: Option<String>,
        /// Execution time - debug only
        duration_ms: u64,
    },

    /// Tool call started
    ToolCall {
        tool_name: String,
        /// Human action description
        action_human: String,
        /// Tool arguments - debug only
        args: Option<serde_json::Value>,
    },

    /// Tool result received
    ToolResult {
        tool_name: String,
        success: bool,
        /// Human description of what was found
        result_human: String,
        /// Raw result - debug only
        result_raw: Option<String>,
        duration_ms: u64,
    },

    /// Classification/translator output
    Classification {
        /// Human description of what was understood
        understood_human: String,
        /// Canonical translator lines - debug only
        canonical_lines: Option<Vec<String>>,
        /// Parse attempts count - debug only
        parse_attempts: Option<u32>,
        /// Whether fallback was used - debug only
        fallback_used: bool,
    },

    /// Reliability score
    Reliability {
        score: u8,
        /// Short rationale for human mode (e.g., "good evidence coverage")
        rationale_short: String,
        /// Full rationale for debug mode
        rationale_full: Option<String>,
        /// Uncited claims - debug only
        uncited_claims: Option<Vec<String>>,
    },

    /// Performance metrics
    Perf {
        /// Total time
        total_ms: u64,
        /// Breakdown by phase - debug only
        breakdown: Option<PerfBreakdownV72>,
    },

    /// Confirmation request (safety prompt)
    Confirmation {
        /// What will change - human readable
        change_description: String,
        /// Risk level
        risk_level: RiskLevelV72,
        /// Exact phrase required to confirm (unchanged)
        confirm_phrase: String,
        /// Rollback plan summary - human mode
        rollback_summary: String,
        /// Full rollback details - debug only
        rollback_details: Option<String>,
    },

    /// Warning or issue
    Warning {
        /// Human-friendly warning message
        message_human: String,
        /// Technical details - debug only
        details_debug: Option<String>,
        /// Warning category
        category: WarningCategoryV72,
    },

    /// Phase separator
    Phase { name: String },

    /// Working/progress indicator
    Working {
        role: RoleV72,
        /// Human-friendly progress message
        message: String,
    },
}

/// Role in the IT department dialogue
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RoleV72 {
    ServiceDesk,
    Network,
    Storage,
    Performance,
    Audio,
    Graphics,
    Boot,
    Security,
    InfoDesk,
    Anna,
    /// Internal roles - hidden in human mode
    Translator,
    Junior,
    Senior,
    Annad,
}

impl RoleV72 {
    /// Display name for human mode
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::ServiceDesk => "Service Desk",
            Self::Network => "Network",
            Self::Storage => "Storage",
            Self::Performance => "Performance",
            Self::Audio => "Audio",
            Self::Graphics => "Graphics",
            Self::Boot => "Boot",
            Self::Security => "Security",
            Self::InfoDesk => "Info Desk",
            Self::Anna => "Anna",
            Self::Translator => "Translator",
            Self::Junior => "Junior",
            Self::Senior => "Senior",
            Self::Annad => "annad",
        }
    }

    /// Tag for transcript (lowercase)
    pub fn tag(&self) -> &'static str {
        match self {
            Self::ServiceDesk => "service desk",
            Self::Network => "network",
            Self::Storage => "storage",
            Self::Performance => "performance",
            Self::Audio => "audio",
            Self::Graphics => "graphics",
            Self::Boot => "boot",
            Self::Security => "security",
            Self::InfoDesk => "info desk",
            Self::Anna => "anna",
            Self::Translator => "translator",
            Self::Junior => "junior",
            Self::Senior => "senior",
            Self::Annad => "annad",
        }
    }

    /// Whether visible in human mode
    pub fn visible_in_human(&self) -> bool {
        !matches!(
            self,
            Self::Translator | Self::Junior | Self::Senior | Self::Annad
        )
    }
}

/// Message tone
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToneV72 {
    #[default]
    Neutral,
    Brisk,
    Skeptical,
    Helpful,
    Cautious,
    Urgent,
}

/// Risk level for confirmations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RiskLevelV72 {
    Low,
    Medium,
    High,
    Destructive,
}

impl RiskLevelV72 {
    pub fn display(&self) -> &'static str {
        match self {
            Self::Low => "Low",
            Self::Medium => "Medium",
            Self::High => "High",
            Self::Destructive => "DESTRUCTIVE",
        }
    }
}

/// Warning category
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WarningCategoryV72 {
    Parse,
    Fallback,
    MissingEvidence,
    LowConfidence,
    Other,
}

/// Performance breakdown
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerfBreakdownV72 {
    pub translation_ms: Option<u64>,
    pub tool_execution_ms: Option<u64>,
    pub synthesis_ms: Option<u64>,
    pub verification_ms: Option<u64>,
}

impl TranscriptEventV72 {
    pub fn new(data: EventDataV72) -> Self {
        Self {
            ts: Utc::now(),
            data,
        }
    }

    pub fn with_timestamp(ts: DateTime<Utc>, data: EventDataV72) -> Self {
        Self { ts, data }
    }
}

/// Event stream collector
#[derive(Debug, Clone, Default)]
pub struct TranscriptStreamV72 {
    events: Vec<TranscriptEventV72>,
    case_id: Option<String>,
}

impl TranscriptStreamV72 {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_case_id(mut self, id: &str) -> Self {
        self.case_id = Some(id.to_string());
        self
    }

    pub fn push(&mut self, event: TranscriptEventV72) {
        self.events.push(event);
    }

    pub fn push_data(&mut self, data: EventDataV72) {
        self.events.push(TranscriptEventV72::new(data));
    }

    pub fn events(&self) -> &[TranscriptEventV72] {
        &self.events
    }

    pub fn case_id(&self) -> Option<&str> {
        self.case_id.as_deref()
    }

    pub fn len(&self) -> usize {
        self.events.len()
    }

    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    pub fn clear(&mut self) {
        self.events.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_role_visibility() {
        assert!(RoleV72::ServiceDesk.visible_in_human());
        assert!(RoleV72::Network.visible_in_human());
        assert!(RoleV72::Anna.visible_in_human());
        assert!(!RoleV72::Translator.visible_in_human());
        assert!(!RoleV72::Junior.visible_in_human());
        assert!(!RoleV72::Annad.visible_in_human());
    }

    #[test]
    fn test_event_creation() {
        let event = TranscriptEventV72::new(EventDataV72::UserMessage {
            text: "What is my CPU?".to_string(),
        });
        if let EventDataV72::UserMessage { text } = &event.data {
            assert_eq!(text, "What is my CPU?");
        } else {
            panic!("Wrong event type");
        }
    }

    #[test]
    fn test_stream_operations() {
        let mut stream = TranscriptStreamV72::new().with_case_id("test-123");
        stream.push_data(EventDataV72::UserMessage {
            text: "test".to_string(),
        });
        assert_eq!(stream.len(), 1);
        assert_eq!(stream.case_id(), Some("test-123"));
    }
}
