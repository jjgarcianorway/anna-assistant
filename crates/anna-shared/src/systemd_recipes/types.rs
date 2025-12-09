//! Systemd unit file recipe types (v0.0.233).

use serde::{Deserialize, Serialize};

/// Systemd unit type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UnitType {
    /// Regular service
    Service,
    /// Timer for scheduled tasks
    Timer,
    /// Mount point
    Mount,
    /// Socket activation
    Socket,
    /// Path-based activation
    Path,
}

impl UnitType {
    pub fn display_name(&self) -> &'static str {
        match self {
            UnitType::Service => "service",
            UnitType::Timer => "timer",
            UnitType::Mount => "mount",
            UnitType::Socket => "socket",
            UnitType::Path => "path",
        }
    }

    pub fn file_extension(&self) -> &'static str {
        match self {
            UnitType::Service => ".service",
            UnitType::Timer => ".timer",
            UnitType::Mount => ".mount",
            UnitType::Socket => ".socket",
            UnitType::Path => ".path",
        }
    }
}

impl std::fmt::Display for UnitType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

/// Systemd recipe features
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SystemdFeature {
    /// Create a basic service unit
    CreateService,
    /// Create a timer unit (replaces cron)
    CreateTimer,
    /// Create a user service (~/.config/systemd/user)
    CreateUserService,
    /// Enable/start a service
    EnableService,
    /// View service logs
    ViewLogs,
    /// Debug a failing service
    DebugService,
    /// Create a socket-activated service
    SocketActivation,
    /// Service hardening (security options)
    HardenService,
}

impl SystemdFeature {
    pub fn display_name(&self) -> &'static str {
        match self {
            SystemdFeature::CreateService => "create systemd service",
            SystemdFeature::CreateTimer => "create systemd timer",
            SystemdFeature::CreateUserService => "create user service",
            SystemdFeature::EnableService => "enable/start service",
            SystemdFeature::ViewLogs => "view service logs",
            SystemdFeature::DebugService => "debug failing service",
            SystemdFeature::SocketActivation => "socket activation",
            SystemdFeature::HardenService => "harden service",
        }
    }

    /// Keywords that indicate this feature
    pub fn keywords(&self) -> &'static [&'static str] {
        match self {
            SystemdFeature::CreateService => {
                &["create", "new", "write", "make", "unit", "service file"]
            }
            SystemdFeature::CreateTimer => &["timer", "schedule", "periodic", "cron", "interval"],
            SystemdFeature::CreateUserService => &["user", "user service", "local", "home"],
            SystemdFeature::EnableService => &["enable", "start", "activate", "boot"],
            SystemdFeature::ViewLogs => &["logs", "journalctl", "journal", "output"],
            SystemdFeature::DebugService => &["debug", "failing", "failed", "fix", "troubleshoot"],
            SystemdFeature::SocketActivation => &["socket", "activation", "on-demand", "inetd"],
            SystemdFeature::HardenService => &["harden", "secure", "sandbox", "security"],
        }
    }
}

/// Service restart policy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum RestartPolicy {
    /// Never restart
    No,
    /// Restart on failure
    #[default]
    OnFailure,
    /// Always restart
    Always,
    /// Restart on abnormal exit
    OnAbnormal,
}

impl RestartPolicy {
    pub fn as_str(&self) -> &'static str {
        match self {
            RestartPolicy::No => "no",
            RestartPolicy::OnFailure => "on-failure",
            RestartPolicy::Always => "always",
            RestartPolicy::OnAbnormal => "on-abnormal",
        }
    }
}

/// A systemd recipe
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemdRecipe {
    pub feature: SystemdFeature,
    pub description: String,
    pub unit_template: Option<String>,
    pub commands: Vec<String>,
    pub answer_template: String,
    pub notes: Vec<String>,
}

impl SystemdRecipe {
    pub fn new(feature: SystemdFeature, description: &str) -> Self {
        Self {
            feature,
            description: description.to_string(),
            unit_template: None,
            commands: Vec::new(),
            answer_template: String::new(),
            notes: Vec::new(),
        }
    }

    pub fn with_template(mut self, template: &str) -> Self {
        self.unit_template = Some(template.to_string());
        self
    }

    pub fn with_command(mut self, cmd: &str) -> Self {
        self.commands.push(cmd.to_string());
        self
    }

    pub fn with_answer(mut self, answer: &str) -> Self {
        self.answer_template = answer.to_string();
        self
    }

    pub fn with_note(mut self, note: &str) -> Self {
        self.notes.push(note.to_string());
        self
    }
}
