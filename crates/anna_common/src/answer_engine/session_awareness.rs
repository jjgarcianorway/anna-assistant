//! Session Awareness - v0.25.0
//!
//! Tracks active desktop sessions and user context.
//! No hardcoded assumptions - all discovered from system state.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::app_awareness::{DesktopEnvironment, DisplayProtocol, WindowManagerType};
use super::protocol_v25::{RankedEntity, RankedEntityType, UsageEvent, UsageEventType};

// ============================================================================
// CONFIGURATION
// ============================================================================

/// Default session timeout (30 minutes of inactivity)
pub const DEFAULT_SESSION_TIMEOUT_SECS: i64 = 1800;

/// Default max sessions to track
pub const DEFAULT_MAX_SESSIONS: usize = 10;

/// Configuration for session awareness
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionConfig {
    /// Session timeout (seconds)
    pub session_timeout_secs: i64,
    /// Max sessions to keep in history
    pub max_sessions: usize,
    /// Track focused window
    pub track_focus: bool,
    /// Track workspace changes
    pub track_workspace: bool,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            session_timeout_secs: DEFAULT_SESSION_TIMEOUT_SECS,
            max_sessions: DEFAULT_MAX_SESSIONS,
            track_focus: true,
            track_workspace: true,
        }
    }
}

// ============================================================================
// SESSION STATE
// ============================================================================

/// Current session state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionState {
    /// Session ID
    pub id: String,
    /// Session start time
    pub started_at: i64,
    /// Last activity time
    pub last_activity: i64,
    /// Desktop environment detected
    pub desktop_env: Option<DesktopEnvironment>,
    /// Window manager detected
    pub window_manager: Option<WindowManagerType>,
    /// Display protocol
    pub display_protocol: Option<DisplayProtocol>,
    /// Currently focused app
    pub focused_app: Option<FocusedApp>,
    /// Current workspace
    pub current_workspace: Option<WorkspaceInfo>,
    /// Apps used in this session (by id)
    pub apps_used: HashMap<String, AppSessionUsage>,
    /// Is session active
    pub is_active: bool,
}

impl SessionState {
    /// Create new session
    pub fn new() -> Self {
        let now = chrono::Utc::now().timestamp();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            started_at: now,
            last_activity: now,
            desktop_env: None,
            window_manager: None,
            display_protocol: None,
            focused_app: None,
            current_workspace: None,
            apps_used: HashMap::new(),
            is_active: true,
        }
    }

    /// Update last activity
    pub fn touch(&mut self) {
        self.last_activity = chrono::Utc::now().timestamp();
    }

    /// Check if session is expired
    pub fn is_expired(&self, timeout_secs: i64) -> bool {
        let now = chrono::Utc::now().timestamp();
        now - self.last_activity > timeout_secs
    }

    /// Session duration in seconds
    pub fn duration(&self) -> i64 {
        self.last_activity - self.started_at
    }

    /// Record app focus
    pub fn set_focus(&mut self, app: FocusedApp) {
        // Update usage for previous focused app
        if let Some(prev) = &self.focused_app {
            if let Some(usage) = self.apps_used.get_mut(&prev.app_id) {
                usage.update_focus_time(prev.focused_at);
            }
        }

        // Record new app usage
        let usage = self.apps_used.entry(app.app_id.clone()).or_default();
        usage.focus_count += 1;
        usage.last_focused = Some(app.focused_at);

        self.focused_app = Some(app);
        self.touch();
    }

    /// Get most used apps in this session
    pub fn top_apps(&self, limit: usize) -> Vec<(&String, &AppSessionUsage)> {
        let mut sorted: Vec<_> = self.apps_used.iter().collect();
        sorted.sort_by(|a, b| b.1.focus_count.cmp(&a.1.focus_count));
        sorted.into_iter().take(limit).collect()
    }
}

impl Default for SessionState {
    fn default() -> Self {
        Self::new()
    }
}

/// Info about the currently focused app
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FocusedApp {
    /// App identifier
    pub app_id: String,
    /// Window title
    pub window_title: Option<String>,
    /// Window class
    pub window_class: Option<String>,
    /// Process ID
    pub pid: Option<u32>,
    /// When focus was gained
    pub focused_at: i64,
}

/// Usage info for an app within a session
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AppSessionUsage {
    /// Number of times focused
    pub focus_count: u32,
    /// Total focus time (seconds)
    pub total_focus_time: i64,
    /// Last time focused
    pub last_focused: Option<i64>,
}

impl AppSessionUsage {
    /// Update focus time when losing focus
    pub fn update_focus_time(&mut self, focus_started: i64) {
        let now = chrono::Utc::now().timestamp();
        self.total_focus_time += now - focus_started;
    }
}

/// Info about current workspace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceInfo {
    /// Workspace index/number
    pub index: u32,
    /// Workspace name (if available)
    pub name: Option<String>,
    /// Monitor/output name
    pub monitor: Option<String>,
}

// ============================================================================
// SESSION MANAGER
// ============================================================================

/// Manages session state and history
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SessionManager {
    /// Configuration
    pub config: SessionConfig,
    /// Current session
    pub current: Option<SessionState>,
    /// Recent sessions (for pattern learning)
    pub history: Vec<SessionSummary>,
}

impl SessionManager {
    /// Create with config
    pub fn with_config(config: SessionConfig) -> Self {
        Self {
            config,
            current: None,
            history: Vec::new(),
        }
    }

    /// Start or resume session
    pub fn ensure_session(&mut self) -> &mut SessionState {
        if self.current.is_none() || self.current.as_ref().map(|s| s.is_expired(self.config.session_timeout_secs)).unwrap_or(true) {
            self.start_new_session();
        }
        self.current.as_mut().unwrap()
    }

    /// Start a new session
    pub fn start_new_session(&mut self) {
        // Archive current session if exists
        if let Some(old) = self.current.take() {
            self.archive_session(old);
        }

        self.current = Some(SessionState::new());
    }

    /// Archive a session to history
    fn archive_session(&mut self, session: SessionState) {
        let summary = SessionSummary::from_session(&session);
        self.history.push(summary);

        // Trim history
        while self.history.len() > self.config.max_sessions {
            self.history.remove(0);
        }
    }

    /// Record activity
    pub fn record_activity(&mut self, event: &UsageEvent) {
        let session = self.ensure_session();
        session.touch();

        // Track app usage
        if matches!(event.event_type, UsageEventType::AppLaunch | UsageEventType::AppFocus) {
            let usage = session.apps_used.entry(event.entity.clone()).or_default();
            usage.focus_count += 1;
            usage.last_focused = Some(event.timestamp);
        }
    }

    /// Get current session ID
    pub fn current_session_id(&self) -> Option<&str> {
        self.current.as_ref().map(|s| s.id.as_str())
    }

    /// Check if session is active
    pub fn is_active(&self) -> bool {
        self.current.as_ref().map(|s| s.is_active).unwrap_or(false)
    }

    /// Get session context for relevance scoring
    pub fn get_session_context(&self) -> SessionContext {
        let Some(session) = &self.current else {
            return SessionContext::default();
        };

        SessionContext {
            session_id: session.id.clone(),
            duration_secs: session.duration(),
            apps_used: session.apps_used.keys().cloned().collect(),
            focused_app: session.focused_app.as_ref().map(|a| a.app_id.clone()),
            desktop_env: session.desktop_env.clone(),
            workspace: session.current_workspace.as_ref().map(|w| w.index),
        }
    }
}

/// Summary of a completed session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSummary {
    /// Session ID
    pub id: String,
    /// Start time
    pub started_at: i64,
    /// End time
    pub ended_at: i64,
    /// Duration in seconds
    pub duration_secs: i64,
    /// Apps used (by focus count)
    pub top_apps: Vec<(String, u32)>,
    /// Desktop environment
    pub desktop_env: Option<DesktopEnvironment>,
}

impl SessionSummary {
    /// Create from session state
    pub fn from_session(session: &SessionState) -> Self {
        let mut top_apps: Vec<_> = session
            .apps_used
            .iter()
            .map(|(id, u)| (id.clone(), u.focus_count))
            .collect();
        top_apps.sort_by(|a, b| b.1.cmp(&a.1));
        top_apps.truncate(10);

        Self {
            id: session.id.clone(),
            started_at: session.started_at,
            ended_at: session.last_activity,
            duration_secs: session.duration(),
            top_apps,
            desktop_env: session.desktop_env.clone(),
        }
    }
}

/// Context from current session for relevance
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SessionContext {
    /// Current session ID
    pub session_id: String,
    /// Session duration so far
    pub duration_secs: i64,
    /// Apps used in session
    pub apps_used: Vec<String>,
    /// Currently focused app
    pub focused_app: Option<String>,
    /// Desktop environment
    pub desktop_env: Option<DesktopEnvironment>,
    /// Current workspace index
    pub workspace: Option<u32>,
}

impl SessionContext {
    /// Check if entity was used in current session
    pub fn was_used(&self, entity_id: &str) -> bool {
        self.apps_used.iter().any(|a| a == entity_id)
    }

    /// Check if entity is currently focused
    pub fn is_focused(&self, entity_id: &str) -> bool {
        self.focused_app.as_ref().map(|a| a == entity_id).unwrap_or(false)
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_creation() {
        let session = SessionState::new();
        assert!(session.is_active);
        assert!(!session.id.is_empty());
    }

    #[test]
    fn test_session_focus_tracking() {
        let mut session = SessionState::new();

        let app = FocusedApp {
            app_id: "firefox".to_string(),
            window_title: Some("Mozilla Firefox".to_string()),
            window_class: Some("firefox".to_string()),
            pid: Some(12345),
            focused_at: chrono::Utc::now().timestamp(),
        };

        session.set_focus(app);

        assert!(session.focused_app.is_some());
        assert!(session.apps_used.contains_key("firefox"));
    }

    #[test]
    fn test_session_manager() {
        let mut manager = SessionManager::default();

        let event = UsageEvent::new(UsageEventType::AppLaunch, "firefox".to_string());
        manager.record_activity(&event);

        assert!(manager.current.is_some());
        assert!(manager.current_session_id().is_some());
    }

    #[test]
    fn test_session_context() {
        let mut manager = SessionManager::default();

        let event = UsageEvent::new(UsageEventType::AppLaunch, "nvim".to_string());
        manager.record_activity(&event);

        let ctx = manager.get_session_context();
        assert!(ctx.was_used("nvim"));
        assert!(!ctx.was_used("firefox"));
    }

    #[test]
    fn test_session_expiry() {
        let mut session = SessionState::new();
        session.last_activity = chrono::Utc::now().timestamp() - 7200; // 2 hours ago

        assert!(session.is_expired(1800)); // 30 min timeout
    }
}
