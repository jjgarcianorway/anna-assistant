//! Helper usage statistics (v0.0.434).

use super::super::helper_entry::HelperCatalog;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Helper usage statistics.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HelperUsageStats {
    /// Per-helper usage.
    pub helpers: HashMap<String, HelperUsage>,
}

impl HelperUsageStats {
    /// Record helper usage.
    pub fn record_use(&mut self, helper_id: &str) {
        let usage = self.helpers.entry(helper_id.to_string()).or_default();
        usage.use_count += 1;
        usage.last_used = Some(timestamp_now());
    }

    /// Format for stats display.
    pub fn format(&self, catalog: &HelperCatalog) -> String {
        let mut lines = Vec::new();
        lines.push("[helper_usage]".to_string());

        for helper in &catalog.helpers {
            if let Some(usage) = self.helpers.get(&helper.id) {
                lines.push(format!(
                    "  {:<18} used {} times, last used {}",
                    helper.id,
                    usage.use_count,
                    usage.last_used.as_deref().unwrap_or("never")
                ));
            } else {
                lines.push(format!("  {:<18} not installed", helper.id));
            }
        }

        lines.join("\n")
    }
}

/// Usage stats for a single helper.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HelperUsage {
    /// Number of uses.
    pub use_count: u64,
    /// Last used timestamp.
    pub last_used: Option<String>,
}

/// Get current timestamp.
fn timestamp_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    format!("{}", secs)
}
