//! Learning System v0.11.0
//!
//! Event-driven learning jobs and system mapping.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Learning job priority
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum JobPriority {
    /// Background, low urgency
    Low,
    /// Normal priority
    Normal,
    /// High priority, process soon
    High,
    /// Critical, process immediately
    Critical,
}

impl JobPriority {
    pub fn as_i32(&self) -> i32 {
        match self {
            JobPriority::Low => 0,
            JobPriority::Normal => 1,
            JobPriority::High => 2,
            JobPriority::Critical => 3,
        }
    }
}

/// Learning job status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum JobStatus {
    /// Waiting to be processed
    Pending,
    /// Currently being processed
    Processing,
    /// Completed successfully
    Completed,
    /// Failed, may retry
    Failed,
    /// Cancelled
    Cancelled,
}

/// Event type that triggers learning
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum LearningEvent {
    /// Package installed
    PackageAdded {
        name: String,
        version: Option<String>,
    },
    /// Package removed
    PackageRemoved { name: String },
    /// Package upgraded
    PackageUpgraded {
        name: String,
        old_version: String,
        new_version: String,
    },
    /// Service state changed
    ServiceChanged { name: String, state: String },
    /// GPU driver availability changed
    GpuDriverChanged {
        available: bool,
        driver: Option<String>,
    },
    /// Config file changed
    ConfigChanged { path: String },
    /// First install - map system
    InitialMapping { phase: MappingPhase },
    /// Scheduled refresh
    ScheduledRefresh { target: String },
    /// User query about a topic (for prioritization)
    UserQuery { topic: String },
    /// Manual learning request
    ManualRequest { description: String },
}

impl LearningEvent {
    pub fn event_type(&self) -> &'static str {
        match self {
            LearningEvent::PackageAdded { .. } => "pkg_added",
            LearningEvent::PackageRemoved { .. } => "pkg_removed",
            LearningEvent::PackageUpgraded { .. } => "pkg_upgraded",
            LearningEvent::ServiceChanged { .. } => "svc_changed",
            LearningEvent::GpuDriverChanged { .. } => "gpu_driver",
            LearningEvent::ConfigChanged { .. } => "cfg_changed",
            LearningEvent::InitialMapping { .. } => "initial_mapping",
            LearningEvent::ScheduledRefresh { .. } => "scheduled_refresh",
            LearningEvent::UserQuery { .. } => "user_query",
            LearningEvent::ManualRequest { .. } => "manual_request",
        }
    }

    pub fn default_priority(&self) -> JobPriority {
        match self {
            LearningEvent::GpuDriverChanged { .. } => JobPriority::High,
            LearningEvent::PackageAdded { .. } => JobPriority::Normal,
            LearningEvent::PackageRemoved { .. } => JobPriority::Normal,
            LearningEvent::PackageUpgraded { .. } => JobPriority::Normal,
            LearningEvent::ServiceChanged { .. } => JobPriority::Normal,
            LearningEvent::ConfigChanged { .. } => JobPriority::Low,
            LearningEvent::InitialMapping { .. } => JobPriority::High,
            LearningEvent::ScheduledRefresh { .. } => JobPriority::Low,
            LearningEvent::UserQuery { .. } => JobPriority::Normal,
            LearningEvent::ManualRequest { .. } => JobPriority::High,
        }
    }
}

/// System mapping phases
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MappingPhase {
    /// Hardware baseline (CPU, RAM, GPU, disks)
    Hardware,
    /// Core software (bootloader, kernel, init, DM)
    CoreSoftware,
    /// Desktop environment and WM
    Desktop,
    /// User context (shell, editor, apps)
    UserContext,
    /// Network configuration
    Network,
    /// Services and daemons
    Services,
}

impl MappingPhase {
    pub fn all() -> Vec<MappingPhase> {
        vec![
            MappingPhase::Hardware,
            MappingPhase::CoreSoftware,
            MappingPhase::Desktop,
            MappingPhase::UserContext,
            MappingPhase::Network,
            MappingPhase::Services,
        ]
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            MappingPhase::Hardware => "hardware",
            MappingPhase::CoreSoftware => "core_software",
            MappingPhase::Desktop => "desktop",
            MappingPhase::UserContext => "user_context",
            MappingPhase::Network => "network",
            MappingPhase::Services => "services",
        }
    }

    /// Probes needed for this phase
    pub fn required_probes(&self) -> Vec<&'static str> {
        match self {
            MappingPhase::Hardware => vec!["cpu.info", "mem.info", "disk.lsblk"],
            MappingPhase::CoreSoftware => vec!["system.kernel"],
            MappingPhase::Desktop => vec![], // Detected via packages
            MappingPhase::UserContext => vec![], // Detected via env and packages
            MappingPhase::Network => vec!["net.links", "net.addr", "net.routes", "dns.resolv"],
            MappingPhase::Services => vec![], // Detected via systemd queries
        }
    }
}

/// A learning job
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningJob {
    /// Unique job ID
    pub id: String,
    /// Event that triggered this job
    pub event: LearningEvent,
    /// Relevant entities
    pub entities: Vec<String>,
    /// Priority
    pub priority: JobPriority,
    /// Current status
    pub status: JobStatus,
    /// Created timestamp
    pub created_at: DateTime<Utc>,
    /// Processing started timestamp
    pub started_at: Option<DateTime<Utc>>,
    /// Completed timestamp
    pub completed_at: Option<DateTime<Utc>>,
    /// Retry count
    pub retries: u32,
    /// Maximum retries
    pub max_retries: u32,
    /// Error message if failed
    pub error: Option<String>,
    /// Facts created or updated by this job
    pub facts_affected: Vec<String>,
}

impl LearningJob {
    /// Create a new learning job
    pub fn new(event: LearningEvent) -> Self {
        let priority = event.default_priority();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            event,
            entities: Vec::new(),
            priority,
            status: JobStatus::Pending,
            created_at: Utc::now(),
            started_at: None,
            completed_at: None,
            retries: 0,
            max_retries: 3,
            error: None,
            facts_affected: Vec::new(),
        }
    }

    /// Create a job with specific entities
    pub fn with_entities(mut self, entities: Vec<String>) -> Self {
        self.entities = entities;
        self
    }

    /// Create a job with specific priority
    pub fn with_priority(mut self, priority: JobPriority) -> Self {
        self.priority = priority;
        self
    }

    /// Mark job as processing
    pub fn start(&mut self) {
        self.status = JobStatus::Processing;
        self.started_at = Some(Utc::now());
    }

    /// Mark job as completed
    pub fn complete(&mut self, facts_affected: Vec<String>) {
        self.status = JobStatus::Completed;
        self.completed_at = Some(Utc::now());
        self.facts_affected = facts_affected;
    }

    /// Mark job as failed
    pub fn fail(&mut self, error: &str) {
        self.retries += 1;
        if self.retries >= self.max_retries {
            self.status = JobStatus::Failed;
        } else {
            self.status = JobStatus::Pending; // Will be retried
        }
        self.error = Some(error.to_string());
    }

    /// Check if job can be retried
    pub fn can_retry(&self) -> bool {
        self.retries < self.max_retries
    }

    /// Get backoff duration for retry (exponential backoff)
    pub fn backoff_seconds(&self) -> u64 {
        // 30s, 60s, 120s, etc.
        30 * 2_u64.pow(self.retries.min(5))
    }
}

/// Proactive notice for user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProactiveNotice {
    /// Notice ID
    pub id: String,
    /// Severity level
    pub severity: NoticeSeverity,
    /// Short message
    pub message: String,
    /// Related entities
    pub entities: Vec<String>,
    /// Evidence sources
    pub evidence: Vec<String>,
    /// Confidence score
    pub confidence: f64,
    /// Created timestamp
    pub created_at: DateTime<Utc>,
    /// Whether it has been shown to user
    pub shown: bool,
}

/// Notice severity
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NoticeSeverity {
    /// Informational
    Info,
    /// Notable change
    Notable,
    /// Important, should see
    Important,
    /// Critical, must see
    Critical,
}

impl NoticeSeverity {
    pub fn as_str(&self) -> &'static str {
        match self {
            NoticeSeverity::Info => "info",
            NoticeSeverity::Notable => "notable",
            NoticeSeverity::Important => "important",
            NoticeSeverity::Critical => "critical",
        }
    }

    pub fn should_show(&self) -> bool {
        // Only show notable and above
        !matches!(self, NoticeSeverity::Info)
    }
}

impl ProactiveNotice {
    pub fn new(severity: NoticeSeverity, message: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            severity,
            message,
            entities: Vec::new(),
            evidence: Vec::new(),
            confidence: 0.9,
            created_at: Utc::now(),
            shown: false,
        }
    }

    pub fn with_entities(mut self, entities: Vec<String>) -> Self {
        self.entities = entities;
        self
    }

    pub fn with_evidence(mut self, evidence: Vec<String>) -> Self {
        self.evidence = evidence;
        self
    }

    pub fn with_confidence(mut self, confidence: f64) -> Self {
        self.confidence = confidence;
        self
    }

    pub fn mark_shown(&mut self) {
        self.shown = true;
    }
}

/// System mapping state
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MappingState {
    /// Whether initial mapping has been completed
    pub initial_complete: bool,
    /// Completed phases
    pub completed_phases: Vec<MappingPhase>,
    /// Last full mapping timestamp
    pub last_full_mapping: Option<DateTime<Utc>>,
    /// Last incremental refresh
    pub last_refresh: Option<DateTime<Utc>>,
}

impl MappingState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn needs_initial_mapping(&self) -> bool {
        !self.initial_complete
    }

    pub fn mark_phase_complete(&mut self, phase: MappingPhase) {
        if !self.completed_phases.contains(&phase) {
            self.completed_phases.push(phase);
        }
        if self.completed_phases.len() == MappingPhase::all().len() {
            self.initial_complete = true;
            self.last_full_mapping = Some(Utc::now());
        }
    }

    pub fn needs_refresh(&self, hours: i64) -> bool {
        match self.last_refresh {
            Some(ts) => {
                let age = Utc::now().signed_duration_since(ts);
                age.num_hours() >= hours
            }
            None => true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_learning_event_priority() {
        let pkg = LearningEvent::PackageAdded {
            name: "vim".to_string(),
            version: Some("9.0".to_string()),
        };
        assert_eq!(pkg.default_priority(), JobPriority::Normal);

        let gpu = LearningEvent::GpuDriverChanged {
            available: true,
            driver: Some("nvidia".to_string()),
        };
        assert_eq!(gpu.default_priority(), JobPriority::High);
    }

    #[test]
    fn test_learning_job_lifecycle() {
        let event = LearningEvent::PackageAdded {
            name: "neovim".to_string(),
            version: None,
        };
        let mut job = LearningJob::new(event);

        assert_eq!(job.status, JobStatus::Pending);

        job.start();
        assert_eq!(job.status, JobStatus::Processing);
        assert!(job.started_at.is_some());

        job.complete(vec!["pkg:neovim".to_string()]);
        assert_eq!(job.status, JobStatus::Completed);
        assert_eq!(job.facts_affected.len(), 1);
    }

    #[test]
    fn test_job_retry() {
        let event = LearningEvent::ScheduledRefresh {
            target: "hardware".to_string(),
        };
        let mut job = LearningJob::new(event);

        job.fail("Connection error");
        assert_eq!(job.status, JobStatus::Pending); // Can retry
        assert_eq!(job.retries, 1);

        job.fail("Connection error");
        job.fail("Connection error");
        assert_eq!(job.status, JobStatus::Failed); // Max retries reached
    }

    #[test]
    fn test_mapping_phases() {
        let mut state = MappingState::new();
        assert!(state.needs_initial_mapping());

        for phase in MappingPhase::all() {
            state.mark_phase_complete(phase);
        }

        assert!(!state.needs_initial_mapping());
        assert!(state.last_full_mapping.is_some());
    }

    #[test]
    fn test_proactive_notice() {
        let notice = ProactiveNotice::new(
            NoticeSeverity::Important,
            "GPU driver now available".to_string(),
        )
        .with_entities(vec!["gpu:0".to_string()])
        .with_confidence(0.95);

        assert!(notice.severity.should_show());
        assert!(!notice.shown);
    }
}
