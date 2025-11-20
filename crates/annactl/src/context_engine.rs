//! Context Engine - Version 150
//!
//! Provides contextual awareness and proactive system administration.
//! This is the "brain" that makes Anna a professional sysadmin, not just a chatbot.
//!
//! Features:
//! 1. Session Context - Welcome messages, time-aware greetings, last login tracking
//! 2. System Monitoring - Proactive alerts for failures, storage, security
//! 3. Usage Patterns - Track common commands and provide intelligent suggestions
//! 4. Release Awareness - Notify about updates and show release notes
//!
//! Architecture:
//! - Stores context in ~/.local/share/anna/context.json
//! - Integrates with unified_query_handler to enrich all query types
//! - Runs health checks on startup and periodically
//! - Tracks user interaction patterns

use anyhow::Result;
use chrono::{DateTime, Local, Timelike, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// Context Engine - Tracks system and user context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextEngine {
    /// Last session timestamp
    pub last_session: Option<DateTime<Utc>>,

    /// Current session start time
    pub session_start: DateTime<Utc>,

    /// Anna version at last session
    pub last_version: Option<String>,

    /// Current Anna version
    pub current_version: String,

    /// System health alerts
    pub health_alerts: Vec<HealthAlert>,

    /// Usage patterns
    pub usage_patterns: UsagePatterns,

    /// Monitoring state
    pub monitoring: MonitoringState,
}

/// Health alert types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthAlert {
    pub alert_type: AlertType,
    pub severity: Severity,
    pub message: String,
    pub detected_at: DateTime<Utc>,
    pub acknowledged: bool,
}

/// Alert types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AlertType {
    FailedService,
    LowDiskSpace,
    SecurityIssue,
    PackageUpdates,
    SystemError,
    HighLoad,
    MemoryPressure,
}

/// Alert severity
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    Info,
    Warning,
    Critical,
}

/// Usage pattern tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsagePatterns {
    /// Command frequency (command -> count)
    pub command_frequency: HashMap<String, u32>,

    /// Question frequency (normalized question -> count)
    pub question_frequency: HashMap<String, u32>,

    /// Last used commands (recent history)
    pub recent_commands: Vec<String>,

    /// Common workflows detected
    pub workflows: Vec<Workflow>,
}

/// Detected workflow pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    pub name: String,
    pub commands: Vec<String>,
    pub frequency: u32,
}

/// Monitoring state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringState {
    /// Last health check timestamp
    pub last_health_check: Option<DateTime<Utc>>,

    /// Storage space last checked
    pub last_storage_check: Option<DateTime<Utc>>,

    /// Services last checked
    pub last_service_check: Option<DateTime<Utc>>,

    /// Known failed services (to avoid duplicate alerts)
    pub known_failed_services: Vec<String>,

    /// Storage warning threshold (GB)
    pub storage_warning_gb: f64,

    /// Storage critical threshold (GB)
    pub storage_critical_gb: f64,
}

impl Default for ContextEngine {
    fn default() -> Self {
        Self {
            last_session: None,
            session_start: Utc::now(),
            last_version: None,
            current_version: env!("CARGO_PKG_VERSION").to_string(),
            health_alerts: Vec::new(),
            usage_patterns: UsagePatterns::default(),
            monitoring: MonitoringState::default(),
        }
    }
}

impl Default for UsagePatterns {
    fn default() -> Self {
        Self {
            command_frequency: HashMap::new(),
            question_frequency: HashMap::new(),
            recent_commands: Vec::new(),
            workflows: Vec::new(),
        }
    }
}

impl Default for MonitoringState {
    fn default() -> Self {
        Self {
            last_health_check: None,
            last_storage_check: None,
            last_service_check: None,
            known_failed_services: Vec::new(),
            storage_warning_gb: 10.0,  // Warn at 10GB free
            storage_critical_gb: 5.0,   // Critical at 5GB free
        }
    }
}

impl ContextEngine {
    /// Load context from disk, or create new if doesn't exist
    pub fn load() -> Result<Self> {
        let path = Self::context_file_path()?;

        if path.exists() {
            let contents = fs::read_to_string(&path)?;
            let mut ctx: ContextEngine = serde_json::from_str(&contents)?;

            // Update session info
            ctx.last_session = Some(ctx.session_start);
            ctx.session_start = Utc::now();
            ctx.last_version = Some(ctx.current_version.clone());
            ctx.current_version = env!("CARGO_PKG_VERSION").to_string();

            Ok(ctx)
        } else {
            // First run
            Ok(ContextEngine::default())
        }
    }

    /// Save context to disk
    pub fn save(&self) -> Result<()> {
        let path = Self::context_file_path()?;

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let contents = serde_json::to_string_pretty(self)?;
        fs::write(&path, contents)?;

        Ok(())
    }

    /// Get context file path
    fn context_file_path() -> Result<PathBuf> {
        let data_dir = dirs::data_local_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not find local data directory"))?;
        Ok(data_dir.join("anna").join("context.json"))
    }

    /// Generate contextual greeting for session start
    pub fn generate_greeting(&self) -> String {
        let local_time = Local::now();
        let hour = local_time.hour();

        // Time-aware greeting
        let greeting = match hour {
            5..=11 => "Good morning",
            12..=17 => "Good afternoon",
            18..=21 => "Good evening",
            _ => "Hello",
        };

        // Check if this is first run
        if self.last_session.is_none() {
            return format!(
                "{}, I'm Anna, your Arch Linux system administrator.\n\
                 This is our first session together. I'm here to help you manage your system.\n\
                 Type 'help' to see what I can do, or just ask me anything!",
                greeting
            );
        }

        // Check if version changed (update/upgrade)
        let version_changed = self.last_version.as_ref() != Some(&self.current_version);

        // Build greeting with context
        let mut msg = format!("{}! Welcome back.", greeting);

        // Add time since last session
        if let Some(last) = self.last_session {
            let duration = Utc::now().signed_duration_since(last);
            if duration.num_days() >= 1 {
                msg.push_str(&format!("\nLast session was {} days ago.", duration.num_days()));
            } else if duration.num_hours() >= 1 {
                msg.push_str(&format!("\nLast session was {} hours ago.", duration.num_hours()));
            }
        }

        // Add version change notice
        if version_changed {
            msg.push_str(&format!(
                "\n\n🎉 Anna has been updated to version {}!",
                self.current_version
            ));
            if let Some(ref old_version) = self.last_version {
                msg.push_str(&format!(" (from {})", old_version));
            }
            msg.push_str("\nCheck release notes with: `anna release-notes`");
        }

        // Add health alerts summary
        let critical_alerts = self.health_alerts.iter()
            .filter(|a| !a.acknowledged && a.severity == Severity::Critical)
            .count();
        let warning_alerts = self.health_alerts.iter()
            .filter(|a| !a.acknowledged && a.severity == Severity::Warning)
            .count();

        if critical_alerts > 0 || warning_alerts > 0 {
            msg.push_str("\n\n⚠️  System Alerts:");
            if critical_alerts > 0 {
                msg.push_str(&format!("\n  • {} critical issue(s)", critical_alerts));
            }
            if warning_alerts > 0 {
                msg.push_str(&format!("\n  • {} warning(s)", warning_alerts));
            }
            msg.push_str("\nType 'alerts' to view details.");
        }

        msg
    }

    /// Run proactive health checks and populate alerts
    pub fn run_health_checks(&mut self, telemetry: &anna_common::telemetry::SystemTelemetry) {
        self.check_storage(telemetry);
        self.check_failed_services(telemetry);
        self.check_system_load(telemetry);

        self.monitoring.last_health_check = Some(Utc::now());
    }

    /// Check storage space
    fn check_storage(&mut self, telemetry: &anna_common::telemetry::SystemTelemetry) {
        for disk in &telemetry.disks {
            if disk.mount_point == "/" {
                let available_gb = (disk.total_mb - disk.used_mb) as f64 / 1024.0;

                // Clear old storage alerts for this mount
                self.health_alerts.retain(|a| {
                    !(a.alert_type == AlertType::LowDiskSpace && a.message.contains(&disk.mount_point))
                });

                if available_gb < self.monitoring.storage_critical_gb {
                    self.health_alerts.push(HealthAlert {
                        alert_type: AlertType::LowDiskSpace,
                        severity: Severity::Critical,
                        message: format!(
                            "Critical: Only {:.1} GB free on {} ({}% used)",
                            available_gb, disk.mount_point, disk.usage_percent
                        ),
                        detected_at: Utc::now(),
                        acknowledged: false,
                    });
                } else if available_gb < self.monitoring.storage_warning_gb {
                    self.health_alerts.push(HealthAlert {
                        alert_type: AlertType::LowDiskSpace,
                        severity: Severity::Warning,
                        message: format!(
                            "Warning: Only {:.1} GB free on {} ({}% used)",
                            available_gb, disk.mount_point, disk.usage_percent
                        ),
                        detected_at: Utc::now(),
                        acknowledged: false,
                    });
                }
            }
        }

        self.monitoring.last_storage_check = Some(Utc::now());
    }

    /// Check for failed systemd services
    fn check_failed_services(&mut self, telemetry: &anna_common::telemetry::SystemTelemetry) {
        // Get failed services from telemetry
        let failed_services: Vec<String> = telemetry.services.failed_units
            .iter()
            .map(|unit| unit.name.clone())
            .collect();

        // Clear old failed service alerts
        self.health_alerts.retain(|a| a.alert_type != AlertType::FailedService);

        // Add alerts for newly failed services
        for service in &failed_services {
            if !self.monitoring.known_failed_services.contains(service) {
                self.health_alerts.push(HealthAlert {
                    alert_type: AlertType::FailedService,
                    severity: Severity::Warning,
                    message: format!("Service failed: {}", service),
                    detected_at: Utc::now(),
                    acknowledged: false,
                });
            }
        }

        self.monitoring.known_failed_services = failed_services;
        self.monitoring.last_service_check = Some(Utc::now());
    }

    /// Check system load
    fn check_system_load(&mut self, telemetry: &anna_common::telemetry::SystemTelemetry) {
        let load_1min = telemetry.cpu.load_avg_1min;
        let cores = telemetry.cpu.cores as f64;

        // Clear old load alerts
        self.health_alerts.retain(|a| a.alert_type != AlertType::HighLoad);

        // Alert if 1-minute load average exceeds core count
        if load_1min > cores * 2.0 {
            self.health_alerts.push(HealthAlert {
                alert_type: AlertType::HighLoad,
                severity: Severity::Warning,
                message: format!(
                    "High system load: {:.2} ({}x normal for {} cores)",
                    load_1min,
                    load_1min / cores,
                    cores
                ),
                detected_at: Utc::now(),
                acknowledged: false,
            });
        }
    }

    /// Track a command execution
    pub fn track_command(&mut self, command: &str) {
        // Update frequency map
        *self.usage_patterns.command_frequency
            .entry(command.to_string())
            .or_insert(0) += 1;

        // Add to recent commands (keep last 50)
        self.usage_patterns.recent_commands.push(command.to_string());
        if self.usage_patterns.recent_commands.len() > 50 {
            self.usage_patterns.recent_commands.remove(0);
        }
    }

    /// Track a user question
    pub fn track_question(&mut self, question: &str) {
        // Normalize question (lowercase, remove punctuation)
        let normalized = question
            .to_lowercase()
            .trim_matches(|c: char| !c.is_alphanumeric() && c != ' ')
            .to_string();

        *self.usage_patterns.question_frequency
            .entry(normalized)
            .or_insert(0) += 1;
    }

    /// Get contextual suggestions based on current system state
    pub fn get_contextual_suggestions(&self, telemetry: &anna_common::telemetry::SystemTelemetry) -> Vec<String> {
        let mut suggestions = Vec::new();

        // Check for package updates
        if telemetry.packages.updates_available > 0 {
            suggestions.push(format!(
                "📦 {} package update(s) available. Run: sudo pacman -Syu",
                telemetry.packages.updates_available
            ));
        }

        // Check storage space
        for disk in &telemetry.disks {
            if disk.mount_point == "/" && disk.usage_percent > 80.0 {
                suggestions.push(
                    "💾 Root partition is over 80% full. Consider cleaning up with: \
                     sudo pacman -Sc (clear cache) or ncdu / (find large files)".to_string()
                );
            }
        }

        // Check failed services
        if !telemetry.services.failed_units.is_empty() {
            suggestions.push(format!(
                "⚙️  {} service(s) failed. Check with: systemctl --failed",
                telemetry.services.failed_units.len()
            ));
        }

        suggestions
    }
}
