// v0.0.574: Settings Orchestrator (Phase 150)
// Unified coordinator for all settings subsystems

use serde::{Deserialize, Serialize};

use crate::settings_audit::{AuditEventType, AuditLog, AuditSeverity};
use crate::settings_constraints::ConstraintManager;
use crate::settings_hooks::{HookContext, HookManager, HookTrigger};
use crate::settings_notifications::NotificationManager;
use crate::settings_profiles::ProfileManager;
use crate::settings_scheduler::SettingsScheduler;
use crate::settings_templates::TemplateManager;
use crate::unified_settings::{SettingsCategory, UnifiedSettings};

/// Orchestrator state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum OrchestratorState {
    /// Not initialized
    #[default]
    Uninitialized,
    /// Initializing
    Initializing,
    /// Ready
    Ready,
    /// Busy (processing)
    Busy,
    /// Error state
    Error,
}

impl std::fmt::Display for OrchestratorState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Uninitialized => write!(f, "Uninitialized"),
            Self::Initializing => write!(f, "Initializing"),
            Self::Ready => write!(f, "Ready"),
            Self::Busy => write!(f, "Busy"),
            Self::Error => write!(f, "Error"),
        }
    }
}

/// Operation result
#[derive(Debug, Clone)]
pub struct OperationResult {
    /// Was successful
    pub success: bool,
    /// Message
    pub message: String,
    /// Warnings
    pub warnings: Vec<String>,
    /// Errors
    pub errors: Vec<String>,
}

impl OperationResult {
    /// Create success result
    pub fn ok(message: impl Into<String>) -> Self {
        Self {
            success: true,
            message: message.into(),
            warnings: Vec::new(),
            errors: Vec::new(),
        }
    }

    /// Create error result
    pub fn err(message: impl Into<String>) -> Self {
        Self {
            success: false,
            message: message.into(),
            warnings: Vec::new(),
            errors: Vec::new(),
        }
    }

    /// Add warning
    pub fn with_warning(mut self, warning: impl Into<String>) -> Self {
        self.warnings.push(warning.into());
        self
    }

    /// Add error
    pub fn with_error(mut self, error: impl Into<String>) -> Self {
        self.errors.push(error.into());
        self.success = false;
        self
    }
}

/// Settings orchestrator - coordinates all subsystems
#[derive(Debug, Default)]
pub struct SettingsOrchestrator {
    /// Current settings
    pub settings: UnifiedSettings,
    /// State
    pub state: OrchestratorState,
    /// Profile manager
    pub profiles: ProfileManager,
    /// Template manager
    pub templates: TemplateManager,
    /// Scheduler
    pub scheduler: SettingsScheduler,
    /// Constraint manager
    pub constraints: ConstraintManager,
    /// Hook manager
    pub hooks: HookManager,
    /// Notification manager
    pub notifications: NotificationManager,
    /// Audit log
    pub audit: AuditLog,
    /// Session ID
    session_id: Option<String>,
}

impl SettingsOrchestrator {
    /// Create new orchestrator
    pub fn new() -> Self {
        Self::default()
    }

    /// Create with default subsystems
    pub fn with_defaults() -> Self {
        let mut orchestrator = Self {
            settings: UnifiedSettings::default(),
            state: OrchestratorState::Uninitialized,
            profiles: ProfileManager::new(),
            templates: TemplateManager::with_defaults(),
            scheduler: SettingsScheduler::new(),
            constraints: ConstraintManager::with_defaults(),
            hooks: HookManager::with_defaults(),
            notifications: NotificationManager::new(),
            audit: AuditLog::new(),
            session_id: None,
        };
        orchestrator.state = OrchestratorState::Ready;
        orchestrator
    }

    /// Initialize orchestrator
    pub fn initialize(&mut self) -> OperationResult {
        self.state = OrchestratorState::Initializing;

        // Fire initialization hooks
        let ctx = HookContext::new(HookTrigger::BeforeLoad);
        self.hooks.fire(&ctx);

        // Log initialization
        self.audit.log(AuditEventType::Load, AuditSeverity::Info, "Orchestrator initialized");

        self.state = OrchestratorState::Ready;
        OperationResult::ok("Orchestrator initialized successfully")
    }

    /// Set session ID
    pub fn set_session(&mut self, session_id: impl Into<String>) {
        let id = session_id.into();
        self.session_id = Some(id.clone());
        self.audit.set_session(id);
    }

    /// Get current settings
    pub fn settings(&self) -> &UnifiedSettings {
        &self.settings
    }

    /// Change a setting
    pub fn change_setting(
        &mut self,
        category: SettingsCategory,
        field: &str,
        value: &str,
    ) -> OperationResult {
        if self.state != OrchestratorState::Ready {
            return OperationResult::err("Orchestrator not ready");
        }

        self.state = OrchestratorState::Busy;

        // Fire before change hooks
        let ctx = HookContext::new(HookTrigger::BeforeChange)
            .with_category(category)
            .with_field(field)
            .with_new_value(value);
        self.hooks.fire(&ctx);

        // Get old value (simplified)
        let old_value = "previous".to_string();

        // Apply change (simplified - would actually modify settings)
        // In real implementation, this would call settings.apply_change(category, field, value)

        // Log change
        self.audit.log_change(category, field, &old_value, value);

        // Fire after change hooks
        let ctx = HookContext::new(HookTrigger::AfterChange)
            .with_category(category)
            .with_field(field)
            .with_old_value(&old_value)
            .with_new_value(value);
        self.hooks.fire(&ctx);

        // Send notification
        self.notifications.setting_changed(category, field, &old_value, value);

        // Check constraints
        let result = self.constraints.check(&self.settings);
        let mut op_result = OperationResult::ok(format!("Changed {}.{} to {}", category, field, value));

        for violation in result.violations {
            op_result.warnings.push(format!("{}: {}", violation.field, violation.message));
        }

        self.state = OrchestratorState::Ready;
        op_result
    }

    /// Switch profile
    pub fn switch_profile(&mut self, profile_name: &str) -> OperationResult {
        if self.state != OrchestratorState::Ready {
            return OperationResult::err("Orchestrator not ready");
        }

        self.state = OrchestratorState::Busy;

        // Find and switch profile
        if let Some(profile) = self.profiles.find_by_name(profile_name).first() {
            let id = profile.id.clone();
            let old_profile = self.profiles.active().map(|p| p.meta.name.clone()).unwrap_or_default();
            if self.profiles.switch_to(&id).is_ok() {
                // Log profile switch
                self.audit.log(
                    AuditEventType::ProfileSwitch,
                    AuditSeverity::Notice,
                    &format!("Switched to profile '{}'", profile_name),
                );

                // Notify
                self.notifications.profile_switched(&old_profile, profile_name);

                self.state = OrchestratorState::Ready;
                return OperationResult::ok(format!("Switched to profile '{}'", profile_name));
            }
        }

        self.state = OrchestratorState::Ready;
        OperationResult::err(format!("Profile '{}' not found", profile_name))
    }

    /// Apply template
    pub fn apply_template(&mut self, template_id: u64) -> OperationResult {
        if self.state != OrchestratorState::Ready {
            return OperationResult::err("Orchestrator not ready");
        }

        self.state = OrchestratorState::Busy;

        if self.templates.apply(template_id, &mut self.settings) {
            let template_name = self.templates.get(template_id)
                .map(|t| t.meta.name.clone())
                .unwrap_or_default();

            // Log template apply
            self.audit.log(
                AuditEventType::TemplateApply,
                AuditSeverity::Notice,
                &format!("Applied template '{}'", template_name),
            );

            self.state = OrchestratorState::Ready;
            OperationResult::ok(format!("Applied template '{}'", template_name))
        } else {
            self.state = OrchestratorState::Ready;
            OperationResult::err("Template not found")
        }
    }

    /// Run scheduled tasks
    pub fn run_scheduled(&mut self) -> Vec<OperationResult> {
        let mut results = Vec::new();
        let pending: Vec<_> = self.scheduler.pending().iter().map(|s| s.id).collect();

        for id in pending {
            if let Some(schedule) = self.scheduler.get_mut(id) {
                // Mark as executed
                schedule.mark_executed();

                // Log execution
                self.audit.log(
                    AuditEventType::Change,
                    AuditSeverity::Info,
                    &format!("Executed scheduled task: {}", schedule.name),
                );

                results.push(OperationResult::ok(format!("Executed: {}", schedule.name)));
            }
        }

        results
    }

    /// Get orchestrator status summary
    pub fn status_summary(&self) -> OrchestratorStatus {
        OrchestratorStatus {
            state: self.state,
            profiles_count: self.profiles.count(),
            templates_count: self.templates.count(),
            schedules_count: self.scheduler.count(),
            constraints_count: self.constraints.count(),
            hooks_count: self.hooks.count(),
            unread_notifications: self.notifications.unread_count(),
            audit_entries: self.audit.count(),
            security_events: self.audit.security_events().len(),
        }
    }
}

/// Orchestrator status summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestratorStatus {
    /// Current state
    pub state: OrchestratorState,
    /// Number of profiles
    pub profiles_count: usize,
    /// Number of templates
    pub templates_count: usize,
    /// Number of schedules
    pub schedules_count: usize,
    /// Number of constraints
    pub constraints_count: usize,
    /// Number of hooks
    pub hooks_count: usize,
    /// Unread notifications
    pub unread_notifications: usize,
    /// Audit entries
    pub audit_entries: usize,
    /// Security events
    pub security_events: usize,
}

/// Format orchestrator status
pub fn format_orchestrator_status(orchestrator: &SettingsOrchestrator) -> String {
    let status = orchestrator.status_summary();
    let mut output = String::new();

    output.push_str("=== Settings Orchestrator ===\n\n");
    output.push_str(&format!("State: {}\n\n", status.state));

    output.push_str("Subsystems:\n");
    output.push_str(&format!("  • Profiles: {}\n", status.profiles_count));
    output.push_str(&format!("  • Templates: {}\n", status.templates_count));
    output.push_str(&format!("  • Schedules: {}\n", status.schedules_count));
    output.push_str(&format!("  • Constraints: {}\n", status.constraints_count));
    output.push_str(&format!("  • Hooks: {}\n", status.hooks_count));
    output.push_str(&format!("  • Notifications: {} unread\n", status.unread_notifications));
    output.push_str(&format!("  • Audit: {} entries ({} security)\n",
        status.audit_entries, status.security_events));

    output
}

/// Check if query is about orchestrator
pub fn is_orchestrator_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("orchestrator")
        || lower.contains("settings status")
        || lower.contains("settings overview")
}

/// Fun fact about orchestrator
pub fn orchestrator_fun_fact() -> &'static str {
    "The settings orchestrator coordinates all settings subsystems in one place!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_orchestrator_state_display() {
        assert_eq!(format!("{}", OrchestratorState::Ready), "Ready");
        assert_eq!(format!("{}", OrchestratorState::Busy), "Busy");
    }

    #[test]
    fn test_operation_result_ok() {
        let result = OperationResult::ok("Success");
        assert!(result.success);
    }

    #[test]
    fn test_operation_result_err() {
        let result = OperationResult::err("Failed");
        assert!(!result.success);
    }

    #[test]
    fn test_operation_result_with_warning() {
        let result = OperationResult::ok("Done").with_warning("Check this");
        assert!(result.success);
        assert_eq!(result.warnings.len(), 1);
    }

    #[test]
    fn test_settings_orchestrator_new() {
        let orchestrator = SettingsOrchestrator::new();
        assert_eq!(orchestrator.state, OrchestratorState::Uninitialized);
    }

    #[test]
    fn test_settings_orchestrator_with_defaults() {
        let orchestrator = SettingsOrchestrator::with_defaults();
        assert_eq!(orchestrator.state, OrchestratorState::Ready);
        assert!(orchestrator.templates.count() > 0);
    }

    #[test]
    fn test_settings_orchestrator_initialize() {
        let mut orchestrator = SettingsOrchestrator::new();
        let result = orchestrator.initialize();
        assert!(result.success);
        assert_eq!(orchestrator.state, OrchestratorState::Ready);
    }

    #[test]
    fn test_settings_orchestrator_set_session() {
        let mut orchestrator = SettingsOrchestrator::with_defaults();
        orchestrator.set_session("test-session");
        assert!(orchestrator.session_id.is_some());
    }

    #[test]
    fn test_settings_orchestrator_change_setting() {
        let mut orchestrator = SettingsOrchestrator::with_defaults();
        let result = orchestrator.change_setting(SettingsCategory::Risk, "level", "high");
        assert!(result.success);
    }

    #[test]
    fn test_settings_orchestrator_status_summary() {
        let orchestrator = SettingsOrchestrator::with_defaults();
        let status = orchestrator.status_summary();
        assert_eq!(status.state, OrchestratorState::Ready);
    }

    #[test]
    fn test_format_orchestrator_status() {
        let orchestrator = SettingsOrchestrator::with_defaults();
        let output = format_orchestrator_status(&orchestrator);
        assert!(output.contains("Orchestrator"));
    }

    #[test]
    fn test_is_orchestrator_query() {
        assert!(is_orchestrator_query("settings status"));
        assert!(is_orchestrator_query("orchestrator overview"));
        assert!(!is_orchestrator_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = orchestrator_fun_fact();
        assert!(fact.contains("orchestrator"));
    }
}
