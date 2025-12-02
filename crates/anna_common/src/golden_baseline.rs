//! Golden Baseline v7.20.0 - Deterministic Baseline Selection
//!
//! Anna remembers a golden baseline of log patterns per component.
//!
//! Baseline selection rule (deterministic):
//! - The baseline is the first boot where:
//!   - There are no error or critical messages for that component
//!   - There are no more than N warning patterns (default N = 3)
//!
//! This is not a guess. It is a strict rule using severity and counts.
//! Persisted under /var/lib/anna/journal/baseline.

use std::collections::HashSet;
use std::fs;
use std::path::Path;
use std::process::Command;
use serde::{Deserialize, Serialize};

use crate::log_atlas::{BASELINE_DIR, normalize_message};

/// Maximum warning patterns allowed in a golden baseline
pub const MAX_BASELINE_WARNINGS: usize = 3;

/// Golden baseline for a component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoldenBaseline {
    /// Component name (service or device)
    pub component: String,
    /// Boot ID (0 = current, negative = historical)
    pub boot_id: i32,
    /// Boot number (for display, like "boot 5")
    pub boot_number: u32,
    /// Timestamp when baseline was recorded
    pub recorded_at: u64,
    /// Pattern IDs in the baseline
    pub pattern_ids: Vec<String>,
    /// Normalized patterns (for matching)
    pub patterns: Vec<String>,
    /// Number of warning patterns
    pub warning_count: usize,
}

impl GoldenBaseline {
    /// Check if a pattern is known in the baseline
    pub fn is_known_pattern(&self, normalized: &str) -> Option<&String> {
        for (i, pattern) in self.patterns.iter().enumerate() {
            if pattern == normalized {
                return self.pattern_ids.get(i);
            }
        }
        None
    }

    /// Save baseline to disk
    pub fn save(&self) -> std::io::Result<()> {
        let dir = Path::new(BASELINE_DIR);
        if !dir.exists() {
            fs::create_dir_all(dir)?;
        }

        let path = dir.join(format!("{}.json", self.component.replace('/', "_")));
        let json = serde_json::to_string_pretty(self)?;
        fs::write(path, json)?;
        Ok(())
    }

    /// Load baseline from disk
    pub fn load(component: &str) -> Option<Self> {
        let path = Path::new(BASELINE_DIR)
            .join(format!("{}.json", component.replace('/', "_")));

        if !path.exists() {
            return None;
        }

        let content = fs::read_to_string(&path).ok()?;
        serde_json::from_str(&content).ok()
    }

    /// Check if baseline exists for component
    pub fn exists(component: &str) -> bool {
        let path = Path::new(BASELINE_DIR)
            .join(format!("{}.json", component.replace('/', "_")));
        path.exists()
    }
}

/// Baseline tag for a log pattern
#[derive(Debug, Clone, PartialEq)]
pub enum BaselineTag {
    /// Pattern exists in baseline: [known, baseline ID]
    Known { baseline_id: String },
    /// Pattern is new: [new since baseline]
    NewSinceBaseline,
    /// No baseline exists for this component
    NoBaseline,
}

impl BaselineTag {
    pub fn format(&self) -> String {
        match self {
            BaselineTag::Known { baseline_id } => {
                format!("[known, baseline {}]", baseline_id)
            }
            BaselineTag::NewSinceBaseline => {
                "[new since baseline]".to_string()
            }
            BaselineTag::NoBaseline => String::new(),
        }
    }
}

/// Boot log summary for baseline analysis
#[derive(Debug, Clone, Default)]
struct BootLogSummary {
    boot_offset: i32,
    error_count: usize,
    critical_count: usize,
    warning_patterns: Vec<(String, String)>,  // (pattern_id, normalized)
}

/// Find or create golden baseline for a service
pub fn find_or_create_service_baseline(unit_name: &str, max_boots_to_scan: u32) -> Option<GoldenBaseline> {
    // Check if baseline already exists
    if let Some(baseline) = GoldenBaseline::load(unit_name) {
        return Some(baseline);
    }

    // Scan recent boots to find a candidate
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    for boot_offset in 0..max_boots_to_scan as i32 {
        let summary = analyze_boot_for_service(unit_name, boot_offset);

        // Check if this boot qualifies as a golden baseline
        if summary.error_count == 0
            && summary.critical_count == 0
            && summary.warning_patterns.len() <= MAX_BASELINE_WARNINGS
        {
            // This boot qualifies!
            let baseline = GoldenBaseline {
                component: unit_name.to_string(),
                boot_id: -boot_offset,
                boot_number: (max_boots_to_scan as i32 - boot_offset) as u32,
                recorded_at: now,
                pattern_ids: summary.warning_patterns.iter().map(|(id, _)| id.clone()).collect(),
                patterns: summary.warning_patterns.iter().map(|(_, n)| n.clone()).collect(),
                warning_count: summary.warning_patterns.len(),
            };

            // Save the baseline
            let _ = baseline.save();
            return Some(baseline);
        }
    }

    None  // No qualifying boot found
}

/// Find or create golden baseline for a device
pub fn find_or_create_device_baseline(device: &str, max_boots_to_scan: u32) -> Option<GoldenBaseline> {
    if let Some(baseline) = GoldenBaseline::load(device) {
        return Some(baseline);
    }

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    for boot_offset in 0..max_boots_to_scan as i32 {
        let summary = analyze_boot_for_device(device, boot_offset);

        if summary.error_count == 0
            && summary.critical_count == 0
            && summary.warning_patterns.len() <= MAX_BASELINE_WARNINGS
        {
            let baseline = GoldenBaseline {
                component: device.to_string(),
                boot_id: -boot_offset,
                boot_number: (max_boots_to_scan as i32 - boot_offset) as u32,
                recorded_at: now,
                pattern_ids: summary.warning_patterns.iter().map(|(id, _)| id.clone()).collect(),
                patterns: summary.warning_patterns.iter().map(|(_, n)| n.clone()).collect(),
                warning_count: summary.warning_patterns.len(),
            };

            let _ = baseline.save();
            return Some(baseline);
        }
    }

    None
}

/// Analyze a single boot for a service
fn analyze_boot_for_service(unit_name: &str, boot_offset: i32) -> BootLogSummary {
    let mut summary = BootLogSummary {
        boot_offset,
        ..Default::default()
    };

    let boot_arg = if boot_offset == 0 {
        "-b".to_string()
    } else {
        format!("-b -{}", boot_offset)
    };

    // Get error and critical messages
    let output = Command::new("journalctl")
        .args([
            "-u", unit_name,
            &boot_arg,
            "-p", "err..alert",
            "--no-pager",
            "-q",
            "-o", "cat",
        ])
        .output();

    if let Ok(out) = output {
        if out.status.success() {
            let stdout = String::from_utf8_lossy(&out.stdout);
            for line in stdout.lines() {
                if line.is_empty() {
                    continue;
                }
                if line.to_lowercase().contains("critical") || line.to_lowercase().contains("crit") {
                    summary.critical_count += 1;
                } else {
                    summary.error_count += 1;
                }
            }
        }
    }

    // Get warning messages and deduplicate by pattern
    let output = Command::new("journalctl")
        .args([
            "-u", unit_name,
            &boot_arg,
            "-p", "warning..warning",
            "--no-pager",
            "-q",
            "-o", "cat",
        ])
        .output();

    if let Ok(out) = output {
        if out.status.success() {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let mut seen_patterns: HashSet<String> = HashSet::new();
            let mut pattern_num = 1;

            for line in stdout.lines() {
                if line.is_empty() {
                    continue;
                }
                let normalized = normalize_message(line);
                if !seen_patterns.contains(&normalized) {
                    seen_patterns.insert(normalized.clone());
                    let pattern_id = format!("W{:02}", pattern_num);
                    summary.warning_patterns.push((pattern_id, normalized));
                    pattern_num += 1;
                }
            }
        }
    }

    summary
}

/// Analyze a single boot for a device
fn analyze_boot_for_device(device: &str, boot_offset: i32) -> BootLogSummary {
    let mut summary = BootLogSummary {
        boot_offset,
        ..Default::default()
    };

    let boot_arg = if boot_offset == 0 {
        "-b".to_string()
    } else {
        format!("-b -{}", boot_offset)
    };

    // Get kernel messages for this device
    let output = Command::new("sh")
        .args([
            "-c",
            &format!(
                "journalctl -k {} --no-pager -q -o cat | grep -i {}",
                boot_arg, device
            ),
        ])
        .output();

    if let Ok(out) = output {
        if out.status.success() {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let mut seen_patterns: HashSet<String> = HashSet::new();
            let mut pattern_num = 1;

            for line in stdout.lines() {
                if line.is_empty() {
                    continue;
                }

                let lower = line.to_lowercase();
                let normalized = normalize_message(line);

                if lower.contains("critical") || lower.contains("crit") {
                    summary.critical_count += 1;
                } else if lower.contains("error") || lower.contains("failed") {
                    summary.error_count += 1;
                } else if lower.contains("warning") || lower.contains("warn") {
                    if !seen_patterns.contains(&normalized) {
                        seen_patterns.insert(normalized.clone());
                        let pattern_id = format!("W{:02}", pattern_num);
                        summary.warning_patterns.push((pattern_id, normalized));
                        pattern_num += 1;
                    }
                }
            }
        }
    }

    summary
}

/// Tag a pattern with baseline status
pub fn tag_pattern(component: &str, normalized: &str) -> BaselineTag {
    match GoldenBaseline::load(component) {
        Some(baseline) => {
            if let Some(id) = baseline.is_known_pattern(normalized) {
                BaselineTag::Known {
                    baseline_id: id.clone(),
                }
            } else {
                BaselineTag::NewSinceBaseline
            }
        }
        None => BaselineTag::NoBaseline,
    }
}

/// Get components with new patterns since baseline (for status summary)
pub fn get_components_with_new_patterns() -> Vec<(String, usize)> {
    let mut result = Vec::new();

    let baseline_dir = Path::new(BASELINE_DIR);
    if !baseline_dir.exists() {
        return result;
    }

    // Read all baseline files
    if let Ok(entries) = fs::read_dir(baseline_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map(|e| e == "json").unwrap_or(false) {
                if let Ok(content) = fs::read_to_string(&path) {
                    if let Ok(baseline) = serde_json::from_str::<GoldenBaseline>(&content) {
                        // Count new patterns in current boot
                        let new_count = count_new_patterns_for_component(&baseline.component, &baseline);
                        if new_count > 0 {
                            result.push((baseline.component.clone(), new_count));
                        }
                    }
                }
            }
        }
    }

    result.sort_by(|a, b| b.1.cmp(&a.1));  // Sort by count descending
    result
}

/// Count new patterns in current boot for a component
fn count_new_patterns_for_component(component: &str, baseline: &GoldenBaseline) -> usize {
    // Check if it's a service or device
    let is_service = component.ends_with(".service") || component.contains(".");

    let output = if is_service {
        Command::new("journalctl")
            .args([
                "-u", component,
                "-b",
                "-p", "warning..alert",
                "--no-pager",
                "-q",
                "-o", "cat",
            ])
            .output()
    } else {
        Command::new("sh")
            .args([
                "-c",
                &format!("journalctl -k -b --no-pager -q -o cat | grep -i {}", component),
            ])
            .output()
    };

    match output {
        Ok(out) if out.status.success() => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let mut seen_patterns: HashSet<String> = HashSet::new();
            let mut new_count = 0;

            for line in stdout.lines() {
                if line.is_empty() {
                    continue;
                }
                let normalized = normalize_message(line);
                if !seen_patterns.contains(&normalized) {
                    seen_patterns.insert(normalized.clone());
                    if baseline.is_known_pattern(&normalized).is_none() {
                        new_count += 1;
                    }
                }
            }

            new_count
        }
        _ => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_baseline_tag_format() {
        let known = BaselineTag::Known {
            baseline_id: "W01".to_string(),
        };
        assert_eq!(known.format(), "[known, baseline W01]");

        let new = BaselineTag::NewSinceBaseline;
        assert_eq!(new.format(), "[new since baseline]");

        let none = BaselineTag::NoBaseline;
        assert_eq!(none.format(), "");
    }

    #[test]
    fn test_baseline_pattern_matching() {
        let baseline = GoldenBaseline {
            component: "test.service".to_string(),
            boot_id: -1,
            boot_number: 5,
            recorded_at: 0,
            pattern_ids: vec!["W01".to_string(), "W02".to_string()],
            patterns: vec![
                "connection timeout".to_string(),
                "retrying operation".to_string(),
            ],
            warning_count: 2,
        };

        assert_eq!(
            baseline.is_known_pattern("connection timeout"),
            Some(&"W01".to_string())
        );
        assert_eq!(
            baseline.is_known_pattern("retrying operation"),
            Some(&"W02".to_string())
        );
        assert_eq!(baseline.is_known_pattern("unknown error"), None);
    }
}
