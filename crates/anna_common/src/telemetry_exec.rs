//! Telemetry Execution Storage v8.0.0
//!
//! Per-object, per-day JSONL storage for command execution telemetry.
//!
//! Storage model:
//! /var/lib/anna/telemetry/<object>/YYYY/MM/DD/exec.jsonl
//!
//! Each line is a JSON record:
//! {"timestamp":"RFC3339","pid":123,"cpu_percent":5.2,"mem_rss_kb":1024,"duration_ms":150,"exit_code":0}
//!
//! Rules:
//! - One record per execution (not per sample)
//! - No aggregation stored; only raw events
//! - Missing fields are omitted, not null
//! - Directory structure enables efficient window queries

use chrono::{DateTime, NaiveDate, TimeZone, Utc};
use serde::{Deserialize, Serialize};
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;

/// Base directory for execution telemetry
pub const EXEC_TELEMETRY_DIR: &str = "/var/lib/anna/telemetry";

/// A single execution event record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionRecord {
    /// RFC3339 timestamp
    pub timestamp: String,
    /// Process ID
    pub pid: u32,
    /// CPU usage percentage at exit (if available)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cpu_percent: Option<f32>,
    /// RSS memory in KiB (if available)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mem_rss_kb: Option<u64>,
    /// Duration in milliseconds (if available)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<u64>,
    /// Exit code (if available)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exit_code: Option<i32>,
}

impl ExecutionRecord {
    /// Create a new execution record with current timestamp
    pub fn new(pid: u32) -> Self {
        Self {
            timestamp: Utc::now().to_rfc3339(),
            pid,
            cpu_percent: None,
            mem_rss_kb: None,
            duration_ms: None,
            exit_code: None,
        }
    }

    /// Get Unix timestamp from RFC3339 string
    pub fn unix_timestamp(&self) -> Option<i64> {
        DateTime::parse_from_rfc3339(&self.timestamp)
            .ok()
            .map(|dt| dt.timestamp())
    }
}

/// Aggregated stats for a time window
#[derive(Debug, Clone, Default)]
pub struct WindowStats {
    /// Total executions in window
    pub exec_count: u64,
    /// Sum of CPU percentages (for averaging)
    pub cpu_sum: f64,
    /// Count of records with CPU data
    pub cpu_count: u64,
    /// Peak CPU percentage
    pub cpu_peak: f32,
    /// Sum of RSS (for averaging)
    pub rss_sum: u64,
    /// Count of records with RSS data
    pub rss_count: u64,
    /// Peak RSS in KiB
    pub rss_peak: u64,
    /// Sum of durations (for averaging)
    pub duration_sum: u64,
    /// Count of records with duration
    pub duration_count: u64,
    /// Peak duration in ms
    pub duration_peak: u64,
}

impl WindowStats {
    /// Check if window has any samples
    pub fn has_samples(&self) -> bool {
        self.exec_count > 0
    }

    /// Get average CPU (returns None if no CPU data)
    pub fn avg_cpu(&self) -> Option<f32> {
        if self.cpu_count > 0 {
            Some((self.cpu_sum / self.cpu_count as f64) as f32)
        } else {
            None
        }
    }

    /// Get peak CPU (returns None if no CPU data)
    pub fn peak_cpu(&self) -> Option<f32> {
        if self.cpu_count > 0 {
            Some(self.cpu_peak)
        } else {
            None
        }
    }

    /// Get average RSS in KiB (returns None if no RSS data)
    pub fn avg_rss_kb(&self) -> Option<u64> {
        if self.rss_count > 0 {
            Some(self.rss_sum / self.rss_count)
        } else {
            None
        }
    }

    /// Get peak RSS in KiB (returns None if no RSS data)
    pub fn peak_rss_kb(&self) -> Option<u64> {
        if self.rss_count > 0 {
            Some(self.rss_peak)
        } else {
            None
        }
    }

    /// Get average duration in ms (returns None if no duration data)
    pub fn avg_duration_ms(&self) -> Option<u64> {
        if self.duration_count > 0 {
            Some(self.duration_sum / self.duration_count)
        } else {
            None
        }
    }

    /// Get peak duration in ms (returns None if no duration data)
    pub fn peak_duration_ms(&self) -> Option<u64> {
        if self.duration_count > 0 {
            Some(self.duration_peak)
        } else {
            None
        }
    }

    /// Add a record to this window's stats
    pub fn add_record(&mut self, record: &ExecutionRecord) {
        self.exec_count += 1;

        if let Some(cpu) = record.cpu_percent {
            self.cpu_sum += cpu as f64;
            self.cpu_count += 1;
            if cpu > self.cpu_peak {
                self.cpu_peak = cpu;
            }
        }

        if let Some(rss) = record.mem_rss_kb {
            self.rss_sum += rss;
            self.rss_count += 1;
            if rss > self.rss_peak {
                self.rss_peak = rss;
            }
        }

        if let Some(dur) = record.duration_ms {
            self.duration_sum += dur;
            self.duration_count += 1;
            if dur > self.duration_peak {
                self.duration_peak = dur;
            }
        }
    }

    /// Format as single-line summary
    pub fn format_line(&self) -> String {
        let mut parts = Vec::new();

        parts.push(format!("Execs {}", self.exec_count));

        if let Some(avg) = self.avg_cpu() {
            parts.push(format!("Avg CPU {:.1}%", avg));
        }
        if let Some(peak) = self.peak_cpu() {
            parts.push(format!("Peak CPU {:.1}%", peak));
        }

        if let Some(avg) = self.avg_rss_kb() {
            parts.push(format!("Avg RAM {}", format_kb(avg)));
        }
        if let Some(peak) = self.peak_rss_kb() {
            parts.push(format!("Peak RAM {}", format_kb(peak)));
        }

        if let Some(avg) = self.avg_duration_ms() {
            parts.push(format!("Avg Dur {} ms", avg));
        }
        if let Some(peak) = self.peak_duration_ms() {
            parts.push(format!("Peak Dur {} ms", peak));
        }

        parts.join(" | ")
    }
}

/// Format KiB to human-readable
fn format_kb(kb: u64) -> String {
    if kb >= 1024 * 1024 {
        format!("{:.1} GiB", kb as f64 / (1024.0 * 1024.0))
    } else if kb >= 1024 {
        format!("{:.1} MiB", kb as f64 / 1024.0)
    } else {
        format!("{} KiB", kb)
    }
}

/// Result of querying telemetry for an object
#[derive(Debug, Clone)]
pub struct ObjectTelemetryResult {
    /// Whether any telemetry has ever been collected for this object
    pub has_any_history: bool,
    /// Stats for last 1 hour
    pub w1h: WindowStats,
    /// Stats for last 24 hours
    pub w24h: WindowStats,
    /// Stats for last 7 days
    pub w7d: WindowStats,
    /// Stats for last 30 days
    pub w30d: WindowStats,
}

impl Default for ObjectTelemetryResult {
    fn default() -> Self {
        Self {
            has_any_history: false,
            w1h: WindowStats::default(),
            w24h: WindowStats::default(),
            w7d: WindowStats::default(),
            w30d: WindowStats::default(),
        }
    }
}

/// Writer for execution telemetry
pub struct ExecTelemetryWriter {
    base_dir: PathBuf,
}

impl Default for ExecTelemetryWriter {
    fn default() -> Self {
        Self::new()
    }
}

impl ExecTelemetryWriter {
    /// Create writer with default base directory
    pub fn new() -> Self {
        Self {
            base_dir: PathBuf::from(EXEC_TELEMETRY_DIR),
        }
    }

    /// Create writer with custom base directory
    pub fn with_dir(base_dir: PathBuf) -> Self {
        Self { base_dir }
    }

    /// Get the path for an object's daily log file
    fn get_log_path(&self, object: &str, date: &NaiveDate) -> PathBuf {
        self.base_dir
            .join(sanitize_object_name(object))
            .join(date.format("%Y").to_string())
            .join(date.format("%m").to_string())
            .join(date.format("%d").to_string())
            .join("exec.jsonl")
    }

    /// Record an execution event
    pub fn record(&self, object: &str, record: &ExecutionRecord) -> std::io::Result<()> {
        // Parse timestamp to get date
        let date = DateTime::parse_from_rfc3339(&record.timestamp)
            .map(|dt| dt.date_naive())
            .unwrap_or_else(|_| Utc::now().date_naive());

        let path = self.get_log_path(object, &date);

        // Ensure directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Append record as JSON line
        let mut file = OpenOptions::new().create(true).append(true).open(&path)?;

        let json = serde_json::to_string(record)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        writeln!(file, "{}", json)
    }
}

/// Reader for execution telemetry
pub struct ExecTelemetryReader {
    base_dir: PathBuf,
}

impl Default for ExecTelemetryReader {
    fn default() -> Self {
        Self::new()
    }
}

impl ExecTelemetryReader {
    /// Create reader with default base directory
    pub fn new() -> Self {
        Self {
            base_dir: PathBuf::from(EXEC_TELEMETRY_DIR),
        }
    }

    /// Create reader with custom base directory
    pub fn with_dir(base_dir: PathBuf) -> Self {
        Self { base_dir }
    }

    /// Get the object's base directory
    fn object_dir(&self, object: &str) -> PathBuf {
        self.base_dir.join(sanitize_object_name(object))
    }

    /// Check if an object has any telemetry history
    pub fn has_history(&self, object: &str) -> bool {
        let obj_dir = self.object_dir(object);
        if !obj_dir.exists() {
            return false;
        }
        // Check if there are any year directories
        if let Ok(entries) = fs::read_dir(&obj_dir) {
            for entry in entries.flatten() {
                if entry.path().is_dir() {
                    return true;
                }
            }
        }
        false
    }

    /// List all dates that have telemetry for an object (within a range)
    fn list_dates_in_range(
        &self,
        object: &str,
        start: NaiveDate,
        end: NaiveDate,
    ) -> Vec<NaiveDate> {
        let obj_dir = self.object_dir(object);
        let mut dates = Vec::new();

        if !obj_dir.exists() {
            return dates;
        }

        // Iterate years
        if let Ok(year_entries) = fs::read_dir(&obj_dir) {
            for year_entry in year_entries.flatten() {
                let year_path = year_entry.path();
                if !year_path.is_dir() {
                    continue;
                }
                let year_str = year_entry.file_name().to_string_lossy().to_string();
                let year: i32 = match year_str.parse() {
                    Ok(y) => y,
                    Err(_) => continue,
                };

                // Iterate months
                if let Ok(month_entries) = fs::read_dir(&year_path) {
                    for month_entry in month_entries.flatten() {
                        let month_path = month_entry.path();
                        if !month_path.is_dir() {
                            continue;
                        }
                        let month_str = month_entry.file_name().to_string_lossy().to_string();
                        let month: u32 = match month_str.parse() {
                            Ok(m) => m,
                            Err(_) => continue,
                        };

                        // Iterate days
                        if let Ok(day_entries) = fs::read_dir(&month_path) {
                            for day_entry in day_entries.flatten() {
                                let day_path = day_entry.path();
                                if !day_path.is_dir() {
                                    continue;
                                }
                                let day_str = day_entry.file_name().to_string_lossy().to_string();
                                let day: u32 = match day_str.parse() {
                                    Ok(d) => d,
                                    Err(_) => continue,
                                };

                                if let Some(date) = NaiveDate::from_ymd_opt(year, month, day) {
                                    if date >= start && date <= end {
                                        // Check if exec.jsonl exists
                                        let log_path = day_path.join("exec.jsonl");
                                        if log_path.exists() {
                                            dates.push(date);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        dates.sort();
        dates
    }

    /// Read records from a specific date's log file
    fn read_date_records(&self, object: &str, date: &NaiveDate) -> Vec<ExecutionRecord> {
        let path = self
            .base_dir
            .join(sanitize_object_name(object))
            .join(date.format("%Y").to_string())
            .join(date.format("%m").to_string())
            .join(date.format("%d").to_string())
            .join("exec.jsonl");

        if !path.exists() {
            return Vec::new();
        }

        let file = match File::open(&path) {
            Ok(f) => f,
            Err(_) => return Vec::new(),
        };

        BufReader::new(file)
            .lines()
            .filter_map(|line| line.ok().and_then(|l| serde_json::from_str(&l).ok()))
            .collect()
    }

    /// Read all records for an object within a Unix timestamp range
    pub fn read_records_in_range(
        &self,
        object: &str,
        since_unix: i64,
        until_unix: i64,
    ) -> Vec<ExecutionRecord> {
        let since_dt = Utc
            .timestamp_opt(since_unix, 0)
            .single()
            .unwrap_or_else(Utc::now);
        let until_dt = Utc
            .timestamp_opt(until_unix, 0)
            .single()
            .unwrap_or_else(Utc::now);

        let start_date = since_dt.date_naive();
        let end_date = until_dt.date_naive();

        let dates = self.list_dates_in_range(object, start_date, end_date);

        let mut records = Vec::new();
        for date in dates {
            let day_records = self.read_date_records(object, &date);
            for record in day_records {
                if let Some(ts) = record.unix_timestamp() {
                    if ts >= since_unix && ts <= until_unix {
                        records.push(record);
                    }
                }
            }
        }

        records
    }

    /// Get aggregated telemetry for an object across all standard windows
    pub fn get_object_telemetry(&self, object: &str) -> ObjectTelemetryResult {
        let has_history = self.has_history(object);

        if !has_history {
            return ObjectTelemetryResult::default();
        }

        let now = Utc::now().timestamp();
        let h1_ago = now - 3600;
        let h24_ago = now - 86400;
        let d7_ago = now - 7 * 86400;
        let d30_ago = now - 30 * 86400;

        // Read all records from the last 30 days (superset)
        let all_records = self.read_records_in_range(object, d30_ago, now);

        let mut result = ObjectTelemetryResult {
            has_any_history: has_history,
            w1h: WindowStats::default(),
            w24h: WindowStats::default(),
            w7d: WindowStats::default(),
            w30d: WindowStats::default(),
        };

        for record in &all_records {
            if let Some(ts) = record.unix_timestamp() {
                // Add to 30d window
                result.w30d.add_record(record);

                // Add to 7d window if within range
                if ts >= d7_ago {
                    result.w7d.add_record(record);
                }

                // Add to 24h window if within range
                if ts >= h24_ago {
                    result.w24h.add_record(record);
                }

                // Add to 1h window if within range
                if ts >= h1_ago {
                    result.w1h.add_record(record);
                }
            }
        }

        result
    }

    /// List all objects that have telemetry
    pub fn list_objects(&self) -> Vec<String> {
        let mut objects = Vec::new();

        if !self.base_dir.exists() {
            return objects;
        }

        if let Ok(entries) = fs::read_dir(&self.base_dir) {
            for entry in entries.flatten() {
                if entry.path().is_dir() {
                    let name = entry.file_name().to_string_lossy().to_string();
                    // Skip hidden directories and special files
                    if !name.starts_with('.') && !name.ends_with(".db") && !name.ends_with(".log") {
                        objects.push(name);
                    }
                }
            }
        }

        objects.sort();
        objects
    }

    /// Get top objects by execution count in a time window
    pub fn top_by_execs(&self, since_unix: i64, limit: usize) -> Vec<(String, u64)> {
        let objects = self.list_objects();
        let now = Utc::now().timestamp();

        let mut counts: Vec<(String, u64)> = objects
            .into_iter()
            .map(|obj| {
                let records = self.read_records_in_range(&obj, since_unix, now);
                (obj, records.len() as u64)
            })
            .filter(|(_, count)| *count > 0)
            .collect();

        counts.sort_by(|a, b| b.1.cmp(&a.1));
        counts.truncate(limit);
        counts
    }

    /// Get top objects by CPU usage in a time window
    pub fn top_by_cpu(&self, since_unix: i64, limit: usize) -> Vec<(String, f32)> {
        let objects = self.list_objects();
        let now = Utc::now().timestamp();

        let mut cpu_totals: Vec<(String, f32)> = objects
            .into_iter()
            .filter_map(|obj| {
                let records = self.read_records_in_range(&obj, since_unix, now);
                let total_cpu: f32 = records.iter().filter_map(|r| r.cpu_percent).sum();
                if total_cpu > 0.0 {
                    Some((obj, total_cpu))
                } else {
                    None
                }
            })
            .collect();

        cpu_totals.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        cpu_totals.truncate(limit);
        cpu_totals
    }
}

/// Sanitize object name for use as directory name
fn sanitize_object_name(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' || c == '.' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_execution_record_serialization() {
        let record = ExecutionRecord {
            timestamp: "2024-01-15T10:30:00Z".to_string(),
            pid: 1234,
            cpu_percent: Some(15.5),
            mem_rss_kb: Some(102400),
            duration_ms: Some(500),
            exit_code: Some(0),
        };

        let json = serde_json::to_string(&record).unwrap();
        assert!(json.contains("\"pid\":1234"));
        assert!(json.contains("\"cpu_percent\":15.5"));

        let parsed: ExecutionRecord = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.pid, 1234);
        assert_eq!(parsed.cpu_percent, Some(15.5));
    }

    #[test]
    fn test_execution_record_omits_none() {
        let record = ExecutionRecord {
            timestamp: "2024-01-15T10:30:00Z".to_string(),
            pid: 1234,
            cpu_percent: None,
            mem_rss_kb: None,
            duration_ms: None,
            exit_code: None,
        };

        let json = serde_json::to_string(&record).unwrap();
        assert!(!json.contains("cpu_percent"));
        assert!(!json.contains("mem_rss_kb"));
        assert!(!json.contains("duration_ms"));
        assert!(!json.contains("exit_code"));
    }

    #[test]
    fn test_window_stats_aggregation() {
        let mut stats = WindowStats::default();

        let r1 = ExecutionRecord {
            timestamp: Utc::now().to_rfc3339(),
            pid: 1,
            cpu_percent: Some(10.0),
            mem_rss_kb: Some(1000),
            duration_ms: Some(100),
            exit_code: Some(0),
        };

        let r2 = ExecutionRecord {
            timestamp: Utc::now().to_rfc3339(),
            pid: 2,
            cpu_percent: Some(20.0),
            mem_rss_kb: Some(2000),
            duration_ms: Some(200),
            exit_code: Some(0),
        };

        stats.add_record(&r1);
        stats.add_record(&r2);

        assert_eq!(stats.exec_count, 2);
        assert_eq!(stats.avg_cpu(), Some(15.0));
        assert_eq!(stats.peak_cpu(), Some(20.0));
        assert_eq!(stats.avg_rss_kb(), Some(1500));
        assert_eq!(stats.peak_rss_kb(), Some(2000));
        assert_eq!(stats.avg_duration_ms(), Some(150));
        assert_eq!(stats.peak_duration_ms(), Some(200));
    }

    #[test]
    fn test_writer_reader_roundtrip() {
        let temp_dir = TempDir::new().unwrap();
        let writer = ExecTelemetryWriter::with_dir(temp_dir.path().to_path_buf());
        let reader = ExecTelemetryReader::with_dir(temp_dir.path().to_path_buf());

        let record = ExecutionRecord {
            timestamp: Utc::now().to_rfc3339(),
            pid: 1234,
            cpu_percent: Some(25.0),
            mem_rss_kb: Some(50000),
            duration_ms: Some(1000),
            exit_code: Some(0),
        };

        writer.record("test_cmd", &record).unwrap();

        assert!(reader.has_history("test_cmd"));

        let result = reader.get_object_telemetry("test_cmd");
        assert!(result.has_any_history);
        assert_eq!(result.w1h.exec_count, 1);
        assert_eq!(result.w24h.exec_count, 1);
        assert_eq!(result.w7d.exec_count, 1);
        assert_eq!(result.w30d.exec_count, 1);
    }

    #[test]
    fn test_sanitize_object_name() {
        assert_eq!(sanitize_object_name("bash"), "bash");
        assert_eq!(sanitize_object_name("my-cmd"), "my-cmd");
        assert_eq!(sanitize_object_name("cmd.sh"), "cmd.sh");
        assert_eq!(sanitize_object_name("foo/bar"), "foo_bar");
        assert_eq!(sanitize_object_name("a b c"), "a_b_c");
    }

    #[test]
    fn test_format_kb() {
        assert_eq!(format_kb(512), "512 KiB");
        assert_eq!(format_kb(1024), "1.0 MiB");
        assert_eq!(format_kb(1536), "1.5 MiB");
        assert_eq!(format_kb(1048576), "1.0 GiB");
    }
}
