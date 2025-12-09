//! Fallback keyword-based translator for when LLM fails.
//!
//! Extracted from translator.rs (v0.0.164) for modularization.

use anna_shared::answer_contract::AnswerContract;
use anna_shared::rpc::{QueryIntent, SpecialistDomain, TranslatorTicket};
use tracing::warn;

/// Fallback keyword-based translation (used when LLM fails)
pub fn translate_fallback(query: &str) -> TranslatorTicket {
    warn!("Using fallback keyword translator");
    let q = query.to_lowercase();

    // v0.0.30: Strip greetings before classification
    let stripped = strip_greetings(&q);

    // v0.0.30: Check for health/status queries FIRST (before domain classification)
    let is_health_query = stripped.contains("how is my computer")
        || stripped.contains("how's my computer")
        || stripped.contains("how is the system")
        || stripped.contains("any errors")
        || stripped.contains("any problems")
        || stripped.contains("problems so far")
        || stripped.contains("what's wrong")
        || stripped.contains("is everything ok")
        || stripped.contains("check my system")
        || stripped.contains("health")
        || stripped.contains("summary")
        || stripped.contains("status report")
        || stripped.contains("overview")
        || q.trim() == "status"
        || q.trim() == "report";

    if is_health_query {
        return TranslatorTicket {
            intent: QueryIntent::Question,
            domain: SpecialistDomain::System,
            entities: vec![],
            needs_probes: vec![
                "memory_info".to_string(),
                "disk_usage".to_string(),
                "cpu_info".to_string(),
                "failed_services".to_string(),
            ],
            clarification_question: None,
            confidence: 0.8, // Higher confidence for health queries
            answer_contract: Some(AnswerContract::from_query(query)),
        };
    }

    let domain = classify_domain(&q);
    let intent = classify_intent(&q);
    let needs_probes = select_probes(&q);

    TranslatorTicket {
        intent,
        domain,
        entities: Vec::new(),
        needs_probes,
        clarification_question: None,
        confidence: 0.3,
        answer_contract: Some(AnswerContract::from_query(query)),
    }
}

/// Classify domain from keywords
fn classify_domain(q: &str) -> SpecialistDomain {
    if q.contains("network")
        || q.contains("ip ")
        || q.contains("interface")
        || q.contains("dns")
        || q.contains("port")
        || q.contains("route")
    {
        SpecialistDomain::Network
    } else if q.contains("disk")
        || q.contains("storage")
        || q.contains("space")
        || q.contains("mount")
        || q.contains("partition")
    {
        SpecialistDomain::Storage
    } else if q.contains("security")
        || q.contains("firewall")
        || q.contains("permission")
        || q.contains("ssh")
    {
        SpecialistDomain::Security
    } else if q.contains("package")
        || q.contains("install")
        || q.contains("pacman")
        || q.contains("apt")
    {
        SpecialistDomain::Packages
    } else {
        SpecialistDomain::System
    }
}

/// Classify intent from keywords
fn classify_intent(q: &str) -> QueryIntent {
    if q.contains("install") || q.contains("start") || q.contains("stop") || q.contains("configure")
    {
        QueryIntent::Request
    } else if q.contains("why") || q.contains("debug") || q.contains("fix") {
        QueryIntent::Investigate
    } else {
        QueryIntent::Question
    }
}

/// Select probes based on keywords
fn select_probes(q: &str) -> Vec<String> {
    let mut needs_probes = Vec::new();
    if q.contains("memory") || q.contains("ram") {
        needs_probes.extend(["top_memory", "memory_info"].map(String::from));
    }
    if q.contains("cpu") {
        needs_probes.extend(["top_cpu", "cpu_info"].map(String::from));
    }
    if q.contains("disk") || q.contains("space") {
        needs_probes.push("disk_usage".to_string());
    }
    if q.contains("network") || q.contains("ip") {
        needs_probes.push("network_addrs".to_string());
    }
    if q.contains("port") || q.contains("listen") {
        needs_probes.push("listening_ports".to_string());
    }
    needs_probes
}

/// Strip greetings for fallback translator
fn strip_greetings(q: &str) -> String {
    let patterns = [
        "hello",
        "hi ",
        "hey ",
        "good morning",
        "good afternoon",
        "good evening",
        "anna",
        ":)",
        ":(",
        ";)",
        ":d",
        ":p",
        "!",
        "?",
        "…",
        "...",
    ];
    let mut result = q.to_string();
    for p in patterns {
        result = result.replace(p, " ");
    }
    result.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fallback_domain_classification() {
        let ticket = translate_fallback("show me memory usage");
        assert_eq!(ticket.domain, SpecialistDomain::System);
        assert!(ticket.needs_probes.contains(&"top_memory".to_string()));

        let ticket = translate_fallback("check network interfaces");
        assert_eq!(ticket.domain, SpecialistDomain::Network);
    }

    #[test]
    fn test_health_query_detection() {
        let ticket = translate_fallback("how is my computer doing?");
        assert_eq!(ticket.domain, SpecialistDomain::System);
        assert!(ticket.needs_probes.contains(&"memory_info".to_string()));
        assert!(ticket.needs_probes.contains(&"disk_usage".to_string()));
    }

    #[test]
    fn test_strip_greetings() {
        assert_eq!(strip_greetings("hello how are you"), "how are you");
        assert_eq!(strip_greetings("hi there :)"), "there");
    }
}
