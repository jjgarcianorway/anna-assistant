//! TUI State Management - Central state for Anna's terminal interface
//!
//! All TUI rendering comes from this state struct. No loose println! after init.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Central TUI state - everything rendered on screen comes from this
#[derive(Debug, Clone)]
pub struct AnnaTuiState {
    /// User's language preference (ISO 639-1 code: "en", "es", "fr", etc.)
    pub language: LanguageCode,

    /// Conversation history
    pub conversation: Vec<ChatItem>,

    /// Current input buffer
    pub input: String,

    /// Input cursor position
    pub cursor_pos: usize,

    /// Scroll offset for conversation pane
    pub scroll_offset: usize,

    /// System telemetry for left panel
    pub system_panel: SystemSummary,

    /// LLM status for left panel
    pub llm_panel: LlmStatus,

    /// Whether telemetry is working
    pub telemetry_ok: bool,

    /// Last LLM reply (for debugging)
    pub last_llm_reply: Option<String>,

    /// Input history for ↑/↓ navigation
    pub input_history: Vec<String>,

    /// Current position in history
    pub history_index: Option<usize>,

    /// Whether help overlay is shown
    pub show_help: bool,

    /// Beta.91: Whether Anna is currently thinking (processing with LLM)
    pub is_thinking: bool,

    /// Beta.91: Animation frame for thinking indicator (0-3 for spinner)
    pub thinking_frame: usize,

    /// Beta.147: Last action plan (for execution)
    pub last_action_plan: Option<Box<anna_common::action_plan_v3::ActionPlan>>,

    /// Beta.218: Brain diagnostic analysis (top 3 insights)
    pub brain_insights: Vec<anna_common::ipc::DiagnosticInsightData>,

    /// Beta.218: Brain analysis timestamp
    pub brain_timestamp: Option<String>,

    /// Beta.218: Whether brain data is available
    pub brain_available: bool,
}

impl Default for AnnaTuiState {
    fn default() -> Self {
        Self {
            language: LanguageCode::English,
            conversation: Vec::new(),
            input: String::new(),
            cursor_pos: 0,
            scroll_offset: 0,
            system_panel: SystemSummary::default(),
            llm_panel: LlmStatus::default(),
            telemetry_ok: false,
            last_llm_reply: None,
            input_history: Vec::new(),
            history_index: None,
            show_help: false,
            is_thinking: false,
            thinking_frame: 0,
            last_action_plan: None,
            brain_insights: Vec::new(),
            brain_timestamp: None,
            brain_available: false,
        }
    }
}

impl AnnaTuiState {
    /// Load persisted state from disk
    pub async fn load() -> anyhow::Result<Self> {
        let state_path = Self::state_file_path();

        if state_path.exists() {
            let contents = tokio::fs::read_to_string(&state_path).await?;
            let persisted: PersistedState = serde_json::from_str(&contents)?;

            let mut state = Self::default();
            state.language = persisted.language;
            state.input_history = persisted.input_history;
            Ok(state)
        } else {
            Ok(Self::default())
        }
    }

    /// Save state to disk
    pub async fn save(&self) -> anyhow::Result<()> {
        let state_path = Self::state_file_path();

        // Create parent directory if needed
        if let Some(parent) = state_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        let persisted = PersistedState {
            language: self.language.clone(),
            input_history: self.input_history.clone(),
        };

        let contents = serde_json::to_string_pretty(&persisted)?;
        tokio::fs::write(&state_path, contents).await?;
        Ok(())
    }

    /// Get state file path
    fn state_file_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join("anna")
            .join("tui_state.json")
    }

    /// Add user message to conversation
    pub fn add_user_message(&mut self, message: String) {
        self.conversation.push(ChatItem::User(message.clone()));

        // Add to history if not empty
        if !message.trim().is_empty() {
            self.input_history.push(message);
            // Limit history to 100 items
            if self.input_history.len() > 100 {
                self.input_history.remove(0);
            }
        }

        self.history_index = None;

        // Auto-scroll to bottom
        self.scroll_to_bottom();
    }

    /// Add Anna's reply to conversation
    pub fn add_anna_reply(&mut self, reply: String) {
        self.conversation.push(ChatItem::Anna(reply.clone()));
        self.last_llm_reply = Some(reply);

        // Auto-scroll to bottom
        self.scroll_to_bottom();
    }

    /// Beta.147: Add structured action plan to conversation
    pub fn add_action_plan(&mut self, plan: anna_common::action_plan_v3::ActionPlan) {
        // Store for potential execution
        self.last_action_plan = Some(Box::new(plan.clone()));

        self.conversation.push(ChatItem::ActionPlan(Box::new(plan)));
        self.is_thinking = false;

        // Auto-scroll to bottom
        self.scroll_to_bottom();
    }

    /// Beta.234: Update brain diagnostics from background task
    pub fn update_brain_diagnostics(&mut self, analysis: anna_common::ipc::BrainAnalysisData) {
        // Take top 3 insights (sorted by severity: critical > warning > info)
        let mut insights = analysis.insights;
        insights.sort_by(|a, b| {
            let a_priority = severity_priority(&a.severity);
            let b_priority = severity_priority(&b.severity);
            b_priority.cmp(&a_priority) // Descending order (higher priority first)
        });
        insights.truncate(3);

        self.brain_insights = insights;
        self.brain_timestamp = Some(analysis.timestamp);
        self.brain_available = true;
    }

    /// Scroll to bottom of conversation
    pub fn scroll_to_bottom(&mut self) {
        // Set scroll offset to a large number - rendering will clamp it
        self.scroll_offset = usize::MAX;
    }

    /// Beta.115: Append chunk to last Anna message (for streaming)
    pub fn append_to_last_anna_reply(&mut self, chunk: String) {
        if let Some(ChatItem::Anna(last_reply)) = self.conversation.last_mut() {
            last_reply.push_str(&chunk);
        } else {
            // No Anna reply exists yet, create one
            self.conversation.push(ChatItem::Anna(chunk));
        }
        // Auto-scroll as chunks arrive
        self.scroll_to_bottom();
    }

    /// Add system message to conversation
    pub fn add_system_message(&mut self, message: String) {
        self.conversation.push(ChatItem::System(message));
    }

    /// Clear conversation history
    pub fn clear_conversation(&mut self) {
        self.conversation.clear();
        self.scroll_offset = 0;
    }

    /// Navigate history up
    pub fn history_up(&mut self) {
        if self.input_history.is_empty() {
            return;
        }

        match self.history_index {
            None => {
                // Start from most recent
                self.history_index = Some(self.input_history.len() - 1);
                self.input = self.input_history[self.input_history.len() - 1].clone();
            }
            Some(idx) if idx > 0 => {
                self.history_index = Some(idx - 1);
                self.input = self.input_history[idx - 1].clone();
            }
            _ => {}
        }
        self.cursor_pos = self.input.len();
    }

    /// Navigate history down
    pub fn history_down(&mut self) {
        match self.history_index {
            Some(idx) if idx < self.input_history.len() - 1 => {
                self.history_index = Some(idx + 1);
                self.input = self.input_history[idx + 1].clone();
            }
            Some(_) => {
                // At bottom of history, clear input
                self.history_index = None;
                self.input.clear();
            }
            None => {}
        }
        self.cursor_pos = self.input.len();
    }
}

/// Language code (ISO 639-1)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LanguageCode {
    #[serde(rename = "en")]
    English,
    #[serde(rename = "es")]
    Spanish,
    #[serde(rename = "fr")]
    French,
    #[serde(rename = "de")]
    German,
    #[serde(rename = "it")]
    Italian,
    #[serde(rename = "pt")]
    Portuguese,
}

impl LanguageCode {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::English => "en",
            Self::Spanish => "es",
            Self::French => "fr",
            Self::German => "de",
            Self::Italian => "it",
            Self::Portuguese => "pt",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::English => "English",
            Self::Spanish => "Español",
            Self::French => "Français",
            Self::German => "Deutsch",
            Self::Italian => "Italiano",
            Self::Portuguese => "Português",
        }
    }
}

/// Chat item in conversation
#[derive(Debug, Clone)]
pub enum ChatItem {
    User(String),
    Anna(String),
    System(String),
    /// Beta.147: Structured action plan display
    ActionPlan(Box<anna_common::action_plan_v3::ActionPlan>),
}

/// System summary for left panel
#[derive(Debug, Clone, Default)]
pub struct SystemSummary {
    pub cpu_model: String,
    pub cpu_load_1min: f64,
    pub cpu_load_5min: f64,
    pub cpu_load_15min: f64,
    pub ram_total_gb: f64,
    pub ram_used_gb: f64,
    pub gpu_name: Option<String>,
    pub gpu_vram_gb: Option<f64>,
    pub desktop_env: Option<String>,
    pub window_manager: Option<String>,
    pub disk_free_gb: f64,
    pub anna_version: String,
    pub uptime_seconds: u64,
    pub outdated_packages: usize,
    pub recent_errors: Vec<String>,
    pub hostname: String, // Version 150: Real hostname from telemetry_truth
}

/// LLM status for left panel
#[derive(Debug, Clone, Default)]
pub struct LlmStatus {
    pub model_name: String,
    pub model_size: String,
    pub mode: String,
    pub available: bool,
}

/// Persisted state (saved to disk)
#[derive(Debug, Clone, Serialize, Deserialize)]
struct PersistedState {
    language: LanguageCode,
    input_history: Vec<String>,
}

/// Beta.234: Get severity priority for sorting (higher = more important)
fn severity_priority(severity: &str) -> u8 {
    match severity.to_lowercase().as_str() {
        "critical" => 3,
        "warning" => 2,
        "info" => 1,
        _ => 0,
    }
}
