# Anna Roadmap

## Current Focus (v0.0.369+)

**Theme**: UI Consistency + Code Quality + Reliability

Anna v0.0.369 focuses on:
- Centralized UI system for consistent terminal output
- Zero compiler warnings for cleaner builds (library + tests)
- Modular code design (all files under 400 lines)
- Enhanced user experience across all displays
- Natural language guidance (no non-existent CLI references)
- Unified symbols across all UI components

### Recent Completions (v0.0.346-369)
- [x] Unified bullet symbol to symbols::BULLET (v0.0.369)
- [x] Documentation updates (v0.0.368)
- [x] Fixed stale CLI command references in 7 files (v0.0.365)
- [x] UI consistency in REPL/uninstall messages (v0.0.366)
- [x] Fixed undo command reference → natural language (v0.0.367)
- [x] Documentation cleanup after revert (v0.0.364)
- [x] UI consistency in errors.rs - print_step() for recovery suggestions (v0.0.356)
- [x] UI consistency in handlers.rs - print_ok/warn for uninstall flow (v0.0.356)
- [x] Zero test warnings - cleaned unused imports/functions (v0.0.357)
- [x] Documentation updates - FEATURES.md, README.md, ROADMAP.md (v0.0.359-360)

---

## Completed

### v0.0.584 - Settings Metrics (Phase 160) ✓
- [x] `settings_metrics.rs` for metrics collection
- [x] `MetricKind` / `MetricUnit` / `MetricValue` / `Metric`
- [x] `SettingsMetrics` collection
- [x] `register()` / `increment()` / `set()` / `record()`
- [x] Default metrics with uptime tracking

### v0.0.583 - Settings Diagnostics (Phase 159) ✓
- [x] `settings_diagnostics.rs` for health checking
- [x] `DiagnosticSeverity` / `DiagnosticType` / `DiagnosticIssue`
- [x] `DiagnosticReport` / `SettingsDiagnostics`
- [x] `run()` / `quick_check()` / `health_score()` / `auto_fixable()`
- [x] Configuration, security, performance checks

### v0.0.582 - Settings Permissions (Phase 158) ✓
- [x] `settings_permissions.rs` for access control
- [x] `PermissionLevel` / `PermissionAction` / `PermissionResult`
- [x] `CategoryPermission` / `PermissionRole` / `PermissionManager`
- [x] `check()` / `add_role()` / `set_active_role()` / `lock_category()`
- [x] Role-based access with built-in roles

### v0.0.581 - Settings Events (Phase 157) ✓
- [x] `settings_events.rs` for pub/sub event system
- [x] `SettingsEventType` / `EventPriority` / `SettingsEvent`
- [x] `EventFilter` / `Subscriber` / `SettingsEventBus`
- [x] `publish()` / `subscribe()` / `query()` / `unsubscribe()`
- [x] Priority-based events with filtering

### v0.0.580 - Settings API (Phase 156) ✓
- [x] `settings_api.rs` for unified API access
- [x] `ApiOperation` / `ApiStatus` / `ApiRequest` / `ApiResponse`
- [x] `SettingValue` / `SettingsApi`
- [x] `handle()` / `get()` / `set()` / `list()` / `search()`
- [x] Request/response pattern with history

### v0.0.579 - Settings Dashboard (Phase 155) ✓
- [x] `settings_dashboard.rs` for unified overview
- [x] `DashboardSection` / `HealthLevel` / `QuickAction`
- [x] `CategorySummary` / `DashboardStats` / `RecentChange`
- [x] `refresh()` / `add_change()` / `categories()` / `stats()`
- [x] Health scoring and section visibility

### v0.0.578 - Settings Recommendations (Phase 154) ✓
- [x] `settings_recommendations.rs` for intelligent recommendations
- [x] `RecommendationPriority` / `RecommendationType` / `RecommendationStatus`
- [x] `Recommendation` / `RecommendationEngine`
- [x] `analyze()` / `active()` / `dismiss()` / `apply()`
- [x] Security, privacy, usability, performance checks

### v0.0.577 - Settings Analytics (Phase 153) ✓
- [x] `settings_analytics.rs` for usage tracking
- [x] `AnalyticsPeriod` / `MetricType` / `AnalyticsEvent`
- [x] `CategoryStats` / `AnalyticsSummary` / `SettingsAnalytics`
- [x] `record()` / `summary()` / `events_for_period()`
- [x] Per-category tracking and activity scoring

### v0.0.576 - Settings Restore (Phase 152) ✓
- [x] `settings_restore.rs` for restoring from backups
- [x] `RestoreMode` / `RestoreStatus` / `RestoreValidation`
- [x] `RestorePoint` / `RestoreRecord` / `RestoreManager`
- [x] `validate()` / `restore()` / `rollback()`
- [x] Pre-restore snapshots and version checking

### v0.0.575 - Settings Backup Manager (Phase 151) ✓
- [x] `settings_backup.rs` for automated backup/restore
- [x] `BackupType` / `BackupStatus` / `BackupMeta`
- [x] `BackupConfig` / `BackupManager`
- [x] `create_backup()` / `manual_backup()` / `pre_change_backup()`
- [x] Scheduled backups and retention limits

### v0.0.574 - Settings Orchestrator (Phase 150) ✓
- [x] `settings_orchestrator.rs` for unified coordination
- [x] `OrchestratorState` / `OperationResult`
- [x] `SettingsOrchestrator` / `OrchestratorStatus`
- [x] `change_setting()` / `switch_profile()` / `apply_template()`
- [x] Unified hooks, audit, notifications, constraints

### v0.0.573 - Settings Audit (Phase 149) ✓
- [x] `settings_audit.rs` for compliance tracking
- [x] `AuditEventType` / `AuditSeverity` enums
- [x] `AuditEntry` / `AuditFilter` / `AuditLog`
- [x] `log()` / `log_change()` / `log_security()`
- [x] Security events and session tracking

### v0.0.572 - Settings Wizard (Phase 148) ✓
- [x] `settings_wizard.rs` for guided configuration
- [x] `WizardStepType` / `WizardChoice` / `WizardStep`
- [x] `WizardState` / `SettingsWizard` / `WizardManager`
- [x] `start()` / `next()` / `back()` / `answer()`
- [x] Built-in wizards (Quick Setup, Privacy Setup)

### v0.0.571 - Settings Hooks (Phase 147) ✓
- [x] `settings_hooks.rs` for callbacks on changes
- [x] `HookTrigger` / `HookResult` / `HookPriority` enums
- [x] `HookContext` / `SettingsHook` / `HookManager`
- [x] `register()` / `fire()` / `should_fire()`
- [x] Built-in hooks and execution history

### v0.0.570 - Settings Constraints (Phase 146) ✓
- [x] `settings_constraints.rs` for settings rules
- [x] `ConstraintSeverity` / `ConstraintType` enums
- [x] `ConstraintViolation` / `SettingsConstraint` / `ConstraintManager`
- [x] `check()` / `add_constraint()` / `set_enabled()`
- [x] Built-in constraints for common conflicts

### v0.0.569 - Settings Templates (Phase 145) ✓
- [x] `settings_templates.rs` for reusable configurations
- [x] `TemplateScope` / `TemplateUseCase` / `TemplateMeta`
- [x] `SettingsTemplate` / `TemplateManager`
- [x] `add()` / `apply()` / `find_by_name()` / `find_by_tag()`
- [x] Built-in templates (Development, Production, Presentation, Learning)

### v0.0.568 - Settings Scheduler (Phase 144) ✓
- [x] `settings_scheduler.rs` for scheduling changes
- [x] `ScheduleTrigger` / `ScheduleEvent` / `ScheduledAction`
- [x] `ScheduledChange` / `SettingsScheduler`
- [x] `add()` / `remove()` / `pending()` / `on_event()`
- [x] Time-based and event-based scheduling

### v0.0.567 - Settings Notifications (Phase 143) ✓
- [x] `settings_notifications.rs` for change notifications
- [x] `NotificationPriority` / `NotificationType` enums
- [x] `SettingsNotification` / `NotificationManager`
- [x] `notify()` / `mark_read()` / `dismiss()` / `unread()`
- [x] Priority-based and category-specific notifications

### v0.0.566 - Settings Search (Phase 142) ✓
- [x] `settings_search.rs` for keyword search
- [x] `MatchType` / `SearchResult` / `SearchResults`
- [x] `SearchOptions` / `SettingsSearcher`
- [x] `search()` / `sort_by_score()` / `by_category()`
- [x] Relevance scoring and category filtering

### v0.0.565 - Settings Profiles (Phase 141) ✓
- [x] `settings_profiles.rs` for named configurations
- [x] `ProfileMeta` / `SettingsProfile` / `ProfileManager`
- [x] `add()` / `remove()` / `switch_to()` / `duplicate()`
- [x] Profile tagging and search
- [x] Active/default profile tracking

### v0.0.564 - Settings Sync (Phase 140) ✓
- [x] `settings_sync.rs` for multi-device sync
- [x] `SyncStatus` / `SyncProvider` / `ConflictResolution`
- [x] `SyncConfig` / `SyncManager` structs
- [x] `push()` / `pull()` / `sync()` / `check_remote()`
- [x] File-based and Git-based sync support
- [x] Configurable sync intervals and conflict resolution

### v0.0.563 - Settings History (Phase 139) ✓
- [x] `settings_history.rs` for change tracking
- [x] `HistoryEntry` / `SettingsHistory` with undo/redo
- [x] `record()` / `undo()` / `redo()` / `recent()`
- [x] Full settings snapshots for reliable restoration
- [x] Human-readable age formatting

### v0.0.562 - Settings Presets (Phase 138) ✓
- [x] `settings_presets.rs` for pre-configured profiles
- [x] `PresetCategory` / `SettingsPreset` / `PresetManager`
- [x] 12 built-in presets (Beginner, Expert, Paranoid, etc)
- [x] `find()` / `by_category()` / `add()` / `remove()`
- [x] Natural language preset matching

### v0.0.561 - Settings Diff (Phase 137) ✓
- [x] `settings_diff.rs` for comparing settings
- [x] `DiffType` / `DiffEntry` / `SettingsDiff` / `SettingsDiffer`
- [x] `diff()` / `is_identical()` / `has_changes()` / `changes_only()`
- [x] Field-level comparison for all 12 categories
- [x] Git-style diff output format

### v0.0.560 - Settings Watcher (Phase 136) ✓
- [x] `settings_watcher.rs` for file change watching
- [x] `SettingsEventType` / `SettingsEvent` / `WatcherConfig`
- [x] `check()` / `emit()` / `start()` / `stop()`
- [x] Event history and callback listener system
- [x] Thread-safe running flag

### v0.0.559 - Settings CLI Interface (Phase 135) ✓
- [x] `settings_cli.rs` for natural language commands
- [x] `SettingsCommand` / `ParseResult` / `SettingsParser`
- [x] `parse()` / `execute_command()` / `is_settings_command()`
- [x] Show/Reset/Change/Export/Import/Validate commands
- [x] Help and category listing

### v0.0.558 - Settings Export/Import (Phase 134) ✓
- [x] `settings_export.rs` for exporting/importing settings
- [x] `ExportFormat` / `ExportOptions` / `ExportMetadata`
- [x] `export_string()` / `import_string()` / `import_and_merge()`
- [x] Multi-format support (JSON, TOML, compact)
- [x] Automatic format detection

### v0.0.557 - Settings Validation (Phase 133) ✓
- [x] `settings_validation.rs` for conflict detection
- [x] `ValidationSeverity` / `ValidationCategory` / `ValidationIssue`
- [x] `validate()` / `check_conflicts()` / `check_security_issues()`
- [x] Strict mode and suggestion system
- [x] Formatted validation reports

### v0.0.556 - Settings Migration (Phase 132) ✓
- [x] `settings_migration.rs` for version migrations
- [x] `MigrationStatus` / `MigrationResult` / `VersionedSettings`
- [x] `migrate()` / `migrate_legacy()` / `needs_migration()`
- [x] Schema version tracking and history
- [x] Dry run mode and automatic backup

### v0.0.555 - Settings Persistence (Phase 131) ✓
- [x] `settings_persistence.rs` for disk storage
- [x] `SettingsError` / `SettingsFormat` enums
- [x] `SettingsPersistence` struct with auto-save
- [x] `load()` / `save()` / `create_backup()` / `restore_latest()`
- [x] Export/import and backup management

### v0.0.554 - Unified Settings Manager (Phase 130) ✓
- [x] `unified_settings.rs` aggregates all 12 config modules
- [x] `SettingsCategory` enum for routing
- [x] `UnifiedSettings` struct with all configs
- [x] `categorize_request()` / `apply_change()` / `reset_all()`
- [x] `format_settings_summary()` / `is_settings_query()`

### v0.0.553 - Model Config (Phase 129) ✓
- [x] `model_config.rs` for LLM model settings
- [x] `ModelSizePreference` / `QualitySpeedBalance` / `ModelManagement` enums
- [x] `ModelConfig` with natural language parser
- [x] Fast/quality/minimal presets
- [x] GPU and auto-download controls

### v0.0.552 - Update Config (Phase 128) ✓
- [x] `update_config.rs` for update settings
- [x] `UpdateCheckFrequency` / `UpdateChannel` / `UpdateAction` enums
- [x] `UpdateConfig` with natural language parser
- [x] Conservative/automatic/bleeding_edge presets
- [x] Check frequency and notification controls

### v0.0.551 - Backup Config (Phase 127) ✓
- [x] `backup_config.rs` for backup settings
- [x] `BackupFrequency` / `BackupType` / `BackupTarget` / `CompressionLevel` enums
- [x] `BackupConfig` with natural language parser
- [x] Minimal/comprehensive/manual_only presets
- [x] Encryption and verification controls

### v0.0.550 - Privacy Config (Phase 126) ✓
- [x] `privacy_config.rs` for privacy settings
- [x] `DataCollectionLevel` / `LogRetention` / `SensitiveDataHandling` enums
- [x] `PrivacyConfig` with natural language parser
- [x] Maximum/balanced/convenience presets
- [x] History storage and anonymization controls

### v0.0.549 - Output Style Config (Phase 125) ✓
- [x] `output_style_config.rs` for output styling
- [x] `ColorScheme` / `ThemeStyle` / `AnimationStyle` / `BorderStyle` enums
- [x] `OutputStyleConfig` with natural language parser
- [x] Hollywood/minimal/hacker/professional/no_color presets
- [x] Color, animation, and compact mode toggles

### v0.0.548 - Timeout Config (Phase 124) ✓
- [x] `timeout_config.rs` for timeout settings
- [x] `TimeoutScope` / `TimeoutAction` / `TimeoutProfile` enums
- [x] `TimeoutConfig` with natural language parser
- [x] Fast/patient/unlimited presets
- [x] Scope-specific timeout values

### v0.0.547 - Confirmation Behavior Config (Phase 123) ✓
- [x] `confirmation_behavior_config.rs` for confirmation behavior
- [x] `ConfirmationStyle` / `TimeoutBehavior` / `ConfirmableAction` enums
- [x] `ConfirmationBehaviorConfig` with natural language parser
- [x] Strict/lenient/silent presets
- [x] Action-specific confirmation requirements

### v0.0.546 - Verbosity Config (Phase 122) ✓
- [x] `verbosity_config.rs` for verbosity and detail level
- [x] `VerbosityLevel` / `DetailLevel` / `OutputContext` enums
- [x] `VerbosityConfig` with natural language parser
- [x] Minimal/verbose/debug presets
- [x] Context-specific detail levels

### v0.0.545 - Escalation Policy Config (Phase 121) ✓
- [x] `escalation_policy_config.rs` for escalation policy
- [x] `EscalationTrigger` / `EscalationPriority` / `EscalationMode` enums
- [x] `EscalationPolicyConfig` with natural language parser
- [x] Lenient/strict/manual presets
- [x] Confidence threshold and auto-escalation controls

### v0.0.544 - Learning Mode Config (Phase 120) ✓
- [x] `learning_mode_config.rs` for learning mode
- [x] `LearningModeLevel` / `ExplanationDepth` enums
- [x] `LearningModeConfig` with natural language parser
- [x] Basic/intermediate/advanced presets
- [x] Command and config explanation toggles

### v0.0.543 - Risk Level Config (Phase 119) ✓
- [x] `risk_level_config.rs` for confirmation skipping
- [x] `RiskLevel` / `ActionCategory` / `ConfirmationMode` enums
- [x] `RiskLevelConfig` with natural language parser
- [x] Risk ordering and category defaults
- [x] Root/delete special confirmation handling

### v0.0.542 - Personality Config (Phase 118) ✓
- [x] `personality_config.rs` for personality settings
- [x] `FormalityLevel` / `FriendlinessLevel` / `HumorLevel` enums
- [x] `PersonalityConfig` with natural language parser
- [x] `apply_change()` for natural language commands
- [x] Greeting style based on personality

### v0.0.541 - Tips System (Phase 117) ✓
- [x] `tips_system.rs` for greeting tips
- [x] `TipCategory` / `TipPriority` enums
- [x] `Tip` / `TipsSystem` with rotation
- [x] 10 default tips for configuration options
- [x] Daily tip limit with automatic rotation

### v0.0.540 - Installation Date Tracker (Phase 116) ✓
- [x] `installation_tracker.rs` for installation tracking
- [x] `InstallMethod` / `InstallStatus` enums
- [x] `InstallationInfo` struct with duration calculations
- [x] Anniversary and milestone detection
- [x] Human-readable uptime strings

### v0.0.539 - Team Consultation Tracker (Phase 115) ✓
- [x] `team_consultation_tracker.rs` for team consultations
- [x] `TeamDepartment` / `ConsultationOutcome` enums
- [x] `ConsultationRecord` / `TeamConsultationTracker` system
- [x] Most consulted team detection for fun stats
- [x] Resolution/escalation rate tracking

### v0.0.538 - Response Time Tracker (Phase 114) ✓
- [x] `response_time_tracker.rs` for response time tracking
- [x] `ResponseType` / `ComplexityLevel` enums
- [x] `ResponseTimeRecord` / `ResponseTimeTracker` system
- [x] Shortest/longest reply detection for fun stats
- [x] Percentile calculations and distribution stats

### v0.0.537 - Query History Tracker (Phase 113) ✓
- [x] `query_history_tracker.rs` for user query tracking
- [x] `QueryCategory` / `QueryOutcome` enums
- [x] `QueryRecord` / `QueryHistoryTracker` system
- [x] Repeated questions detection for fun stats
- [x] Topic most asked about analytics

### v0.0.536 - Display Mode Manager (Phase 112) ✓
- [x] `display_mode_manager.rs` for debug vs fly-on-the-wall display
- [x] `DisplayMode` enum (FlyOnTheWall, Debug, Minimal, Verbose)
- [x] `OutputSection` enum (Greeting, InternalComms, etc.)
- [x] `VisibilityRule` / `DisplayModeManager` system
- [x] Section visibility per mode with rules

### v0.0.535 - Greeting Generator (Phase 111) ✓
- [x] `greeting_generator.rs` for personalized greetings
- [x] `TimeOfDay` / `InsightType` enums
- [x] `GreetingInsight` / `GreetingContext` / `GreetingGenerator` system
- [x] Time-appropriate greetings with insights
- [x] Error/warning announcements in REPL greeting

### v0.0.534 - Long Task Manager (Phase 110) ✓
- [x] `long_task_manager.rs` for long-running tasks
- [x] `LongTaskStatus` / `LongTaskType` enums
- [x] `LongTaskRecord` / `LongTaskManager` system
- [x] Idle-time execution with `wait_for_idle()`
- [x] Chain of thought and email notification

### v0.0.533 - Notification Tracker (Phase 109) ✓
- [x] `notification_tracker.rs` for user notifications
- [x] `NotificationChannel` / `NotificationPriority` / `DeliveryStatus` enums
- [x] `NotificationRecord` / `NotificationTracker` system
- [x] Anti-spam cooldown with `should_suppress()`
- [x] Email, libnotify, wall, terminal channels

### v0.0.532 - Helper Install Tracker (Phase 108) ✓
- [x] `helper_install_tracker.rs` for helper tools
- [x] `HelperInstaller` / `HelperCategory` / `HelperStatus` enums
- [x] `HelperRecord` / `HelperInstallTracker` system
- [x] Track Anna vs user installations
- [x] `would_be_useless()` hardware check

### v0.0.531 - LLM Model Registry (Phase 107) ✓
- [x] `llm_model_registry.rs` for installed LLM models
- [x] `ModelCapability` / `ModelStatus` / `InstalledBy` enums
- [x] `ModelRecord` / `LlmModelRegistry` system
- [x] Specialist assignment and usage tracking
- [x] VRAM and disk resource tracking

### v0.0.530 - Knowledge Citation Tracker (Phase 106) ✓
- [x] `knowledge_citation.rs` for authoritative sources
- [x] `CitationSource` / `CitationReliability` enums
- [x] `CitationRecord` / `KnowledgeCitationTracker` system
- [x] Track Arch Wiki, man pages, --help, official docs
- [x] Usage tracking and source stats

### v0.0.529 - Escalation Tracker (Phase 105) ✓
- [x] `escalation_tracker.rs` for junior-to-senior escalations
- [x] `EscalationReason` / `EscalationOutcome` enums
- [x] `EscalationRecord` / `EscalationTracker` system
- [x] `escalate()` / `resolve()` with full analytics
- [x] Senior resolution rate and reason stats

### v0.0.528 - Team Specialist Roster (Phase 104) ✓
- [x] `team_specialist_roster.rs` for IT department roster
- [x] `SeniorityLevel` / `Department` / `AvailabilityStatus` enums
- [x] `Specialist` / `TeamSpecialistRoster` system
- [x] `find_available()` / `find_senior()` for escalation
- [x] Ticket assignment with stats tracking

### v0.0.527 - Skill Proficiency Tracker (Phase 103) ✓
- [x] `skill_proficiency.rs` for tracking Anna's learned skills
- [x] `SkillDomain` / `ProficiencyLevel` enums
- [x] `SkillRecord` / `SkillProficiencyTracker` system
- [x] XP-based leveling system (Novice to Master)
- [x] `learn()` / `use_skill()` / `top_skills()` / `needs_practice()`

### v0.0.526 - Context Memory Store (Phase 102) ✓
- [x] `context_memory_store.rs` for storing conversational context
- [x] `MemoryType` / `MemoryImportance` enums
- [x] `MemoryEntry` / `ContextMemoryStore` system
- [x] `store()` / `retrieve()` / `search()` / `prune()`
- [x] Automatic cleanup of low-priority memories

### v0.0.525 - Workflow Automation Tracker (Phase 101) ✓
- [x] `workflow_automation.rs` for automated workflows
- [x] `WorkflowTrigger` / `WorkflowStatus` enums
- [x] `WorkflowStep` / `WorkflowRecord` / `WorkflowAutomationTracker` system
- [x] Multi-step workflow execution
- [x] Success rate tracking

### v0.0.524 - Anna Metrics Dashboard (Phase 100!) 🎉
- [x] `anna_metrics_dashboard.rs` - Comprehensive dashboard
- [x] `DashboardSection` / `HealthStatus` enums
- [x] `MetricEntry` / `AnnaMetricsDashboard` system
- [x] Trend tracking (positive/negative)
- [x] **MILESTONE: Phase 100 reached!**

### v0.0.523 - Task Priority Manager (Phase 99) ✓
- [x] `task_priority_manager.rs` for managing task priority
- [x] `TaskPriority` / `TaskState` enums
- [x] `ManagedTask` / `TaskPriorityManager` system
- [x] `add()` / `start()` / `complete()` lifecycle
- [x] Priority-sorted pending queue

### v0.0.522 - User Preference Learner (Phase 98) ✓
- [x] `user_preference_learner.rs` for learning user preferences
- [x] `PreferenceCategory` / `LearnConfidence` enums
- [x] `LearnedPreference` / `UserPreferenceLearner` system
- [x] `learn()` / `confirm()` adaptive learning
- [x] Confidence increases with repeated observations

### v0.0.521 - Error Recovery Tracker (Phase 97) ✓
- [x] `error_recovery_tracker.rs` for tracking error recovery
- [x] `ErrorCategory` / `RecoveryOutcome` enums
- [x] `ErrorRecoveryRecord` / `ErrorRecoveryTracker` system
- [x] `strategy_rate()` / `best_strategies()` analytics
- [x] Overall recovery rate tracking

### v0.0.520 - Resource Usage Tracker (Phase 96) ✓
- [x] `resource_usage_tracker.rs` for tracking system resources
- [x] `ResourceType` / `UsageLevel` enums
- [x] `UsageSample` / `ResourceUsageTracker` system
- [x] `record()` / `current()` / `peak()` / `average()` tracking
- [x] Critical/High usage detection

### v0.0.519 - Query Pattern Analyzer (Phase 95) ✓
- [x] `query_pattern_analyzer.rs` for analyzing query patterns
- [x] `PatternCategory` / `ConfidenceLevel` enums
- [x] `QueryPattern` / `QueryPatternAnalyzer` system
- [x] `record_match()` / `most_used()` for pattern analytics
- [x] `COMMON_PATTERNS` constant

### v0.0.518 - Session History Tracker (Phase 94) ✓
- [x] `session_history.rs` for tracking user sessions
- [x] `SessionOutcome` / `SessionType` enums
- [x] `SessionRecord` / `SessionHistoryTracker` system
- [x] `start_session()` / `end_session()` lifecycle
- [x] Session analytics (avg_duration, queries, tickets)

### v0.0.517 - Dependency Tracker (Phase 93) ✓
- [x] `dependency_tracker.rs` for tracking software dependencies
- [x] `DependencyType` / `DependencyStatus` enums
- [x] `DependencyRecord` / `DependencyTracker` system
- [x] `reverse_deps()` / `safe_to_remove()` for safe package removal
- [x] Tracks broken packages with missing deps

### v0.0.516 - Hardware Capability Detector (Phase 92) ✓
- [x] `hardware_capability.rs` for detecting what hardware exists
- [x] `HardwareCategory` / `HardwareStatus` enums
- [x] `HardwareCapability` / `HardwareCapabilityTracker` system
- [x] `is_helper_useful()` / `useless_helpers()` for smart helper installation
- [x] `COMMON_CAPABILITIES` constant mapping hardware to helpers
- [x] Per VISION.md: "Never install useless helpers"

### v0.0.515 - Strategic Thinking Tracker (Phase 91) ✓
- [x] `strategic_thinking.rs` for idle-time analysis
- [x] `ThinkingStatus` / `ThinkingCategory` / `ThinkingPriority` enums
- [x] `ThinkingTask` / `StrategicThinkingTracker` system
- [x] `start()` / `pause()` / `resume()` interruptible workflow
- [x] Tracks resume_point for interrupted tasks

### v0.0.514 - Alarm Scheduler (Phase 90) ✓
- [x] `alarm_scheduler.rs` for recurring notifications
- [x] `AlarmFrequency` / `DayOfWeek` / `AlarmStatus` enums
- [x] `AlarmRecord` / `AlarmScheduler` system
- [x] Natural language alarm parsing

### v0.0.513 - Dialogue Renderer (Phase 89) ✓
- [x] `dialogue_renderer.rs` for fly-on-the-wall display
- [x] `Speaker` / `DialogueMood` enums
- [x] Specialist dialogue formatting

### v0.0.512 - LLM Assignment Tracker (Phase 88) ✓
- [x] `llm_assignment.rs` for model-to-specialist mapping
- [x] `ModelTier` enum (Light, Standard, Heavy, DeepThinking)
- [x] `COMMON_MODELS` constant

### v0.0.511 - Specialist Roster (Phase 87) ✓
- [x] `specialist_roster.rs` with diverse human names
- [x] `SPECIALIST_NAMES` constant with 20 diverse names

### v0.0.510 - Ticket Resolution Stats (Phase 86) ✓
- [x] `ticket_resolution_stats.rs` for Anna vs specialist tracking
- [x] `Resolver` / `ResolutionMethod` enums

### v0.0.509 - Idle Time Detector (Phase 85) ✓
- [x] `idle_time_detector.rs` for background research
- [x] `IdleState` enum (Active, Idle, DeepIdle)

### v0.0.508 - Email Notification System (Phase 84) ✓
- [x] `email_notification.rs` for long-running task notifications
- [x] `EmailConfig` with consent, daily limits, DND hours

### v0.0.507 - Helper Tracker (Phase 83) ✓
- [x] `helper_tracker.rs` tracking helpers Anna vs user installed
- [x] `InstalledBy` enum (Anna, User, System)

### v0.0.506 - Config Change Tracker (Phase 82) ✓
- [x] `config_change_tracker.rs` for configuration changes
- [x] `ConfigScope` / `ChangeType` enums

### v0.0.337-363 - Centralized UI System (Phase 28) ✓
- [x] `anna_shared::ui::printing` module with consistent helpers
- [x] print_header, print_title, print_footer, print_hr
- [x] print_section_header, print_label, print_hint, print_step
- [x] print_ok, print_err, print_warn with symbols
- [x] kv() and kv_colored() for key-value display
- [x] colors and symbols constants centralized
- [x] Updated greeting/status.rs, greeting/personal.rs
- [x] Updated errors.rs, handlers.rs, progress_display.rs
- [x] Updated change_commands.rs, learning.rs
- [x] Zero compiler warnings in release build + tests (v0.0.357)
- [x] Removed unused DecayResult import
- [x] Removed unused VerificationInput.id field
- [x] Removed unused VerificationResult.score field
- [x] Removed unused test imports and functions (v0.0.357)
- [x] Removed dead code files (v0.0.363)

### v0.0.103 - Recipe Feedback System (Phase 23) ✓
- [x] `FeedbackRequest` struct - Anna asks user for feedback when uncertain
- [x] `feedback_request` field in ServiceDeskResult
- [x] Anna asks for feedback on borderline confidence (60-75) or new recipes (<3 uses)
- [x] Interactive feedback handling in REPL and one-shot modes
- [x] Feedback adjusts recipe reliability_score (+1 helpful, -5 not-helpful)
- [x] Feedback history logged to ~/.anna/feedback_history.jsonl

### v0.0.102 - Recipe Direct Answers (Phase 22) ✓
- [x] Direct answer from recipe template (skip probes too)
- [x] `build_recipe_result()` creates ServiceDeskResult from recipe
- [x] `can_answer_directly()` checks for answer template
- [x] Instant responses for learned patterns

### v0.0.101 - Recipe Fast Path Integration (Phase 21) ✓
- [x] Recipe index built at daemon startup
- [x] Recipe check BEFORE calling LLM translator
- [x] High-confidence recipes skip LLM (score >= 70)
- [x] ConfigureShell and ConfigureGit query classes
- [x] Shell/git config query routing

### v0.0.100 - Recipe Matcher & Config Recipes (Phase 20) ✓
- [x] Recipe matcher for translator fast-path
- [x] Shell configuration recipes (bash, zsh, fish)
- [x] Git configuration recipes
- [x] New RecipeKind variants (ShellConfig, GitConfig)

### v0.0.99 - Natural Language Package & Service Management (Phase 19) ✓
- [x] Package install via natural language ("install htop")
- [x] Service management via natural language ("restart docker")
- [x] Cross-distro package name mapping
- [x] Protected service detection
- [x] QueryClass: InstallPackage, ManageService

### v0.0.98 - Multi-file Transactions & Recipe Systems (Phase 18) ✓
- [x] ChangeTransaction for atomic multi-file changes
- [x] Automatic rollback on failure
- [x] Package recipes with multi-manager support (pacman, apt, dnf, flatpak, snap)
- [x] Service recipes with risk levels and protected services
- [x] Cross-distro package name mapping

### v0.0.97 - Change History and Undo (Phase 17) ✓
- [x] Change history tracking in ~/.anna/change_history.jsonl
- [x] `annactl history` command
- [x] `annactl undo <id>` command
- [x] Backup-based restoration

### v0.0.96 - Desktop Team Editor Config Flow (Phase 16) ✓
- [x] Natural language editor configuration ("enable syntax highlighting")
- [x] proposed_change field in ServiceDeskResult
- [x] CLI confirmation flow for config changes
- [x] Integration with Safe Change Engine

### v0.0.95 - Safe Change Engine (Phase 15) ✓
- [x] PlanChange, ApplyChange, RollbackChange RPC methods
- [x] Backup-first, idempotent config modifications
- [x] Extracted editor_recipe_data.rs module
- [x] All files under 400 lines

### v0.0.94 - Recipe Learning System (Phase 14) ✓
- [x] Automatic recipe learning from successful queries
- [x] Learning criteria: verified=true, reliability >= 80
- [x] Recipe persistence in ~/.anna/recipes/
- [x] Team assignment from domain

### v0.0.93 - Documentation Update (Phase 13) ✓
- [x] Updated README, ROADMAP, FEATURES for current version
- [x] Hollywood IT aesthetic documentation

### v0.0.92 - Codebase Hygiene (Phase 12) ✓
- [x] Zero compiler warnings across entire workspace
- [x] Fixed unused methods, variables, and imports
- [x] Applied cargo fix to test files

### v0.0.91 - ASCII-Style Achievement Badges (Phase 11) ✓
- [x] Replaced emoji badges with ASCII art symbols
- [x] Badge styles: `[1]` `<3d>` `(90+)` `{*}` `~00~` `|7d|`
- [x] Hollywood IT aesthetic consistency

### v0.0.90 - Achievement Badges (Phase 10) ✓
- [x] 22 unique achievements across 6 categories
- [x] Milestones, Streaks, Quality, Teams, Special, Tenure
- [x] Integration with stats display

### v0.0.89 - Personalized Greetings (Phase 9) ✓
- [x] Time-of-day awareness (Morning, Afternoon, Evening, Night)
- [x] User personalization from $USER
- [x] Domain-specific follow-up prompts
- [x] New greetings.rs module

### v0.0.88 - Warning Cleanup (Phase 8) ✓
- [x] Removed all compiler warnings
- [x] Fixed unused imports across workspace

### v0.0.87 - Dialogue Variety (Phase 7) ✓
- [x] Varied junior approval phrases
- [x] Varied escalation requests
- [x] Varied senior responses
- [x] Seed-based deterministic variety

### v0.0.81-86 - Service Desk Theatre ✓
- [x] Named IT personas with roles
- [x] Cinematic narrative rendering
- [x] Internal communications mode (-i flag)
- [x] Streak tracking and XP system

### v0.0.75 - RPG Stats System ✓
- [x] Event logging with JSONL store
- [x] XP calculation and level progression
- [x] Titles from Trainee to Principal Engineer
- [x] Stats display with progress bars

### v0.0.71 - Version Truth ✓
- [x] Single source of truth: workspace Cargo.toml version only
- [x] Unified version display: annactl/annad --version format consistent
- [x] Status shows: installed (annactl), daemon_ver (annad), available, last_check, next_check, auto_update
- [x] Hard gate tests: CI fails if annactl/annad version != workspace version
- [x] No hardcoded version strings in tests (compare against VERSION constant)
- [x] Auto-update semantic comparison with no-downgrade guarantee

### v0.0.70 - Version Unification + Release Hygiene ✓
- [x] Single source of truth: workspace Cargo.toml version is authoritative
- [x] All crates use version.workspace = true
- [x] anna_shared::VERSION uses env!("CARGO_PKG_VERSION")
- [x] install.sh fetches version from GitHub releases API (no hardcoding)
- [x] Version consistency tests validate all sources
- [x] Status output shows: installed, available, last_check, next_check, auto_update
- [x] Auto-update uses semantic version comparison (no string comparison)
- [x] No downgrade guarantee: newer installed version is never replaced

### v0.0.69 - Unified Versioning + REPL Enhancements ✓
- [x] Single source of truth for version (workspace Cargo.toml)
- [x] REPL "since last time" summary with snapshot comparison
- [x] Delta tracking for failed services, disk, memory changes
- [x] Version consistency tests
- [x] Documentation updates (CHANGELOG, FEATURES, README)

### v0.0.68 - Audio Parse Correctness + ConfigureEditor Grounding ✓
- [x] Audio deterministic answer handles "Multimedia audio controller"
- [x] ConfigureEditor uses full router probe list (skip spine override)
- [x] Clarification prompts end with period, not question mark

### v0.0.67 - Service Desk Theatre UX ✓
- [x] Service desk narrative renderer (render.rs)
- [x] REPL narrative header with boot status, critical issues
- [x] Stats RPG system with XP calculation
- [x] Local citations system (citations.rs)

### v0.0.66 - Version Normalization + Regressions ✓
- [x] Version consolidation across all sources
- [x] Audio evidence parsing for lspci PCI class codes
- [x] ConfigureEditor numbered options without question marks

### v0.0.63 - Service Desk Theatre Renderer ✓
- [x] Narrative flow in normal mode ("Checking X...")
- [x] Evidence source in footer when grounded
- [x] Clarification options numbered display
- [x] New transcript events (EvidenceSummary, DeterministicPath, ProposedAction, ActionConfirmationRequest)
- [x] Debug mode rendering for all new events

### v0.0.62 - ConfigureEditor Grounding ✓
- [x] Proper probe accounting with valid_evidence_count
- [x] Execution trace for all ConfigureEditor paths
- [x] Grounding signals based on valid evidence

### v0.0.61 - HardwareAudio Parser ✓
- [x] Content-based audio detection (not just command pattern)
- [x] pactl detection by "Card #" blocks
- [x] Evidence merge from lspci + pactl

### v0.0.45 - Query Classification & Probe Planning ✓
- [x] New QueryClass variants: InstalledToolCheck, HardwareAudio, CpuTemp, CpuCores, PackageCount, MemoryFree
- [x] Modularized router.rs + query_classify.rs
- [x] Stabilization golden tests
- [x] ReliabilityInput builder methods

### v0.0.26 - Team-Scoped Review System ✓
- [x] SPECIALISTS Registry: Team-scoped roles (Translator, Junior, Senior)
- [x] 8 Teams: Desktop, Storage, Network, Performance, Services, Security, Hardware, General
- [x] Deterministic Review Gate: Hybrid logic that minimizes LLM calls
- [x] Team-specific junior/senior review prompts
- [x] Review gate transcript events
- [x] Trace enhancements (ReviewerOutcome, FallbackUsed::Timeout)

### v0.0.23 - TRACE + TRUST+ + RESCUE ✓
- [x] Execution trace for debugging degraded paths
- [x] Enhanced reliability explanations
- [x] Explicit threshold constants for scoring

### v0.0.18 and earlier ✓
- [x] Core pipeline with grounded responses
- [x] Deterministic probe routing
- [x] Auto-update mechanism
- [x] Per-stage latency tracking
- [x] Hardware-aware model selection

### v0.0.235 - Docker Compose Recipes (Phase 27) ✓
- [x] DockerFeature enum (CreateCompose, StartServices, StopServices, ViewLogs, etc.)
- [x] DockerRecipe with answer templates
- [x] 10 built-in recipes for Docker Compose workflows
- [x] Query matcher integration with recipe_fast_path
- [x] RecipeKind::DockerCompose variant

### v0.0.234 - Cron Job Recipes (Phase 26) ✓
- [x] CronFeature enum (AddJob, ListJobs, EditCrontab, RemoveJob, etc.)
- [x] CronPreset enum (Hourly, Daily, Weekly, Monthly, etc.)
- [x] CronRecipe with syntax help and examples
- [x] 8 built-in recipes for cron management
- [x] Query matcher integration with recipe_fast_path
- [x] RecipeKind::CronJob variant

### v0.0.233 - Systemd Unit File Recipes (Phase 25) ✓
- [x] SystemdFeature enum (CreateService, CreateTimer, EnableService, ViewLogs, etc.)
- [x] UnitType, RestartPolicy enums
- [x] SystemdRecipe with unit file templates
- [x] 8 built-in recipes for systemd management
- [x] Query matcher integration with recipe_fast_path
- [x] RecipeKind::SystemdUnit variant

### v0.0.104 - SSH Key Management Recipes (Phase 24) ✓
- [x] SshFeature enum (GenerateKey, CopyKey, Config, Agent, GitHub, etc.)
- [x] SshRecipe with answer templates
- [x] 7 built-in recipes for SSH key management
- [x] Query matcher for SSH-related queries
- [x] RecipeKind::SshConfig variant

## Planned

### Phase 29 - Enhanced Status Display ✓ (v0.0.463)
- [x] Show installed vs available GitHub version with asset verification
- [x] Display update check pace, last check, next scheduled check
- [x] Show all user groups and folder permissions (FolderPermission struct)
- [x] Show Ollama status separately from daemon status
- [x] List helpers with "installed by Anna" vs "installed by user" labels
- [x] Display which specialist uses which LLM
- [x] Show all config settings in status (not just debug mode)
- [x] REPL greeting announces errors (SystemError struct, add_error(), render())

### Phase 30 - Enhanced Stats Display ✓ (v0.0.464)
- [x] Non-linear XP progression (0-100 RPG style, logistic curve)
- [x] Funny titles based on level (Trainee -> Linus's Chosen One)
- [x] Recipes categorized by type (in by_handler)
- [x] Average interaction count between Anna and specialists
- [x] Number of times Anna managed on her own (anna_solo_count)
- [x] Number of recipes learned (recipes_learned)
- [x] Longest/shortest resolution times (min/max_duration_ms)
- [x] Most consulted team (most_consulted_team)
- [x] Repeated questions tracking (repeated_queries)
- [x] Topic most asked about (top_topic)
- [x] Longest/shortest reply (tracked via duration proxy)
- [x] Installation date display (first_event_ts)
- [x] `annactl stats <category>` for detailed views (rpg, learning, outcomes, topics, repeated)

### Phase 31 - Natural Language Dialog ✓ (v0.0.465)
- [x] Anna-to-Specialist dialog rendering in normal mode (TranscriptSegment system)
- [x] Case number display (CN-XXXX-DDMMYYYY format via format_case_with_date)
- [x] Specialist responses with reliability assessments (junior_approval, senior_response)
- [x] Anna confirming recipe storage after high-reliability answers (anna_recipe_storage_confirmation)
- [x] Real-time streaming word-by-word from LLM (Progress segment kind)
- [x] Screen updates after each LLM call (transcript segments streamed)

### Phase 32 - Smart Helper Management ✓ (v0.0.466)
- [x] Auto-install helpers learned from specialists (register_anna_installed)
- [x] Track helper source (Anna vs User) (InstallSource enum)
- [x] Remove only Anna-installed helpers on uninstall (remove_anna_installed)
- [x] Skip useless helpers (no ethtool without ethernet) (hardware_requirement, is_useful)
- [x] Display helper last_used timestamp (last_used, last_used_display)

### Phase 33 - Dynamic Team Availability ✓ (v0.0.454)
- [x] Detect hardware capabilities (sound, network, etc.)
- [x] Hide teams for missing hardware (no Sound team if no audio)
- [x] Show available team count in status

### Phase 34 - Long-Running Task Handling ✓ (v0.0.455)
- [x] Detect tasks taking > X minutes
- [x] Ask for email (store for reuse)
- [x] Move to background with notification
- [ ] Investigate during idle time (deferred to Phase 37)
- [ ] Email with chain of thoughts and conclusion (deferred to Phase 37)
- [ ] Second email if internet research needed (deferred to Phase 37)

### Phase 35 - User Notifications ✓ (v0.0.456)
- [x] Email notifications for long tasks
- [x] libnotify integration for desktop alerts
- [x] wall for terminal broadcasts (optional)
- [x] Custom alarms via natural language ("notify me every Monday at 9 about storage")

### Phase 36 - Knowledge & Citations ✓ (v0.0.457)
- [x] Citations from Arch Wiki, man pages, --help commands
- [x] Local cache of Arch Wiki pages linked from official docs
- [x] Learning mode explains why commands are run
- [x] Learning mode explains how commands work

### Phase 37 - Idle Time Learning ✓ (v0.0.458)
- [x] Senior specialists think strategically during idle time
- [x] Resumable tasks if interrupted
- [x] Email notification when idle task completes

### Phase 38 - Kubernetes Recipes ✓ (v0.0.459)
- [x] K8sFeature enum (19 features for pods, deployments, services, etc.)
- [x] K8sRecipe with answer templates and commands
- [x] Query matcher with longest-keyword matching
- [x] Builtin recipes for all common kubectl operations
- [x] Debugging recipes for CrashLoopBackOff, ImagePullBackOff

### Phase 39 - Web Server Recipes ✓ (v0.0.460)
- [x] WebServerFeature enum (15 features for Nginx/Apache)
- [x] WebServerRecipe with config examples and commands
- [x] Installation recipes for Arch, Debian, Fedora
- [x] SSL/TLS with Let's Encrypt/Certbot
- [x] Reverse proxy, load balancing, performance optimization

### Phase 40 - Database Recipes ✓ (v0.0.461)
- [x] DatabaseFeature enum (15 features for backup, restore, management)
- [x] DatabaseRecipe with multi-DB support
- [x] PostgreSQL, MySQL/MariaDB, SQLite, MongoDB, Redis
- [x] User/permission management recipes
- [x] Import/export data recipes

### Phase 41 - Network Troubleshooting Recipes ✓ (v0.0.462)
- [x] NetworkFeature enum (15 features for diagnostics)
- [x] NetworkRecipe with tool requirements
- [x] Connectivity, DNS, traceroute recipes
- [x] Port scanning, firewall, listening ports
- [x] SSL certificate, HTTP testing
- [x] WiFi diagnostics and VPN status

### Phase 43 - Personality Configuration via Natural Language ✓ (v0.0.467)
- [x] ConfigChange enum for 7 preference categories
- [x] detect_config_change() natural language parser
- [x] is_show_preferences() query detection
- [x] apply_config_change() updates UserProfile
- [x] format_preferences() for display
- [x] Supports: learning mode, verbosity, auto-confirm, internal comms
- [x] Personality: formality, humor, technical depth

### Phase 44 - Tips in Greetings ✓ (v0.0.468)
- [x] greeting_tips.rs module for configuration tips
- [x] Tips based on current user settings
- [x] Randomized selection with variety
- [x] Non-intrusive (1 in 3 greetings probability)
- [x] Categories: Learning, Personality, Safety, Display, Notifications
- [x] get_random_greeting_tip() convenience function

### Phase 45 - Monitoring & Custom Alarms ✓ (v0.0.469)
- [x] system_monitors.rs for proactive monitoring
- [x] CheckType enum for 5 metric types
- [x] Platform-specific monitoring (reads /proc, uses df, systemctl)
- [x] evaluate_condition() for alarm condition checking
- [x] check_conditional_alarms() integrates with AlarmStore
- [x] run_all_checks() for full system health
- [x] format_monitor_results() for display

### Phase 46 - Notification Configuration via Natural Language ✓ (v0.0.470)
- [x] notification_config.rs for notification settings
- [x] NotifyConfigChange enum for 7 setting types
- [x] Email configuration via "set my email to X"
- [x] Desktop notification toggle
- [x] Wall broadcast toggle
- [x] Quiet hours configuration
- [x] format_notification_settings() for display

### Phase 47 - Facts Lifecycle Management ✓ (v0.0.471)
- [x] facts_maintenance.rs for scheduled cleanup
- [x] MaintenanceResult tracks transitions
- [x] FactsHealth with statistics by category
- [x] run_maintenance() for lifecycle transitions
- [x] get_health() for facts statistics
- [x] Health ratio check (< 30% stale = healthy)

### Phase 48 - Arch Wiki Local Caching ✓ (v0.0.472)
- [x] wiki_cache.rs for local caching
- [x] WikiCacheEntry with metadata
- [x] WikiCacheIndex for management
- [x] essential_pages() and missing_essential()
- [x] prune_stale() and prune_to_size()
- [x] Content hash for change detection

### Phase 49 - Debug Configuration via Natural Language ✓ (v0.0.473)
- [x] debug_config.rs for NL debug settings
- [x] DebugConfigChange enum (SetLevel, EnableLogFile, Redact*)
- [x] detect_debug_config() parses natural language
- [x] apply_debug_change() updates DebugConfig
- [x] format_debug_settings() displays settings
- [x] parse_debug_level() parses level strings

### Phase 50 - Risk Level Configuration via Natural Language ✓ (v0.0.474)
- [x] risk_config.rs for NL risk tolerance settings
- [x] RiskTolerance struct with auto-confirm, warnings, protection
- [x] RiskConfigChange enum for config changes
- [x] Presets: cautious, balanced, confident, expert
- [x] detect_risk_config() parses natural language
- [x] should_auto_confirm() checks risk against tolerance

### Phase 51 - Session Summary Display ✓ (v0.0.475)
- [x] session_display.rs for session info display
- [x] SessionStats for current session statistics
- [x] format_current_session() displays stats
- [x] format_session_history() shows past sessions
- [x] is_session_query() detects session queries
- [x] get_since_last_time() for greeting integration

### Phase 52 - Unified Settings Display ✓ (v0.0.476)
- [x] settings_display.rs for unified settings view
- [x] AllSettings struct combining all config types
- [x] format_all_settings() shows all sections
- [x] format_settings_summary() compact summary
- [x] is_all_settings_query() query detection
- [x] get_settings_section() section filtering

### Phase 53 - Learning Mode Explanations Enhancement ✓ (v0.0.477)
- [x] Split into module directory (mod.rs, commands.rs)
- [x] Added 15+ new command explanations
- [x] list_explained_commands() for discovery
- [x] Improved display format (cleaner output)
- [x] Total 29+ explained commands

### Phase 54 - XP/Level Display Enhancement ✓ (v0.0.478)
- [x] xp_display.rs for RPG-style progression
- [x] AnnaXP struct with level, XP, progress, title
- [x] Non-linear XP thresholds (10 levels)
- [x] Funny level titles (Intern → Chief Technology Guru)
- [x] format_xp_display() with progress bar
- [x] format_xp_compact() for greetings

### Phase 55 - Fun Statistics Display ✓ (v0.0.479)
- [x] fun_stats_display.rs for VISION.md Fun Statistics
- [x] FunStats struct from AggregatedEvents
- [x] Installation date and days active
- [x] Most consulted team with count
- [x] Lucky team with success rate
- [x] Anna solo count and percentage
- [x] Longest/shortest reply times
- [x] Current and best streak
- [x] format_fun_stats() full display
- [x] format_fun_stats_compact() for greetings
- [x] generate_fun_fact() random facts
- [x] is_fun_stats_query() query detection

### Phase 56 - Capabilities Display ✓ (v0.0.480)
- [x] capabilities_display.rs for "what can you do?" queries
- [x] CapabilityCategory enum (9 categories)
- [x] Examples and descriptions for each category
- [x] format_capabilities() full display
- [x] format_capabilities_compact() one-line
- [x] format_capability_category() single category
- [x] format_capabilities_with_teams() with team count
- [x] is_capabilities_query() query detection
- [x] parse_capability_category() parse from query
- [x] capability_facts() and random_capability_fact()

### Phase 57 - Query Type Router ✓ (v0.0.481)
- [x] query_type_router.rs consolidates query detection
- [x] QueryType enum for all informational queries
- [x] QueryType::detect() from natural language
- [x] route_query() central routing function
- [x] should_handle_locally() skip specialist check
- [x] suggest_display() display function suggestion
- [x] is_status_query() status detection
- [x] extract_help_context() for "help with X"
- [x] is_informational() pure info check

### Phase 58 - Contextual Tips System ✓ (v0.0.482)
- [x] contextual_tips.rs for context-aware tips
- [x] TipContext for topic/command tracking
- [x] Topic detection from queries (8 categories)
- [x] get_contextual_tips() returns relevant tips
- [x] Tips with related actions
- [x] Learning mode tips
- [x] General fallback tips
- [x] format_tip() with action hints

### Phase 59 - Command Shortcuts ✓ (v0.0.483)
- [x] command_shortcuts.rs for quick aliases
- [x] 25+ built-in shortcuts (8 categories)
- [x] expand_shortcut() expands short to full
- [x] is_shortcut() detection
- [x] shortcuts_by_category() filtering
- [x] format_shortcuts() display
- [x] is_shortcuts_query() query detection
- [x] Case-insensitive matching

### Phase 60 - Quick Status Summary ✓ (v0.0.484)
- [x] quick_status.rs for at-a-glance status
- [x] HealthLevel enum (Good, Warning, Critical, Unknown)
- [x] StatusItem struct with name, health, message, value
- [x] QuickStatus struct with items, overall health, summary
- [x] memory_status() - Memory health from percentage
- [x] disk_status() - Disk health from percentage
- [x] cpu_status() - CPU health from load/cores
- [x] service_status() - Service health from state
- [x] format_quick_status_oneline() - One-line summary
- [x] format_quick_status_compact() - Issues only
- [x] format_quick_status_full() - Full display
- [x] is_quick_status_query() - Query detection

### Phase 61 - Repeated Questions Detection ✓ (v0.0.485)
- [x] repeated_questions.rs for tracking similar questions
- [x] RecordedQuestion struct with variants, count, timestamps
- [x] QuestionHistory for storing and querying
- [x] normalize_question() - Remove filler words
- [x] calculate_similarity() - Jaccard-like similarity
- [x] detect_category() - Topic categorization
- [x] record() with similarity grouping
- [x] get_repeated(), top_repeated(), by_category()
- [x] unresolved_repeated() - Pending questions
- [x] format_repeated_questions/compact() - Display
- [x] is_repeated_questions_query() - Query detection

### Phase 62 - Response Length Tracking ✓ (v0.0.486)
- [x] response_length.rs for tracking response lengths
- [x] RecordedResponse struct with char/word/line counts
- [x] ResponseLengthTracker for statistics
- [x] record() / record_with_category() - Track responses
- [x] average_chars() / average_words() - Averages
- [x] Longest/shortest by chars and words
- [x] Recent responses (last 10)
- [x] format_response_lengths() - Full display
- [x] format_response_lengths_compact() - Compact
- [x] response_length_fun_fact() - Fun facts
- [x] is_response_length_query() - Query detection

### Phase 63 - Resolution Time Tracking ✓ (v0.0.487)
- [x] resolution_time.rs for tracking resolution times
- [x] ResolutionRecord struct with timing and metadata
- [x] ResolutionTimeTracker for comprehensive stats
- [x] record() / record_simple() - Track resolutions
- [x] average_ms(), success_rate(), escalation_rate()
- [x] Fastest/slowest resolutions
- [x] Per-category stats (count, total, avg, range)
- [x] Recent resolutions (last 20)
- [x] format_resolution_times() - Full display
- [x] format_resolution_times_compact() - Compact
- [x] format_duration_ms() - Human-readable
- [x] resolution_time_fun_fact() - Fun facts

### Phase 64 - Interaction Counter ✓ (v0.0.488)
- [x] interaction_counter.rs for specialist interactions
- [x] InteractionRecord struct with from/to/type
- [x] InteractionType enum (Dispatch, Response, etc.)
- [x] SpecialistStats per-specialist tracking
- [x] InteractionCounter comprehensive tracker
- [x] record() / record_anna_solo() - Track interactions
- [x] average_per_ticket() - Avg interactions
- [x] most_consulted() / least_consulted()
- [x] fastest_responder() - Quickest specialist
- [x] anna_solo_rate() - Anna independence
- [x] format_interactions/compact() - Display
- [x] interaction_fun_fact() - Fun facts

### Phase 65 - Expert Ticket Statistics ✓ (v0.0.489)
- [x] expert_stats.rs for tickets per expert
- [x] ExpertLevel enum (Junior, Senior)
- [x] Expert struct with id, name, department
- [x] ExpertStatistics per-expert tracking
- [x] ExpertStatsTracker comprehensive tracker
- [x] register_expert() / record_closed()
- [x] record_escalation() / record_anna_solo()
- [x] top_performers() - Rankings
- [x] by_department() / by_level() - Filtering
- [x] most_reliable() / fastest_responder()
- [x] format_expert_stats/compact() - Display
- [x] expert_stats_fun_fact() - Fun facts

### Phase 66 - Recipe Statistics Display ✓ (v0.0.490)
- [x] recipe_stats_display.rs for recipe statistics
- [x] RecipeCategory enum (11 categories)
- [x] RecipeOriginType enum (Seed, Learned, etc.)
- [x] RecipeStats per-recipe tracking
- [x] RecipeStatsTracker comprehensive tracker
- [x] register() / record_use() - Track recipes
- [x] most_used() / recipes_by_category()
- [x] most_reliable() / recently_learned()
- [x] top_category() - Popular category
- [x] format_recipe_stats/compact() - Display
- [x] recipe_stats_fun_fact() - Fun facts

### Phase 67 - Aggregated Stats Dashboard ✓ (v0.0.491)
- [x] stats_dashboard.rs for unified view
- [x] DashboardSection enum (8 sections)
- [x] StatMetric with name, value, trend
- [x] StatTrend enum (Up, Down, Stable)
- [x] StatsDashboard with health score
- [x] DashboardBuilder for easy construction
- [x] with_summary/resolutions/interactions/etc
- [x] format_dashboard() - Full display
- [x] format_dashboard_compact/oneline()
- [x] generate_health_bar() - ASCII indicator
- [x] is_dashboard_query() / detect_section()

### Phase 68 - Uptime Tracking ✓ (v0.0.492)
- [x] uptime_tracker.rs for uptime and availability
- [x] UptimeRecord struct for sessions
- [x] UptimeTracker comprehensive stats
- [x] start_session() / end_session()
- [x] current_session_duration()
- [x] days_since_install() / total_uptime
- [x] avg_session_duration() / uptime_percentage()
- [x] clean_shutdown_rate() - Stability
- [x] format_uptime() / compact / oneline
- [x] format_duration_secs() - Human readable
- [x] uptime_fun_fact() / is_uptime_query()

### Phase 69 - Ticket History Display ✓ (v0.0.493)
- [x] ticket_history_display.rs for past ticket viewing
- [x] TicketOutcome enum (6 states)
- [x] HistoricalTicket struct with details
- [x] TicketHistory storage and query
- [x] add() / recent() / by_outcome()
- [x] by_department() / open_tickets()
- [x] success_rate() / most_active_department()
- [x] format_ticket_history() / compact / oneline
- [x] format_timestamp() / format_duration()
- [x] ticket_history_fun_fact()
- [x] is_ticket_history_query()

### Phase 70 - Error Summary Display ✓ (v0.0.494)
- [x] error_summary_display.rs for error tracking
- [x] ErrorSeverity enum (Critical, Error, Warning, Info)
- [x] ErrorCategory enum (9 categories)
- [x] ErrorEntry with duplicate detection
- [x] ErrorSummary storage and query
- [x] unacknowledged() / critical() / by_severity()
- [x] acknowledge() / acknowledge_all()
- [x] has_active_critical() / categorize_error()
- [x] format_error_summary() / compact / oneline
- [x] error_health_message() / is_error_summary_query()

### Phase 71 - Team Performance Display ✓ (v0.0.495)
- [x] team_performance_display.rs for team metrics
- [x] TeamId enum (8 IT teams)
- [x] TeamMetrics with comprehensive stats
- [x] TeamPerformance tracker
- [x] record_ticket() / record_escalation()
- [x] success_rate() / escalation_rate() / avg_resolution_ms()
- [x] by_activity() / by_success_rate() / by_speed()
- [x] most_active() / best_performing() / fastest()
- [x] team_grade() - A+ to F grading
- [x] format_team_performance() / compact / oneline
- [x] team_performance_fun_fact() / is_team_performance_query()

### Phase 72 - Anna Progress Report ✓ (v0.0.496)
- [x] anna_progress_report.rs for progress tracking
- [x] TimePeriod enum (Day, Week, Month, AllTime)
- [x] Trend enum with symbols
- [x] ProgressMetric with trend/change tracking
- [x] Milestone with progress tracking
- [x] PeriodSnapshot for comparisons
- [x] ProgressReport comprehensive container
- [x] calculate_trend() / calculate_change_percent()
- [x] progress_bar() - ASCII visualization
- [x] default_milestones() - 10 standard milestones
- [x] format_progress_report() / compact / oneline
- [x] is_progress_query() / progress_summary_message()

### Phase 73 - User Activity Summary ✓ (v0.0.497)
- [x] user_activity_summary.rs for usage patterns
- [x] TimeOfDay enum (Morning, Afternoon, Evening, Night)
- [x] DayOfWeek enum with display names
- [x] ActivityRecord for interactions
- [x] UserActivitySummary comprehensive tracker
- [x] most_active_time() / most_active_day()
- [x] top_topic() / top_activity_type()
- [x] days_active() / avg_interactions_per_day()
- [x] detect_topic() - 8 topic categories
- [x] format_activity_summary() / compact / oneline
- [x] activity_insight() / is_activity_query()

### Phase 74 - System Health Score ✓ (v0.0.498)
- [x] system_health_score.rs for unified health
- [x] HealthGrade enum (A-F) with descriptions
- [x] HealthCategory enum (8 categories with weights)
- [x] HealthMetric with recommendations
- [x] SystemHealthScore unified tracker
- [x] cpu_health() / memory_health() / disk_health()
- [x] services_health() / network_health() / daemon_health()
- [x] calculate_overall() with weighted scores
- [x] health_bar() - ASCII visualization
- [x] format_health_score() / compact / oneline
- [x] health_summary_message() / is_health_query()

### Phase 75 - Knowledge Base Stats ✓ (v0.0.499)
- [x] knowledge_base_stats.rs for knowledge tracking
- [x] KnowledgeType enum (Recipe, Fact, WikiPage, ManPage, etc.)
- [x] KnowledgeSource enum (Seed, Specialist, User, ArchWiki, etc.)
- [x] KnowledgeEntry struct with usage tracking
- [x] KnowledgeBaseStats comprehensive tracker
- [x] add_entry() / record_use() / mark_stale() / refresh()
- [x] by_type() / by_source() / by_topic() - Filtering
- [x] stale() / recently_acquired() / most_used() - Queries
- [x] acquisition_rate() / usage_rate() - Statistics
- [x] format_knowledge_stats() / compact / oneline
- [x] knowledge_fun_fact() / is_knowledge_query()

### Phase 76 - Boot Time Tracking ✓ (v0.0.500)
- [x] boot_time_tracking.rs for boot analysis
- [x] BootRecord struct with kernel/userspace times
- [x] SlowService struct for problem services
- [x] BootTrend enum (Faster, Slower, Stable)
- [x] BootTimeTracker comprehensive tracker
- [x] parse_systemd_analyze() - Parse systemd output
- [x] change_from_previous() / trend() / average_boot_secs()
- [x] top_slow_services() - Problem service detection
- [x] format_boot_stats() / compact / oneline
- [x] boot_time_greeting() / boot_time_fun_fact()
- [x] is_boot_time_query() - Query detection

### Phase 77 - Command Execution Logging ✓ (v0.0.501)
- [x] command_execution_log.rs for command tracking
- [x] ExecStatus enum (Success, Failed, Timeout, etc.)
- [x] CommandRisk enum (ReadOnly to Critical)
- [x] ExecutionRecord struct with full execution details
- [x] ExecutionLog comprehensive tracker
- [x] classify_risk() - Risk classification
- [x] most_used() / most_failed() - Command patterns
- [x] success_rate() / average_duration_ms()
- [x] format_execution_log() / compact / oneline
- [x] execution_fun_fact() / is_execution_log_query()

### Phase 78 - Specialist Conversation Display ✓ (v0.0.502)
- [x] specialist_conversation.rs for conversation tracking
- [x] Speaker enum (Anna, Junior, Senior, User)
- [x] MessageType enum (Query, Response, etc.)
- [x] ConversationMessage / Conversation / ConversationHistory
- [x] add_message() / resolve() - Conversation management
- [x] participants() / messages_by_speaker() - Analysis
- [x] avg_messages_per_conversation() / avg_resolution_secs()
- [x] format_conversation() - Fly-on-the-wall style
- [x] format_conversation_history() / compact / oneline
- [x] conversation_fun_fact() / is_conversation_query()

### Phase 79 - Backup History Tracking ✓ (v0.0.503)
- [x] backup_history.rs for backup tracking
- [x] BackupStatus enum (Active, Restored, Expired, Deleted)
- [x] BackupType enum (ConfigFile, SystemFile, etc.)
- [x] BackupRecord / BackupHistory comprehensive tracker
- [x] mark_restored() / mark_deleted() - Status changes
- [x] by_backup_type() / for_file() / for_change() - Queries
- [x] expire_old() / active_size_bytes() - Maintenance
- [x] format_backup_history() / compact / oneline
- [x] backup_fun_fact() / is_backup_query()

### Phase 80 - Package Installation Tracker ✓ (v0.0.504)
- [x] package_install_tracker.rs for package tracking
- [x] InstalledBy enum (Anna, User, System, Unknown)
- [x] PackageManager enum (Pacman, Apt, Dnf, etc.)
- [x] PackageRecord / PackageTracker comprehensive tracker
- [x] record_install() / record_removal() - Management
- [x] anna_installed() / user_installed() - Filtering
- [x] by_package_manager() / installed() / removed()
- [x] format_package_tracker() / compact / oneline
- [x] package_fun_fact() / is_package_tracker_query()

### Phase 81 - Service Management Tracker ✓ (v0.0.505)
- [x] service_management_tracker.rs for service tracking
- [x] ServiceOperation enum (Start, Stop, Restart, etc.)
- [x] OperationResult enum (Success, Failed, Skipped, Pending)
- [x] ServiceRecord / ServiceTracker comprehensive tracker
- [x] by_operation_type() / for_service() - Filtering
- [x] success_rate() / most_managed() / most_common_op()
- [x] format_service_tracker() / compact / oneline
- [x] service_fun_fact() / is_service_tracker_query()

### Phase 82 - Config Change Tracker ✓ (v0.0.506)
- [x] config_change_tracker.rs for config file change tracking
- [x] ChangeType enum (Add, Modify, Delete, Replace, Append, Comment, Uncomment)
- [x] ConfigCategory enum (Shell, Editor, Git, System, Service, etc.)
- [x] ConfigChangeRecord / ConfigChangeTracker comprehensive tracker
- [x] detect_category() - Auto-detect config category from path
- [x] mark_rolled_back() / for_file() / by_config_category()
- [x] rolled_back() / active() / most_changed_file()
- [x] format_config_tracker() / compact / oneline
- [x] config_fun_fact() / is_config_tracker_query()

### Phase 83 - Helper Tracker ✓ (v0.0.507)
- [x] helper_tracker.rs for helper tool tracking
- [x] InstallerSource enum (Anna, User, System, Unknown)
- [x] HelperPurpose enum (SystemInfo, NetworkDiag, DiskUtil, etc.)
- [x] HelperRecord / HelperTracker comprehensive tracker
- [x] register() / record_usage() / mark_unavailable()
- [x] anna_installed() / user_installed() - Source filtering
- [x] removable_on_uninstall() - VISION.md feature
- [x] detect_purpose() - Auto-detect helper purpose
- [x] format_helper_tracker() / compact / oneline
- [x] helper_fun_fact() / is_helper_query()

### Phase 84 - Email Notification System ✓ (v0.0.508)
- [x] email_notification.rs for email tracking
- [x] NotificationStatus enum (Pending, Sent, Failed, etc.)
- [x] NotificationType enum (TaskComplete, TaskFailed, etc.)
- [x] EmailConfig with consent, daily limits, DND hours
- [x] NotificationRecord / EmailNotificationTracker
- [x] configure() / is_configured() - Consent management
- [x] daily_limit_reached() / reset_daily() - Rate limiting
- [x] pending() / sent() / failed() / success_rate()
- [x] format_email_tracker() / compact / oneline
- [x] email_fun_fact() / is_email_notification_query()

### Phase 85 - Idle Time Detector ✓ (v0.0.509)
- [x] idle_time_detector.rs for idle detection
- [x] IdleState enum (Active, Idle, DeepIdle, Suspended, Unknown)
- [x] ActivityLevel enum (High, Medium, Low, Minimal)
- [x] IdleConfig with thresholds, quiet hours
- [x] IdlePeriod / IdleTimeTracker comprehensive tracker
- [x] record_activity() / check_idle() - State management
- [x] can_do_background_work() / is_quiet_hours()
- [x] record_task_completed() - Background task tracking
- [x] avg_idle_duration() / longest_idle()
- [x] format_idle_tracker() / compact / oneline
- [x] idle_fun_fact() / is_idle_query()

### Phase 86 - Ticket Resolution Stats ✓ (v0.0.510)
- [x] ticket_resolution_stats.rs for resolution tracking
- [x] Resolver enum (Anna, Junior, Senior, Escalated, User)
- [x] ResolutionMethod enum (Recipe, Specialist, DirectAnswer, etc.)
- [x] ResolutionRecord / TicketResolutionStats tracker
- [x] record() / anna_rate() - Core tracking
- [x] by_res() / by_res_method() - Filtering
- [x] avg_resolution_time() / fastest / slowest
- [x] anna_improving() - Learning progress detection
- [x] format_resolution_stats() / compact / oneline
- [x] resolution_fun_fact() / is_resolution_stats_query()

### Phase 87 - Specialist Roster ✓ (v0.0.511)
- [x] specialist_roster.rs for team management
- [x] SpecialistLevel enum (Junior, Senior, Lead)
- [x] Department enum (Desktop, Network, Security, etc.)
- [x] SpecialistProfile / SpecialistRoster tracker
- [x] add() / get() / get_by_name() - Management
- [x] record_resolution() / set_available()
- [x] by_dept() / by_lvl() / juniors() / seniors()
- [x] top_performer() / SPECIALIST_NAMES
- [x] format_specialist_roster() / compact / oneline
- [x] roster_fun_fact() / is_specialist_roster_query()

### Phase 88 - LLM Assignment Tracker ✓ (v0.0.512)
- [x] llm_assignment.rs for model assignment tracking
- [x] ModelTier enum (Light, Standard, Heavy, DeepThinking)
- [x] AssignmentReason enum (Default, HardwareLimit, etc.)
- [x] LlmAssignment / LlmAssignmentTracker tracker
- [x] assign() / get_assignment() - Assignment management
- [x] add_available_model() / set_recommended_tier()
- [x] by_llm_model() / by_model_tier() - Filtering
- [x] models_in_use() / most_used_model()
- [x] COMMON_MODELS / get_model_tier()
- [x] format_llm_tracker() / compact / oneline
- [x] llm_fun_fact() / is_llm_query()

### Phase 89 - Dialogue Renderer ✓ (v0.0.513)
- [x] dialogue_renderer.rs for fly-on-the-wall display
- [x] Speaker enum (Anna, User, Junior, Senior, Lead, System)
- [x] DialogueMood enum (Neutral, Confident, Uncertain, etc.)
- [x] DialogueTurn / Dialogue conversation model
- [x] anna_says() / user_says() / specialist_says()
- [x] internal_turns() / external_turns() - Filtering
- [x] Color codes per speaker type
- [x] render_dialogue() / render_dialogue_plain()
- [x] render_dialogue_compact()
- [x] is_dialogue_query() / dialogue_fun_fact()

### Phase 90 - Alarm Scheduler ✓ (v0.0.514)
- [x] alarm_scheduler.rs for recurring notifications
- [x] AlarmFrequency enum (Once, Daily, Weekly, etc.)
- [x] DayOfWeek enum with name/short
- [x] AlarmStatus enum (Active, Paused, Triggered, etc.)
- [x] AlarmRecord / AlarmScheduler comprehensive system
- [x] add() / get() / trigger() - Management
- [x] pause() / resume() / cancel() - Status control
- [x] active() / due_at() - Query alarms
- [x] parse_day_of_week() - NL day parsing
- [x] format_alarm_scheduler() / compact / oneline
- [x] alarm_fun_fact() / is_alarm_query()

### Phase 91 - Strategic Thinking Tracker ✓ (v0.0.515)
- [x] strategic_thinking.rs for idle-time analysis
- [x] ThinkingStatus enum (Pending, InProgress, Paused, etc.)
- [x] ThinkingCategory / ThinkingPriority enums
- [x] ThinkingTask / StrategicThinkingTracker tracker
- [x] start() / pause() / resume() - Interruptible workflow
- [x] complete() - Findings and recommendations
- [x] pending() / paused() / completed() - Filtering
- [x] high_priority() / resume_point tracking
- [x] format_strategic_tracker() / compact / oneline
- [x] strategic_fun_fact() / is_strategic_query()

### Future
- All VISION.md features implemented!
