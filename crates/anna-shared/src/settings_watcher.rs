// v0.0.560: Settings Watcher (Phase 136)
// Watches for settings file changes and emits events

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, SystemTime};

use crate::settings_persistence::SettingsPersistence;
use crate::unified_settings::{SettingsCategory, UnifiedSettings};

/// Settings change event type
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SettingsEventType {
    /// Settings file was created
    Created,
    /// Settings file was modified
    Modified,
    /// Settings file was deleted
    Deleted,
    /// Settings were reloaded
    Reloaded,
    /// Specific category changed
    CategoryChanged(SettingsCategory),
    /// Validation warning detected
    ValidationWarning,
}

impl std::fmt::Display for SettingsEventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Created => write!(f, "Created"),
            Self::Modified => write!(f, "Modified"),
            Self::Deleted => write!(f, "Deleted"),
            Self::Reloaded => write!(f, "Reloaded"),
            Self::CategoryChanged(cat) => write!(f, "{} Changed", cat),
            Self::ValidationWarning => write!(f, "Validation Warning"),
        }
    }
}

/// Settings change event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingsEvent {
    /// Event type
    pub event_type: SettingsEventType,
    /// When the event occurred
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Path to settings file
    pub path: Option<PathBuf>,
    /// Optional description
    pub description: Option<String>,
}

impl SettingsEvent {
    /// Create a new settings event
    pub fn new(event_type: SettingsEventType) -> Self {
        Self {
            event_type,
            timestamp: chrono::Utc::now(),
            path: SettingsPersistence::settings_path(),
            description: None,
        }
    }

    /// Add a description
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }
}

/// Watcher configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatcherConfig {
    /// Check interval in milliseconds
    pub check_interval_ms: u64,
    /// Auto-reload on change
    pub auto_reload: bool,
    /// Validate on reload
    pub validate_on_reload: bool,
    /// Maximum events to keep in history
    pub max_history: usize,
}

impl Default for WatcherConfig {
    fn default() -> Self {
        Self {
            check_interval_ms: 1000,
            auto_reload: true,
            validate_on_reload: true,
            max_history: 100,
        }
    }
}

/// Settings file state
#[derive(Debug, Clone)]
struct FileState {
    /// Last modification time
    modified: Option<SystemTime>,
    /// File exists
    exists: bool,
}

impl FileState {
    fn new() -> Self {
        Self {
            modified: None,
            exists: false,
        }
    }

    fn from_path(path: &PathBuf) -> Self {
        if path.exists() {
            let modified = std::fs::metadata(path)
                .ok()
                .and_then(|m| m.modified().ok());
            Self {
                modified,
                exists: true,
            }
        } else {
            Self {
                modified: None,
                exists: false,
            }
        }
    }

    fn changed(&self, other: &FileState) -> Option<SettingsEventType> {
        match (self.exists, other.exists) {
            (false, true) => Some(SettingsEventType::Created),
            (true, false) => Some(SettingsEventType::Deleted),
            (true, true) if self.modified != other.modified => Some(SettingsEventType::Modified),
            _ => None,
        }
    }
}

/// Settings watcher
#[derive(Debug)]
pub struct SettingsWatcher {
    /// Watcher configuration
    pub config: WatcherConfig,
    /// Event history
    events: Vec<SettingsEvent>,
    /// Last known file state
    last_state: FileState,
    /// Running flag
    running: Arc<AtomicBool>,
}

impl Default for SettingsWatcher {
    fn default() -> Self {
        Self::new()
    }
}

impl SettingsWatcher {
    /// Create new settings watcher
    pub fn new() -> Self {
        Self {
            config: WatcherConfig::default(),
            events: vec![],
            last_state: FileState::new(),
            running: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Configure the watcher
    pub fn with_config(mut self, config: WatcherConfig) -> Self {
        self.config = config;
        self
    }

    /// Set check interval
    pub fn check_interval(mut self, ms: u64) -> Self {
        self.config.check_interval_ms = ms;
        self
    }

    /// Disable auto-reload
    pub fn no_auto_reload(mut self) -> Self {
        self.config.auto_reload = false;
        self
    }

    /// Check for changes (single poll)
    pub fn check(&mut self) -> Option<SettingsEvent> {
        let path = SettingsPersistence::settings_path()?;
        let current_state = FileState::from_path(&path);

        if let Some(event_type) = self.last_state.changed(&current_state) {
            let event = SettingsEvent::new(event_type);
            self.record_event(event.clone());
            self.last_state = current_state;
            return Some(event);
        }

        self.last_state = current_state;
        None
    }

    /// Record an event
    fn record_event(&mut self, event: SettingsEvent) {
        self.events.push(event);
        if self.events.len() > self.config.max_history {
            self.events.remove(0);
        }
    }

    /// Manually emit an event
    pub fn emit(&mut self, event_type: SettingsEventType) {
        let event = SettingsEvent::new(event_type);
        self.record_event(event);
    }

    /// Emit a category change event
    pub fn emit_category_change(&mut self, category: SettingsCategory) {
        let event = SettingsEvent::new(SettingsEventType::CategoryChanged(category));
        self.record_event(event);
    }

    /// Get event history
    pub fn history(&self) -> &[SettingsEvent] {
        &self.events
    }

    /// Get recent events (last n)
    pub fn recent_events(&self, count: usize) -> Vec<&SettingsEvent> {
        self.events.iter().rev().take(count).collect()
    }

    /// Clear event history
    pub fn clear_history(&mut self) {
        self.events.clear();
    }

    /// Is the watcher running?
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    /// Start watching (marks as running)
    pub fn start(&self) {
        self.running.store(true, Ordering::SeqCst);
    }

    /// Stop watching
    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
    }

    /// Get running flag (for async loops)
    pub fn running_flag(&self) -> Arc<AtomicBool> {
        self.running.clone()
    }

    /// Poll interval as Duration
    pub fn poll_interval(&self) -> Duration {
        Duration::from_millis(self.config.check_interval_ms)
    }

    /// Event count
    pub fn event_count(&self) -> usize {
        self.events.len()
    }

    /// Has events?
    pub fn has_events(&self) -> bool {
        !self.events.is_empty()
    }
}

/// Callback type for settings changes
pub type SettingsCallback = Box<dyn Fn(&SettingsEvent, &UnifiedSettings) + Send + Sync>;

/// Settings change listener
pub struct SettingsListener {
    /// Registered callbacks
    callbacks: Vec<SettingsCallback>,
}

impl Default for SettingsListener {
    fn default() -> Self {
        Self::new()
    }
}

impl SettingsListener {
    /// Create new listener
    pub fn new() -> Self {
        Self { callbacks: vec![] }
    }

    /// Register a callback
    pub fn on_change(&mut self, callback: SettingsCallback) {
        self.callbacks.push(callback);
    }

    /// Notify all callbacks
    pub fn notify(&self, event: &SettingsEvent, settings: &UnifiedSettings) {
        for callback in &self.callbacks {
            callback(event, settings);
        }
    }

    /// Callback count
    pub fn callback_count(&self) -> usize {
        self.callbacks.len()
    }
}

/// Format watcher status for display
pub fn format_watcher_status(watcher: &SettingsWatcher) -> String {
    let mut output = String::new();

    output.push_str("=== Settings Watcher ===\n\n");
    output.push_str(&format!(
        "Status: {}\n",
        if watcher.is_running() {
            "Running"
        } else {
            "Stopped"
        }
    ));
    output.push_str(&format!(
        "Check interval: {}ms\n",
        watcher.config.check_interval_ms
    ));
    output.push_str(&format!("Auto-reload: {}\n", watcher.config.auto_reload));
    output.push_str(&format!("Event count: {}\n", watcher.event_count()));

    if let Some(event) = watcher.events.last() {
        output.push_str(&format!("\nLast event: {} at {}\n", event.event_type, event.timestamp));
    }

    output
}

/// Check if settings file has changed since timestamp
pub fn has_changed_since(since: chrono::DateTime<chrono::Utc>) -> bool {
    if let Some(path) = SettingsPersistence::settings_path() {
        if let Ok(metadata) = std::fs::metadata(&path) {
            if let Ok(modified) = metadata.modified() {
                let modified_dt: chrono::DateTime<chrono::Utc> = modified.into();
                return modified_dt > since;
            }
        }
    }
    false
}

/// Fun fact about settings watcher
pub fn settings_watcher_fun_fact() -> &'static str {
    "Anna watches your settings file and automatically reloads when you make changes!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_type_display() {
        assert_eq!(format!("{}", SettingsEventType::Created), "Created");
        assert_eq!(format!("{}", SettingsEventType::Modified), "Modified");
        assert_eq!(
            format!("{}", SettingsEventType::CategoryChanged(SettingsCategory::Privacy)),
            "Privacy Changed"
        );
    }

    #[test]
    fn test_settings_event_new() {
        let event = SettingsEvent::new(SettingsEventType::Modified);
        assert_eq!(event.event_type, SettingsEventType::Modified);
        assert!(event.description.is_none());
    }

    #[test]
    fn test_settings_event_with_description() {
        let event =
            SettingsEvent::new(SettingsEventType::Created).with_description("Initial creation");
        assert!(event.description.is_some());
    }

    #[test]
    fn test_watcher_config_default() {
        let config = WatcherConfig::default();
        assert_eq!(config.check_interval_ms, 1000);
        assert!(config.auto_reload);
    }

    #[test]
    fn test_watcher_new() {
        let watcher = SettingsWatcher::new();
        assert!(!watcher.is_running());
        assert!(!watcher.has_events());
    }

    #[test]
    fn test_watcher_start_stop() {
        let watcher = SettingsWatcher::new();
        assert!(!watcher.is_running());

        watcher.start();
        assert!(watcher.is_running());

        watcher.stop();
        assert!(!watcher.is_running());
    }

    #[test]
    fn test_watcher_emit() {
        let mut watcher = SettingsWatcher::new();
        watcher.emit(SettingsEventType::Modified);
        assert!(watcher.has_events());
        assert_eq!(watcher.event_count(), 1);
    }

    #[test]
    fn test_watcher_emit_category() {
        let mut watcher = SettingsWatcher::new();
        watcher.emit_category_change(SettingsCategory::Privacy);
        assert!(watcher.has_events());
    }

    #[test]
    fn test_watcher_clear_history() {
        let mut watcher = SettingsWatcher::new();
        watcher.emit(SettingsEventType::Modified);
        assert!(watcher.has_events());
        watcher.clear_history();
        assert!(!watcher.has_events());
    }

    #[test]
    fn test_watcher_recent_events() {
        let mut watcher = SettingsWatcher::new();
        watcher.emit(SettingsEventType::Created);
        watcher.emit(SettingsEventType::Modified);
        let recent = watcher.recent_events(1);
        assert_eq!(recent.len(), 1);
        assert_eq!(recent[0].event_type, SettingsEventType::Modified);
    }

    #[test]
    fn test_listener_new() {
        let listener = SettingsListener::new();
        assert_eq!(listener.callback_count(), 0);
    }

    #[test]
    fn test_format_watcher_status() {
        let watcher = SettingsWatcher::new();
        let status = format_watcher_status(&watcher);
        assert!(status.contains("Watcher"));
        assert!(status.contains("Stopped"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = settings_watcher_fun_fact();
        assert!(fact.contains("watch"));
    }
}
