//! Event Model for Transcript System v0.0.70
//!
//! Defines the event types and transcript stream for dual-mode rendering.

use super::topics::EvidenceTopicV70;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Actors in the transcript
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActorV70 {
    You,
    ServiceDesk,
    Networking,
    Storage,
    Boot,
    Audio,
    Graphics,
    Security,
    Performance,
    InfoDesk,
    // Internal actors (hidden in human mode)
    Translator,
    Junior,
    Senior,
    Annad,
}

impl ActorV70 {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::You => "you",
            Self::ServiceDesk => "service-desk",
            Self::Networking => "networking",
            Self::Storage => "storage",
            Self::Boot => "boot",
            Self::Audio => "audio",
            Self::Graphics => "graphics",
            Self::Security => "security",
            Self::Performance => "performance",
            Self::InfoDesk => "info-desk",
            Self::Translator => "translator",
            Self::Junior => "junior",
            Self::Senior => "senior",
            Self::Annad => "annad",
        }
    }

    /// Whether this actor is visible in human mode
    pub fn visible_in_human(&self) -> bool {
        !matches!(
            self,
            Self::Translator | Self::Junior | Self::Senior | Self::Annad
        )
    }
}

/// Transcript event types for v0.0.70
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum EventV70 {
    /// User's request
    UserToAnna { text: String },

    /// Staff message
    StaffMessage {
        from: ActorV70,
        to: ActorV70,
        message_human: String,
        message_debug: String,
    },

    /// Evidence gathered (with topic abstraction)
    Evidence {
        actor: ActorV70,
        topic: EvidenceTopicV70,
        summary_human: String,
        summary_debug: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        tool_name: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        evidence_id: Option<String>,
        duration_ms: u64,
    },

    /// Tool call (debug only)
    ToolCall {
        tool_name: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        args: Option<String>,
        duration_ms: u64,
    },

    /// Tool result (debug only)
    ToolResult {
        tool_name: String,
        success: bool,
        #[serde(skip_serializing_if = "Option::is_none")]
        raw_excerpt: Option<String>,
    },

    /// Parse warning (debug only)
    ParseWarning {
        subsystem: String,
        details: String,
        fallback_used: bool,
    },

    /// Translator canonical 6-line output (debug only)
    TranslatorCanonical {
        intent: String,
        target: String,
        depth: String,
        topics: Vec<String>,
        actions: Vec<String>,
        safety: String,
    },

    /// Reliability score
    Reliability {
        score: u8,
        rationale_human: String,
        rationale_debug: String,
    },

    /// Performance metrics (debug only)
    Perf {
        total_ms: u64,
        #[serde(skip_serializing_if = "Option::is_none")]
        llm_ms: Option<u64>,
        tool_ms: u64,
        tool_count: usize,
        retry_count: usize,
    },

    /// Retry occurred (debug only)
    Retry {
        subsystem: String,
        attempt: usize,
        reason: String,
    },

    /// Final answer
    FinalAnswer {
        text: String,
        reliability: u8,
        reliability_reason: String,
    },

    /// Phase separator
    Phase { name: String },

    /// Working indicator
    Working { actor: ActorV70, message: String },
}

/// Timestamped event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimestampedEventV70 {
    pub ts: DateTime<Utc>,
    pub event: EventV70,
}

/// Statistics for debug mode display
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TranscriptStatsV70 {
    pub parse_warning_count: usize,
    pub retry_count: usize,
    pub fallback_count: usize,
    pub tool_call_count: usize,
    pub total_tool_ms: u64,
}

/// Transcript stream collector
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TranscriptStreamV70 {
    pub events: Vec<TimestampedEventV70>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub case_id: Option<String>,
    pub stats: TranscriptStatsV70,
}

impl TranscriptStreamV70 {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_case_id(mut self, case_id: &str) -> Self {
        self.case_id = Some(case_id.to_string());
        self
    }

    pub fn push(&mut self, event: EventV70) {
        // Update stats
        match &event {
            EventV70::ParseWarning { fallback_used, .. } => {
                self.stats.parse_warning_count += 1;
                if *fallback_used {
                    self.stats.fallback_count += 1;
                }
            }
            EventV70::Retry { .. } => {
                self.stats.retry_count += 1;
            }
            EventV70::ToolCall { duration_ms, .. } => {
                self.stats.tool_call_count += 1;
                self.stats.total_tool_ms += duration_ms;
            }
            _ => {}
        }

        self.events.push(TimestampedEventV70 {
            ts: Utc::now(),
            event,
        });
    }

    pub fn record_fallback(&mut self) {
        self.stats.fallback_count += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_internal_actors_hidden() {
        assert!(!ActorV70::Translator.visible_in_human());
        assert!(!ActorV70::Junior.visible_in_human());
        assert!(!ActorV70::Senior.visible_in_human());
        assert!(!ActorV70::Annad.visible_in_human());

        assert!(ActorV70::You.visible_in_human());
        assert!(ActorV70::ServiceDesk.visible_in_human());
        assert!(ActorV70::Networking.visible_in_human());
        assert!(ActorV70::Storage.visible_in_human());
    }

    #[test]
    fn test_stats_tracking() {
        let mut stream = TranscriptStreamV70::new();

        stream.push(EventV70::ToolCall {
            tool_name: "test".to_string(),
            args: None,
            duration_ms: 100,
        });
        stream.push(EventV70::ToolCall {
            tool_name: "test2".to_string(),
            args: None,
            duration_ms: 200,
        });
        stream.push(EventV70::ParseWarning {
            subsystem: "translator".to_string(),
            details: "test".to_string(),
            fallback_used: true,
        });
        stream.push(EventV70::Retry {
            subsystem: "junior".to_string(),
            attempt: 1,
            reason: "test".to_string(),
        });

        assert_eq!(stream.stats.tool_call_count, 2);
        assert_eq!(stream.stats.total_tool_ms, 300);
        assert_eq!(stream.stats.parse_warning_count, 1);
        assert_eq!(stream.stats.fallback_count, 1);
        assert_eq!(stream.stats.retry_count, 1);
    }
}
