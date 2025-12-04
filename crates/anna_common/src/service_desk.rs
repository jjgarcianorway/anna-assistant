//! Service Desk v0.0.59 - Departmental Routing Logic
//!
//! Two-step routing:
//! 1. Service Desk triage (Anna + Translator): classify department from targets/keywords/alerts
//! 2. Specialist dispatch (DoctorRegistry): pick doctor module(s) based on department/symptoms
//!
//! Rules:
//! - If doctor exists for department: use it
//! - If multiple likely: select primary, optionally consult secondary (max 2)
//! - If ambiguous: ask user for clarification

use crate::case_lifecycle::{CaseFileV2, CaseStatus, Department};
use crate::doctor_registry::{DoctorRegistry, DoctorSelection};
use crate::proactive_alerts::{AlertType, ProactiveAlertsState};

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
// Triage Result
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

    TriageResult {
        department: if final_score < 30 {
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
        assert!(result.confidence >= 30);
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
        let result = triage_request("help", &[]);
        assert_eq!(result.department, Department::ServiceDesk);
        assert!(result.confidence < 30);
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
}
