//! Service Desk v0.0.66 - IT Department Dispatcher
//!
//! The Service Desk is the central dispatcher for all user requests:
//! 1. Creates a Ticket for every request (id, category, severity, type)
//! 2. Determines routing via RoutingPlan (which doctors to invoke)
//! 3. Generates WorkOrder with goals and required evidence topics
//! 4. Generates HumanNarrationPlan for readable dialogue
//!
//! v0.0.66: Enhanced with TicketType classification and WorkOrder generation
//! - TicketType: question vs incident vs change_request
//! - WorkOrder: department, goals, required_topics
//! - Multi-department escalation support (max 2 departments)
//!
//! Severity heuristic:
//! - Problem reports ("wifi down", "no sound", "disk full") => elevated
//! - Informational queries ("how much memory") => low
//!
//! Routing rules:
//! - Problem reports route to specialist Doctors
//! - Informational queries use Evidence Topic Router (no doctors)
//!
//! v0.0.64: Full dispatcher model with Ticket, RoutingPlan, Severity

use crate::case_lifecycle::{CaseFileV2, CaseStatus, Department, TicketType};
use crate::department_protocol::{DepartmentName, RoutingDecision, WorkOrder};
use crate::doctor_registry::{DoctorRegistry, DoctorSelection};
use crate::evidence_topic::{detect_topic, EvidenceTopic};
use crate::proactive_alerts::{AlertType, ProactiveAlertsState};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// ============================================================================
// Triage Keywords
// ============================================================================

/// Keywords that map to departments
const NETWORKING_KEYWORDS: &[&str] = &[
    "wifi",
    "network",
    "internet",
    "dns",
    "ethernet",
    "connection",
    "ping",
    "route",
    "ip",
    "firewall",
    "vpn",
    "proxy",
    "socket",
    "port",
    "networkmanager",
    "nm-",
    "wlan",
    "wireless",
    "dhcp",
    "gateway",
];

const STORAGE_KEYWORDS: &[&str] = &[
    "disk",
    "storage",
    "space",
    "mount",
    "filesystem",
    "btrfs",
    "ext4",
    "partition",
    "drive",
    "ssd",
    "hdd",
    "nvme",
    "usb",
    "smart",
    "fstab",
    "lvm",
    "raid",
    "swap",
    "full",
];

const BOOT_KEYWORDS: &[&str] = &[
    "boot",
    "startup",
    "slow boot",
    "grub",
    "systemd",
    "init",
    "service",
    "failed",
    "timeout",
    "hung",
    "freeze",
    "reboot",
    "shutdown",
    "hibernate",
    "suspend",
    "resume",
];

const AUDIO_KEYWORDS: &[&str] = &[
    "audio",
    "sound",
    "speaker",
    "microphone",
    "headphone",
    "pipewire",
    "pulseaudio",
    "alsa",
    "bluetooth audio",
    "volume",
    "mute",
    "sink",
    "source",
    "wireplumber",
    "no sound",
];

const GRAPHICS_KEYWORDS: &[&str] = &[
    "graphics",
    "gpu",
    "display",
    "monitor",
    "screen",
    "resolution",
    "wayland",
    "x11",
    "xorg",
    "compositor",
    "nvidia",
    "amd",
    "intel",
    "mesa",
    "vulkan",
    "opengl",
    "tearing",
    "flicker",
    "xdg",
    "portal",
];

const SECURITY_KEYWORDS: &[&str] = &[
    "permission",
    "denied",
    "access",
    "sudo",
    "root",
    "password",
    "ssh",
    "gpg",
    "key",
    "certificate",
    "ssl",
    "tls",
    "firewall",
    "selinux",
    "apparmor",
    "polkit",
];

const PERFORMANCE_KEYWORDS: &[&str] = &[
    "slow",
    "performance",
    "cpu",
    "memory",
    "ram",
    "load",
    "freeze",
    "lag",
    "unresponsive",
    "thermal",
    "temperature",
    "hot",
    "throttle",
    "fan",
    "power",
    "battery",
];

// ============================================================================
// v0.0.64: Problem Report Detection
// ============================================================================

/// Keywords indicating a problem report (not just informational)
const PROBLEM_KEYWORDS: &[&str] = &[
    "down", "broken", "not working", "doesn't work", "can't", "cannot",
    "failed", "failing", "error", "problem", "issue", "wrong", "slow",
    "dead", "no sound", "no audio", "no internet", "no connection",
    "keeps", "disconnecting", "crashing", "freezing", "stuck",
    "won't", "help", "fix", "why is", "what's wrong",
];

/// Check if request looks like a problem report (vs informational query)
pub fn is_problem_report(request: &str) -> bool {
    let lower = request.to_lowercase();

    // Problem indicators
    for kw in PROBLEM_KEYWORDS {
        if lower.contains(kw) {
            return true;
        }
    }

    // Informational queries start with "what", "how much", "which", etc.
    let informational_starts = [
        "what is", "what's my", "how much", "how many",
        "which", "show me", "tell me about", "list",
    ];
    for start in informational_starts {
        if lower.starts_with(start) {
            return false;
        }
    }

    // Default: if we matched a department keyword, assume problem
    false
}

// ============================================================================
// v0.0.66: Ticket Type Detection
// ============================================================================

/// Keywords that indicate a change request
const CHANGE_REQUEST_KEYWORDS: &[&str] = &[
    "install", "remove", "uninstall", "update", "upgrade",
    "enable", "disable", "start", "stop", "restart",
    "configure", "setup", "set up", "change", "modify",
    "add", "delete", "create", "edit", "fix",
];

/// Detect the ticket type from request text
pub fn detect_ticket_type(request: &str) -> TicketType {
    let lower = request.to_lowercase();

    // Check for change request indicators first
    for kw in CHANGE_REQUEST_KEYWORDS {
        if lower.contains(kw) {
            // Make sure it's imperative, not just asking about it
            // "how do I install" is a question, "install X" is a change
            let is_imperative = lower.starts_with(kw)
                || lower.contains(&format!("please {}", kw))
                || lower.contains(&format!("can you {}", kw))
                || lower.contains(&format!("could you {}", kw));
            if is_imperative {
                return TicketType::ChangeRequest;
            }
        }
    }

    // Check for incident (problem report)
    if is_problem_report(request) {
        return TicketType::Incident;
    }

    // Default to question
    TicketType::Question
}

// ============================================================================
// v0.0.66: Work Order Generation
// ============================================================================

/// Create a work order from ticket and routing plan
pub fn create_work_order(ticket: &Ticket, routing: &RoutingPlan) -> WorkOrder {
    let department = DepartmentName::from_department(
        ticket.category.to_department()
    );

    let goals = generate_goals(ticket);
    let topics = routing.evidence_topics.clone();

    WorkOrder::new(department, goals, topics)
}

/// Generate investigation goals from ticket
fn generate_goals(ticket: &Ticket) -> Vec<String> {
    let mut goals = Vec::new();

    match ticket.category {
        TicketCategory::Networking => {
            goals.push("Check network interface status".to_string());
            goals.push("Verify IP configuration".to_string());
            goals.push("Test connectivity".to_string());
        }
        TicketCategory::Storage => {
            goals.push("Check disk usage".to_string());
            goals.push("Verify filesystem health".to_string());
            goals.push("Check for I/O errors".to_string());
        }
        TicketCategory::Audio => {
            goals.push("Check audio stack status".to_string());
            goals.push("Verify sink/source configuration".to_string());
            goals.push("Check for conflicts".to_string());
        }
        TicketCategory::Boot => {
            goals.push("Analyze boot timeline".to_string());
            goals.push("Check for slow services".to_string());
            goals.push("Look for recent changes".to_string());
        }
        TicketCategory::Graphics => {
            goals.push("Check display configuration".to_string());
            goals.push("Verify driver status".to_string());
            goals.push("Check compositor health".to_string());
        }
        TicketCategory::Performance => {
            goals.push("Check resource usage".to_string());
            goals.push("Identify top consumers".to_string());
            goals.push("Look for anomalies".to_string());
        }
        _ => {
            goals.push("Gather relevant information".to_string());
        }
    }

    goals
}

/// Create routing decision from triage result
pub fn create_routing_decision(triage: &TriageResult) -> RoutingDecision {
    let department = DepartmentName::from_department(triage.department);
    RoutingDecision::new(department, triage.confidence, &triage.reason)
        .with_keywords(triage.matched_keywords.clone())
}

// ============================================================================
// v0.0.64: Ticket and Severity
// ============================================================================

/// Severity level for tickets
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TicketSeverity {
    /// Read-only informational query
    Low,
    /// Something might be wrong
    Medium,
    /// User reports outage/failure
    High,
    /// Critical system issue
    Critical,
}

impl TicketSeverity {
    pub fn as_str(&self) -> &'static str {
        match self {
            TicketSeverity::Low => "low",
            TicketSeverity::Medium => "medium",
            TicketSeverity::High => "high",
            TicketSeverity::Critical => "critical",
        }
    }
}

impl std::fmt::Display for TicketSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Request category (maps to departments + general types)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TicketCategory {
    /// Network issues
    Networking,
    /// Storage/disk issues
    Storage,
    /// Audio/sound issues
    Audio,
    /// Boot/startup issues
    Boot,
    /// Graphics/display issues
    Graphics,
    /// Security/permissions issues
    Security,
    /// Performance issues
    Performance,
    /// Service management
    Services,
    /// Package management
    Packages,
    /// General system query
    General,
}

impl TicketCategory {
    pub fn from_department(dept: Department) -> Self {
        match dept {
            Department::Networking => TicketCategory::Networking,
            Department::Storage => TicketCategory::Storage,
            Department::Audio => TicketCategory::Audio,
            Department::Boot => TicketCategory::Boot,
            Department::Graphics => TicketCategory::Graphics,
            Department::Security => TicketCategory::Security,
            Department::Performance => TicketCategory::Performance,
            Department::ServiceDesk => TicketCategory::General,
        }
    }

    pub fn to_department(&self) -> Department {
        match self {
            TicketCategory::Networking => Department::Networking,
            TicketCategory::Storage => Department::Storage,
            TicketCategory::Audio => Department::Audio,
            TicketCategory::Boot => Department::Boot,
            TicketCategory::Graphics => Department::Graphics,
            TicketCategory::Security => Department::Security,
            TicketCategory::Performance => Department::Performance,
            TicketCategory::Services | TicketCategory::Packages | TicketCategory::General => {
                Department::ServiceDesk
            }
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            TicketCategory::Networking => "networking",
            TicketCategory::Storage => "storage",
            TicketCategory::Audio => "audio",
            TicketCategory::Boot => "boot",
            TicketCategory::Graphics => "graphics",
            TicketCategory::Security => "security",
            TicketCategory::Performance => "performance",
            TicketCategory::Services => "services",
            TicketCategory::Packages => "packages",
            TicketCategory::General => "general",
        }
    }

    pub fn human_label(&self) -> &'static str {
        match self {
            TicketCategory::Networking => "Network",
            TicketCategory::Storage => "Storage",
            TicketCategory::Audio => "Audio",
            TicketCategory::Boot => "Boot",
            TicketCategory::Graphics => "Graphics",
            TicketCategory::Security => "Security",
            TicketCategory::Performance => "Performance",
            TicketCategory::Services => "Services",
            TicketCategory::Packages => "Packages",
            TicketCategory::General => "General",
        }
    }
}

impl std::fmt::Display for TicketCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.human_label())
    }
}

/// Service Desk Ticket - created for every request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ticket {
    /// Ticket ID (format: A-YYYYMMDD-XXXX)
    pub id: String,
    /// Category of the request
    pub category: TicketCategory,
    /// Severity level
    pub severity: TicketSeverity,
    /// Confidence in categorization (0-100)
    pub confidence: u8,
    /// Suspected domains (departments that might be involved)
    pub suspected_domains: Vec<TicketCategory>,
    /// Whether this is a problem report (vs informational query)
    pub is_problem: bool,
    /// Original request text
    pub request: String,
    /// When ticket was created
    pub created_at: DateTime<Utc>,
    /// Matched keywords for categorization
    pub matched_keywords: Vec<String>,
}

impl Ticket {
    /// Generate a new ticket ID
    pub fn generate_id() -> String {
        let now = Utc::now();
        let random = simple_rand_u16();
        format!("A-{}-{:04}", now.format("%Y%m%d"), random)
    }
}

/// Simple random u16 (no external deps)
fn simple_rand_u16() -> u16 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    use std::time::Instant;

    let mut hasher = DefaultHasher::new();
    Instant::now().hash(&mut hasher);
    std::process::id().hash(&mut hasher);
    (hasher.finish() % 10000) as u16
}

// ============================================================================
// v0.0.64: Routing Plan
// ============================================================================

/// Routing plan - which doctors to invoke and what evidence to collect
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingPlan {
    /// Primary doctor to handle the request (if any)
    pub primary_doctor: Option<String>,
    /// Secondary doctors to consult (different domains)
    pub secondary_doctors: Vec<String>,
    /// Evidence topics to collect
    pub evidence_topics: Vec<EvidenceTopic>,
    /// Whether to use doctor flow (vs evidence topic router)
    pub use_doctor_flow: bool,
    /// Reasoning for this routing
    pub reasoning: String,
}

impl RoutingPlan {
    /// Create a plan for informational query (no doctor)
    pub fn informational(topic: EvidenceTopic) -> Self {
        Self {
            primary_doctor: None,
            secondary_doctors: Vec::new(),
            evidence_topics: vec![topic],
            use_doctor_flow: false,
            reasoning: "Informational query - using evidence topic router".to_string(),
        }
    }

    /// Create a plan that routes to a doctor
    pub fn doctor(doctor_name: &str, topics: Vec<EvidenceTopic>, reason: &str) -> Self {
        Self {
            primary_doctor: Some(doctor_name.to_string()),
            secondary_doctors: Vec::new(),
            evidence_topics: topics,
            use_doctor_flow: true,
            reasoning: reason.to_string(),
        }
    }
}

// ============================================================================
// v0.0.64: Human Narration Plan
// ============================================================================

/// Human narration plan - how to describe routing in Human Mode
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HumanNarrationPlan {
    /// Opening message (e.g., "Opening ticket #A-20251204-1234")
    pub opening: String,
    /// Triage summary (e.g., "Triage: Networking. Severity: medium.")
    pub triage_summary: String,
    /// Routing message (e.g., "Routing this to Network team.")
    pub routing: String,
    /// What evidence will be gathered (human descriptions)
    pub evidence_descriptions: Vec<String>,
}

impl HumanNarrationPlan {
    pub fn from_ticket_and_routing(ticket: &Ticket, routing: &RoutingPlan) -> Self {
        let opening = format!("Opening ticket #{}.", ticket.id);

        let triage_summary = format!(
            "Triage: {}. Severity: {}.",
            ticket.category, ticket.severity
        );

        let routing_msg = if let Some(doctor) = &routing.primary_doctor {
            format!("Routing this to {} team.", doctor)
        } else {
            "I'll gather the information for you.".to_string()
        };

        let evidence_descriptions = routing.evidence_topics.iter()
            .map(|t| t.human_label().to_string())
            .collect();

        Self {
            opening,
            triage_summary,
            routing: routing_msg,
            evidence_descriptions,
        }
    }
}

// ============================================================================
// v0.0.64: Dispatch Result
// ============================================================================

/// Complete dispatch result from Service Desk
#[derive(Debug, Clone)]
pub struct DispatchResult {
    /// The ticket created
    pub ticket: Ticket,
    /// Routing plan
    pub routing: RoutingPlan,
    /// Human narration plan
    pub narration: HumanNarrationPlan,
    /// Legacy triage result (for compatibility)
    pub triage: TriageResult,
    /// v0.0.66: Work order for department
    pub work_order: WorkOrder,
    /// v0.0.66: Routing decision
    pub routing_decision: RoutingDecision,
    /// v0.0.66: Ticket type
    pub ticket_type: TicketType,
}

/// Main dispatch function - entry point for all requests
pub fn dispatch_request(request: &str, targets: &[String]) -> DispatchResult {
    let request_lower = request.to_lowercase();

    // 1. Determine if this is a problem report
    let is_problem = is_problem_report(request);

    // 2. v0.0.66: Detect ticket type
    let ticket_type = detect_ticket_type(request);

    // 3. Run keyword-based triage
    let triage = triage_request(request, targets);

    // 4. Detect evidence topic (for informational queries)
    let topic_detection = detect_topic(&request_lower);

    // 5. Determine severity
    let severity = determine_severity(request, &triage, is_problem);

    // 6. Create ticket
    let ticket = Ticket {
        id: Ticket::generate_id(),
        category: TicketCategory::from_department(triage.department),
        severity,
        confidence: triage.confidence,
        suspected_domains: if let Some(sec) = triage.secondary {
            vec![TicketCategory::from_department(triage.department),
                 TicketCategory::from_department(sec)]
        } else {
            vec![TicketCategory::from_department(triage.department)]
        },
        is_problem,
        request: request.to_string(),
        created_at: Utc::now(),
        matched_keywords: triage.matched_keywords.clone(),
    };

    // 7. Create routing plan
    let routing = create_routing_plan(&ticket, &triage, topic_detection.topic, is_problem);

    // 8. Create narration plan
    let narration = HumanNarrationPlan::from_ticket_and_routing(&ticket, &routing);

    // 9. v0.0.66: Create work order and routing decision
    let work_order = create_work_order(&ticket, &routing);
    let routing_decision = create_routing_decision(&triage);

    DispatchResult {
        ticket,
        routing,
        narration,
        triage,
        work_order,
        routing_decision,
        ticket_type,
    }
}

/// Determine severity based on request analysis
fn determine_severity(request: &str, triage: &TriageResult, is_problem: bool) -> TicketSeverity {
    let lower = request.to_lowercase();

    // Critical: explicit outage words
    if lower.contains("completely")
        || lower.contains("total")
        || lower.contains("nothing works")
        || lower.contains("critical")
    {
        return TicketSeverity::Critical;
    }

    // High: outage indicators
    if lower.contains("down")
        || lower.contains("dead")
        || lower.contains("no internet")
        || lower.contains("no sound")
        || lower.contains("not booting")
        || lower.contains("can't boot")
    {
        return TicketSeverity::High;
    }

    // Medium: problem but not outage
    if is_problem || triage.linked_alerts.iter().any(|(_, t)| *t != AlertType::JournalErrorBurst) {
        return TicketSeverity::Medium;
    }

    // Low: informational
    TicketSeverity::Low
}

/// Create routing plan based on ticket and analysis
fn create_routing_plan(
    ticket: &Ticket,
    triage: &TriageResult,
    topic: EvidenceTopic,
    is_problem: bool,
) -> RoutingPlan {
    // If not a problem, use evidence topic router
    if !is_problem && topic != EvidenceTopic::Unknown {
        return RoutingPlan::informational(topic);
    }

    // Map category to doctor
    let doctor_name = match ticket.category {
        TicketCategory::Networking => Some("Network Doctor"),
        TicketCategory::Storage => Some("Storage Doctor"),
        TicketCategory::Audio => Some("Audio Doctor"),
        TicketCategory::Boot => Some("Boot Doctor"),
        TicketCategory::Graphics => Some("Graphics Doctor"),
        _ => None,
    };

    if let Some(name) = doctor_name {
        // Map category to relevant evidence topics
        let topics = category_to_evidence_topics(ticket.category);
        RoutingPlan::doctor(
            name,
            topics,
            &format!("Problem report routed to {}", name),
        )
    } else {
        // Fallback to evidence topic router
        if topic != EvidenceTopic::Unknown {
            RoutingPlan::informational(topic)
        } else {
            RoutingPlan {
                primary_doctor: None,
                secondary_doctors: Vec::new(),
                evidence_topics: vec![EvidenceTopic::Unknown],
                use_doctor_flow: false,
                reasoning: "General query - no specific doctor available".to_string(),
            }
        }
    }
}

/// Map category to relevant evidence topics
fn category_to_evidence_topics(category: TicketCategory) -> Vec<EvidenceTopic> {
    match category {
        TicketCategory::Networking => vec![EvidenceTopic::NetworkStatus],
        TicketCategory::Storage => vec![EvidenceTopic::DiskFree],
        TicketCategory::Audio => vec![EvidenceTopic::AudioStatus],
        TicketCategory::Boot => vec![EvidenceTopic::BootTime],
        TicketCategory::Graphics => vec![EvidenceTopic::GraphicsStatus],
        TicketCategory::Performance => vec![EvidenceTopic::MemoryInfo, EvidenceTopic::CpuInfo],
        _ => vec![EvidenceTopic::Unknown],
    }
}

// ============================================================================
// Triage Result (legacy, kept for compatibility)
// ============================================================================

/// Result of service desk triage
#[derive(Debug, Clone)]
pub struct TriageResult {
    /// Primary department
    pub department: Department,
    /// Confidence (0-100)
    pub confidence: u8,
    /// Matched keywords
    pub matched_keywords: Vec<String>,
    /// Reason for assignment
    pub reason: String,
    /// Secondary department if applicable
    pub secondary: Option<Department>,
    /// Linked alerts (if any)
    pub linked_alerts: Vec<(String, AlertType)>,
}

impl TriageResult {
    /// Check if triage is confident enough to proceed
    pub fn is_confident(&self) -> bool {
        self.confidence >= 60
    }

    /// Check if clarification is needed
    pub fn needs_clarification(&self) -> bool {
        self.confidence < 60 || self.secondary.is_some()
    }
}

// ============================================================================
// Service Desk Triage
// ============================================================================

/// Triage a request to determine department
pub fn triage_request(request: &str, targets: &[String]) -> TriageResult {
    let request_lower = request.to_lowercase();
    let mut scores: Vec<(Department, u8, Vec<String>)> = Vec::new();

    // Score each department
    scores.push(score_department(
        &request_lower,
        targets,
        Department::Networking,
        NETWORKING_KEYWORDS,
    ));
    scores.push(score_department(
        &request_lower,
        targets,
        Department::Storage,
        STORAGE_KEYWORDS,
    ));
    scores.push(score_department(
        &request_lower,
        targets,
        Department::Boot,
        BOOT_KEYWORDS,
    ));
    scores.push(score_department(
        &request_lower,
        targets,
        Department::Audio,
        AUDIO_KEYWORDS,
    ));
    scores.push(score_department(
        &request_lower,
        targets,
        Department::Graphics,
        GRAPHICS_KEYWORDS,
    ));
    scores.push(score_department(
        &request_lower,
        targets,
        Department::Security,
        SECURITY_KEYWORDS,
    ));
    scores.push(score_department(
        &request_lower,
        targets,
        Department::Performance,
        PERFORMANCE_KEYWORDS,
    ));

    // Sort by score descending
    scores.sort_by(|a, b| b.1.cmp(&a.1));

    let (primary_dept, primary_score, primary_keywords) = scores.remove(0);
    let secondary = if !scores.is_empty() && scores[0].1 > 30 {
        Some(scores[0].0)
    } else {
        None
    };

    // Check for linked alerts
    let linked_alerts = find_linked_alerts(&request_lower);

    // Adjust score based on alerts
    let final_score = if !linked_alerts.is_empty() {
        // Alert linkage boosts confidence
        primary_score.saturating_add(20).min(100)
    } else {
        primary_score
    };

    let reason = if primary_keywords.is_empty() {
        "No specific keywords matched; defaulting to service desk".to_string()
    } else {
        format!("Matched keywords: {}", primary_keywords.join(", "))
    };

    // v0.0.64: Lower threshold for department routing (was 30, now 15)
    // A single keyword match is enough to route to a department
    TriageResult {
        department: if final_score < 15 {
            Department::ServiceDesk
        } else {
            primary_dept
        },
        confidence: final_score,
        matched_keywords: primary_keywords,
        reason,
        secondary,
        linked_alerts,
    }
}

/// Score a department based on keyword matches
fn score_department(
    request: &str,
    targets: &[String],
    department: Department,
    keywords: &[&str],
) -> (Department, u8, Vec<String>) {
    let mut matched = Vec::new();
    let mut score: u32 = 0;

    // Check request text
    for keyword in keywords {
        if request.contains(keyword) {
            matched.push(keyword.to_string());
            score += 15;
        }
    }

    // Check targets (higher weight)
    for target in targets {
        let target_lower = target.to_lowercase();
        for keyword in keywords {
            if target_lower.contains(keyword) {
                if !matched.contains(&keyword.to_string()) {
                    matched.push(keyword.to_string());
                }
                score += 25;
            }
        }
    }

    // Cap at 100
    let final_score = score.min(100) as u8;

    (department, final_score, matched)
}

/// Find alerts that might be related to the request
fn find_linked_alerts(request: &str) -> Vec<(String, AlertType)> {
    let state = ProactiveAlertsState::load();
    let active = state.get_active();
    let mut linked = Vec::new();

    for alert in active {
        let should_link = match alert.alert_type {
            AlertType::DiskPressure => {
                request.contains("disk")
                    || request.contains("space")
                    || request.contains("storage")
                    || request.contains("full")
            }
            AlertType::BootRegression => {
                request.contains("boot")
                    || request.contains("slow")
                    || request.contains("startup")
            }
            AlertType::ServiceFailed => {
                request.contains("service")
                    || request.contains("failed")
                    || request.contains(&alert.dedupe_key)
            }
            AlertType::ThermalThrottling => {
                request.contains("hot")
                    || request.contains("thermal")
                    || request.contains("slow")
                    || request.contains("temperature")
            }
            AlertType::JournalErrorBurst => {
                request.contains("error") || request.contains(&alert.dedupe_key)
            }
        };

        if should_link {
            linked.push((alert.id.clone(), alert.alert_type));
        }
    }

    linked
}

// ============================================================================
// Specialist Dispatch
// ============================================================================

/// Dispatch to specialist doctor based on triage result
pub fn dispatch_to_specialist(
    triage: &TriageResult,
    registry: &DoctorRegistry,
    _symptoms: &[String],
) -> Option<DoctorSelection> {
    // Service desk doesn't dispatch to specialists
    if triage.department == Department::ServiceDesk {
        return None;
    }

    // Build request from matched keywords for doctor selection
    let request = triage.matched_keywords.join(" ");
    let intent_tags = vec![format!("{}", triage.department.actor_name())];

    // Select doctor from registry using the correct API
    registry.select_doctors(&request, &intent_tags)
}

// ============================================================================
// Auto-Case Opening from Alerts
// ============================================================================

/// Check if a case should be auto-opened from an alert
pub fn should_auto_open_case(request: &str) -> Option<(String, AlertType, Department)> {
    let linked = find_linked_alerts(&request.to_lowercase());
    if let Some((alert_id, alert_type)) = linked.first() {
        let dept = Department::from_alert_type(*alert_type);
        Some((alert_id.clone(), *alert_type, dept))
    } else {
        None
    }
}

/// Find existing case for an alert
pub fn find_case_for_alert(alert_id: &str) -> Option<CaseFileV2> {
    let cases_dir = std::path::Path::new(crate::case_lifecycle::CASE_FILES_DIR);
    if !cases_dir.exists() {
        return None;
    }

    for entry in std::fs::read_dir(cases_dir).ok()? {
        let entry = entry.ok()?;
        if entry.path().is_dir() {
            if let Ok(case) = crate::case_lifecycle::load_case_v2(&entry.file_name().to_string_lossy())
            {
                if case.linked_alert_ids.contains(&alert_id.to_string()) && case.status.is_active()
                {
                    return Some(case);
                }
            }
        }
    }

    None
}

/// Open or reuse a case for an alert-related request
pub fn open_case_for_alert(
    case_id: &str,
    request: &str,
    alert_id: &str,
    alert_type: AlertType,
) -> CaseFileV2 {
    // Check for existing case
    if let Some(existing) = find_case_for_alert(alert_id) {
        return existing;
    }

    // Create new case
    let mut case = CaseFileV2::new(case_id, request);
    let dept = Department::from_alert_type(alert_type);

    case.link_alert(alert_id, alert_type);
    case.assign_department(dept, &format!("linked to {:?} alert", alert_type));
    case.set_status(CaseStatus::Triaged);

    case
}

// ============================================================================
// Case Progression
// ============================================================================

/// Progress a case through the triage phase
pub fn progress_case_triage(case: &mut CaseFileV2, request: &str, targets: &[String]) {
    // Run triage
    let triage = triage_request(request, targets);

    // Update case
    case.targets = targets.to_vec();
    case.assign_department(triage.department, &triage.reason);

    // Link any found alerts
    for (alert_id, alert_type) in &triage.linked_alerts {
        case.link_alert(alert_id, *alert_type);
    }

    // Transition to triaged
    case.set_status(CaseStatus::Triaged);

    // Add translator as participant
    case.add_participant(crate::case_lifecycle::Participant::Translator);
}

/// Progress a case to investigating phase
pub fn progress_case_investigating(case: &mut CaseFileV2) {
    case.add_participant(crate::case_lifecycle::Participant::Junior);
    case.set_status(CaseStatus::Investigating);
}

/// Progress a case to plan_ready phase
pub fn progress_case_plan_ready(case: &mut CaseFileV2) {
    case.set_status(CaseStatus::PlanReady);
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_triage_networking() {
        let result = triage_request("my wifi keeps disconnecting", &[]);
        assert_eq!(result.department, Department::Networking);
        assert!(result.confidence >= 15);  // v0.0.64: lowered threshold
        assert!(result.matched_keywords.contains(&"wifi".to_string()));
    }

    #[test]
    fn test_triage_storage() {
        let result = triage_request("running out of disk space", &[]);
        assert_eq!(result.department, Department::Storage);
        assert!(result.matched_keywords.contains(&"disk".to_string()));
    }

    #[test]
    fn test_triage_boot() {
        let result = triage_request("why is my boot so slow", &[]);
        assert_eq!(result.department, Department::Boot);
        assert!(result.matched_keywords.contains(&"boot".to_string()));
    }

    #[test]
    fn test_triage_audio() {
        let result = triage_request("no sound from speakers", &[]);
        assert_eq!(result.department, Department::Audio);
    }

    #[test]
    fn test_triage_with_targets() {
        let result = triage_request("what's wrong", &["network_status".to_string()]);
        assert_eq!(result.department, Department::Networking);
        assert!(result.confidence >= 25);
    }

    #[test]
    fn test_triage_ambiguous() {
        let result = triage_request("hello", &[]);  // "help" triggers PROBLEM_KEYWORDS
        assert_eq!(result.department, Department::ServiceDesk);
        assert!(result.confidence < 15);
    }

    #[test]
    fn test_department_from_alert() {
        assert_eq!(
            Department::from_alert_type(AlertType::DiskPressure),
            Department::Storage
        );
        assert_eq!(
            Department::from_alert_type(AlertType::BootRegression),
            Department::Boot
        );
    }

    // v0.0.64: New dispatch tests
    #[test]
    fn test_is_problem_report() {
        // Problem reports
        assert!(is_problem_report("wifi keeps disconnecting"));
        assert!(is_problem_report("no sound from speakers"));
        assert!(is_problem_report("network is down"));
        assert!(is_problem_report("can't connect to internet"));
        assert!(is_problem_report("why is my boot so slow"));

        // Informational queries (NOT problems)
        assert!(!is_problem_report("how much memory do I have"));
        assert!(!is_problem_report("what is my kernel version"));
        assert!(!is_problem_report("show me disk usage"));
    }

    #[test]
    fn test_dispatch_problem_uses_doctor() {
        let dispatch = dispatch_request("wifi keeps disconnecting", &[]);

        assert!(dispatch.ticket.is_problem);
        assert!(dispatch.routing.use_doctor_flow);
        assert!(dispatch.routing.primary_doctor.is_some());
        assert!(dispatch.routing.primary_doctor.unwrap().contains("Network"));
    }

    #[test]
    fn test_dispatch_informational_no_doctor() {
        let dispatch = dispatch_request("how much memory do I have", &[]);

        assert!(!dispatch.ticket.is_problem);
        assert!(!dispatch.routing.use_doctor_flow);
        assert!(dispatch.routing.primary_doctor.is_none());
    }

    #[test]
    fn test_severity_high_for_outage() {
        let dispatch = dispatch_request("internet is down", &[]);
        assert_eq!(dispatch.ticket.severity, TicketSeverity::High);
    }

    #[test]
    fn test_severity_low_for_query() {
        let dispatch = dispatch_request("what kernel version am I running", &[]);
        assert_eq!(dispatch.ticket.severity, TicketSeverity::Low);
    }

    #[test]
    fn test_ticket_id_format() {
        let id = Ticket::generate_id();
        assert!(id.starts_with("A-"));
        assert!(id.contains("-"));
    }

    #[test]
    fn test_narration_plan_has_ticket_id() {
        let dispatch = dispatch_request("no sound", &[]);
        assert!(dispatch.narration.opening.contains(&dispatch.ticket.id));
    }

    // v0.0.66: Ticket Type Detection Tests
    #[test]
    fn test_ticket_type_question() {
        assert_eq!(detect_ticket_type("how much memory do I have"), TicketType::Question);
        assert_eq!(detect_ticket_type("what is my kernel version"), TicketType::Question);
        assert_eq!(detect_ticket_type("show me disk usage"), TicketType::Question);
    }

    #[test]
    fn test_ticket_type_incident() {
        assert_eq!(detect_ticket_type("wifi keeps disconnecting"), TicketType::Incident);
        assert_eq!(detect_ticket_type("no sound from speakers"), TicketType::Incident);
        assert_eq!(detect_ticket_type("network is down"), TicketType::Incident);
    }

    #[test]
    fn test_ticket_type_change_request() {
        assert_eq!(detect_ticket_type("install firefox"), TicketType::ChangeRequest);
        assert_eq!(detect_ticket_type("please restart the service"), TicketType::ChangeRequest);
        assert_eq!(detect_ticket_type("can you enable bluetooth"), TicketType::ChangeRequest);
    }

    // v0.0.66: Work Order Tests
    #[test]
    fn test_work_order_created() {
        let dispatch = dispatch_request("wifi disconnecting", &[]);
        assert!(!dispatch.work_order.goals.is_empty());
        assert!(dispatch.work_order.can_escalate());
    }

    #[test]
    fn test_routing_decision_created() {
        let dispatch = dispatch_request("disk is full", &[]);
        assert!(dispatch.routing_decision.confidence > 0);
        assert!(!dispatch.routing_decision.reason.is_empty());
    }

    #[test]
    fn test_dispatch_includes_ticket_type() {
        let dispatch = dispatch_request("how much memory do I have", &[]);
        assert_eq!(dispatch.ticket_type, TicketType::Question);

        let dispatch2 = dispatch_request("wifi not working", &[]);
        assert_eq!(dispatch2.ticket_type, TicketType::Incident);
    }
}
