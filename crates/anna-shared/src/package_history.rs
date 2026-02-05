//! Package History - Track package installations and changes over time.
//!
//! v0.3.124: Comprehensive package telemetry for visualization and analysis.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use chrono::{DateTime, Utc, Duration, Datelike};

/// A record of a package change (install/remove/upgrade).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageEvent {
    /// Timestamp
    pub timestamp: String,
    /// Package name
    pub package: String,
    /// Event type
    pub event_type: PackageEventType,
    /// Version installed/removed
    pub version: String,
    /// Whether this was installed by Anna herself
    pub anna_installed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PackageEventType {
    Install,
    Remove,
    Upgrade,
}

/// Package history store.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PackageHistory {
    /// All package events
    pub events: Vec<PackageEvent>,
}

impl PackageHistory {
    fn path() -> PathBuf {
        PathBuf::from("/var/lib/anna/package_history.json")
    }

    /// Load from disk.
    pub fn load() -> Self {
        std::fs::read_to_string(Self::path())
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    /// Save to disk.
    pub fn save(&self) -> std::io::Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(Self::path(), json)
    }

    /// Record a package event.
    pub fn record(&mut self, package: &str, event_type: PackageEventType, version: &str, anna_installed: bool) {
        self.events.push(PackageEvent {
            timestamp: Utc::now().to_rfc3339(),
            package: package.to_string(),
            event_type,
            version: version.to_string(),
            anna_installed,
        });

        // Keep only last 10,000 events
        if self.events.len() > 10000 {
            self.events.drain(0..self.events.len() - 10000);
        }
    }

    /// Get events within a time range.
    pub fn events_in_range(&self, start: DateTime<Utc>, end: DateTime<Utc>) -> Vec<&PackageEvent> {
        self.events
            .iter()
            .filter(|e| {
                if let Ok(ts) = DateTime::parse_from_rfc3339(&e.timestamp) {
                    let ts_utc = ts.with_timezone(&Utc);
                    ts_utc >= start && ts_utc <= end
                } else {
                    false
                }
            })
            .collect()
    }

    /// Get all Anna-installed packages (not yet removed).
    pub fn anna_installed_packages(&self) -> Vec<String> {
        let mut installed: HashMap<String, bool> = HashMap::new();

        for event in &self.events {
            if event.anna_installed {
                match event.event_type {
                    PackageEventType::Install => {
                        installed.insert(event.package.clone(), true);
                    }
                    PackageEventType::Remove => {
                        installed.remove(&event.package);
                    }
                    PackageEventType::Upgrade => {}
                }
            }
        }

        installed.keys().cloned().collect()
    }

    /// Get installation counts by period.
    pub fn installations_by_period(&self, days: i64) -> Vec<(String, usize)> {
        let end = Utc::now();
        let start = end - Duration::days(days);
        let events = self.events_in_range(start, end);

        // Group by day
        let mut by_day: HashMap<String, usize> = HashMap::new();
        for event in events {
            if event.event_type == PackageEventType::Install {
                if let Ok(ts) = DateTime::parse_from_rfc3339(&event.timestamp) {
                    let day = ts.format("%Y-%m-%d").to_string();
                    *by_day.entry(day).or_insert(0) += 1;
                }
            }
        }

        let mut result: Vec<_> = by_day.into_iter().collect();
        result.sort_by(|a, b| a.0.cmp(&b.0));
        result
    }

    /// Get installation counts by week.
    pub fn installations_by_week(&self, weeks: i64) -> Vec<(String, usize)> {
        let end = Utc::now();
        let start = end - Duration::weeks(weeks);
        let events = self.events_in_range(start, end);

        let mut by_week: HashMap<String, usize> = HashMap::new();
        for event in events {
            if event.event_type == PackageEventType::Install {
                if let Ok(ts) = DateTime::parse_from_rfc3339(&event.timestamp) {
                    let ts_utc = ts.with_timezone(&Utc);
                    let week = ts_utc.iso_week().week();
                    let year = ts_utc.year();
                    let week_key = format!("{}-W{:02}", year, week);
                    *by_week.entry(week_key).or_insert(0) += 1;
                }
            }
        }

        let mut result: Vec<_> = by_week.into_iter().collect();
        result.sort_by(|a, b| a.0.cmp(&b.0));
        result
    }

    /// Get installation counts by month.
    pub fn installations_by_month(&self, months: i64) -> Vec<(String, usize)> {
        let end = Utc::now();
        let start = end - Duration::days(months * 30);
        let events = self.events_in_range(start, end);

        let mut by_month: HashMap<String, usize> = HashMap::new();
        for event in events {
            if event.event_type == PackageEventType::Install {
                if let Ok(ts) = DateTime::parse_from_rfc3339(&event.timestamp) {
                    let month = ts.format("%Y-%m").to_string();
                    *by_month.entry(month).or_insert(0) += 1;
                }
            }
        }

        let mut result: Vec<_> = by_month.into_iter().collect();
        result.sort_by(|a, b| a.0.cmp(&b.0));
        result
    }

    /// Get most installed packages.
    pub fn most_installed(&self, limit: usize) -> Vec<(String, usize)> {
        let mut counts: HashMap<String, usize> = HashMap::new();
        for event in &self.events {
            if event.event_type == PackageEventType::Install {
                *counts.entry(event.package.clone()).or_insert(0) += 1;
            }
        }

        let mut result: Vec<_> = counts.into_iter().collect();
        result.sort_by(|a, b| b.1.cmp(&a.1));
        result.truncate(limit);
        result
    }

    /// Get recent installations.
    pub fn recent_installations(&self, days: i64, limit: usize) -> Vec<PackageEvent> {
        let cutoff = Utc::now() - Duration::days(days);
        let mut recent: Vec<_> = self.events
            .iter()
            .filter(|e| {
                e.event_type == PackageEventType::Install &&
                DateTime::parse_from_rfc3339(&e.timestamp)
                    .map(|ts| ts.with_timezone(&Utc) > cutoff)
                    .unwrap_or(false)
            })
            .cloned()
            .collect();

        recent.reverse(); // Most recent first
        recent.truncate(limit);
        recent
    }

    /// Generate ASCII chart for installations by period.
    pub fn chart_installations(&self, days: i64) -> String {
        use crate::charts::BarChart;

        let data = if days <= 30 {
            self.installations_by_period(days)
        } else if days <= 180 {
            self.installations_by_week(days / 7)
        } else {
            self.installations_by_month(days / 30)
        };

        if data.is_empty() {
            return "No package installations in this period.".to_string();
        }

        let period = if days <= 30 { "day" } else if days <= 180 { "week" } else { "month" };
        let title = format!("Package Installations by {} (last {} days)", period, days);

        let mut chart = BarChart::new(&title);
        for (label, value) in data {
            chart.add(&label, value as f64);
        }

        chart.render()
    }
}

/// Sync package history from pacman log.
pub fn sync_from_pacman_log() -> std::io::Result<()> {
    let log = std::fs::read_to_string("/var/log/pacman.log")?;
    let mut history = PackageHistory::load();

    // Parse pacman log
    for line in log.lines() {
        if line.contains("[ALPM] installed") {
            if let Some(package_part) = line.split("[ALPM] installed ").nth(1) {
                let parts: Vec<&str> = package_part.split(' ').collect();
                if parts.len() >= 2 {
                    let package = parts[0];
                    let version = parts[1].trim_matches('(').trim_matches(')');

                    // Extract timestamp from log line
                    if let Some(ts_part) = line.split('[').nth(1) {
                        if let Some(ts) = ts_part.split(']').next() {
                            // Simple record (we don't have exact timestamp parsing here)
                            history.record(package, PackageEventType::Install, version, false);
                        }
                    }
                }
            }
        } else if line.contains("[ALPM] removed") {
            if let Some(package_part) = line.split("[ALPM] removed ").nth(1) {
                let parts: Vec<&str> = package_part.split(' ').collect();
                if !parts.is_empty() {
                    let package = parts[0];
                    history.record(package, PackageEventType::Remove, "", false);
                }
            }
        } else if line.contains("[ALPM] upgraded") {
            if let Some(package_part) = line.split("[ALPM] upgraded ").nth(1) {
                let parts: Vec<&str> = package_part.split(' ').collect();
                if parts.len() >= 2 {
                    let package = parts[0];
                    let version = parts[2].trim_matches('(').trim_matches(')');
                    history.record(package, PackageEventType::Upgrade, version, false);
                }
            }
        }
    }

    history.save()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_event() {
        let mut history = PackageHistory::default();
        history.record("test-pkg", PackageEventType::Install, "1.0.0", false);
        assert_eq!(history.events.len(), 1);
        assert_eq!(history.events[0].package, "test-pkg");
    }

    #[test]
    fn test_anna_installed_tracking() {
        let mut history = PackageHistory::default();
        history.record("tool1", PackageEventType::Install, "1.0", true);
        history.record("tool2", PackageEventType::Install, "2.0", true);
        history.record("tool1", PackageEventType::Remove, "", true);

        let installed = history.anna_installed_packages();
        assert_eq!(installed.len(), 1);
        assert!(installed.contains(&"tool2".to_string()));
    }
}
