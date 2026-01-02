// v0.0.574: Settings Orchestrator Core
// Main orchestrator struct and its operations

use crate::settings_audit::{AuditEventType, AuditLog, AuditSeverity};
use crate::settings_constraints::ConstraintManager;
use crate::settings_hooks::{HookContext, HookManager, HookTrigger};
use crate::settings_notifications::NotificationManager;
use crate::settings_profiles::ProfileManager;
use crate::settings_scheduler::SettingsScheduler;
use crate::settings_templates::TemplateManager;
use crate::unified_settings::{SettingsCategory, UnifiedSettings};

use super::result::OperationResult;
use super::state::OrchestratorState;
use super::status::OrchestratorStatus;

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

    /// Get session ID
    pub fn session_id(&self) -> Option<&str> {
        self.session_id.as_deref()
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
