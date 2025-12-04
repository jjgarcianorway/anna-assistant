//! Intent Taxonomy v0.0.55 - Deterministic intent classification
//!
//! 5 canonical intents:
//! - SYSTEM_QUERY: "what CPU", "how much RAM", "disk space", "kernel version"
//! - DIAGNOSE: "X not working", "fix Y", problem phrases
//! - ACTION_REQUEST: "install X", "restart Y", "edit Z"
//! - HOWTO: "how do I", "how can I", "what is"
//! - META: "status", "what can you do", introspection

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
const SYSTEM_QUERY_PATTERNS: &[(&str, u8)] = &[
    ("what cpu", 95), ("which cpu", 90), ("cpu model", 90),
    ("what processor", 90), ("how many cores", 85), ("cpu info", 85),
    ("how much memory", 95), ("how much ram", 95), ("ram available", 90),
    ("total memory", 90), ("memory usage", 85),
    ("disk space", 95), ("free space", 90), ("disk usage", 85),
    ("how much space", 90), ("storage space", 85),
    ("kernel version", 95), ("what kernel", 90), ("linux version", 85),
    ("which kernel", 90), ("running kernel", 90),
    ("system info", 80), ("what's my", 75), ("show me my", 75),
    ("tell me my", 75), ("what is my", 80),
];

/// HOWTO patterns - "how do I" questions
const HOWTO_PATTERNS: &[(&str, u8)] = &[
    ("how do i", 95), ("how can i", 90), ("how to", 90),
    ("how would i", 85), ("what's the way to", 85),
    ("what is the command", 85), ("what command", 80),
    ("explain how", 85), ("teach me", 80), ("show me how", 90),
    ("guide me", 80), ("help me understand", 80),
];

/// ACTION_REQUEST patterns - mutation requests
const ACTION_PATTERNS: &[(&str, u8)] = &[
    ("install", 90), ("uninstall", 90), ("remove package", 90),
    ("restart", 85), ("start service", 90), ("stop service", 90),
    ("enable service", 85), ("disable service", 85),
    ("edit", 80), ("modify", 80), ("change", 75), ("update", 75),
    ("create file", 85), ("delete file", 90), ("add line", 85),
    ("append to", 85), ("set", 75),
];

/// META patterns - introspection and status
const META_PATTERNS: &[(&str, u8)] = &[
    ("status", 90), ("your status", 95), ("anna status", 95),
    ("what can you do", 95), ("what are you", 90),
    ("help", 70), ("show recipes", 90), ("list recipes", 90),
    ("show memory", 85), ("show cases", 85),
    ("version", 80), ("about anna", 90), ("who are you", 85),
    ("your level", 85), ("your xp", 85), ("show stats", 85),
];

// ============================================================================
// Classification Logic
// ============================================================================

/// Classify user request into canonical intent type
pub fn classify_intent(request: &str) -> IntentClassification {
    let request_lower = request.to_lowercase();
    let mut matched_patterns = Vec::new();

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

    // 2. Check for ACTION_REQUEST (mutation patterns)
    let (action_confidence, action_matches, action_type) = check_action_patterns(&request_lower);
    if action_confidence >= 75 {
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

    // 3. Check for META (introspection)
    let (meta_confidence, meta_matches) = check_patterns(&request_lower, META_PATTERNS);
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

    // 5. Check for SYSTEM_QUERY (factual questions about the system)
    let (query_confidence, query_matches) = check_patterns(&request_lower, SYSTEM_QUERY_PATTERNS);
    let (query_target, target_confidence) = detect_target(request);

    if query_confidence >= 70 || target_confidence >= 70 {
        let confidence = query_confidence.max(target_confidence);
        matched_patterns.extend(query_matches);
        return IntentClassification {
            intent: IntentType::SystemQuery,
            confidence,
            matched_patterns,
            query_target: if query_target != QueryTarget::Unknown { Some(query_target) } else { None },
            problem_domain: None,
            action_type: None,
            reasoning: "Detected system query".to_string(),
        };
    }

    // 6. Default: HOWTO for questions, SYSTEM_QUERY for statements
    if request_lower.contains('?') || request_lower.starts_with("what")
        || request_lower.starts_with("which") || request_lower.starts_with("where")
        || request_lower.starts_with("when") || request_lower.starts_with("why")
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
fn check_action_patterns(request: &str) -> (u8, Vec<String>, String) {
    let (confidence, matches) = check_patterns(request, ACTION_PATTERNS);

    // Determine action type from matches
    let action_type = if request.contains("install") {
        "package_install"
    } else if request.contains("uninstall") || request.contains("remove package") {
        "package_remove"
    } else if request.contains("restart") || request.contains("start service")
        || request.contains("stop service")
    {
        "service_action"
    } else if request.contains("edit") || request.contains("modify") || request.contains("add line") {
        "file_edit"
    } else {
        "unknown_action"
    };

    (confidence, matches, action_type.to_string())
}

/// Detect problem domain from request
fn detect_problem_domain(request: &str) -> String {
    if request.contains("wifi") || request.contains("network") || request.contains("internet")
        || request.contains("ethernet") || request.contains("connect")
    {
        "network".to_string()
    } else if request.contains("audio") || request.contains("sound") || request.contains("speaker")
        || request.contains("headphone") || request.contains("pipewire")
    {
        "audio".to_string()
    } else if request.contains("boot") || request.contains("startup") || request.contains("slow boot") {
        "boot".to_string()
    } else if request.contains("disk") || request.contains("storage") || request.contains("mount")
        || request.contains("btrfs")
    {
        "storage".to_string()
    } else if request.contains("gpu") || request.contains("graphics") || request.contains("display")
        || request.contains("screen") || request.contains("monitor")
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
}
