//! Status Enhancements for Anna v0.0.81
//!
//! Additional status information:
//! - G1: Debug mode indicator
//! - G2: Last 3 cases
//! - G3: Mutation stats (mutations/rollbacks)
//! - G4: Doctor stats (invocations/domains)

use crate::case_file_v081::{CaseFileV081, CaseSummaryV081, CASES_DIR};
use crate::transcript_config::get_transcript_mode;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;

/// Mutation stats directory
pub const MUTATION_STATS_FILE: &str = "/var/lib/anna/internal/mutation_stats.json";

/// Doctor stats directory
pub const DOCTOR_STATS_FILE: &str = "/var/lib/anna/internal/doctor_stats.json";

/// Enhanced status for v0.0.81
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusEnhancedV081 {
    /// Debug mode status
    pub debug_mode: DebugModeStatus,
    /// Last few cases
    pub recent_cases: Vec<CaseSummaryV081>,
    /// Mutation statistics
    pub mutation_stats: MutationStats,
    /// Doctor statistics
    pub doctor_stats: DoctorStats,
}

/// Debug mode status (G1)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugModeStatus {
    /// Whether debug mode is active
    pub enabled: bool,
    /// How it was enabled (env var, config, etc.)
    pub source: String,
}

/// Mutation statistics (G3)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MutationStats {
    /// Total mutations executed
    pub total_mutations: u64,
    /// Successful mutations
    pub successful: u64,
    /// Failed mutations
    pub failed: u64,
    /// Rollbacks performed
    pub rollbacks: u64,
    /// Mutations by type
    pub by_type: HashMap<String, u64>,
    /// Last mutation timestamp
    pub last_mutation: Option<DateTime<Utc>>,
}

/// Doctor statistics (G4)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DoctorStats {
    /// Total doctor invocations
    pub total_invocations: u64,
    /// Successful diagnoses
    pub successful_diagnoses: u64,
    /// Invocations by domain
    pub by_domain: HashMap<String, u64>,
    /// Most common domain
    pub most_common_domain: Option<String>,
    /// Last invocation timestamp
    pub last_invocation: Option<DateTime<Utc>>,
}

impl MutationStats {
    /// Load from disk
    pub fn load() -> Self {
        fs::read_to_string(MUTATION_STATS_FILE)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    /// Save to disk
    pub fn save(&self) -> Result<(), String> {
        let dir = std::path::Path::new(MUTATION_STATS_FILE)
            .parent()
            .ok_or("Invalid path")?;
        fs::create_dir_all(dir).map_err(|e| format!("Cannot create dir: {}", e))?;

        let json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Cannot serialize: {}", e))?;
        fs::write(MUTATION_STATS_FILE, json).map_err(|e| format!("Cannot write: {}", e))?;
        Ok(())
    }

    /// Record a mutation
    pub fn record_mutation(&mut self, mutation_type: &str, success: bool) {
        self.total_mutations += 1;
        if success {
            self.successful += 1;
        } else {
            self.failed += 1;
        }

        *self.by_type.entry(mutation_type.to_string()).or_insert(0) += 1;
        self.last_mutation = Some(Utc::now());
    }

    /// Record a rollback
    pub fn record_rollback(&mut self) {
        self.rollbacks += 1;
    }

    /// Format for display
    pub fn format_human(&self) -> String {
        let mut lines = Vec::new();

        lines.push(format!(
            "Mutations: {} total ({} ok, {} failed, {} rollbacks)",
            self.total_mutations, self.successful, self.failed, self.rollbacks
        ));

        if !self.by_type.is_empty() {
            let mut types: Vec<_> = self.by_type.iter().collect();
            types.sort_by(|a, b| b.1.cmp(a.1));
            let type_strs: Vec<_> = types.iter().take(3).map(|(k, v)| format!("{}: {}", k, v)).collect();
            lines.push(format!("  By type: {}", type_strs.join(", ")));
        }

        if let Some(last) = self.last_mutation {
            lines.push(format!("  Last: {}", last.format("%Y-%m-%d %H:%M")));
        }

        lines.join("\n")
    }
}

impl DoctorStats {
    /// Load from disk
    pub fn load() -> Self {
        fs::read_to_string(DOCTOR_STATS_FILE)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    /// Save to disk
    pub fn save(&self) -> Result<(), String> {
        let dir = std::path::Path::new(DOCTOR_STATS_FILE)
            .parent()
            .ok_or("Invalid path")?;
        fs::create_dir_all(dir).map_err(|e| format!("Cannot create dir: {}", e))?;

        let json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Cannot serialize: {}", e))?;
        fs::write(DOCTOR_STATS_FILE, json).map_err(|e| format!("Cannot write: {}", e))?;
        Ok(())
    }

    /// Record a doctor invocation
    pub fn record_invocation(&mut self, domain: &str, success: bool) {
        self.total_invocations += 1;
        if success {
            self.successful_diagnoses += 1;
        }

        *self.by_domain.entry(domain.to_string()).or_insert(0) += 1;
        self.last_invocation = Some(Utc::now());

        // Update most common domain
        if let Some((top_domain, _)) = self.by_domain.iter().max_by_key(|(_, &count)| count) {
            self.most_common_domain = Some(top_domain.clone());
        }
    }

    /// Format for display
    pub fn format_human(&self) -> String {
        let mut lines = Vec::new();

        lines.push(format!(
            "Doctor: {} invocations ({} successful)",
            self.total_invocations, self.successful_diagnoses
        ));

        if !self.by_domain.is_empty() {
            let mut domains: Vec<_> = self.by_domain.iter().collect();
            domains.sort_by(|a, b| b.1.cmp(a.1));
            let domain_strs: Vec<_> = domains
                .iter()
                .take(3)
                .map(|(k, v)| format!("{}: {}", k, v))
                .collect();
            lines.push(format!("  By domain: {}", domain_strs.join(", ")));
        }

        if let Some(ref domain) = self.most_common_domain {
            lines.push(format!("  Most common: {}", domain));
        }

        if let Some(last) = self.last_invocation {
            lines.push(format!("  Last: {}", last.format("%Y-%m-%d %H:%M")));
        }

        lines.join("\n")
    }
}

/// Get debug mode status (G1)
pub fn get_debug_mode_status() -> DebugModeStatus {
    let mode = get_transcript_mode();
    let is_debug = matches!(mode, crate::transcript_config::TranscriptMode::Debug);

    let source = if std::env::var("ANNA_DEBUG_TRANSCRIPT").is_ok() {
        "ANNA_DEBUG_TRANSCRIPT env var"
    } else if std::env::var("ANNA_UI_TRANSCRIPT_MODE")
        .map(|v| v.eq_ignore_ascii_case("debug"))
        .unwrap_or(false)
    {
        "ANNA_UI_TRANSCRIPT_MODE env var"
    } else if is_debug {
        "config file"
    } else {
        "default (human)"
    };

    DebugModeStatus {
        enabled: is_debug,
        source: source.to_string(),
    }
}

/// Get recent cases (G2)
pub fn get_recent_cases(limit: usize) -> Vec<CaseSummaryV081> {
    CaseFileV081::list_recent(limit)
}

/// Get enhanced status
pub fn get_enhanced_status() -> StatusEnhancedV081 {
    StatusEnhancedV081 {
        debug_mode: get_debug_mode_status(),
        recent_cases: get_recent_cases(3),
        mutation_stats: MutationStats::load(),
        doctor_stats: DoctorStats::load(),
    }
}

/// Format enhanced status for display
pub fn format_enhanced_status(status: &StatusEnhancedV081) -> String {
    let mut lines = Vec::new();

    // Debug mode (G1)
    lines.push(format!(
        "Transcript mode: {} ({})",
        if status.debug_mode.enabled { "DEBUG" } else { "human" },
        status.debug_mode.source
    ));
    lines.push(String::new());

    // Recent cases (G2)
    lines.push("Recent Cases:".to_string());
    if status.recent_cases.is_empty() {
        lines.push("  (no recent cases)".to_string());
    } else {
        for case in &status.recent_cases {
            let mut_indicator = if case.is_mutation { " [MUT]" } else { "" };
            lines.push(format!(
                "  {} {} ({}%){}",
                case.created_at.format("%Y-%m-%d %H:%M"),
                truncate(&case.request, 35),
                case.reliability_score,
                mut_indicator
            ));
        }
    }
    lines.push(String::new());

    // Mutation stats (G3)
    lines.push(status.mutation_stats.format_human());
    lines.push(String::new());

    // Doctor stats (G4)
    lines.push(status.doctor_stats.format_human());

    lines.join("\n")
}

/// Truncate string for display
fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max.saturating_sub(3)])
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mutation_stats_default() {
        let stats = MutationStats::default();
        assert_eq!(stats.total_mutations, 0);
        assert_eq!(stats.rollbacks, 0);
    }

    #[test]
    fn test_mutation_stats_record() {
        let mut stats = MutationStats::default();
        stats.record_mutation("service_restart", true);
        stats.record_mutation("file_edit", true);
        stats.record_mutation("service_restart", false);

        assert_eq!(stats.total_mutations, 3);
        assert_eq!(stats.successful, 2);
        assert_eq!(stats.failed, 1);
        assert_eq!(*stats.by_type.get("service_restart").unwrap(), 2);
    }

    #[test]
    fn test_doctor_stats_record() {
        let mut stats = DoctorStats::default();
        stats.record_invocation("network", true);
        stats.record_invocation("audio", true);
        stats.record_invocation("network", true);

        assert_eq!(stats.total_invocations, 3);
        assert_eq!(stats.successful_diagnoses, 3);
        assert_eq!(stats.most_common_domain, Some("network".to_string()));
    }

    #[test]
    fn test_debug_mode_status() {
        let status = get_debug_mode_status();
        // Default should be human mode
        assert!(!status.enabled || status.source.contains("env"));
    }

    #[test]
    fn test_format_enhanced_status() {
        let status = StatusEnhancedV081 {
            debug_mode: DebugModeStatus {
                enabled: false,
                source: "default".to_string(),
            },
            recent_cases: Vec::new(),
            mutation_stats: MutationStats::default(),
            doctor_stats: DoctorStats::default(),
        };

        let formatted = format_enhanced_status(&status);
        assert!(formatted.contains("Transcript mode"));
        assert!(formatted.contains("Recent Cases"));
        assert!(formatted.contains("Mutations"));
        assert!(formatted.contains("Doctor"));
    }
}
