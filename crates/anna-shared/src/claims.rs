//! Claim extraction from LLM answer text.
//!
//! Extracts strictly-scoped, auditable claims that can be verified against
//! typed evidence from STRUCT-lite parsers. No fuzzy matching, no ML.
//!
//! # Supported Claim Types
//!
//! - **Numeric**: `<subject> uses <size>` (memory/process claims)
//! - **Percent**: `<mount> is <N>% full` (disk usage claims)
//! - **Status**: `<service> is <state>` (service state claims)
//!
//! # Intentionally NOT Parsed
//!
//! - Vague claims without subjects ("memory is low", "disk is full")
//! - Relative claims ("more than", "less than", "about", "around")
//! - Time-based claims ("was running", "will start")
//! - Comparative claims ("faster than", "bigger than")

use crate::parsers::{normalize_service_name, parse_display_size, resolve_mount_alias, ServiceState};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;

/// A canonically-keyed claim extracted from answer text.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Claim {
    /// Numeric claim: subject uses N bytes
    Numeric(NumericClaim),
    /// Percent claim: mount is N% full
    Percent(PercentClaim),
    /// Status claim: service is in state
    Status(StatusClaim),
}

/// Numeric claim about memory/process usage.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NumericClaim {
    /// Canonical subject key (lowercase, trimmed)
    pub subject: String,
    /// Claimed value in bytes (normalized from display units)
    pub bytes: u64,
    /// Original text that was parsed
    pub raw: String,
}

/// Disk usage percent claim.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PercentClaim {
    /// Canonical mount path (resolved from alias if needed)
    pub mount: String,
    /// Claimed percent used (0-100)
    pub percent: u8,
    /// Original text that was parsed
    pub raw: String,
}

/// Service status claim.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StatusClaim {
    /// Canonical service name (normalized with .service suffix)
    pub service: String,
    /// Claimed state
    pub state: ServiceState,
    /// Original text that was parsed
    pub raw: String,
}

// === Regex patterns (compiled once) ===

// Numeric: "<subject> uses <number><unit>" or "<subject> is using <number><unit>"
// Subject must be a word (process name, "memory", "RAM", etc.)
// Excludes vague patterns like "it uses" or "this uses"
static NUMERIC_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?i)\b([a-z][a-z0-9_-]*)\s+(?:uses?|is using|consuming|took)\s+(\d+(?:\.\d+)?)\s*(GB|GiB|MB|MiB|KB|KiB|B)"
    ).unwrap()
});

// Percent: "<mount> is <N>% full" or "<mount> at <N>%"
// Mount must be explicit path (starting with /) or recognized alias (root, home, etc.)
// The path pattern allows just "/" or "/path/segments"
static PERCENT_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?i)((?:/(?:[a-z0-9_-]+)?)+|root|home|var|tmp|boot)\s+(?:is|at)\s+(\d{1,3})%\s*(?:full|used|capacity)?"
    ).unwrap()
});

// Status: "<service> is <state>"
// Service must be a word, state must be known enum value
static STATUS_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?i)\b([a-z][a-z0-9_@-]*(?:\.service|\.socket|\.timer)?)\s+is\s+(running|active|failed|inactive|activating|deactivating|reloading)\b"
    ).unwrap()
});

/// Extract all auditable claims from answer text.
///
/// Returns only strictly-scoped claims that can be verified against typed evidence.
/// Vague, relative, or unkeyed claims are intentionally ignored.
pub fn extract_claims(answer: &str) -> Vec<Claim> {
    let mut claims = Vec::new();

    // Extract numeric claims
    for cap in NUMERIC_PATTERN.captures_iter(answer) {
        if let Some(claim) = parse_numeric_capture(&cap) {
            claims.push(Claim::Numeric(claim));
        }
    }

    // Extract percent claims
    for cap in PERCENT_PATTERN.captures_iter(answer) {
        if let Some(claim) = parse_percent_capture(&cap) {
            claims.push(Claim::Percent(claim));
        }
    }

    // Extract status claims
    for cap in STATUS_PATTERN.captures_iter(answer) {
        if let Some(claim) = parse_status_capture(&cap) {
            claims.push(Claim::Status(claim));
        }
    }

    claims
}

/// Parse a numeric claim from regex capture.
fn parse_numeric_capture(cap: &regex::Captures) -> Option<NumericClaim> {
    let subject = cap.get(1)?.as_str().to_lowercase();
    let number = cap.get(2)?.as_str();
    let unit = cap.get(3)?.as_str();

    // Skip vague subjects
    if matches!(subject.as_str(), "it" | "this" | "that" | "which") {
        return None;
    }

    // Construct size string and parse to bytes using display-friendly parser
    let size_str = format!("{}{}", number, unit);
    let bytes = parse_display_size(&size_str).ok()?;

    Some(NumericClaim {
        subject,
        bytes,
        raw: cap.get(0)?.as_str().to_string(),
    })
}

/// Parse a percent claim from regex capture.
fn parse_percent_capture(cap: &regex::Captures) -> Option<PercentClaim> {
    let mount_raw = cap.get(1)?.as_str();
    let percent_str = cap.get(2)?.as_str();

    // Resolve mount alias to canonical path
    let mount = if mount_raw.starts_with('/') {
        mount_raw.to_string()
    } else {
        // Must be a known alias, otherwise reject
        resolve_mount_alias(mount_raw)?.to_string()
    };

    let percent: u8 = percent_str.parse().ok()?;
    if percent > 100 {
        return None;
    }

    Some(PercentClaim {
        mount,
        percent,
        raw: cap.get(0)?.as_str().to_string(),
    })
}

/// Parse a status claim from regex capture.
fn parse_status_capture(cap: &regex::Captures) -> Option<StatusClaim> {
    let service_raw = cap.get(1)?.as_str();
    let state_str = cap.get(2)?.as_str();

    let service = normalize_service_name(service_raw);
    let state = ServiceState::from_str(state_str);

    // Reject Unknown state (shouldn't happen with our regex, but defensive)
    if state == ServiceState::Unknown {
        return None;
    }

    Some(StatusClaim {
        service,
        state,
        raw: cap.get(0)?.as_str().to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    // === Numeric claim extraction ===

    #[test]
    fn golden_extract_numeric_uses() {
        let claims = extract_claims("firefox uses 4.2GB of memory");
        assert_eq!(claims.len(), 1);
        if let Claim::Numeric(c) = &claims[0] {
            assert_eq!(c.subject, "firefox");
            // 4.2GB = 4.2 * 1024^3 = 4509715661 bytes (half-up)
            assert_eq!(c.bytes, 4_509_715_661);
        } else {
            panic!("Expected numeric claim");
        }
    }

    #[test]
    fn golden_extract_numeric_is_using() {
        let claims = extract_claims("chrome is using 2GiB");
        assert_eq!(claims.len(), 1);
        if let Claim::Numeric(c) = &claims[0] {
            assert_eq!(c.subject, "chrome");
            assert_eq!(c.bytes, 2_147_483_648); // 2 * 1024^3
        } else {
            panic!("Expected numeric claim");
        }
    }

    #[test]
    fn golden_extract_numeric_mb() {
        let claims = extract_claims("vim uses 50MB");
        assert_eq!(claims.len(), 1);
        if let Claim::Numeric(c) = &claims[0] {
            assert_eq!(c.subject, "vim");
            assert_eq!(c.bytes, 52_428_800); // 50 * 1024^2
        } else {
            panic!("Expected numeric claim");
        }
    }

    #[test]
    fn golden_reject_vague_numeric() {
        // "it uses" should not be extracted
        let claims = extract_claims("it uses 4GB");
        assert!(claims.is_empty());
    }

    // === Percent claim extraction ===

    #[test]
    fn golden_extract_percent_path() {
        let claims = extract_claims("/ is 85% full");
        assert_eq!(claims.len(), 1);
        if let Claim::Percent(c) = &claims[0] {
            assert_eq!(c.mount, "/");
            assert_eq!(c.percent, 85);
        } else {
            panic!("Expected percent claim");
        }
    }

    #[test]
    fn golden_extract_percent_alias() {
        let claims = extract_claims("root is 75% full");
        assert_eq!(claims.len(), 1);
        if let Claim::Percent(c) = &claims[0] {
            assert_eq!(c.mount, "/"); // resolved from alias
            assert_eq!(c.percent, 75);
        } else {
            panic!("Expected percent claim");
        }
    }

    #[test]
    fn golden_extract_percent_home() {
        let claims = extract_claims("home at 90% capacity");
        assert_eq!(claims.len(), 1);
        if let Claim::Percent(c) = &claims[0] {
            assert_eq!(c.mount, "/home");
            assert_eq!(c.percent, 90);
        } else {
            panic!("Expected percent claim");
        }
    }

    #[test]
    fn golden_reject_vague_percent() {
        // "disk is 85% full" without mount key → not auditable
        let claims = extract_claims("disk is 85% full");
        assert!(claims.is_empty());
    }

    // === Status claim extraction ===

    #[test]
    fn golden_extract_status_running() {
        let claims = extract_claims("nginx is running");
        assert_eq!(claims.len(), 1);
        if let Claim::Status(c) = &claims[0] {
            assert_eq!(c.service, "nginx.service");
            assert_eq!(c.state, ServiceState::Running);
        } else {
            panic!("Expected status claim");
        }
    }

    #[test]
    fn golden_extract_status_failed() {
        let claims = extract_claims("postgresql.service is failed");
        assert_eq!(claims.len(), 1);
        if let Claim::Status(c) = &claims[0] {
            assert_eq!(c.service, "postgresql.service");
            assert_eq!(c.state, ServiceState::Failed);
        } else {
            panic!("Expected status claim");
        }
    }

    #[test]
    fn golden_extract_status_inactive() {
        let claims = extract_claims("sshd is inactive");
        assert_eq!(claims.len(), 1);
        if let Claim::Status(c) = &claims[0] {
            assert_eq!(c.service, "sshd.service");
            assert_eq!(c.state, ServiceState::Inactive);
        } else {
            panic!("Expected status claim");
        }
    }

    // === Multiple claims ===

    #[test]
    fn golden_extract_multiple_claims() {
        let answer = "firefox uses 4GB and nginx is running. root is 80% full.";
        let claims = extract_claims(answer);
        assert_eq!(claims.len(), 3);
    }

    // === No claims ===

    #[test]
    fn golden_extract_no_claims() {
        let claims = extract_claims("Everything looks fine on your system.");
        assert!(claims.is_empty());
    }

    #[test]
    fn golden_extract_vague_statements() {
        // None of these should produce claims
        let claims = extract_claims(
            "Memory is low. Disk is almost full. The service might be down.",
        );
        assert!(claims.is_empty());
    }
}
