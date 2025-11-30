//! Memory Summary Helper v0.80.0
//!
//! Extracts meaningful memory information from /proc/meminfo output.
//! Used to provide clear evidence summary to Senior auditor.

use serde::{Deserialize, Serialize};

/// Structured memory summary extracted from /proc/meminfo
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MemSummary {
    /// Total installed RAM in kB
    pub mem_total_kb: u64,
    /// Free memory in kB
    pub mem_free_kb: u64,
    /// Available memory in kB (more accurate than free)
    pub mem_available_kb: u64,
    /// Total swap in kB
    pub swap_total_kb: u64,
    /// Free swap in kB
    pub swap_free_kb: u64,
}

impl MemSummary {
    /// Total RAM in GiB (binary, 1024-based)
    pub fn total_gib(&self) -> f64 {
        self.mem_total_kb as f64 / 1024.0 / 1024.0
    }

    /// Available RAM in GiB
    pub fn available_gib(&self) -> f64 {
        self.mem_available_kb as f64 / 1024.0 / 1024.0
    }

    /// Used RAM in GiB (total - available)
    pub fn used_gib(&self) -> f64 {
        let used_kb = self.mem_total_kb.saturating_sub(self.mem_available_kb);
        used_kb as f64 / 1024.0 / 1024.0
    }

    /// Free RAM in GiB
    pub fn free_gib(&self) -> f64 {
        self.mem_free_kb as f64 / 1024.0 / 1024.0
    }

    /// Total swap in GiB
    pub fn swap_total_gib(&self) -> f64 {
        self.swap_total_kb as f64 / 1024.0 / 1024.0
    }

    /// Format as human-readable summary for Senior prompt
    pub fn format_for_senior(&self) -> String {
        format!(
            r#"MEM SUMMARY (from mem.info):
- total_gib: {:.1}
- available_gib: {:.1}
- used_gib: {:.1}
- swap_total_gib: {:.1}"#,
            self.total_gib(),
            self.available_gib(),
            self.used_gib(),
            self.swap_total_gib()
        )
    }

    /// Format as compact JSON for probe summary
    pub fn to_compact_json(&self) -> String {
        format!(
            r#"{{"total_gib":{:.1},"available_gib":{:.1},"used_gib":{:.1}}}"#,
            self.total_gib(),
            self.available_gib(),
            self.used_gib()
        )
    }
}

/// Parse /proc/meminfo text output to extract memory summary
///
/// Format:
/// ```text
/// MemTotal:       32768000 kB
/// MemFree:         8000000 kB
/// MemAvailable:   24000000 kB
/// SwapTotal:       8000000 kB
/// SwapFree:        8000000 kB
/// ```
pub fn summarize_mem_from_text(text: &str) -> MemSummary {
    let mut summary = MemSummary::default();

    for line in text.lines() {
        let line = line.trim();
        if let Some((key, value)) = line.split_once(':') {
            let key = key.trim();
            // Value is like "32768000 kB" - extract just the number
            let value = value.split_whitespace().next().unwrap_or("0");
            let value_kb: u64 = value.parse().unwrap_or(0);

            match key {
                "MemTotal" => summary.mem_total_kb = value_kb,
                "MemFree" => summary.mem_free_kb = value_kb,
                "MemAvailable" => summary.mem_available_kb = value_kb,
                "SwapTotal" => summary.swap_total_kb = value_kb,
                "SwapFree" => summary.swap_free_kb = value_kb,
                _ => {}
            }
        }
    }

    summary
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_summarize_mem_from_text() {
        let text = r#"
MemTotal:       32768000 kB
MemFree:         8000000 kB
MemAvailable:   24000000 kB
Buffers:         1000000 kB
Cached:          8000000 kB
SwapTotal:       8000000 kB
SwapFree:        8000000 kB
"#;

        let summary = summarize_mem_from_text(text);
        assert_eq!(summary.mem_total_kb, 32768000);
        assert_eq!(summary.mem_free_kb, 8000000);
        assert_eq!(summary.mem_available_kb, 24000000);
        assert_eq!(summary.swap_total_kb, 8000000);

        // Test GiB calculations
        assert!((summary.total_gib() - 31.25).abs() < 0.1); // 32768000 / 1024 / 1024 ≈ 31.25
        assert!((summary.available_gib() - 22.88).abs() < 0.1);
    }

    #[test]
    fn test_format_for_senior() {
        let summary = MemSummary {
            mem_total_kb: 32768000,
            mem_free_kb: 8000000,
            mem_available_kb: 24000000,
            swap_total_kb: 8000000,
            swap_free_kb: 8000000,
        };

        let formatted = summary.format_for_senior();
        assert!(formatted.contains("MEM SUMMARY"));
        assert!(formatted.contains("total_gib:"));
        assert!(formatted.contains("available_gib:"));
    }

    #[test]
    fn test_to_compact_json() {
        let summary = MemSummary {
            mem_total_kb: 33554432, // 32 GiB
            mem_free_kb: 8000000,
            mem_available_kb: 25165824, // 24 GiB
            swap_total_kb: 0,
            swap_free_kb: 0,
        };

        let json = summary.to_compact_json();
        assert!(json.contains("total_gib"));
        assert!(json.contains("available_gib"));
        assert!(json.contains("used_gib"));
    }

    #[test]
    fn test_used_gib_calculation() {
        let summary = MemSummary {
            mem_total_kb: 33554432,    // 32 GiB
            mem_free_kb: 0,
            mem_available_kb: 8388608, // 8 GiB available
            swap_total_kb: 0,
            swap_free_kb: 0,
        };

        // Used = Total - Available = 32 - 8 = 24 GiB
        assert!((summary.used_gib() - 24.0).abs() < 0.1);
    }
}
