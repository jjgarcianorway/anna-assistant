//! Intent Taxonomy v0.0.74 - Deterministic intent classification
//!
//! 5 canonical intents:
//! - SYSTEM_QUERY: "what CPU", "how much RAM", "disk space", "kernel version"
//! - DIAGNOSE: "X not working", "fix Y", problem phrases
//! - ACTION_REQUEST: "install X", "restart Y", "edit Z"
//! - HOWTO: "how do I", "how can I", "what is"
//! - META: "status", "what can you do", introspection
//!
//! v0.0.74: Strengthened deterministic classification
//! - INFO_QUERY patterns recognized before action patterns
//! - "is X running" NEVER classified as action
//! - Direct answers for RAM/kernel/disk/network queries

use serde::{Deserialize, Serialize};

use crate::case_engine::IntentType;
use crate::doctor_flow::detect_problem_phrase;
use crate::system_query_router::{detect_target, QueryTarget};

// ============================================================================
// Intent Classification Result
// ============================================================================

/// Result of intent classification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntentClassification {
    /// Primary intent type
    pub intent: IntentType,
    /// Confidence (0-100)
    pub confidence: u8,
    /// Matched patterns that led to this classification
    pub matched_patterns: Vec<String>,
    /// For SYSTEM_QUERY: the specific query target
    pub query_target: Option<QueryTarget>,
    /// For DIAGNOSE: detected problem domain
    pub problem_domain: Option<String>,
    /// For ACTION_REQUEST: the action type
    pub action_type: Option<String>,
    /// Reasoning for classification
    pub reasoning: String,
}

// ============================================================================
// Pattern Definitions
// ============================================================================

/// System query patterns with confidence
/// v0.0.74: Extended with more direct query patterns
const SYSTEM_QUERY_PATTERNS: &[(&str, u8)] = &[
    // CPU queries
    ("what cpu", 95),
    ("which cpu", 90),
    ("cpu model", 90),
    ("what processor", 90),
    ("how many cores", 85),
    ("cpu info", 85),
    ("cpu do i have", 95),
    // Memory queries
    ("how much memory", 95),
    ("how much ram", 95),
    ("ram available", 90),
    ("total memory", 90),
    ("memory usage", 85),
    ("ram do i have", 95),
    ("memory do i have", 95),
    // Disk queries
    ("disk space", 95),
    ("free space", 90),
    ("disk usage", 85),
    ("how much space", 90),
    ("storage space", 85),
    ("space free", 90),
    ("space left", 90),
    // Kernel queries
    ("kernel version", 95),
    ("what kernel", 90),
    ("linux version", 85),
    ("which kernel", 90),
    ("running kernel", 90),
    ("kernel am i", 95),
    // Network status queries (NOT diagnostic)
    ("network status", 90),
    ("network up", 85),
    ("is network up", 90),
    ("am i connected", 90),
    ("am i online", 90),
    // Service status queries (read-only checks)
    ("is running", 85),
    ("is started", 85),
    ("is enabled", 85),
    ("is active", 85),
    // General info queries
    ("system info", 80),
    ("what's my", 75),
    ("show me my", 75),
    ("tell me my", 75),
    ("what is my", 80),
    ("uptime", 85),
];

/// HOWTO patterns - "how do I" questions
const HOWTO_PATTERNS: &[(&str, u8)] = &[
    ("how do i", 95),
    ("how can i", 90),
    ("how to", 90),
    ("how would i", 85),
    ("what's the way to", 85),
    ("what is the command", 85),
    ("what command", 80),
    ("explain how", 85),
    ("teach me", 80),
    ("show me how", 90),
    ("guide me", 80),
    ("help me understand", 80),
];

/// ACTION_REQUEST patterns - mutation requests
/// v0.0.74: Added standalone service verbs (stop, start, kill)
const ACTION_PATTERNS: &[(&str, u8)] = &[
    // Package management
    ("install", 90),
    ("uninstall", 90),
    ("remove package", 90),
    // Service management - full patterns
    ("restart", 85),
    ("start service", 90),
    ("stop service", 90),
    ("enable service", 85),
    ("disable service", 85),
    // v0.0.74: Standalone service verbs at start of request
    // These are detected by position to avoid false positives
    // File operations
    ("edit", 80),
    ("modify", 80),
    ("change", 75),
    ("update", 75),
    ("create file", 85),
    ("delete file", 90),
    ("add line", 85),
    ("append to", 85),
    ("set", 75),
];

/// META patterns - introspection and status
const META_PATTERNS: &[(&str, u8)] = &[
    ("status", 90),
    ("your status", 95),
    ("anna status", 95),
    ("what can you do", 95),
    ("what are you", 90),
    ("help", 70),
    ("show recipes", 90),
    ("list recipes", 90),
    ("show memory", 85),
    ("show cases", 85),
    ("anna version", 80),
    ("your version", 80),
    ("annactl version", 85),
    ("about anna", 90),
    ("who are you", 85),
    ("your level", 85),
    ("your xp", 85),
    ("show stats", 85),
];

// ============================================================================
// Classification Logic
// ============================================================================

/// Classify user request into canonical intent type
/// v0.0.74: Reordered to check SYSTEM_QUERY before ACTION_REQUEST
pub fn classify_intent(request: &str) -> IntentClassification {
    let request_lower = request.to_lowercase();
    let mut matched_patterns = Vec::new();

    // v0.0.74: Detect if this is clearly a read-only query
    // These patterns should NEVER become action requests
    let is_clearly_read_only = is_read_only_query(&request_lower);

    // 1. Check for DIAGNOSE first (problem phrases have priority)
    let (is_problem, problem_confidence, problem_matches) = detect_problem_phrase(request);
    if is_problem && problem_confidence >= 25 {
        let domain = detect_problem_domain(&request_lower);
        return IntentClassification {
            intent: IntentType::Diagnose,
            confidence: problem_confidence,
            matched_patterns: problem_matches,
            query_target: None,
            problem_domain: Some(domain),
            action_type: None,
            reasoning: "Detected problem/diagnostic keywords".to_string(),
        };
    }

    // 2. Check for META first if it's about Anna/introspection
    // v0.0.74: META patterns with "your"/"anna" take priority over status queries
    let (meta_confidence, meta_matches) = check_patterns(&request_lower, META_PATTERNS);
    let is_anna_introspection = request_lower.contains("your ")
        || request_lower.contains("anna")
        || request_lower.contains("what can you");

    if meta_confidence >= 70 && is_anna_introspection {
        return IntentClassification {
            intent: IntentType::Meta,
            confidence: meta_confidence,
            matched_patterns: meta_matches,
            query_target: None,
            problem_domain: None,
            action_type: None,
            reasoning: "Detected meta/introspection request".to_string(),
        };
    }

    // 3. v0.0.74: Check for SYSTEM_QUERY BEFORE action patterns
    // This ensures "how much RAM" never becomes an action request
    let (query_confidence, query_matches) = check_patterns(&request_lower, SYSTEM_QUERY_PATTERNS);
    let (query_target, target_confidence) = detect_target(request);

    if query_confidence >= 70 || target_confidence >= 70 || is_clearly_read_only {
        let confidence = query_confidence
            .max(target_confidence)
            .max(if is_clearly_read_only { 85 } else { 0 });
        matched_patterns.extend(query_matches);
        return IntentClassification {
            intent: IntentType::SystemQuery,
            confidence,
            matched_patterns,
            query_target: if query_target != QueryTarget::Unknown {
                Some(query_target)
            } else {
                None
            },
            problem_domain: None,
            action_type: None,
            reasoning: "Detected system query".to_string(),
        };
    }

    // 4. Check for META (other introspection cases)
    if meta_confidence >= 70 {
        return IntentClassification {
            intent: IntentType::Meta,
            confidence: meta_confidence,
            matched_patterns: meta_matches,
            query_target: None,
            problem_domain: None,
            action_type: None,
            reasoning: "Detected meta/introspection request".to_string(),
        };
    }

    // 4. Check for HOWTO (explain/how-to questions)
    let (howto_confidence, howto_matches) = check_patterns(&request_lower, HOWTO_PATTERNS);
    if howto_confidence >= 75 {
        return IntentClassification {
            intent: IntentType::Howto,
            confidence: howto_confidence,
            matched_patterns: howto_matches,
            query_target: None,
            problem_domain: None,
            action_type: None,
            reasoning: "Detected how-to question".to_string(),
        };
    }

    // 5. Check for ACTION_REQUEST (mutation patterns) - AFTER system query
    // v0.0.74: Only if NOT clearly read-only
    let (action_confidence, action_matches, action_type) = check_action_patterns(&request_lower);
    if action_confidence >= 75 && !is_clearly_read_only {
        return IntentClassification {
            intent: IntentType::ActionRequest,
            confidence: action_confidence,
            matched_patterns: action_matches,
            query_target: None,
            problem_domain: None,
            action_type: Some(action_type),
            reasoning: "Detected action/mutation request".to_string(),
        };
    }

    // 6. Default: HOWTO for questions, SYSTEM_QUERY for statements
    if request_lower.contains('?')
        || request_lower.starts_with("what")
        || request_lower.starts_with("which")
        || request_lower.starts_with("where")
        || request_lower.starts_with("when")
        || request_lower.starts_with("why")
    {
        IntentClassification {
            intent: IntentType::Howto,
            confidence: 50,
            matched_patterns: vec!["question format".to_string()],
            query_target: None,
            problem_domain: None,
            action_type: None,
            reasoning: "Defaulted to HOWTO for question format".to_string(),
        }
    } else {
        IntentClassification {
            intent: IntentType::SystemQuery,
            confidence: 40,
            matched_patterns: vec![],
            query_target: None,
            problem_domain: None,
            action_type: None,
            reasoning: "Defaulted to SYSTEM_QUERY".to_string(),
        }
    }
}

/// v0.0.74: Detect if request is clearly a read-only query
/// These patterns should NEVER become action requests
fn is_read_only_query(request: &str) -> bool {
    // Direct informational patterns
    let info_patterns = [
        "how much",
        "how many",
        "tell me",
        "show me my",
        "display my",
        "do i have",
        "am i connected",
        "am i online",
        "is my network",
        "my disk space",
        "my memory",
        "my cpu",
        "my kernel",
    ];

    // If it matches info patterns, it's read-only
    if info_patterns.iter().any(|p| request.contains(p)) {
        return true;
    }

    // Check for mutation verbs FIRST - if present, NOT read-only
    let mutation_verbs = [
        "restart", "stop", "start", "enable", "disable", "install", "remove", "kill",
    ];
    let has_mutation_verb = mutation_verbs.iter().any(|v| {
        // Mutation verb at start or after space (not part of "is X running")
        request.starts_with(v) || request.contains(&format!(" {}", v))
    });

    if has_mutation_verb {
        return false;
    }

    // v0.0.74: Check for "is X running/started/enabled/active" pattern
    // This catches "is docker running", "is nginx started", etc.
    let is_service_check = (request.starts_with("is ") || request.contains(" is "))
        && (request.ends_with(" running")
            || request.ends_with(" started")
            || request.ends_with(" enabled")
            || request.ends_with(" active")
            || request.ends_with(" stopped")
            || request.ends_with(" disabled")
            || request.contains("running?")
            || request.contains(" status"));

    if is_service_check {
        return true;
    }

    // Also check for direct status patterns
    let status_patterns = ["is running", "is started", "is enabled", "is active"];
    status_patterns.iter().any(|p| request.contains(p))
}

/// Check patterns and return highest confidence match
fn check_patterns(request: &str, patterns: &[(&str, u8)]) -> (u8, Vec<String>) {
    let mut max_confidence: u8 = 0;
    let mut matches = Vec::new();

    for (pattern, confidence) in patterns {
        if request.contains(pattern) {
            matches.push(pattern.to_string());
            if *confidence > max_confidence {
                max_confidence = *confidence;
            }
        }
    }

    (max_confidence, matches)
}

/// Check action patterns and extract action type
/// v0.0.74: Enhanced to detect standalone service verbs like "stop docker"
fn check_action_patterns(request: &str) -> (u8, Vec<String>, String) {
    let (mut confidence, mut matches) = check_patterns(request, ACTION_PATTERNS);

    // v0.0.74: Check for standalone service verbs at start of request
    // "stop docker", "start nginx", "kill redis", etc.
    let service_verbs = [
        "stop ", "start ", "kill ", "restart ", "enable ", "disable ",
    ];
    for verb in service_verbs {
        if request.starts_with(verb) {
            // High confidence if request starts with a service verb
            if confidence < 85 {
                confidence = 85;
                matches.push(format!("{} (service verb)", verb.trim()));
            }
        }
    }

    // Determine action type from matches
    let action_type = if request.contains("install") {
        "package_install"
    } else if request.contains("uninstall") || request.contains("remove package") {
        "package_remove"
    } else if request.contains("restart")
        || request.contains("start service")
        || request.contains("stop service")
        || request.starts_with("stop ")
        || request.starts_with("start ")
        || request.starts_with("restart ")
        || request.starts_with("kill ")
        || request.starts_with("enable ")
        || request.starts_with("disable ")
    {
        "service_action"
    } else if request.contains("edit") || request.contains("modify") || request.contains("add line")
    {
        "file_edit"
    } else {
        "unknown_action"
    };

    (confidence, matches, action_type.to_string())
}

/// Detect problem domain from request
fn detect_problem_domain(request: &str) -> String {
    if request.contains("wifi")
        || request.contains("network")
        || request.contains("internet")
        || request.contains("ethernet")
        || request.contains("connect")
    {
        "network".to_string()
    } else if request.contains("audio")
        || request.contains("sound")
        || request.contains("speaker")
        || request.contains("headphone")
        || request.contains("pipewire")
    {
        "audio".to_string()
    } else if request.contains("boot")
        || request.contains("startup")
        || request.contains("slow boot")
    {
        "boot".to_string()
    } else if request.contains("disk")
        || request.contains("storage")
        || request.contains("mount")
        || request.contains("btrfs")
    {
        "storage".to_string()
    } else if request.contains("gpu")
        || request.contains("graphics")
        || request.contains("display")
        || request.contains("screen")
        || request.contains("monitor")
    {
        "graphics".to_string()
    } else {
        "general".to_string()
    }
}

/// Map problem domain to doctor domain
pub fn domain_to_doctor(domain: &str) -> Option<crate::doctor_registry::DoctorDomain> {
    match domain {
        "network" => Some(crate::doctor_registry::DoctorDomain::Network),
        "audio" => Some(crate::doctor_registry::DoctorDomain::Audio),
        "boot" => Some(crate::doctor_registry::DoctorDomain::Boot),
        "storage" => Some(crate::doctor_registry::DoctorDomain::Storage),
        "graphics" => Some(crate::doctor_registry::DoctorDomain::Graphics),
        "general" => Some(crate::doctor_registry::DoctorDomain::System),
        _ => None,
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_query_cpu() {
        let result = classify_intent("what cpu do I have");
        assert_eq!(result.intent, IntentType::SystemQuery);
        assert!(result.confidence >= 80);
    }

    #[test]
    fn test_system_query_disk() {
        let result = classify_intent("how much disk space is free");
        assert_eq!(result.intent, IntentType::SystemQuery);
        assert!(result.confidence >= 80);
    }

    #[test]
    fn test_system_query_memory() {
        let result = classify_intent("how much ram do I have");
        assert_eq!(result.intent, IntentType::SystemQuery);
        assert!(result.confidence >= 80);
    }

    #[test]
    fn test_system_query_kernel() {
        let result = classify_intent("what kernel version am I using");
        assert_eq!(result.intent, IntentType::SystemQuery);
        assert!(result.confidence >= 80);
    }

    #[test]
    fn test_diagnose_network() {
        let result = classify_intent("wifi keeps disconnecting");
        assert_eq!(result.intent, IntentType::Diagnose);
        assert_eq!(result.problem_domain, Some("network".to_string()));
    }

    #[test]
    fn test_diagnose_audio() {
        let result = classify_intent("no sound from speakers");
        assert_eq!(result.intent, IntentType::Diagnose);
        assert_eq!(result.problem_domain, Some("audio".to_string()));
    }

    #[test]
    fn test_diagnose_boot() {
        let result = classify_intent("my system has slow boot");
        assert_eq!(result.intent, IntentType::Diagnose);
        assert_eq!(result.problem_domain, Some("boot".to_string()));
    }

    #[test]
    fn test_action_install() {
        let result = classify_intent("install nginx");
        assert_eq!(result.intent, IntentType::ActionRequest);
        assert_eq!(result.action_type, Some("package_install".to_string()));
    }

    #[test]
    fn test_action_restart() {
        let result = classify_intent("restart nginx service");
        assert_eq!(result.intent, IntentType::ActionRequest);
        assert_eq!(result.action_type, Some("service_action".to_string()));
    }

    #[test]
    fn test_howto() {
        let result = classify_intent("how do I configure nginx");
        assert_eq!(result.intent, IntentType::Howto);
        assert!(result.confidence >= 80);
    }

    #[test]
    fn test_meta_status() {
        let result = classify_intent("what's your status");
        assert_eq!(result.intent, IntentType::Meta);
    }

    #[test]
    fn test_meta_capabilities() {
        let result = classify_intent("what can you do");
        assert_eq!(result.intent, IntentType::Meta);
    }

    // v0.0.74: Tests for read-only query detection
    #[test]
    fn test_is_docker_running_is_system_query() {
        let result = classify_intent("is docker running");
        assert_eq!(
            result.intent,
            IntentType::SystemQuery,
            "is X running should be SystemQuery, not ActionRequest"
        );
    }

    #[test]
    fn test_is_systemd_running_is_system_query() {
        let result = classify_intent("is systemd running");
        assert_eq!(
            result.intent,
            IntentType::SystemQuery,
            "is systemd running should be SystemQuery"
        );
    }

    #[test]
    fn test_service_status_check_is_system_query() {
        let result = classify_intent("what is the status of nginx");
        assert_eq!(
            result.intent,
            IntentType::SystemQuery,
            "service status check should be SystemQuery"
        );
    }

    #[test]
    fn test_network_status_is_system_query() {
        let result = classify_intent("what is my network status");
        assert_eq!(
            result.intent,
            IntentType::SystemQuery,
            "network status should be SystemQuery"
        );
    }

    #[test]
    fn test_am_i_connected_is_system_query() {
        let result = classify_intent("am I connected to the internet");
        assert_eq!(
            result.intent,
            IntentType::SystemQuery,
            "connectivity check should be SystemQuery"
        );
    }

    #[test]
    fn test_uptime_is_system_query() {
        let result = classify_intent("what is my uptime");
        assert_eq!(
            result.intent,
            IntentType::SystemQuery,
            "uptime should be SystemQuery"
        );
    }

    // Ensure actual mutations are still ActionRequest
    #[test]
    fn test_restart_nginx_is_action_request() {
        let result = classify_intent("restart nginx");
        assert_eq!(
            result.intent,
            IntentType::ActionRequest,
            "restart should still be ActionRequest"
        );
    }

    #[test]
    fn test_stop_docker_is_action_request() {
        let result = classify_intent("stop docker");
        assert_eq!(
            result.intent,
            IntentType::ActionRequest,
            "stop should still be ActionRequest. Got: {:?}, reasoning: {}",
            result.intent,
            result.reasoning
        );
    }

    #[test]
    fn test_is_read_only_stop_docker() {
        assert!(
            !is_read_only_query("stop docker"),
            "stop docker should NOT be read-only"
        );
    }

    #[test]
    fn test_is_read_only_is_docker_running() {
        assert!(
            is_read_only_query("is docker running"),
            "is docker running SHOULD be read-only"
        );
    }
}
