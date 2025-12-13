//! Resource Usage Tracker - Phase 96
//!
//! Tracks system resource usage over time.
//! Helps identify trends and potential issues.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Resource type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum ResourceType {
    #[default]
    Cpu,
    Memory,
    Disk,
    Network,
    Gpu,
    Swap,
}

impl ResourceType {
    pub fn name(&self) -> &'static str {
        match self {
            ResourceType::Cpu => "CPU",
            ResourceType::Memory => "Memory",
            ResourceType::Disk => "Disk",
            ResourceType::Network => "Network",
            ResourceType::Gpu => "GPU",
            ResourceType::Swap => "Swap",
        }
    }

    pub fn unit(&self) -> &'static str {
        match self {
            ResourceType::Cpu => "%",
            ResourceType::Memory => "%",
            ResourceType::Disk => "%",
            ResourceType::Network => "KB/s",
            ResourceType::Gpu => "%",
            ResourceType::Swap => "%",
        }
    }
}

/// Usage level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum UsageLevel {
    #[default]
    Low,
    Normal,
    Elevated,
    High,
    Critical,
}

impl UsageLevel {
    pub fn name(&self) -> &'static str {
        match self {
            UsageLevel::Low => "Low",
            UsageLevel::Normal => "Normal",
            UsageLevel::Elevated => "Elevated",
            UsageLevel::High => "High",
            UsageLevel::Critical => "Critical",
        }
    }

    pub fn symbol(&self) -> &'static str {
        match self {
            UsageLevel::Low => "▁",
            UsageLevel::Normal => "▃",
            UsageLevel::Elevated => "▅",
            UsageLevel::High => "▇",
            UsageLevel::Critical => "█",
        }
    }

    pub fn from_percent(percent: u8) -> Self {
        match percent {
            0..=20 => UsageLevel::Low,
            21..=50 => UsageLevel::Normal,
            51..=70 => UsageLevel::Elevated,
            71..=90 => UsageLevel::High,
            _ => UsageLevel::Critical,
        }
    }
}

/// A usage sample
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageSample {
    /// Resource type
    pub resource: ResourceType,
    /// Value (percentage or KB/s)
    pub value: f64,
    /// Usage level
    pub level: UsageLevel,
    /// Timestamp
    pub timestamp: u64,
}

/// Resource usage tracker
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ResourceUsageTracker {
    /// Recent samples (last N)
    pub samples: Vec<UsageSample>,
    /// Max samples to keep
    pub max_samples: usize,
    /// Count by resource type
    pub by_resource: HashMap<String, u64>,
    /// Count by level
    pub by_level: HashMap<String, u64>,
    /// Peak values per resource
    pub peaks: HashMap<String, f64>,
    /// Last sample per resource
    pub last_values: HashMap<String, f64>,
}

impl ResourceUsageTracker {
    pub fn new() -> Self {
        Self {
            max_samples: 1000,
            ..Default::default()
        }
    }

    /// Record a sample
    pub fn record(&mut self, resource: ResourceType, value: f64, timestamp: u64) {
        let level = UsageLevel::from_percent(value as u8);
        let sample = UsageSample { resource, value, level, timestamp };

        *self.by_resource.entry(resource.name().to_string()).or_insert(0) += 1;
        *self.by_level.entry(level.name().to_string()).or_insert(0) += 1;

        // Update peak
        let peak = self.peaks.entry(resource.name().to_string()).or_insert(0.0);
        if value > *peak {
            *peak = value;
        }

        // Update last value
        self.last_values.insert(resource.name().to_string(), value);

        self.samples.push(sample);

        // Trim old samples
        if self.samples.len() > self.max_samples {
            self.samples.remove(0);
        }
    }

    /// Get current value for resource
    pub fn current(&self, resource: ResourceType) -> Option<f64> {
        self.last_values.get(resource.name()).copied()
    }

    /// Get peak value for resource
    pub fn peak(&self, resource: ResourceType) -> Option<f64> {
        self.peaks.get(resource.name()).copied()
    }

    /// Get average for resource
    pub fn average(&self, resource: ResourceType) -> f64 {
        let matching: Vec<f64> = self
            .samples
            .iter()
            .filter(|s| s.resource == resource)
            .map(|s| s.value)
            .collect();
        if matching.is_empty() {
            0.0
        } else {
            matching.iter().sum::<f64>() / matching.len() as f64
        }
    }

    /// Get samples by resource
    pub fn by_res_type(&self, resource: ResourceType) -> Vec<&UsageSample> {
        self.samples.iter().filter(|s| s.resource == resource).collect()
    }

    /// Get samples by level
    pub fn by_usage_level(&self, level: UsageLevel) -> Vec<&UsageSample> {
        self.samples.iter().filter(|s| s.level == level).collect()
    }

    /// Get critical samples
    pub fn critical(&self) -> Vec<&UsageSample> {
        self.by_usage_level(UsageLevel::Critical)
    }

    /// Get high samples
    pub fn high(&self) -> Vec<&UsageSample> {
        self.by_usage_level(UsageLevel::High)
    }

    /// Total sample count
    pub fn total_count(&self) -> usize {
        self.samples.len()
    }

    /// Critical count
    pub fn critical_count(&self) -> usize {
        self.critical().len()
    }
}

/// Format resource tracker for display
pub fn format_resource_tracker(tracker: &ResourceUsageTracker) -> String {
    let mut lines = vec!["=== Resource Usage Tracker ===".to_string()];
    lines.push(String::new());

    if tracker.samples.is_empty() {
        lines.push("No resource samples recorded yet.".to_string());
        return lines.join("\n");
    }

    // Summary
    lines.push(format!("Total samples: {}", tracker.total_count()));
    lines.push(format!("Critical events: {}", tracker.critical_count()));

    // Current values
    lines.push(String::new());
    lines.push("Current values:".to_string());
    for (res, value) in &tracker.last_values {
        let level = UsageLevel::from_percent(*value as u8);
        lines.push(format!("  {} {}: {:.1}", level.symbol(), res, value));
    }

    // Peaks
    if !tracker.peaks.is_empty() {
        lines.push(String::new());
        lines.push("Peak values:".to_string());
        for (res, value) in &tracker.peaks {
            lines.push(format!("  {}: {:.1}", res, value));
        }
    }

    lines.join("\n")
}

/// Format resource tracker compact
pub fn format_resource_tracker_compact(tracker: &ResourceUsageTracker) -> String {
    let cpu = tracker.current(ResourceType::Cpu).unwrap_or(0.0);
    let mem = tracker.current(ResourceType::Memory).unwrap_or(0.0);
    format!(
        "Resources: CPU {:.0}% | MEM {:.0}% | {} samples",
        cpu,
        mem,
        tracker.total_count()
    )
}

/// Format resource tracker one-line
pub fn format_resource_tracker_oneline(tracker: &ResourceUsageTracker) -> String {
    let cpu = tracker.current(ResourceType::Cpu).unwrap_or(0.0);
    let mem = tracker.current(ResourceType::Memory).unwrap_or(0.0);
    format!("CPU {:.0}% | MEM {:.0}%", cpu, mem)
}

/// Check if query is about resources
pub fn is_resource_query(query: &str) -> bool {
    let q = query.to_lowercase();
    let keywords = [
        "resource",
        "resources",
        "cpu usage",
        "memory usage",
        "disk usage",
        "system load",
        "how much ram",
    ];
    keywords.iter().any(|k| q.contains(k))
}

/// Generate fun fact about resources
pub fn resource_fun_fact(tracker: &ResourceUsageTracker) -> String {
    if tracker.samples.is_empty() {
        return "No resource data collected yet!".to_string();
    }

    let facts = [
        format!("Anna has recorded {} resource samples.", tracker.total_count()),
        format!("{} critical resource events detected.", tracker.critical_count()),
        format!(
            "Average CPU usage: {:.1}%.",
            tracker.average(ResourceType::Cpu)
        ),
    ];

    facts[tracker.total_count() % facts.len()].clone()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resource_type() {
        assert_eq!(ResourceType::Cpu.name(), "CPU");
        assert_eq!(ResourceType::Memory.unit(), "%");
    }

    #[test]
    fn test_usage_level() {
        assert_eq!(UsageLevel::High.name(), "High");
        assert_eq!(UsageLevel::from_percent(75), UsageLevel::High);
        assert_eq!(UsageLevel::from_percent(95), UsageLevel::Critical);
    }

    #[test]
    fn test_record_sample() {
        let mut tracker = ResourceUsageTracker::new();
        tracker.record(ResourceType::Cpu, 45.0, 1000);

        assert_eq!(tracker.total_count(), 1);
        assert_eq!(tracker.current(ResourceType::Cpu), Some(45.0));
    }

    #[test]
    fn test_peak_tracking() {
        let mut tracker = ResourceUsageTracker::new();
        tracker.record(ResourceType::Cpu, 45.0, 1000);
        tracker.record(ResourceType::Cpu, 80.0, 2000);
        tracker.record(ResourceType::Cpu, 60.0, 3000);

        assert_eq!(tracker.peak(ResourceType::Cpu), Some(80.0));
    }

    #[test]
    fn test_average() {
        let mut tracker = ResourceUsageTracker::new();
        tracker.record(ResourceType::Cpu, 40.0, 1000);
        tracker.record(ResourceType::Cpu, 60.0, 2000);

        assert_eq!(tracker.average(ResourceType::Cpu), 50.0);
    }

    #[test]
    fn test_by_resource() {
        let mut tracker = ResourceUsageTracker::new();
        tracker.record(ResourceType::Cpu, 45.0, 1000);
        tracker.record(ResourceType::Memory, 60.0, 1000);

        assert_eq!(tracker.by_res_type(ResourceType::Cpu).len(), 1);
        assert_eq!(tracker.by_res_type(ResourceType::Memory).len(), 1);
    }

    #[test]
    fn test_critical_detection() {
        let mut tracker = ResourceUsageTracker::new();
        tracker.record(ResourceType::Cpu, 95.0, 1000);
        tracker.record(ResourceType::Memory, 45.0, 1000);

        assert_eq!(tracker.critical_count(), 1);
    }

    #[test]
    fn test_sample_trimming() {
        let mut tracker = ResourceUsageTracker::new();
        tracker.max_samples = 5;

        for i in 0..10 {
            tracker.record(ResourceType::Cpu, i as f64 * 10.0, i);
        }

        assert_eq!(tracker.total_count(), 5);
    }

    #[test]
    fn test_format_tracker() {
        let mut tracker = ResourceUsageTracker::new();
        tracker.record(ResourceType::Cpu, 45.0, 1000);

        let output = format_resource_tracker(&tracker);
        assert!(output.contains("Resource Usage Tracker"));
        assert!(output.contains("CPU"));
    }

    #[test]
    fn test_is_resource_query() {
        assert!(is_resource_query("show cpu usage"));
        assert!(is_resource_query("how much ram is used?"));
        assert!(is_resource_query("check system load"));
        assert!(!is_resource_query("what is the weather?"));
    }

    #[test]
    fn test_fun_fact() {
        let mut tracker = ResourceUsageTracker::new();
        tracker.record(ResourceType::Cpu, 45.0, 1000);

        let fact = resource_fun_fact(&tracker);
        assert!(!fact.is_empty());
    }
}
