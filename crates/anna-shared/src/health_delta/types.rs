//! Health delta types (v0.0.225).

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::snapshot::{diff_snapshots, DeltaItem, SystemSnapshot};

/// Health delta showing what changed
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HealthDelta {
    pub changed_fields: Vec<String>,
    pub prev_values: BTreeMap<String, String>,
    pub new_values: BTreeMap<String, String>,
    pub delta_items: Vec<DeltaItem>,
    pub summary: String,
}

impl HealthDelta {
    /// Create from comparing two snapshots
    pub fn from_snapshots(prev: &SystemSnapshot, curr: &SystemSnapshot) -> Self {
        let delta_items = diff_snapshots(prev, curr);
        let mut changed_fields = Vec::new();
        let mut prev_values = BTreeMap::new();
        let mut new_values = BTreeMap::new();

        // Track disk changes
        for (mount, &curr_pct) in &curr.disk {
            let prev_pct = prev.disk.get(mount).copied().unwrap_or(0);
            if curr_pct != prev_pct {
                let field = format!("disk:{}", mount);
                changed_fields.push(field.clone());
                prev_values.insert(field.clone(), format!("{}%", prev_pct));
                new_values.insert(field, format!("{}%", curr_pct));
            }
        }

        // Track memory changes
        let prev_mem = prev.memory_percent();
        let curr_mem = curr.memory_percent();
        if prev_mem != curr_mem {
            changed_fields.push("memory".to_string());
            prev_values.insert("memory".to_string(), format!("{}%", prev_mem));
            new_values.insert("memory".to_string(), format!("{}%", curr_mem));
        }

        // Track failed services changes
        let prev_failed: std::collections::BTreeSet<_> = prev.failed_services.iter().collect();
        let curr_failed: std::collections::BTreeSet<_> = curr.failed_services.iter().collect();

        let new_failed: Vec<_> = curr_failed.difference(&prev_failed).collect();
        let recovered: Vec<_> = prev_failed.difference(&curr_failed).collect();

        if !new_failed.is_empty() || !recovered.is_empty() {
            changed_fields.push("services".to_string());
            prev_values.insert(
                "services".to_string(),
                format!("{} failed", prev.failed_services.len()),
            );
            new_values.insert(
                "services".to_string(),
                format!("{} failed", curr.failed_services.len()),
            );
        }

        // Generate summary
        let summary = super::helpers::generate_summary(&delta_items, curr);

        Self {
            changed_fields,
            prev_values,
            new_values,
            delta_items,
            summary,
        }
    }

    /// Check if there are any changes
    pub fn has_changes(&self) -> bool {
        !self.changed_fields.is_empty()
    }

    /// Check if there are actionable items (errors/warnings)
    pub fn has_actionable(&self) -> bool {
        self.delta_items
            .iter()
            .any(|d| d.is_error() || d.is_warning())
    }

    /// Get count of errors
    pub fn error_count(&self) -> usize {
        self.delta_items.iter().filter(|d| d.is_error()).count()
    }

    /// Get count of warnings
    pub fn warning_count(&self) -> usize {
        self.delta_items.iter().filter(|d| d.is_warning()).count()
    }
}
