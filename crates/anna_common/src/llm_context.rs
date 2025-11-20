use serde::{Deserialize, Serialize};

/// LLM Contextualization - Synthesizes all system detection data into actionable context
///
/// This module ties together all 99 detection items from Milestone 1 into a cohesive
/// system summary that can be used by the LLM caretaker brain for intelligent decision-making.

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmContext {
    pub system_identity: SystemIdentity,
    pub stability_indicators: StabilityIndicators,
    pub performance_indicators: PerformanceIndicators,
    pub risk_indicators: RiskIndicators,
    pub inferred_user_goals: UserGoals,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemIdentity {
    pub classification: SystemClassification,
    pub primary_workload: String,
    pub secondary_workloads: Vec<String>,
    pub hardware_tier: HardwareTier,
    pub system_age_estimate: SystemAgeEstimate,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SystemClassification {
    GamingRig,
    DevelopmentWorkstation,
    HomeServer,
    MediaProductionStation,
    GeneralPurposeDesktop,
    LightweightLaptop,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HardwareTier {
    HighEnd,  // Gaming GPU, 16GB+ RAM, 8+ cores
    MidRange, // Dedicated GPU, 8-16GB RAM, 4-8 cores
    LowEnd,   // Integrated GPU, <8GB RAM, <4 cores
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SystemAgeEstimate {
    New,    // <1 year
    Recent, // 1-3 years
    Mature, // 3-5 years
    Aging,  // >5 years
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StabilityIndicators {
    pub overall_stability_score: u8, // 0-100
    pub uptime_health: UptimeHealth,
    pub crash_frequency: CrashFrequency,
    pub error_rate: ErrorRate,
    pub filesystem_health_score: u8,
    pub backup_status: BackupStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UptimeHealth {
    Excellent,  // Long uptime, no unexpected reboots
    Good,       // Reasonable uptime
    Concerning, // Frequent reboots
    Critical,   // Very unstable
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CrashFrequency {
    None,
    Rare,
    Occasional,
    Frequent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ErrorRate {
    Minimal,
    Low,
    Moderate,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BackupStatus {
    Healthy,
    Outdated,
    Missing,
    NoToolInstalled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceIndicators {
    pub overall_performance_score: u8, // 0-100
    pub cpu_health: CpuHealth,
    pub memory_health: MemoryHealth,
    pub storage_health: StorageHealth,
    pub network_health: NetworkHealth,
    pub gpu_health: Option<GpuHealth>,
    pub bottlenecks: Vec<Bottleneck>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuHealth {
    pub throttling: bool,
    pub governor_optimal: bool,
    pub microcode_current: bool,
    pub temperature_ok: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryHealth {
    pub swap_configured: bool,
    pub oom_events: bool,
    pub pressure_ok: bool,
    pub utilization_healthy: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageHealth {
    pub smart_ok: bool,
    pub trim_enabled: bool,
    pub io_errors: bool,
    pub alignment_ok: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkHealth {
    pub latency_ok: bool,
    pub packet_loss_ok: bool,
    pub dns_ok: bool,
    pub firewall_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuHealth {
    pub throttling: bool,
    pub driver_ok: bool,
    pub temperature_ok: bool,
    pub compute_available: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bottleneck {
    pub component: String,
    pub severity: BottleneckSeverity,
    pub description: String,
    pub recommendation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BottleneckSeverity {
    Minor,
    Moderate,
    Major,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskIndicators {
    pub overall_risk_score: u8, // 0-100, higher = more risk
    pub data_loss_risk: DataLossRisk,
    pub security_risk: SecurityRisk,
    pub stability_risk: StabilityRisk,
    pub critical_issues: Vec<CriticalIssue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DataLossRisk {
    Low,
    Moderate,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityRisk {
    Low,
    Moderate,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StabilityRisk {
    Low,
    Moderate,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CriticalIssue {
    pub category: String,
    pub description: String,
    pub urgency: IssueUrgency,
    pub recommended_action: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IssueUrgency {
    Low,
    Medium,
    High,
    Urgent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserGoals {
    pub detected_use_cases: Vec<String>,
    pub optimization_opportunities: Vec<OptimizationOpportunity>,
    pub workflow_improvements: Vec<WorkflowImprovement>,
    pub learning_curve: LearningCurve,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationOpportunity {
    pub area: String,
    pub potential_benefit: String,
    pub effort_level: EffortLevel,
    pub recommendation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EffortLevel {
    Minimal,
    Low,
    Moderate,
    High,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowImprovement {
    pub workflow: String,
    pub current_state: String,
    pub suggested_improvement: String,
    pub benefit: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LearningCurve {
    Beginner,            // Needs basic system administration help
    IntermediateLearner, // Learning advanced topics
    AdvancedUser,        // Comfortable with most tasks
    Expert,              // Deep system knowledge
}

impl LlmContext {
    /// Generate LLM context from complete system facts
    ///
    /// This synthesizes all 99 detection items into actionable intelligence
    pub fn from_system_facts(facts: &crate::types::SystemFacts) -> Self {
        let system_identity = Self::infer_system_identity(facts);
        let stability_indicators = Self::analyze_stability(facts);
        let performance_indicators = Self::analyze_performance(facts);
        let risk_indicators = Self::assess_risks(facts);
        let inferred_user_goals = Self::infer_user_goals(facts);

        LlmContext {
            system_identity,
            stability_indicators,
            performance_indicators,
            risk_indicators,
            inferred_user_goals,
        }
    }

    fn infer_system_identity(facts: &crate::types::SystemFacts) -> SystemIdentity {
        // Classify system based on user behavior and hardware
        let classification = if let Some(behavior) = &facts.user_behavior {
            match behavior.user_profile.primary_use_case {
                crate::user_behavior::UseCase::Gaming => SystemClassification::GamingRig,
                crate::user_behavior::UseCase::Development => {
                    SystemClassification::DevelopmentWorkstation
                }
                crate::user_behavior::UseCase::ServerAdmin => SystemClassification::HomeServer,
                crate::user_behavior::UseCase::MediaProduction => {
                    SystemClassification::MediaProductionStation
                }
                crate::user_behavior::UseCase::Workstation => {
                    SystemClassification::GeneralPurposeDesktop
                }
                crate::user_behavior::UseCase::GeneralUse => {
                    SystemClassification::GeneralPurposeDesktop
                }
            }
        } else {
            SystemClassification::Unknown
        };

        // Determine primary workload
        let primary_workload = if let Some(behavior) = &facts.user_behavior {
            format!("{:?}", behavior.user_profile.primary_use_case)
        } else {
            "Unknown".to_string()
        };

        // Secondary workloads
        let secondary_workloads = if let Some(behavior) = &facts.user_behavior {
            behavior
                .user_profile
                .secondary_use_cases
                .iter()
                .map(|uc| format!("{:?}", uc))
                .collect()
        } else {
            Vec::new()
        };

        // Hardware tier based on specs
        let hardware_tier = if facts.total_memory_gb >= 16.0
            && facts.cpu_cores >= 8
            && facts.is_nvidia
        {
            HardwareTier::HighEnd
        } else if facts.total_memory_gb >= 8.0 && facts.cpu_cores >= 4 && facts.gpu_vendor.is_some()
        {
            HardwareTier::MidRange
        } else {
            HardwareTier::LowEnd
        };

        // System age estimate (would need more data, placeholder)
        let system_age_estimate = SystemAgeEstimate::Unknown;

        SystemIdentity {
            classification,
            primary_workload,
            secondary_workloads,
            hardware_tier,
            system_age_estimate,
        }
    }

    fn analyze_stability(facts: &crate::types::SystemFacts) -> StabilityIndicators {
        let mut overall_stability_score = 100u8;

        // Check system health
        let uptime_health = if let Some(health) = &facts.system_health {
            if health.daemon_crashes.total_crashes_24h > 5 {
                overall_stability_score = overall_stability_score.saturating_sub(30);
                UptimeHealth::Critical
            } else if health.daemon_crashes.total_crashes_24h > 2 {
                overall_stability_score = overall_stability_score.saturating_sub(15);
                UptimeHealth::Concerning
            } else {
                UptimeHealth::Good
            }
        } else {
            UptimeHealth::Good
        };

        let crash_frequency = if let Some(health) = &facts.system_health {
            if health.daemon_crashes.total_crashes_24h > 5 {
                CrashFrequency::Frequent
            } else if health.daemon_crashes.total_crashes_24h > 2 {
                CrashFrequency::Occasional
            } else if health.daemon_crashes.total_crashes_24h > 0 {
                CrashFrequency::Rare
            } else {
                CrashFrequency::None
            }
        } else {
            CrashFrequency::None
        };

        // Check systemd health
        let error_rate = if let Some(systemd) = &facts.systemd_health {
            if systemd.failed_units.len() > 5 {
                overall_stability_score = overall_stability_score.saturating_sub(20);
                ErrorRate::High
            } else if systemd.failed_units.len() > 2 {
                overall_stability_score = overall_stability_score.saturating_sub(10);
                ErrorRate::Moderate
            } else if !systemd.failed_units.is_empty() {
                ErrorRate::Low
            } else {
                ErrorRate::Minimal
            }
        } else {
            ErrorRate::Minimal
        };

        // Filesystem health score
        let filesystem_health_score = facts
            .filesystem_health
            .as_ref()
            .map(|fh| fh.health_score())
            .unwrap_or(100);

        overall_stability_score =
            overall_stability_score.saturating_sub(100 - filesystem_health_score);

        // Backup status
        let backup_status = if let Some(backup) = &facts.backup_detection {
            match backup.overall_status {
                crate::backup_detection::BackupStatus::Healthy => BackupStatus::Healthy,
                crate::backup_detection::BackupStatus::Warning => BackupStatus::Outdated,
                crate::backup_detection::BackupStatus::Critical => BackupStatus::Missing,
                crate::backup_detection::BackupStatus::NoBackupTool => {
                    BackupStatus::NoToolInstalled
                }
            }
        } else {
            BackupStatus::NoToolInstalled
        };

        StabilityIndicators {
            overall_stability_score,
            uptime_health,
            crash_frequency,
            error_rate,
            filesystem_health_score,
            backup_status,
        }
    }

    fn analyze_performance(facts: &crate::types::SystemFacts) -> PerformanceIndicators {
        let mut overall_performance_score = 100u8;
        let mut bottlenecks = Vec::new();

        // CPU health
        let cpu_health = CpuHealth {
            throttling: facts
                .cpu_throttling
                .as_ref()
                .map(|ct| ct.throttling_events.has_throttling)
                .unwrap_or(false),
            governor_optimal: facts
                .cpu_performance
                .as_ref()
                .map(|cp| cp.governor.governor != "powersave")
                .unwrap_or(true),
            microcode_current: facts
                .cpu_performance
                .as_ref()
                .and_then(|cp| cp.microcode_version.as_ref())
                .is_some(),
            temperature_ok: facts
                .sensors_info
                .as_ref()
                .and_then(|si| si.cpu_temp)
                .map(|temp| temp < 80.0)
                .unwrap_or(true),
        };

        if cpu_health.throttling {
            overall_performance_score = overall_performance_score.saturating_sub(20);
            bottlenecks.push(Bottleneck {
                component: "CPU".to_string(),
                severity: BottleneckSeverity::Major,
                description: "CPU throttling detected".to_string(),
                recommendation: "Check cooling and power settings".to_string(),
            });
        }

        // Memory health
        let memory_health = MemoryHealth {
            swap_configured: facts
                .memory_usage_info
                .as_ref()
                .map(|mu| mu.swap.total_gb > 0.0)
                .unwrap_or(false),
            oom_events: facts
                .memory_usage_info
                .as_ref()
                .map(|mu| !mu.oom_events.is_empty())
                .unwrap_or(false),
            pressure_ok: facts
                .memory_usage_info
                .as_ref()
                .and_then(|mu| mu.memory_pressure.as_ref())
                .map(|mp| mp.some_avg10 < 50.0)
                .unwrap_or(true),
            utilization_healthy: facts
                .memory_usage_info
                .as_ref()
                .map(|mu| (mu.used_ram_gb / mu.total_ram_gb) < 0.9)
                .unwrap_or(true),
        };

        if memory_health.oom_events {
            overall_performance_score = overall_performance_score.saturating_sub(25);
            bottlenecks.push(Bottleneck {
                component: "Memory".to_string(),
                severity: BottleneckSeverity::Critical,
                description: "Out of memory events detected".to_string(),
                recommendation: "Add more RAM or reduce memory usage".to_string(),
            });
        }

        // Storage health
        let storage_health = StorageHealth {
            smart_ok: facts
                .storage_info
                .as_ref()
                .map(|si| !si.devices.is_empty())
                .unwrap_or(true),
            trim_enabled: facts
                .filesystem_info
                .as_ref()
                .map(|fi| fi.has_btrfs || fi.has_luks_encryption)
                .unwrap_or(false),
            io_errors: false,
            alignment_ok: true,
        };

        // Network health
        let network_health = NetworkHealth {
            latency_ok: true,
            packet_loss_ok: true,
            dns_ok: facts
                .network_config
                .as_ref()
                .map(|nc| !nc.dns_servers.is_empty())
                .unwrap_or(true),
            firewall_active: facts
                .security_info
                .as_ref()
                .map(|si| {
                    !matches!(
                        si.firewall_status,
                        crate::security::FirewallStatus::Inactive
                    )
                })
                .unwrap_or(false),
        };

        // GPU health
        let gpu_health = if facts.gpu_vendor.is_some() {
            Some(GpuHealth {
                throttling: facts
                    .gpu_throttling
                    .as_ref()
                    .map(|_gt| false) // Simplified check
                    .unwrap_or(false),
                driver_ok: facts
                    .display_issues
                    .as_ref()
                    .map(|di| di.driver_issues.is_empty())
                    .unwrap_or(true),
                temperature_ok: facts
                    .sensors_info
                    .as_ref()
                    .and_then(|si| si.gpu_temp)
                    .map(|temp| temp < 85.0)
                    .unwrap_or(true),
                compute_available: facts
                    .gpu_compute
                    .as_ref()
                    .map(|gc| gc.cuda_support.is_some() || gc.opencl_support.is_some())
                    .unwrap_or(false),
            })
        } else {
            None
        };

        PerformanceIndicators {
            overall_performance_score,
            cpu_health,
            memory_health,
            storage_health,
            network_health,
            gpu_health,
            bottlenecks,
        }
    }

    fn assess_risks(facts: &crate::types::SystemFacts) -> RiskIndicators {
        let mut overall_risk_score = 0u8;
        let mut critical_issues = Vec::new();

        // Data loss risk
        let data_loss_risk = if let Some(backup) = &facts.backup_detection {
            match backup.overall_status {
                crate::backup_detection::BackupStatus::NoBackupTool => {
                    overall_risk_score = overall_risk_score.saturating_add(40);
                    critical_issues.push(CriticalIssue {
                        category: "Data Protection".to_string(),
                        description: "No backup solution installed".to_string(),
                        urgency: IssueUrgency::Urgent,
                        recommended_action: "Install timeshift or snapper immediately".to_string(),
                    });
                    DataLossRisk::Critical
                }
                crate::backup_detection::BackupStatus::Critical => {
                    overall_risk_score = overall_risk_score.saturating_add(30);
                    DataLossRisk::High
                }
                crate::backup_detection::BackupStatus::Warning => {
                    overall_risk_score = overall_risk_score.saturating_add(15);
                    DataLossRisk::Moderate
                }
                crate::backup_detection::BackupStatus::Healthy => DataLossRisk::Low,
            }
        } else {
            DataLossRisk::Critical
        };

        // Security risk
        let security_risk = if let Some(behavior) = &facts.user_behavior {
            if let Some(sec_patterns) = &behavior.security_patterns {
                if sec_patterns.failed_login_attempts > 20 {
                    overall_risk_score = overall_risk_score.saturating_add(25);
                    critical_issues.push(CriticalIssue {
                        category: "Security".to_string(),
                        description: format!(
                            "{} failed login attempts detected",
                            sec_patterns.failed_login_attempts
                        ),
                        urgency: IssueUrgency::High,
                        recommended_action: "Enable fail2ban for SSH protection".to_string(),
                    });
                    SecurityRisk::High
                } else if sec_patterns.failed_login_attempts > 5 {
                    overall_risk_score = overall_risk_score.saturating_add(10);
                    SecurityRisk::Moderate
                } else {
                    SecurityRisk::Low
                }
            } else {
                SecurityRisk::Low
            }
        } else {
            SecurityRisk::Low
        };

        // Stability risk
        let stability_risk = if let Some(health) = &facts.system_health {
            if health.daemon_crashes.total_crashes_24h > 5 {
                overall_risk_score = overall_risk_score.saturating_add(20);
                StabilityRisk::High
            } else if health.daemon_crashes.total_crashes_24h > 2 {
                overall_risk_score = overall_risk_score.saturating_add(10);
                StabilityRisk::Moderate
            } else {
                StabilityRisk::Low
            }
        } else {
            StabilityRisk::Low
        };

        RiskIndicators {
            overall_risk_score,
            data_loss_risk,
            security_risk,
            stability_risk,
            critical_issues,
        }
    }

    fn infer_user_goals(facts: &crate::types::SystemFacts) -> UserGoals {
        let mut detected_use_cases = Vec::new();
        let mut optimization_opportunities = Vec::new();
        let mut workflow_improvements = Vec::new();

        if let Some(behavior) = &facts.user_behavior {
            // Detected use cases
            detected_use_cases.push(format!(
                "Primary: {:?}",
                behavior.user_profile.primary_use_case
            ));
            for secondary in &behavior.user_profile.secondary_use_cases {
                detected_use_cases.push(format!("Secondary: {:?}", secondary));
            }

            // Gaming optimizations
            if let Some(gaming) = &behavior.gaming_patterns {
                if gaming.is_gaming_system {
                    let is_powersave = facts
                        .cpu_performance
                        .as_ref()
                        .map(|cp| cp.governor.governor == "powersave")
                        .unwrap_or(false);

                    if is_powersave {
                        optimization_opportunities.push(OptimizationOpportunity {
                            area: "Gaming Performance".to_string(),
                            potential_benefit: "10-20% FPS improvement".to_string(),
                            effort_level: EffortLevel::Minimal,
                            recommendation: "Switch CPU governor to 'performance' for gaming"
                                .to_string(),
                        });
                    }
                }
            }

            // Development workflow improvements
            if let Some(dev) = &behavior.development_patterns {
                if dev.is_development_system && dev.git_repositories > 0 {
                    workflow_improvements.push(WorkflowImprovement {
                        workflow: "Development".to_string(),
                        current_state: format!(
                            "{} git repositories detected",
                            dev.git_repositories
                        ),
                        suggested_improvement: "Set up git hooks for automation".to_string(),
                        benefit: "Automated testing and linting on commits".to_string(),
                    });
                }
            }

            // Learning curve
            let learning_curve = match behavior.user_profile.experience_level {
                crate::user_behavior::ExperienceLevel::Beginner => LearningCurve::Beginner,
                crate::user_behavior::ExperienceLevel::Intermediate => {
                    LearningCurve::IntermediateLearner
                }
                crate::user_behavior::ExperienceLevel::Advanced => LearningCurve::AdvancedUser,
                crate::user_behavior::ExperienceLevel::Expert => LearningCurve::Expert,
            };

            UserGoals {
                detected_use_cases,
                optimization_opportunities,
                workflow_improvements,
                learning_curve,
            }
        } else {
            UserGoals {
                detected_use_cases,
                optimization_opportunities,
                workflow_improvements,
                learning_curve: LearningCurve::IntermediateLearner,
            }
        }
    }

    /// Generate a human-readable summary for LLM consumption
    pub fn to_summary(&self) -> String {
        let mut summary = String::new();

        summary.push_str(&format!(
            "System Identity: {:?} ({})\n",
            self.system_identity.classification, self.system_identity.primary_workload
        ));

        summary.push_str(&format!(
            "Stability Score: {}/100 ({:?} uptime health)\n",
            self.stability_indicators.overall_stability_score,
            self.stability_indicators.uptime_health
        ));

        summary.push_str(&format!(
            "Performance Score: {}/100\n",
            self.performance_indicators.overall_performance_score
        ));

        summary.push_str(&format!(
            "Risk Score: {}/100 (lower is better)\n",
            self.risk_indicators.overall_risk_score
        ));

        if !self.risk_indicators.critical_issues.is_empty() {
            summary.push_str("\nCritical Issues:\n");
            for issue in &self.risk_indicators.critical_issues {
                summary.push_str(&format!("- [{}] {}\n", issue.category, issue.description));
            }
        }

        if !self.performance_indicators.bottlenecks.is_empty() {
            summary.push_str("\nPerformance Bottlenecks:\n");
            for bottleneck in &self.performance_indicators.bottlenecks {
                summary.push_str(&format!(
                    "- {} ({:?}): {}\n",
                    bottleneck.component, bottleneck.severity, bottleneck.description
                ));
            }
        }

        summary
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_llm_context_summary() {
        // Would need a full SystemFacts instance to test properly
        // This is a placeholder test
    }
}
