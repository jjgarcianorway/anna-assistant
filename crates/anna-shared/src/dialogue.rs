//! Dialogue variety system for Service Desk Theatre (v0.0.87).
//!
//! Provides varied phrases for natural-feeling IT department conversations.
//! Uses deterministic randomness based on case IDs for consistency.

use crate::roster::{person_for, Tier};
use crate::teams::Team;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// Pick a varied dialogue line based on a seed (for consistency)
fn pick_varied<'a>(options: &[&'a str], seed: u64) -> &'a str {
    if options.is_empty() {
        return "";
    }
    let idx = (seed as usize) % options.len();
    options[idx]
}

/// Generate a hash seed from a string
pub fn seed_from_str(s: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    s.hash(&mut hasher);
    hasher.finish()
}

/// Anna's dispatch greetings (to team member)
pub fn anna_dispatch_greeting(team: Team, case_id: &str) -> String {
    let person = person_for(team, Tier::Junior);
    let seed = seed_from_str(case_id);
    let short_id = &case_id[..8.min(case_id.len())];

    let greetings = [
        format!("Hey {}! I have a case for you. {}", person.display_name, short_id),
        format!("{}, got a ticket coming your way. Case {}", person.display_name, short_id),
        format!("{}, when you have a moment - new request in. {}", person.display_name, short_id),
        format!("Quick one for you, {}. Case number {}", person.display_name, short_id),
    ];

    greetings[(seed as usize) % greetings.len()].clone()
}

/// Junior's acknowledgment phrases
pub fn junior_acknowledgment(_team: Team, seed: u64) -> String {
    let acks = [
        "Got it, Anna.",
        "On it.",
        "Looking at it now.",
        "Pulling up the data.",
        "Let me check.",
    ];

    pick_varied(&acks, seed).to_string()
}

/// Junior's approval phrases (when answer looks good)
pub fn junior_approval(score: u8, seed: u64) -> String {
    let phrases = if score >= 90 {
        vec![
            format!("Looks solid, confidence {}%. Good to go.", score),
            format!("All checks pass. {}% confidence. Approved.", score),
            format!("Data matches up. {}% - sending it back.", score),
        ]
    } else if score >= 80 {
        vec![
            format!("Looks good, confidence {}%.", score),
            format!("I've reviewed the data. {}% confident.", score),
            format!("Checks out. {}% confidence.", score),
        ]
    } else {
        vec![
            format!("It's acceptable. {}% confidence.", score),
            format!("Marginal but okay. {}%.", score),
        ]
    };

    phrases[(seed as usize) % phrases.len()].clone()
}

/// Junior's escalation phrases (when not confident)
pub fn junior_escalation_request(team: Team, score: u8, seed: u64) -> String {
    let senior = person_for(team, Tier::Senior);

    let phrases = [
        format!("{}, can you take a look? Only {}% confident.", senior.display_name, score),
        format!("Need a second opinion, {}. Score's at {}%.", senior.display_name, score),
        format!("Escalating to {}. {}% isn't enough.", senior.display_name, score),
        format!("{}, I'm not sure about this one. {}%.", senior.display_name, score),
    ];

    phrases[(seed as usize) % phrases.len()].clone()
}

/// Senior's response phrases
pub fn senior_response(approved: bool, seed: u64) -> String {
    let phrases = if approved {
        vec![
            "I've reviewed it. Looks correct.",
            "Confirmed. The analysis is sound.",
            "Good catch escalating, but it checks out.",
            "Verified. You can send this back.",
        ]
    } else {
        vec![
            "Let me handle this one.",
            "I see the issue. Give me a moment.",
            "Good call escalating. This needs work.",
        ]
    };

    phrases[(seed as usize) % phrases.len()].to_string()
}

/// Anna's response after team review
pub fn anna_after_review(had_escalation: bool, seed: u64) -> String {
    let phrases = if had_escalation {
        vec![
            "Thanks for waiting. I've consulted with the team.",
            "Apologies for the delay - wanted to make sure I got this right.",
            "Sorry for the wait. I checked with a specialist.",
        ]
    } else {
        vec![
            "Here's what I found.",
            "Got the information you need.",
            "Based on my checks:",
        ]
    };

    phrases[(seed as usize) % phrases.len()].to_string()
}

/// Team-specific checking phrases
pub fn team_checking_phrase(team: Team, seed: u64) -> String {
    let phrases = match team {
        Team::Storage => vec![
            "Checking disk space and mounts...",
            "Looking at storage configuration...",
            "Analyzing filesystem status...",
        ],
        Team::Performance => vec![
            "Checking CPU and memory usage...",
            "Analyzing system performance...",
            "Running performance diagnostics...",
        ],
        Team::Network => vec![
            "Checking network interfaces...",
            "Analyzing connectivity...",
            "Looking at network configuration...",
        ],
        Team::Hardware => vec![
            "Probing hardware configuration...",
            "Gathering hardware inventory...",
            "Checking device status...",
        ],
        Team::Services => vec![
            "Checking service statuses...",
            "Looking at systemd units...",
            "Analyzing service health...",
        ],
        _ => vec![
            "Checking system data...",
            "Running diagnostics...",
            "Gathering information...",
        ],
    };

    phrases[(seed as usize) % phrases.len()].to_string()
}

/// Anna greeting based on query domain
pub fn anna_domain_greeting(domain: &str) -> String {
    match domain.to_lowercase().as_str() {
        "storage" | "disk" => "Let me check that storage information for you.",
        "memory" | "ram" => "I'll look into the memory right away.",
        "network" | "wifi" => "Let me examine your network configuration.",
        "performance" | "cpu" | "slow" => "I'll analyze the system performance.",
        "service" | "services" => "Let me check those service statuses.",
        "security" => "I'll review the security information carefully.",
        "hardware" | "audio" => "Let me gather the hardware details.",
        "desktop" | "editor" => "I'll check that for you.",
        _ => "Let me look into that for you.",
    }.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_seed_deterministic() {
        let seed1 = seed_from_str("abc123");
        let seed2 = seed_from_str("abc123");
        assert_eq!(seed1, seed2);
    }

    #[test]
    fn test_different_seeds() {
        let seed1 = seed_from_str("abc123");
        let seed2 = seed_from_str("xyz789");
        assert_ne!(seed1, seed2);
    }

    #[test]
    fn test_junior_approval_varies() {
        let p1 = junior_approval(95, 0);
        let p2 = junior_approval(95, 1);
        // Different seeds should give different phrases
        // (may be same by chance, but usually different)
        assert!(p1.contains("95"));
        assert!(p2.contains("95"));
    }

    #[test]
    fn test_anna_dispatch_includes_name() {
        let greeting = anna_dispatch_greeting(Team::Storage, "case12345");
        assert!(greeting.contains("Lars")); // Storage junior
    }

    #[test]
    fn test_escalation_includes_senior() {
        let request = junior_escalation_request(Team::Network, 65, 0);
        assert!(request.contains("Ana")); // Network senior
    }

    #[test]
    fn test_domain_greetings() {
        assert!(anna_domain_greeting("storage").contains("storage"));
        assert!(anna_domain_greeting("network").contains("network"));
        assert!(anna_domain_greeting("unknown").contains("look into"));
    }
}
