//! Advisor module for Anna v0.13.2 "Orion II"
//!
//! Centralized recommendation engine with extensible rule system

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

use crate::radar_cmd::{HardwareRadar, RadarSnapshot, SoftwareRadar, UserRadar};
use crate::learning::LearningEngine;

/// Recommendation rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    pub id: String,
    pub category: String,       // "hardware", "software", "user"
    pub priority: String,       // "critical", "high", "medium", "low"
    pub title: String,
    pub condition: Condition,
    pub message: String,
    pub action: String,
    pub impact: String,         // e.g., "+3 Software, +1 User"
    pub emoji: String,
}

/// Rule condition
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Condition {
    #[serde(rename = "threshold")]
    Threshold {
        metric: String,
        operator: String,  // "<=", ">=", "==", "!=", "<", ">"
        value: u8,
    },
    #[serde(rename = "and")]
    And {
        conditions: Vec<Condition>,
    },
    #[serde(rename = "or")]
    Or {
        conditions: Vec<Condition>,
    },
}

/// Recommendation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recommendation {
    pub priority: String,
    pub category: String,
    pub title: String,
    pub reason: String,
    pub action: String,
    pub emoji: String,
    pub impact: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub learned_weight: Option<f32>,   // User response weight from learning
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_confidence: Option<f32>,  // Automation confidence
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trust_level: Option<String>,   // high, neutral, low, untrusted
}

/// Advisor engine
pub struct Advisor {
    rules: Vec<Rule>,
    learning: Option<LearningEngine>,
}

impl Advisor {
    /// Create new advisor with default rules
    pub fn new() -> Result<Self> {
        let mut rules = Self::default_rules();

        // Load custom rules from config directory
        if let Ok(custom_rules) = Self::load_custom_rules() {
            rules.extend(custom_rules);
        }

        // Initialize learning engine
        let learning = LearningEngine::new().ok();

        Ok(Self { rules, learning })
    }

    /// Default built-in rules
    fn default_rules() -> Vec<Rule> {
        vec![
            // ========== CRITICAL RULES ==========
            Rule {
                id: "critical_performance_drift".to_string(),
                category: "software".to_string(),
                priority: "critical".to_string(),
                title: "Anna performance degradation detected".to_string(),
                condition: Condition::Threshold {
                    metric: "software.overall".to_string(),
                    operator: ">=".to_string(),
                    value: 0,  // Always check (actual detection happens in anomaly system)
                },
                message: "Anna's own performance has degraded significantly. Self-monitoring detected persistent issues.".to_string(),
                action: "Check profiler: annactl profiled --status. Review anomalies: annactl anomalies".to_string(),
                impact: "System Stability".to_string(),
                emoji: "🔴".to_string(),
            },
            Rule {
                id: "critical_security_updates".to_string(),
                category: "software".to_string(),
                priority: "critical".to_string(),
                title: "Security updates pending".to_string(),
                condition: Condition::Threshold {
                    metric: "software.os_freshness".to_string(),
                    operator: "<=".to_string(),
                    value: 5,
                },
                message: "System has not been updated recently. Security vulnerabilities may be present.".to_string(),
                action: "Apply updates: sudo pacman -Syu or sudo apt update && sudo apt upgrade".to_string(),
                impact: "+3 Software".to_string(),
                emoji: "🔒".to_string(),
            },
            Rule {
                id: "critical_disk_space".to_string(),
                category: "hardware".to_string(),
                priority: "critical".to_string(),
                title: "Critical disk space low".to_string(),
                condition: Condition::Threshold {
                    metric: "hardware.disk_free".to_string(),
                    operator: "<=".to_string(),
                    value: 3,
                },
                message: "Less than 10% disk space remaining. System instability imminent.".to_string(),
                action: "Free space: sudo pacman -Sc (Arch) or sudo apt clean && sudo apt autoremove (Debian/Ubuntu)".to_string(),
                impact: "+5 Hardware".to_string(),
                emoji: "💾".to_string(),
            },
            Rule {
                id: "critical_security_hardening".to_string(),
                category: "software".to_string(),
                priority: "critical".to_string(),
                title: "Security hardening needed".to_string(),
                condition: Condition::Threshold {
                    metric: "software.security".to_string(),
                    operator: "<=".to_string(),
                    value: 4,
                },
                message: "System security posture is weak. Immediate hardening required.".to_string(),
                action: "Enable firewall, configure SELinux/AppArmor, review ssh config".to_string(),
                impact: "+4 Software".to_string(),
                emoji: "🛡️".to_string(),
            },

            // ========== HIGH PRIORITY RULES ==========
            Rule {
                id: "high_thermal_issues".to_string(),
                category: "hardware".to_string(),
                priority: "high".to_string(),
                title: "High CPU temperatures detected".to_string(),
                condition: Condition::Threshold {
                    metric: "hardware.cpu_thermal".to_string(),
                    operator: "<=".to_string(),
                    value: 5,
                },
                message: "CPU may be throttling due to high temperatures".to_string(),
                action: "Check cooling system, clean dust filters, verify fan operation".to_string(),
                impact: "+3 Hardware".to_string(),
                emoji: "🌡️".to_string(),
            },
            Rule {
                id: "high_memory_pressure".to_string(),
                category: "hardware".to_string(),
                priority: "high".to_string(),
                title: "High memory pressure".to_string(),
                condition: Condition::Threshold {
                    metric: "hardware.memory".to_string(),
                    operator: "<=".to_string(),
                    value: 5,
                },
                message: "System is running low on available memory".to_string(),
                action: "Close unused applications, consider adding swap, enable zram".to_string(),
                impact: "+2 Hardware".to_string(),
                emoji: "⚠️".to_string(),
            },
            Rule {
                id: "high_package_updates".to_string(),
                category: "software".to_string(),
                priority: "high".to_string(),
                title: "Package updates available".to_string(),
                condition: Condition::Threshold {
                    metric: "software.packages".to_string(),
                    operator: "<=".to_string(),
                    value: 6,
                },
                message: "Multiple packages have available updates".to_string(),
                action: "Review and apply: sudo pacman -Syu or sudo apt update && sudo apt upgrade".to_string(),
                impact: "+2 Software".to_string(),
                emoji: "📦".to_string(),
            },
            Rule {
                id: "high_failing_services".to_string(),
                category: "software".to_string(),
                priority: "high".to_string(),
                title: "Critical services failing".to_string(),
                condition: Condition::Threshold {
                    metric: "software.services".to_string(),
                    operator: "<=".to_string(),
                    value: 6,
                },
                message: "One or more critical system services are not running correctly".to_string(),
                action: "Check status: systemctl --failed, review logs: journalctl -xe".to_string(),
                impact: "+3 Software".to_string(),
                emoji: "🔧".to_string(),
            },
            Rule {
                id: "high_no_backups".to_string(),
                category: "software".to_string(),
                priority: "high".to_string(),
                title: "No backup system detected".to_string(),
                condition: Condition::Threshold {
                    metric: "software.backups".to_string(),
                    operator: "<=".to_string(),
                    value: 5,
                },
                message: "System has no automated backup solution configured".to_string(),
                action: "Set up backups: timeshift, restic, or borg. Configure automatic snapshots.".to_string(),
                impact: "+4 Software".to_string(),
                emoji: "💿".to_string(),
            },

            // ========== MEDIUM PRIORITY RULES ==========
            Rule {
                id: "medium_disk_space_warning".to_string(),
                category: "hardware".to_string(),
                priority: "medium".to_string(),
                title: "Disk space running low".to_string(),
                condition: Condition::Threshold {
                    metric: "hardware.disk_free".to_string(),
                    operator: "<=".to_string(),
                    value: 6,
                },
                message: "Disk space is below 30%. Consider cleanup.".to_string(),
                action: "Clean package cache, remove old logs: sudo journalctl --vacuum-time=30d".to_string(),
                impact: "+2 Hardware".to_string(),
                emoji: "💾".to_string(),
            },
            Rule {
                id: "medium_basic_fs".to_string(),
                category: "hardware".to_string(),
                priority: "medium".to_string(),
                title: "Basic filesystem detected".to_string(),
                condition: Condition::Threshold {
                    metric: "hardware.fs_features".to_string(),
                    operator: "<=".to_string(),
                    value: 6,
                },
                message: "Consider modern filesystem (btrfs, zfs) for snapshots and data integrity".to_string(),
                action: "Research btrfs or zfs benefits. Plan migration during next reinstall.".to_string(),
                impact: "+2 Hardware".to_string(),
                emoji: "📁".to_string(),
            },
            Rule {
                id: "medium_log_noise".to_string(),
                category: "software".to_string(),
                priority: "medium".to_string(),
                title: "Excessive log errors".to_string(),
                condition: Condition::Threshold {
                    metric: "software.log_noise".to_string(),
                    operator: "<=".to_string(),
                    value: 6,
                },
                message: "System logs contain many warnings or errors".to_string(),
                action: "Review logs: journalctl -p err -b, investigate root causes".to_string(),
                impact: "+1 Software".to_string(),
                emoji: "📋".to_string(),
            },
            Rule {
                id: "medium_user_irregularity".to_string(),
                category: "user".to_string(),
                priority: "medium".to_string(),
                title: "Irregular usage patterns".to_string(),
                condition: Condition::Threshold {
                    metric: "user.regularity".to_string(),
                    operator: "<=".to_string(),
                    value: 6,
                },
                message: "Usage patterns suggest inconsistent maintenance routines".to_string(),
                action: "Establish regular maintenance schedule: weekly updates, monthly cleanup".to_string(),
                impact: "+2 User".to_string(),
                emoji: "📅".to_string(),
            },
            Rule {
                id: "medium_workspace_clutter".to_string(),
                category: "user".to_string(),
                priority: "medium".to_string(),
                title: "Workspace organization needed".to_string(),
                condition: Condition::Threshold {
                    metric: "user.workspace".to_string(),
                    operator: "<=".to_string(),
                    value: 6,
                },
                message: "Home directory or workspace could use organization".to_string(),
                action: "Organize ~/Downloads, ~/Desktop. Archive old projects. Clean temp files.".to_string(),
                impact: "+1 User".to_string(),
                emoji: "🗂️".to_string(),
            },

            // ========== LOW PRIORITY RULES ==========
            Rule {
                id: "low_gpu_optimization".to_string(),
                category: "hardware".to_string(),
                priority: "low".to_string(),
                title: "GPU optimization opportunity".to_string(),
                condition: Condition::Threshold {
                    metric: "hardware.gpu".to_string(),
                    operator: "<=".to_string(),
                    value: 7,
                },
                message: "GPU drivers or configuration could be optimized".to_string(),
                action: "Update GPU drivers, enable hardware acceleration in browser".to_string(),
                impact: "+1 Hardware".to_string(),
                emoji: "🎮".to_string(),
            },
            Rule {
                id: "low_boot_optimization".to_string(),
                category: "hardware".to_string(),
                priority: "low".to_string(),
                title: "Boot time optimization".to_string(),
                condition: Condition::Threshold {
                    metric: "hardware.boot".to_string(),
                    operator: "<=".to_string(),
                    value: 7,
                },
                message: "Boot time could be improved".to_string(),
                action: "Analyze boot: systemd-analyze blame. Disable unused services.".to_string(),
                impact: "+1 Hardware".to_string(),
                emoji: "⚡".to_string(),
            },
            Rule {
                id: "low_containers_optimization".to_string(),
                category: "software".to_string(),
                priority: "low".to_string(),
                title: "Container environment optimization".to_string(),
                condition: Condition::Threshold {
                    metric: "software.containers".to_string(),
                    operator: "<=".to_string(),
                    value: 7,
                },
                message: "Container setup (docker/podman) could be improved".to_string(),
                action: "Review container config, clean unused images: docker system prune".to_string(),
                impact: "+1 Software".to_string(),
                emoji: "🐳".to_string(),
            },
            Rule {
                id: "low_power_management".to_string(),
                category: "user".to_string(),
                priority: "low".to_string(),
                title: "Power management tuning".to_string(),
                condition: Condition::Threshold {
                    metric: "user.power".to_string(),
                    operator: "<=".to_string(),
                    value: 7,
                },
                message: "Power settings could be optimized for battery life or performance".to_string(),
                action: "Install powertop or tlp. Review power profiles with asusctl (ASUS laptops)".to_string(),
                impact: "+1 User".to_string(),
                emoji: "🔋".to_string(),
            },
        ]
    }

    /// Load custom rules from ~/.config/anna/advisor.d/
    fn load_custom_rules() -> Result<Vec<Rule>> {
        let config_dir = Self::get_config_dir()?;
        let advisor_dir = config_dir.join("advisor.d");

        if !advisor_dir.exists() {
            return Ok(Vec::new());
        }

        let mut rules = Vec::new();

        // Load all .json and .yaml files
        for entry in fs::read_dir(advisor_dir)? {
            let entry = entry?;
            let path = entry.path();

            if !path.is_file() {
                continue;
            }

            let ext = path.extension().and_then(|s| s.to_str());
            if ext != Some("json") && ext != Some("yaml") && ext != Some("yml") {
                continue;
            }

            match Self::load_rules_from_file(&path) {
                Ok(mut file_rules) => rules.append(&mut file_rules),
                Err(e) => {
                    eprintln!("Warning: Failed to load rules from {:?}: {}", path, e);
                }
            }
        }

        Ok(rules)
    }

    /// Load rules from a single file
    fn load_rules_from_file(path: &PathBuf) -> Result<Vec<Rule>> {
        let content = fs::read_to_string(path)?;

        // Try JSON first, then YAML
        if let Ok(rules) = serde_json::from_str::<Vec<Rule>>(&content) {
            return Ok(rules);
        }

        // If YAML support is added later, parse here
        // For now, just JSON

        Ok(Vec::new())
    }

    /// Get config directory
    fn get_config_dir() -> Result<PathBuf> {
        let home = std::env::var("HOME").context("HOME not set")?;
        Ok(PathBuf::from(home).join(".config/anna"))
    }

    /// Evaluate all rules against a radar snapshot
    pub fn evaluate(&self, snapshot: &RadarSnapshot) -> Vec<Recommendation> {
        let mut recommendations = Vec::new();

        for rule in &self.rules {
            if self.check_condition(&rule.condition, snapshot) {
                // Get learned weight if available
                let (learned_weight, auto_confidence, trust_level) = if let Some(learning) = &self.learning {
                    if let Some(weight) = learning.get_weight(&rule.id) {
                        (
                            Some(weight.user_response_weight),
                            Some(weight.auto_confidence),
                            Some(weight.trust_level().to_string()),
                        )
                    } else {
                        (None, None, None)
                    }
                } else {
                    (None, None, None)
                };

                recommendations.push(Recommendation {
                    priority: rule.priority.clone(),
                    category: rule.category.clone(),
                    title: rule.title.clone(),
                    reason: rule.message.clone(),
                    action: rule.action.clone(),
                    emoji: rule.emoji.clone(),
                    impact: rule.impact.clone(),
                    learned_weight,
                    auto_confidence,
                    trust_level,
                });
            }
        }

        // Sort by priority AND learned weights
        recommendations.sort_by(|a, b| {
            let priority_order = |p: &str| match p {
                "critical" => 0,
                "high" => 1,
                "medium" => 2,
                "low" => 3,
                _ => 4,
            };

            // First compare priority
            let pri_cmp = priority_order(&a.priority).cmp(&priority_order(&b.priority));
            if pri_cmp != std::cmp::Ordering::Equal {
                return pri_cmp;
            }

            // Within same priority, sort by learned weight (higher weight first)
            let weight_a = a.learned_weight.unwrap_or(0.0);
            let weight_b = b.learned_weight.unwrap_or(0.0);
            weight_b.partial_cmp(&weight_a).unwrap_or(std::cmp::Ordering::Equal)
        });

        recommendations
    }

    /// Check if a condition is met
    fn check_condition(&self, condition: &Condition, snapshot: &RadarSnapshot) -> bool {
        match condition {
            Condition::Threshold { metric, operator, value } => {
                let metric_value = self.get_metric_value(metric, snapshot);
                self.compare(metric_value, operator, *value)
            }
            Condition::And { conditions } => {
                conditions.iter().all(|c| self.check_condition(c, snapshot))
            }
            Condition::Or { conditions } => {
                conditions.iter().any(|c| self.check_condition(c, snapshot))
            }
        }
    }

    /// Get metric value from snapshot
    fn get_metric_value(&self, metric: &str, snapshot: &RadarSnapshot) -> u8 {
        let parts: Vec<&str> = metric.split('.').collect();

        if parts.len() != 2 {
            return 10; // Default to max if metric format is invalid
        }

        match (parts[0], parts[1]) {
            // Hardware metrics
            ("hardware", "overall") => snapshot.hardware.overall,
            ("hardware", "cpu_throughput") => snapshot.hardware.cpu_throughput,
            ("hardware", "cpu_thermal") => snapshot.hardware.cpu_thermal,
            ("hardware", "memory") => snapshot.hardware.memory,
            ("hardware", "disk_health") => snapshot.hardware.disk_health,
            ("hardware", "disk_free") => snapshot.hardware.disk_free,
            ("hardware", "fs_features") => snapshot.hardware.fs_features,
            ("hardware", "gpu") => snapshot.hardware.gpu,
            ("hardware", "network") => snapshot.hardware.network,
            ("hardware", "boot") => snapshot.hardware.boot,

            // Software metrics
            ("software", "overall") => snapshot.software.overall,
            ("software", "os_freshness") => snapshot.software.os_freshness,
            ("software", "kernel") => snapshot.software.kernel,
            ("software", "packages") => snapshot.software.packages,
            ("software", "services") => snapshot.software.services,
            ("software", "security") => snapshot.software.security,
            ("software", "containers") => snapshot.software.containers,
            ("software", "fs_integrity") => snapshot.software.fs_integrity,
            ("software", "backups") => snapshot.software.backups,
            ("software", "log_noise") => snapshot.software.log_noise,

            // User metrics
            ("user", "overall") => snapshot.user.overall,
            ("user", "regularity") => snapshot.user.regularity,
            ("user", "workspace") => snapshot.user.workspace,
            ("user", "updates") => snapshot.user.updates,
            ("user", "backups") => snapshot.user.backups,
            ("user", "risk") => snapshot.user.risk,
            ("user", "connectivity") => snapshot.user.connectivity,
            ("user", "power") => snapshot.user.power,
            ("user", "warnings") => snapshot.user.warnings,

            _ => 10, // Unknown metric defaults to max
        }
    }

    /// Compare values based on operator
    fn compare(&self, left: u8, operator: &str, right: u8) -> bool {
        match operator {
            "<=" => left <= right,
            ">=" => left >= right,
            "==" => left == right,
            "!=" => left != right,
            "<" => left < right,
            ">" => left > right,
            _ => false,
        }
    }

    /// Get top N recommendations
    pub fn top_recommendations(&self, snapshot: &RadarSnapshot, n: usize) -> Vec<Recommendation> {
        let mut all = self.evaluate(snapshot);
        all.truncate(n);
        all
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mock_snapshot() -> RadarSnapshot {
        RadarSnapshot {
            hardware: HardwareRadar {
                overall: 7,
                cpu_throughput: 8,
                cpu_thermal: 7,
                memory: 6,
                disk_health: 9,
                disk_free: 5,
                fs_features: 6,
                gpu: 7,
                network: 8,
                boot: 7,
            },
            software: SoftwareRadar {
                overall: 6,
                os_freshness: 4,
                kernel: 8,
                packages: 6,
                services: 7,
                security: 5,
                containers: 7,
                fs_integrity: 8,
                backups: 4,
                log_noise: 6,
            },
            user: UserRadar {
                overall: 7,
                regularity: 6,
                workspace: 6,
                updates: 7,
                backups: 7,
                risk: 8,
                connectivity: 8,
                power: 7,
                warnings: 8,
            },
        }
    }

    #[test]
    fn test_threshold_condition() {
        let advisor = Advisor::new().unwrap();
        let snapshot = mock_snapshot();

        let condition = Condition::Threshold {
            metric: "software.os_freshness".to_string(),
            operator: "<=".to_string(),
            value: 5,
        };

        assert!(advisor.check_condition(&condition, &snapshot));
    }

    #[test]
    fn test_and_condition() {
        let advisor = Advisor::new().unwrap();
        let snapshot = mock_snapshot();

        let condition = Condition::And {
            conditions: vec![
                Condition::Threshold {
                    metric: "software.os_freshness".to_string(),
                    operator: "<=".to_string(),
                    value: 5,
                },
                Condition::Threshold {
                    metric: "software.backups".to_string(),
                    operator: "<=".to_string(),
                    value: 5,
                },
            ],
        };

        assert!(advisor.check_condition(&condition, &snapshot));
    }

    #[test]
    fn test_or_condition() {
        let advisor = Advisor::new().unwrap();
        let snapshot = mock_snapshot();

        let condition = Condition::Or {
            conditions: vec![
                Condition::Threshold {
                    metric: "software.os_freshness".to_string(),
                    operator: "<=".to_string(),
                    value: 5,
                },
                Condition::Threshold {
                    metric: "hardware.cpu_throughput".to_string(),
                    operator: ">=".to_string(),
                    value: 9,
                },
            ],
        };

        // First condition is true (os_freshness=4 <= 5)
        assert!(advisor.check_condition(&condition, &snapshot));
    }

    #[test]
    fn test_evaluate_returns_sorted() {
        let advisor = Advisor::new().unwrap();
        let snapshot = mock_snapshot();

        let recs = advisor.evaluate(&snapshot);

        // Should have recommendations (os_freshness=4, disk_free=5, security=5, backups=4)
        assert!(!recs.is_empty());

        // First should be critical
        if !recs.is_empty() {
            assert_eq!(recs[0].priority, "critical");
        }
    }

    #[test]
    fn test_top_recommendations_truncates() {
        let advisor = Advisor::new().unwrap();
        let snapshot = mock_snapshot();

        let top_3 = advisor.top_recommendations(&snapshot, 3);

        assert!(top_3.len() <= 3);
    }

    #[test]
    fn test_metric_parsing() {
        let advisor = Advisor::new().unwrap();
        let snapshot = mock_snapshot();

        assert_eq!(advisor.get_metric_value("hardware.disk_free", &snapshot), 5);
        assert_eq!(advisor.get_metric_value("software.os_freshness", &snapshot), 4);
        assert_eq!(advisor.get_metric_value("user.regularity", &snapshot), 6);
    }

    #[test]
    fn test_comparison_operators() {
        let advisor = Advisor::new().unwrap();

        assert!(advisor.compare(5, "<=", 5));
        assert!(advisor.compare(4, "<=", 5));
        assert!(!advisor.compare(6, "<=", 5));

        assert!(advisor.compare(5, ">=", 5));
        assert!(advisor.compare(6, ">=", 5));
        assert!(!advisor.compare(4, ">=", 5));

        assert!(advisor.compare(5, "==", 5));
        assert!(!advisor.compare(4, "==", 5));

        assert!(advisor.compare(4, "!=", 5));
        assert!(!advisor.compare(5, "!=", 5));

        assert!(advisor.compare(4, "<", 5));
        assert!(!advisor.compare(5, "<", 5));

        assert!(advisor.compare(6, ">", 5));
        assert!(!advisor.compare(5, ">", 5));
    }

    #[test]
    fn test_priority_ordering() {
        let advisor = Advisor::new().unwrap();
        let snapshot = mock_snapshot();

        let recs = advisor.evaluate(&snapshot);

        // Verify critical comes before high, high before medium, etc.
        let mut last_priority_order = -1;
        for rec in recs {
            let order = match rec.priority.as_str() {
                "critical" => 0,
                "high" => 1,
                "medium" => 2,
                "low" => 3,
                _ => 4,
            };
            assert!(order >= last_priority_order);
            last_priority_order = order;
        }
    }
}
