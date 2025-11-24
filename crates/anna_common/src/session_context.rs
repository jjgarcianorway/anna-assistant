//! Session Context - Lightweight context for follow-up queries and proactive commentary
//!
//! v6.26.0: Deep Context Memory and Proactive Commentary
//!
//! ## Architecture
//!
//! SessionContext stores lightweight state about the current user session:
//! - Last query intent and structured result
//! - User preferences inferred from query patterns
//! - Timestamp for staleness detection
//!
//! ## Design Principles
//!
//! 1. **Temporary File Storage**: Context persists in `/tmp/anna-session-{uid}.json`
//!    to survive across annactl invocations. Cleaned up after 5 minutes of inactivity.
//!
//! 2. **Deterministic Preferences**: All preference inference is rule-based,
//!    no LLM involvement.
//!
//! 3. **Lightweight**: Stores references/summaries, not full results.
//!
//! 4. **Stale-Safe**: Old context automatically invalidated after timeout.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Maximum age for session context before it's considered stale
const MAX_CONTEXT_AGE_SECS: u64 = 300; // 5 minutes

/// Session context for follow-up queries and preference tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionContext {
    /// Last query intent
    pub last_intent: Option<QueryIntent>,

    /// Last structured answer summary
    pub last_answer: Option<LastAnswerSummary>,

    /// Inferred user preferences
    pub preferences: UserPreferences,

    /// When this context was last updated (Unix timestamp)
    last_updated_secs: u64,

    /// Query count in this session (for preference inference)
    query_count: usize,
}

/// High-level query intent classification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QueryIntent {
    /// System status check
    Status,

    /// Wiki reasoning on specific topic
    WikiReasoning { topic: String },

    /// System diagnostics
    Diagnostics,

    /// Configuration query
    Config,

    /// Action plan execution
    ActionPlan { plan_type: String },

    /// Follow-up request (more details, just commands, etc.)
    FollowUp { follow_up_type: FollowUpType },

    /// Generic query
    Generic,
}

/// Type of follow-up request
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum FollowUpType {
    /// User wants more details on last answer
    MoreDetails,

    /// User wants just the commands to execute
    JustCommands,

    /// User wants to clarify or refine last answer
    Clarification,
}

/// Summary of last answer for follow-up support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LastAnswerSummary {
    /// Last WikiAdvice result
    WikiAdvice {
        topic: String,
        advice_key: String, // Hash or ID for caching
        commands: Vec<String>,
        summary: String,
    },

    /// Last Insights from Insights Engine
    Insights {
        insight_ids: Vec<String>,
        top_severity: String, // "Critical", "Warning", "Info"
        summary: String,
    },

    /// Last Status data
    Status {
        summary: String,
        critical_issues: usize,
        warnings: usize,
    },

    /// Generic answer (no specific structure)
    Generic {
        summary: String,
    },
}

/// User preferences inferred from query patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPreferences {
    /// CLI vs GUI bias
    pub interface_bias: InterfaceBias,

    /// Detail level preference
    pub detail_level: DetailPreference,

    /// Counts for preference inference
    cli_mentions: usize,
    gui_mentions: usize,
    detail_requests: usize,
    brief_requests: usize,
}

/// Interface preference (CLI vs GUI)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InterfaceBias {
    Unknown,
    PreferCli,
    PreferGui,
}

/// Detail level preference
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DetailPreference {
    Normal,
    Verbose,
    Short,
}

impl SessionContext {
    /// Create a new empty session context
    pub fn new() -> Self {
        let now_secs = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Self {
            last_intent: None,
            last_answer: None,
            preferences: UserPreferences::default(),
            last_updated_secs: now_secs,
            query_count: 0,
        }
    }

    /// Load session context from file, or create new if missing/stale
    pub fn load_or_new() -> Self {
        let path = Self::session_file_path();

        // Try to load existing context
        if let Ok(contents) = fs::read_to_string(&path) {
            if let Ok(mut ctx) = serde_json::from_str::<SessionContext>(&contents) {
                // Check if context is fresh
                if !ctx.is_stale() {
                    return ctx;
                }
                // Context is stale, invalidate and return
                ctx.invalidate_if_stale();
                return ctx;
            }
        }

        // No valid context found, create new
        Self::new()
    }

    /// Save session context to file
    pub fn save(&self) -> Result<(), std::io::Error> {
        let path = Self::session_file_path();

        // Create parent directory if needed
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let json = serde_json::to_string_pretty(self)?;
        fs::write(&path, json)?;

        Ok(())
    }

    /// Get path to session context file
    fn session_file_path() -> PathBuf {
        // Use /tmp/anna-session-{uid}.json
        let uid = unsafe { libc::getuid() };
        PathBuf::from(format!("/tmp/anna-session-{}.json", uid))
    }

    /// Update context from a new query and result
    pub fn update_from_query(
        &mut self,
        intent: QueryIntent,
        answer_summary: Option<LastAnswerSummary>,
        user_query: &str,
    ) {
        let now_secs = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        self.last_intent = Some(intent);
        self.last_answer = answer_summary;
        self.last_updated_secs = now_secs;
        self.query_count += 1;

        // Infer preferences from query text
        self.infer_preferences_from_query(user_query);

        // Save to file after update
        let _ = self.save();
    }

    /// Infer user preferences from query text (rule-based, deterministic)
    fn infer_preferences_from_query(&mut self, query: &str) {
        let query_lower = query.to_lowercase();

        // CLI vs GUI bias detection
        let cli_keywords = ["terminal", "command", "cli", "shell", "bash", "pacman", "systemctl"];
        let gui_keywords = ["gui", "window", "desktop", "kde", "gnome", "xfce", "app", "application"];

        for keyword in &cli_keywords {
            if query_lower.contains(keyword) {
                self.preferences.cli_mentions += 1;
                break;
            }
        }

        for keyword in &gui_keywords {
            if query_lower.contains(keyword) {
                self.preferences.gui_mentions += 1;
                break;
            }
        }

        // Update interface bias if we have enough data (at least 3 queries)
        if self.query_count >= 3 {
            if self.preferences.cli_mentions > self.preferences.gui_mentions * 2 {
                self.preferences.interface_bias = InterfaceBias::PreferCli;
            } else if self.preferences.gui_mentions > self.preferences.cli_mentions * 2 {
                self.preferences.interface_bias = InterfaceBias::PreferGui;
            }
        }

        // Detail level detection
        let detail_keywords = ["more detail", "more information", "explain", "verbose", "elaborate"];
        let brief_keywords = ["just the command", "brief", "short", "quick", "summary", "tldr"];

        for keyword in &detail_keywords {
            if query_lower.contains(keyword) {
                self.preferences.detail_requests += 1;
                break;
            }
        }

        for keyword in &brief_keywords {
            if query_lower.contains(keyword) {
                self.preferences.brief_requests += 1;
                break;
            }
        }

        // Update detail preference
        if self.preferences.detail_requests > 1 {
            self.preferences.detail_level = DetailPreference::Verbose;
        } else if self.preferences.brief_requests > 1 {
            self.preferences.detail_level = DetailPreference::Short;
        }
    }

    /// Get follow-up context if available and fresh
    pub fn get_followup_context(&self) -> Option<(&QueryIntent, &LastAnswerSummary)> {
        if self.is_stale() {
            return None;
        }

        match (&self.last_intent, &self.last_answer) {
            (Some(intent), Some(answer)) => Some((intent, answer)),
            _ => None,
        }
    }

    /// Check if context is stale (too old to be useful)
    pub fn is_stale(&self) -> bool {
        let now_secs = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        (now_secs - self.last_updated_secs) > MAX_CONTEXT_AGE_SECS
    }

    /// Invalidate context if older than max age
    pub fn invalidate_if_stale(&mut self) {
        if self.is_stale() {
            self.last_intent = None;
            self.last_answer = None;
        }
    }

    /// Detect if query is a follow-up request
    pub fn detect_followup_type(query: &str) -> Option<FollowUpType> {
        let query_lower = query.to_lowercase();

        // "more details" patterns
        let detail_patterns = [
            "more detail",
            "more information",
            "can you give me more",
            "tell me more",
            "elaborate",
            "expand on that",
        ];

        for pattern in &detail_patterns {
            if query_lower.contains(pattern) {
                return Some(FollowUpType::MoreDetails);
            }
        }

        // "just commands" patterns
        let command_patterns = [
            "just the command",
            "just show me the command",
            "only the command",
            "commands to fix",
            "how do i fix it",
            "what should i run",
        ];

        for pattern in &command_patterns {
            if query_lower.contains(pattern) {
                return Some(FollowUpType::JustCommands);
            }
        }

        // Clarification patterns (generic follow-up)
        let clarify_patterns = [
            "can you clarify",
            "what do you mean",
            "explain that",
            "i don't understand",
        ];

        for pattern in &clarify_patterns {
            if query_lower.contains(pattern) {
                return Some(FollowUpType::Clarification);
            }
        }

        None
    }
}

impl Default for SessionContext {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for UserPreferences {
    fn default() -> Self {
        Self {
            interface_bias: InterfaceBias::Unknown,
            detail_level: DetailPreference::Normal,
            cli_mentions: 0,
            gui_mentions: 0,
            detail_requests: 0,
            brief_requests: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_context_creation() {
        let ctx = SessionContext::new();
        assert!(ctx.last_intent.is_none());
        assert!(ctx.last_answer.is_none());
        assert_eq!(ctx.preferences.interface_bias, InterfaceBias::Unknown);
        assert_eq!(ctx.preferences.detail_level, DetailPreference::Normal);
    }

    #[test]
    fn test_followup_type_detection_more_details() {
        assert_eq!(
            SessionContext::detect_followup_type("can you give me more details"),
            Some(FollowUpType::MoreDetails)
        );

        assert_eq!(
            SessionContext::detect_followup_type("tell me more about that"),
            Some(FollowUpType::MoreDetails)
        );
    }

    #[test]
    fn test_followup_type_detection_just_commands() {
        assert_eq!(
            SessionContext::detect_followup_type("just show me the commands to fix it"),
            Some(FollowUpType::JustCommands)
        );

        assert_eq!(
            SessionContext::detect_followup_type("what should i run"),
            Some(FollowUpType::JustCommands)
        );
    }

    #[test]
    fn test_preference_inference_cli_bias() {
        let mut ctx = SessionContext::new();

        // Need at least 3 queries for bias to activate
        ctx.update_from_query(
            QueryIntent::Generic,
            None,
            "how do i use the terminal",
        );
        ctx.update_from_query(
            QueryIntent::Generic,
            None,
            "show me the bash command",
        );
        ctx.update_from_query(
            QueryIntent::Generic,
            None,
            "use pacman to install",
        );

        assert_eq!(ctx.preferences.interface_bias, InterfaceBias::PreferCli);
    }

    #[test]
    fn test_preference_inference_gui_bias() {
        let mut ctx = SessionContext::new();

        ctx.update_from_query(
            QueryIntent::Generic,
            None,
            "open the kde settings app",
        );
        ctx.update_from_query(
            QueryIntent::Generic,
            None,
            "use the gui application",
        );
        ctx.update_from_query(
            QueryIntent::Generic,
            None,
            "which desktop environment",
        );

        assert_eq!(ctx.preferences.interface_bias, InterfaceBias::PreferGui);
    }

    #[test]
    fn test_preference_inference_detail_level() {
        let mut ctx = SessionContext::new();

        // After 2+ "more detail" requests, should prefer verbose
        ctx.update_from_query(
            QueryIntent::Generic,
            None,
            "can you give me more detail",
        );
        ctx.update_from_query(
            QueryIntent::Generic,
            None,
            "explain that more",
        );

        assert_eq!(ctx.preferences.detail_level, DetailPreference::Verbose);
    }

    #[test]
    fn test_preference_inference_brief() {
        let mut ctx = SessionContext::new();

        ctx.update_from_query(
            QueryIntent::Generic,
            None,
            "just the commands please",
        );
        ctx.update_from_query(
            QueryIntent::Generic,
            None,
            "give me the brief version",
        );

        assert_eq!(ctx.preferences.detail_level, DetailPreference::Short);
    }

    #[test]
    fn test_context_not_stale_immediately() {
        let ctx = SessionContext::new();
        assert!(!ctx.is_stale());
    }

    #[test]
    fn test_get_followup_context_with_data() {
        let mut ctx = SessionContext::new();

        ctx.update_from_query(
            QueryIntent::WikiReasoning {
                topic: "Networking".to_string(),
            },
            Some(LastAnswerSummary::WikiAdvice {
                topic: "Networking".to_string(),
                advice_key: "net_001".to_string(),
                commands: vec!["nmcli device show".to_string()],
                summary: "Check network status".to_string(),
            }),
            "wifi issues",
        );

        let result = ctx.get_followup_context();
        assert!(result.is_some());

        let (intent, _answer) = result.unwrap();
        match intent {
            QueryIntent::WikiReasoning { topic } => {
                assert_eq!(topic, "Networking");
            }
            _ => panic!("Expected WikiReasoning intent"),
        }
    }

    #[test]
    fn test_get_followup_context_no_data() {
        let ctx = SessionContext::new();
        assert!(ctx.get_followup_context().is_none());
    }

    #[test]
    fn test_followup_commands_extraction() {
        let mut ctx = SessionContext::new();

        // Set up WikiAdvice with commands
        ctx.update_from_query(
            QueryIntent::WikiReasoning {
                topic: "Networking".to_string(),
            },
            Some(LastAnswerSummary::WikiAdvice {
                topic: "Networking".to_string(),
                advice_key: "net_001".to_string(),
                commands: vec![
                    "nmcli device show".to_string(),
                    "ping 8.8.8.8".to_string(),
                ],
                summary: "Check network status".to_string(),
            }),
            "wifi issues",
        );

        // Verify context stored commands
        if let Some((_, LastAnswerSummary::WikiAdvice { commands, .. })) = ctx.get_followup_context() {
            assert_eq!(commands.len(), 2);
            assert!(commands[0].contains("nmcli"));
        } else {
            panic!("Expected WikiAdvice in context");
        }
    }

    #[test]
    fn test_followup_no_commands_case() {
        let mut ctx = SessionContext::new();

        // Set up WikiAdvice with NO commands (informational only)
        ctx.update_from_query(
            QueryIntent::WikiReasoning {
                topic: "General".to_string(),
            },
            Some(LastAnswerSummary::WikiAdvice {
                topic: "General".to_string(),
                advice_key: "gen_001".to_string(),
                commands: vec![], // Empty commands
                summary: "This is informational advice".to_string(),
            }),
            "general question",
        );

        // Verify empty commands
        if let Some((_, LastAnswerSummary::WikiAdvice { commands, .. })) = ctx.get_followup_context() {
            assert!(commands.is_empty());
        } else {
            panic!("Expected WikiAdvice in context");
        }
    }

    #[test]
    fn test_session_persistence() {
        // Create and save a session
        let mut ctx1 = SessionContext::new();
        ctx1.update_from_query(
            QueryIntent::Status,
            Some(LastAnswerSummary::Status {
                summary: "System OK".to_string(),
                critical_issues: 0,
                warnings: 1,
            }),
            "how is my system",
        );

        // Save is automatic in update_from_query
        // Now load in a new context
        let ctx2 = SessionContext::load_or_new();

        // Should have loaded the saved context
        assert!(ctx2.last_intent.is_some());
        assert!(ctx2.last_answer.is_some());
    }

    #[test]
    fn test_proactive_commentary_conditions() {
        let ctx = SessionContext::new();

        // Test different detail levels
        assert_eq!(ctx.preferences.detail_level, DetailPreference::Normal);

        // After brief requests, should switch to Short
        let mut ctx2 = SessionContext::new();
        ctx2.update_from_query(QueryIntent::Generic, None, "just the commands");
        ctx2.update_from_query(QueryIntent::Generic, None, "brief please");

        assert_eq!(ctx2.preferences.detail_level, DetailPreference::Short);
    }
}
