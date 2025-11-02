//! Snapshot Diff Engine for Anna Assistant
//!
//! Compares two telemetry snapshots to identify changes in system state.
//! Supports hierarchical diff output with color-coded indicators.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents a change between two snapshot values
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DiffChange {
    /// Value added in new snapshot
    Added { value: String },

    /// Value removed from old snapshot
    Removed { value: String },

    /// Value modified between snapshots
    Modified { old_value: String, new_value: String },

    /// Value unchanged
    Unchanged { value: String },
}

/// Hierarchical diff node representing a field or section
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffNode {
    /// Field name or path
    pub path: String,

    /// Change type
    pub change: DiffChange,

    /// Child nodes (for nested structures)
    pub children: Vec<DiffNode>,

    /// Metadata (type, size, etc.)
    pub metadata: Option<DiffMetadata>,
}

/// Metadata about a diff node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffMetadata {
    /// Field type (cpu, memory, disk, etc.)
    pub field_type: String,

    /// Delta value (for numeric fields)
    pub delta: Option<f64>,

    /// Delta percentage
    pub delta_pct: Option<f64>,

    /// Severity of change (0.0 = insignificant, 1.0 = critical)
    pub severity: f64,
}

/// Summary statistics for a diff operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffSummary {
    /// Total fields compared
    pub total_fields: usize,

    /// Fields added in new snapshot
    pub added_count: usize,

    /// Fields removed from old snapshot
    pub removed_count: usize,

    /// Fields modified between snapshots
    pub modified_count: usize,

    /// Fields unchanged
    pub unchanged_count: usize,

    /// Significant changes (severity > 0.5)
    pub significant_changes: usize,

    /// Time difference between snapshots (seconds)
    pub time_delta_secs: i64,
}

/// Main diff result structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotDiff {
    /// Root diff nodes
    pub nodes: Vec<DiffNode>,

    /// Summary statistics
    pub summary: DiffSummary,

    /// Old snapshot timestamp
    pub old_timestamp: String,

    /// New snapshot timestamp
    pub new_timestamp: String,
}

impl SnapshotDiff {
    /// Create a new empty diff
    pub fn new(old_timestamp: String, new_timestamp: String) -> Self {
        Self {
            nodes: Vec::new(),
            summary: DiffSummary {
                total_fields: 0,
                added_count: 0,
                removed_count: 0,
                modified_count: 0,
                unchanged_count: 0,
                significant_changes: 0,
                time_delta_secs: 0,
            },
            old_timestamp,
            new_timestamp,
        }
    }

    /// Add a diff node
    pub fn add_node(&mut self, node: DiffNode) {
        self.update_summary(&node);
        self.nodes.push(node);
    }

    /// Update summary statistics based on a node
    fn update_summary(&mut self, node: &DiffNode) {
        self.summary.total_fields += 1;

        match &node.change {
            DiffChange::Added { .. } => self.summary.added_count += 1,
            DiffChange::Removed { .. } => self.summary.removed_count += 1,
            DiffChange::Modified { .. } => self.summary.modified_count += 1,
            DiffChange::Unchanged { .. } => self.summary.unchanged_count += 1,
        }

        if let Some(ref meta) = node.metadata {
            if meta.severity > 0.5 {
                self.summary.significant_changes += 1;
            }
        }

        // Recursively update for children
        for child in &node.children {
            self.update_summary(child);
        }
    }

    /// Get flattened list of all changes (excluding unchanged)
    pub fn get_changes(&self) -> Vec<&DiffNode> {
        let mut changes = Vec::new();
        self.collect_changes(&self.nodes, &mut changes);
        changes
    }

    fn collect_changes<'a>(&'a self, nodes: &'a [DiffNode], changes: &mut Vec<&'a DiffNode>) {
        for node in nodes {
            if !matches!(node.change, DiffChange::Unchanged { .. }) {
                changes.push(node);
            }
            self.collect_changes(&node.children, changes);
        }
    }
}

/// Diff engine for comparing JSON-based snapshots
pub struct DiffEngine {
    /// Include unchanged fields in output
    include_unchanged: bool,

    /// Minimum severity threshold for significance
    significance_threshold: f64,
}

impl Default for DiffEngine {
    fn default() -> Self {
        Self {
            include_unchanged: false,
            significance_threshold: 0.5,
        }
    }
}

impl DiffEngine {
    /// Create a new diff engine with custom settings
    pub fn new(include_unchanged: bool, significance_threshold: f64) -> Self {
        Self {
            include_unchanged,
            significance_threshold,
        }
    }

    /// Compare two JSON values and produce a diff
    pub fn diff_json(
        &self,
        old: &serde_json::Value,
        new: &serde_json::Value,
        old_timestamp: String,
        new_timestamp: String,
    ) -> SnapshotDiff {
        let mut diff = SnapshotDiff::new(old_timestamp, new_timestamp);

        // Diff root level
        self.diff_json_recursive(old, new, "", &mut diff.nodes);

        // Calculate summary
        diff.summary = self.calculate_summary(&diff.nodes);

        diff
    }

    fn diff_json_recursive(
        &self,
        old: &serde_json::Value,
        new: &serde_json::Value,
        path: &str,
        nodes: &mut Vec<DiffNode>,
    ) {
        match (old, new) {
            // Both objects - diff keys
            (serde_json::Value::Object(old_map), serde_json::Value::Object(new_map)) => {
                let mut all_keys: std::collections::HashSet<String> =
                    old_map.keys().chain(new_map.keys()).cloned().collect();

                for key in all_keys.drain() {
                    let new_path = if path.is_empty() {
                        key.clone()
                    } else {
                        format!("{}.{}", path, key)
                    };

                    match (old_map.get(&key), new_map.get(&key)) {
                        (Some(old_val), Some(new_val)) => {
                            // Key exists in both - recurse or compare
                            if old_val == new_val {
                                if self.include_unchanged {
                                    nodes.push(DiffNode {
                                        path: new_path,
                                        change: DiffChange::Unchanged {
                                            value: value_to_string(new_val),
                                        },
                                        children: Vec::new(),
                                        metadata: None,
                                    });
                                }
                            } else {
                                // Check if values are containers (objects or arrays)
                                match (old_val, new_val) {
                                    (serde_json::Value::Object(_), serde_json::Value::Object(_)) |
                                    (serde_json::Value::Array(_), serde_json::Value::Array(_)) => {
                                        // Containers - recurse
                                        let mut children = Vec::new();
                                        self.diff_json_recursive(old_val, new_val, &new_path, &mut children);

                                        if !children.is_empty() {
                                            // Container changed - add parent node
                                            nodes.push(DiffNode {
                                                path: new_path,
                                                change: DiffChange::Modified {
                                                    old_value: "{ ... }".to_string(),
                                                    new_value: "{ ... }".to_string(),
                                                },
                                                children,
                                                metadata: None,
                                            });
                                        }
                                    }
                                    _ => {
                                        // Leaf values - compare directly
                                        let metadata = self.calculate_metadata(&key, old_val, new_val);
                                        nodes.push(DiffNode {
                                            path: new_path,
                                            change: DiffChange::Modified {
                                                old_value: value_to_string(old_val),
                                                new_value: value_to_string(new_val),
                                            },
                                            children: Vec::new(),
                                            metadata: Some(metadata),
                                        });
                                    }
                                }
                            }
                        }
                        (Some(old_val), None) => {
                            // Key removed
                            nodes.push(DiffNode {
                                path: new_path,
                                change: DiffChange::Removed {
                                    value: value_to_string(old_val),
                                },
                                children: Vec::new(),
                                metadata: None,
                            });
                        }
                        (None, Some(new_val)) => {
                            // Key added
                            nodes.push(DiffNode {
                                path: new_path,
                                change: DiffChange::Added {
                                    value: value_to_string(new_val),
                                },
                                children: Vec::new(),
                                metadata: None,
                            });
                        }
                        (None, None) => unreachable!(),
                    }
                }
            }

            // Arrays - simple element-by-element comparison
            (serde_json::Value::Array(old_arr), serde_json::Value::Array(new_arr)) => {
                let max_len = old_arr.len().max(new_arr.len());
                for i in 0..max_len {
                    let new_path = format!("{}[{}]", path, i);
                    match (old_arr.get(i), new_arr.get(i)) {
                        (Some(old_val), Some(new_val)) => {
                            if old_val != new_val {
                                let metadata = self.calculate_metadata("array_element", old_val, new_val);
                                nodes.push(DiffNode {
                                    path: new_path,
                                    change: DiffChange::Modified {
                                        old_value: value_to_string(old_val),
                                        new_value: value_to_string(new_val),
                                    },
                                    children: Vec::new(),
                                    metadata: Some(metadata),
                                });
                            }
                        }
                        (Some(old_val), None) => {
                            nodes.push(DiffNode {
                                path: new_path,
                                change: DiffChange::Removed {
                                    value: value_to_string(old_val),
                                },
                                children: Vec::new(),
                                metadata: None,
                            });
                        }
                        (None, Some(new_val)) => {
                            nodes.push(DiffNode {
                                path: new_path,
                                change: DiffChange::Added {
                                    value: value_to_string(new_val),
                                },
                                children: Vec::new(),
                                metadata: None,
                            });
                        }
                        (None, None) => unreachable!(),
                    }
                }
            }

            // Different types - treat as modification
            _ => {
                if old != new {
                    let metadata = self.calculate_metadata(path, old, new);
                    nodes.push(DiffNode {
                        path: path.to_string(),
                        change: DiffChange::Modified {
                            old_value: value_to_string(old),
                            new_value: value_to_string(new),
                        },
                        children: Vec::new(),
                        metadata: Some(metadata),
                    });
                }
            }
        }
    }

    fn calculate_metadata(
        &self,
        field_name: &str,
        old_val: &serde_json::Value,
        new_val: &serde_json::Value,
    ) -> DiffMetadata {
        let field_type = self.classify_field(field_name);
        let (delta, delta_pct) = self.calculate_delta(old_val, new_val);
        let severity = self.calculate_severity(field_name, delta_pct);

        DiffMetadata {
            field_type,
            delta,
            delta_pct,
            severity,
        }
    }

    fn classify_field(&self, field_name: &str) -> String {
        let lower = field_name.to_lowercase();
        if lower.contains("cpu") || lower.contains("util") {
            "cpu".to_string()
        } else if lower.contains("mem") || lower.contains("memory") {
            "memory".to_string()
        } else if lower.contains("disk") || lower.contains("storage") {
            "storage".to_string()
        } else if lower.contains("net") || lower.contains("network") {
            "network".to_string()
        } else if lower.contains("temp") {
            "temperature".to_string()
        } else {
            "other".to_string()
        }
    }

    fn calculate_delta(
        &self,
        old_val: &serde_json::Value,
        new_val: &serde_json::Value,
    ) -> (Option<f64>, Option<f64>) {
        match (old_val.as_f64(), new_val.as_f64()) {
            (Some(old_num), Some(new_num)) => {
                let delta = new_num - old_num;
                let delta_pct = if old_num.abs() > 0.001 {
                    (delta / old_num.abs()) * 100.0
                } else {
                    0.0
                };
                (Some(delta), Some(delta_pct))
            }
            _ => (None, None),
        }
    }

    fn calculate_severity(&self, field_name: &str, delta_pct: Option<f64>) -> f64 {
        let delta_pct = match delta_pct {
            Some(pct) => pct.abs(),
            None => return 0.0,
        };

        let lower = field_name.to_lowercase();

        // CPU/Memory utilization - high severity if >20% change
        if lower.contains("util") || lower.contains("mem_used") {
            if delta_pct > 50.0 {
                return 1.0;
            } else if delta_pct > 20.0 {
                return 0.7;
            } else if delta_pct > 10.0 {
                return 0.4;
            }
        }

        // Temperature - high severity if >10% change
        if lower.contains("temp") {
            if delta_pct > 20.0 {
                return 1.0;
            } else if delta_pct > 10.0 {
                return 0.6;
            }
        }

        // Disk usage - medium severity if >10% change
        if lower.contains("disk") || lower.contains("storage") {
            if delta_pct > 30.0 {
                return 0.8;
            } else if delta_pct > 10.0 {
                return 0.5;
            }
        }

        // Default severity based on magnitude
        if delta_pct > 100.0 {
            0.9
        } else if delta_pct > 50.0 {
            0.6
        } else if delta_pct > 25.0 {
            0.4
        } else if delta_pct > 10.0 {
            0.2
        } else {
            0.1
        }
    }

    fn calculate_summary(&self, nodes: &[DiffNode]) -> DiffSummary {
        let mut summary = DiffSummary {
            total_fields: 0,
            added_count: 0,
            removed_count: 0,
            modified_count: 0,
            unchanged_count: 0,
            significant_changes: 0,
            time_delta_secs: 0,
        };

        self.accumulate_summary(nodes, &mut summary);
        summary
    }

    fn accumulate_summary(&self, nodes: &[DiffNode], summary: &mut DiffSummary) {
        for node in nodes {
            // Only count leaf nodes (nodes without children)
            // Container nodes are just for hierarchical display
            if node.children.is_empty() {
                summary.total_fields += 1;

                match &node.change {
                    DiffChange::Added { .. } => summary.added_count += 1,
                    DiffChange::Removed { .. } => summary.removed_count += 1,
                    DiffChange::Modified { .. } => summary.modified_count += 1,
                    DiffChange::Unchanged { .. } => summary.unchanged_count += 1,
                }

                if let Some(ref meta) = node.metadata {
                    if meta.severity >= self.significance_threshold {
                        summary.significant_changes += 1;
                    }
                }
            }

            // Recursively process children
            self.accumulate_summary(&node.children, summary);
        }
    }
}

/// Convert JSON value to display string
fn value_to_string(val: &serde_json::Value) -> String {
    match val {
        serde_json::Value::Null => "null".to_string(),
        serde_json::Value::Bool(b) => b.to_string(),
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::String(s) => format!("\"{}\"", s),
        serde_json::Value::Array(arr) => format!("[{} items]", arr.len()),
        serde_json::Value::Object(obj) => format!("{{ {} fields }}", obj.len()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_simple_value_change() {
        let engine = DiffEngine::default();
        let old = json!({"cpu_util": 50.0});
        let new = json!({"cpu_util": 75.0});

        let diff = engine.diff_json(&old, &new, "t1".to_string(), "t2".to_string());

        assert_eq!(diff.summary.modified_count, 1);
        assert_eq!(diff.summary.added_count, 0);
        assert_eq!(diff.summary.removed_count, 0);
        assert_eq!(diff.summary.total_fields, 1);
    }

    #[test]
    fn test_field_added() {
        let engine = DiffEngine::default();
        let old = json!({"cpu_util": 50.0});
        let new = json!({"cpu_util": 50.0, "mem_used": 1024});

        let diff = engine.diff_json(&old, &new, "t1".to_string(), "t2".to_string());

        assert_eq!(diff.summary.added_count, 1);
        assert!(diff.nodes.iter().any(|n| matches!(n.change, DiffChange::Added { .. })));
    }

    #[test]
    fn test_field_removed() {
        let engine = DiffEngine::default();
        let old = json!({"cpu_util": 50.0, "mem_used": 1024});
        let new = json!({"cpu_util": 50.0});

        let diff = engine.diff_json(&old, &new, "t1".to_string(), "t2".to_string());

        assert_eq!(diff.summary.removed_count, 1);
        assert!(diff.nodes.iter().any(|n| matches!(n.change, DiffChange::Removed { .. })));
    }

    #[test]
    fn test_nested_changes() {
        let engine = DiffEngine::default();
        let old = json!({
            "system": {
                "cpu": {"util": 50.0},
                "memory": {"used_mb": 1024}
            }
        });
        let new = json!({
            "system": {
                "cpu": {"util": 75.0},
                "memory": {"used_mb": 1024}
            }
        });

        let diff = engine.diff_json(&old, &new, "t1".to_string(), "t2".to_string());

        // Only leaf nodes are counted (system.cpu.util changed, system.memory.used_mb unchanged)
        assert_eq!(diff.summary.modified_count, 1); // system.cpu.util
        assert_eq!(diff.summary.total_fields, 1); // Only the changed leaf
    }

    #[test]
    fn test_severity_calculation() {
        let engine = DiffEngine::default();

        // High CPU change should have high severity
        let old = json!({"cpu_util_pct": 20.0});
        let new = json!({"cpu_util_pct": 80.0});

        let diff = engine.diff_json(&old, &new, "t1".to_string(), "t2".to_string());

        // Check that there's a modified node
        assert_eq!(diff.summary.modified_count, 1);

        // Find the node and check its metadata
        let node = diff.nodes.iter().find(|n| matches!(n.change, DiffChange::Modified { .. }));
        assert!(node.is_some());
        let node = node.unwrap();
        assert!(node.metadata.is_some());
        assert!(node.metadata.as_ref().unwrap().severity > 0.5);
    }

    #[test]
    fn test_delta_calculation() {
        let engine = DiffEngine::default();
        let old = json!({"value": 100.0});
        let new = json!({"value": 150.0});

        let diff = engine.diff_json(&old, &new, "t1".to_string(), "t2".to_string());

        // Find the modified node
        let node = diff.nodes.iter().find(|n| matches!(n.change, DiffChange::Modified { .. }));
        assert!(node.is_some());
        let node = node.unwrap();
        assert!(node.metadata.is_some());

        let meta = node.metadata.as_ref().unwrap();
        assert_eq!(meta.delta, Some(50.0));
        assert_eq!(meta.delta_pct, Some(50.0));
    }

    #[test]
    fn test_array_diff() {
        let engine = DiffEngine::default();
        let old = json!({"items": [1, 2, 3]});
        let new = json!({"items": [1, 5, 3]});

        let diff = engine.diff_json(&old, &new, "t1".to_string(), "t2".to_string());

        // Should detect the change in the second element
        assert!(diff.summary.modified_count > 0);
    }

    #[test]
    fn test_include_unchanged() {
        let mut engine = DiffEngine::default();
        engine.include_unchanged = true;

        let old = json!({"a": 1, "b": 2});
        let new = json!({"a": 1, "b": 3});

        let diff = engine.diff_json(&old, &new, "t1".to_string(), "t2".to_string());

        // Should include unchanged field "a" and modified field "b"
        assert_eq!(diff.summary.unchanged_count, 1); // "a"
        assert_eq!(diff.summary.modified_count, 1);  // "b"
        assert_eq!(diff.summary.total_fields, 2);    // Both fields counted
    }
}
