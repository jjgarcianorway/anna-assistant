//! Experiment Scoring - Calculate risk vs information gain.

use super::{SideEffect, SandboxType};
use serde::{Deserialize, Serialize};

/// Score for an experiment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperimentScore {
    /// Overall risk score (0.0-1.0, higher = more risky)
    pub risk_score: f32,
    /// Information gain score (0.0-1.0, higher = more useful)
    pub information_gain: f32,
    /// Net benefit (information_gain - risk_score)
    pub net_benefit: f32,
    /// Confidence in this scoring
    pub confidence: f32,
    /// Factors contributing to risk
    pub risk_factors: Vec<RiskFactor>,
    /// Recommendation
    pub recommendation: ScoreRecommendation,
}

/// A factor contributing to risk
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskFactor {
    /// Factor name
    pub name: String,
    /// Contribution to risk (0.0-1.0)
    pub contribution: f32,
    /// Description
    pub description: String,
}

/// Recommendation based on score
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScoreRecommendation {
    /// Safe to run directly
    RunDirect,
    /// Run with dry-run first
    DryRunFirst,
    /// Run in sandbox
    Sandbox,
    /// Review manually before running
    ManualReview,
    /// Do not run
    DoNotRun,
}

/// Calculate experiment score for commands
pub fn calculate_experiment_score(
    commands: &[String],
    predictions: &[SideEffect],
    sandbox: SandboxType,
) -> ExperimentScore {
    let mut risk_factors = Vec::new();

    // 1. Side effect risk
    let side_effect_risk = calculate_side_effect_risk(predictions);
    if side_effect_risk > 0.1 {
        risk_factors.push(RiskFactor {
            name: "side_effects".to_string(),
            contribution: side_effect_risk * 0.4,
            description: format!(
                "{} predicted side effects",
                predictions.len()
            ),
        });
    }

    // 2. Reversibility risk
    let irreversible_count = predictions.iter().filter(|p| !p.reversible).count();
    let reversibility_risk = if predictions.is_empty() {
        0.0
    } else {
        (irreversible_count as f32 / predictions.len() as f32) * 0.5
    };
    if reversibility_risk > 0.1 {
        risk_factors.push(RiskFactor {
            name: "irreversibility".to_string(),
            contribution: reversibility_risk * 0.3,
            description: format!(
                "{} irreversible operations",
                irreversible_count
            ),
        });
    }

    // 3. Command complexity risk
    let complexity_risk = calculate_complexity_risk(commands);
    if complexity_risk > 0.1 {
        risk_factors.push(RiskFactor {
            name: "complexity".to_string(),
            contribution: complexity_risk * 0.2,
            description: "Command complexity".to_string(),
        });
    }

    // 4. Privilege escalation risk
    let privilege_risk = calculate_privilege_risk(commands);
    if privilege_risk > 0.1 {
        risk_factors.push(RiskFactor {
            name: "privilege".to_string(),
            contribution: privilege_risk * 0.3,
            description: "Requires elevated privileges".to_string(),
        });
    }

    // 5. System-wide impact risk
    let system_risk = calculate_system_impact_risk(predictions);
    if system_risk > 0.1 {
        risk_factors.push(RiskFactor {
            name: "system_impact".to_string(),
            contribution: system_risk * 0.4,
            description: "System-wide impact".to_string(),
        });
    }

    // Calculate total risk
    let total_risk: f32 = risk_factors.iter().map(|f| f.contribution).sum();
    let risk_score = total_risk.min(1.0);

    // Calculate information gain
    let information_gain = calculate_information_gain(commands, predictions);

    // Net benefit
    let net_benefit = information_gain - risk_score;

    // Sandbox mitigation
    let mitigated_risk = risk_score * (1.0 - sandbox.isolation_level() as f32 / 10.0);

    // Determine recommendation
    let recommendation = if mitigated_risk < 0.1 {
        ScoreRecommendation::RunDirect
    } else if mitigated_risk < 0.3 && has_dry_run(commands) {
        ScoreRecommendation::DryRunFirst
    } else if mitigated_risk < 0.5 && sandbox.isolation_level() >= 4 {
        ScoreRecommendation::Sandbox
    } else if mitigated_risk < 0.7 {
        ScoreRecommendation::ManualReview
    } else {
        ScoreRecommendation::DoNotRun
    };

    // Confidence based on prediction quality
    let confidence = if predictions.is_empty() {
        0.5
    } else {
        predictions.iter().map(|p| p.confidence).sum::<f32>() / predictions.len() as f32
    };

    ExperimentScore {
        risk_score,
        information_gain,
        net_benefit,
        confidence,
        risk_factors,
        recommendation,
    }
}

/// Calculate risk from side effects
fn calculate_side_effect_risk(predictions: &[SideEffect]) -> f32 {
    if predictions.is_empty() {
        return 0.0;
    }

    let total_risk: f32 = predictions
        .iter()
        .map(|p| p.effect_type.risk_level() * p.confidence)
        .sum();

    (total_risk / predictions.len() as f32).min(1.0)
}

/// Calculate risk from command complexity
fn calculate_complexity_risk(commands: &[String]) -> f32 {
    let mut risk = 0.0;

    for cmd in commands {
        // Pipes increase complexity
        let pipe_count = cmd.matches('|').count();
        risk += pipe_count as f32 * 0.1;

        // Subshells increase complexity
        let subshell_count = cmd.matches("$(").count() + cmd.matches('`').count();
        risk += subshell_count as f32 * 0.15;

        // Redirections increase risk
        let redirect_count = cmd.matches('>').count();
        risk += redirect_count as f32 * 0.1;

        // Semicolons (multiple commands)
        let semicolon_count = cmd.matches(';').count();
        risk += semicolon_count as f32 * 0.05;

        // && and || chains
        let chain_count = cmd.matches("&&").count() + cmd.matches("||").count();
        risk += chain_count as f32 * 0.05;
    }

    (risk / commands.len().max(1) as f32).min(1.0)
}

/// Calculate risk from privilege requirements
fn calculate_privilege_risk(commands: &[String]) -> f32 {
    let mut needs_root = false;
    let mut root_commands = 0;

    for cmd in commands {
        if cmd.starts_with("sudo ")
            || cmd.contains("pacman -S")
            || cmd.contains("pacman -R")
            || cmd.contains("systemctl ")
            || cmd.contains("mount ")
            || cmd.contains("umount ")
            || cmd.contains("iptables ")
            || cmd.contains("useradd ")
            || cmd.contains("userdel ")
        {
            needs_root = true;
            root_commands += 1;
        }
    }

    if !needs_root {
        return 0.0;
    }

    0.3 + (root_commands as f32 / commands.len() as f32) * 0.3
}

/// Calculate risk from system-wide impact
fn calculate_system_impact_risk(predictions: &[SideEffect]) -> f32 {
    let high_impact_effects = [
        super::SideEffectType::PackageUpgrade,
        super::SideEffectType::SystemReboot,
        super::SideEffectType::KernelModule,
        super::SideEffectType::NetworkChange,
        super::SideEffectType::FirewallRule,
    ];

    let high_impact_count = predictions
        .iter()
        .filter(|p| high_impact_effects.contains(&p.effect_type))
        .count();

    if high_impact_count == 0 {
        return 0.0;
    }

    0.4 + (high_impact_count as f32 / predictions.len().max(1) as f32) * 0.4
}

/// Calculate information gain from commands
fn calculate_information_gain(commands: &[String], predictions: &[SideEffect]) -> f32 {
    let mut gain = 0.0;

    for cmd in commands {
        // Read-only commands have high information gain, low risk
        if is_query_command(cmd) {
            gain += 0.8;
        }
        // Installation commands teach about package management
        else if cmd.contains("pacman -S") || cmd.contains("yay -S") {
            gain += 0.5;
        }
        // Service commands teach about system state
        else if cmd.contains("systemctl") {
            gain += 0.6;
        }
        // File modifications teach about filesystem
        else if cmd.contains("cp ") || cmd.contains("mv ") || cmd.contains("rm ") {
            gain += 0.3;
        }
        // Unknown commands have moderate information value
        else {
            gain += 0.4;
        }
    }

    // Adjust based on prediction count (more effects = more to learn)
    let prediction_bonus = (predictions.len() as f32 * 0.05).min(0.2);

    ((gain / commands.len().max(1) as f32) + prediction_bonus).min(1.0)
}

/// Check if command is a query/read-only command
fn is_query_command(cmd: &str) -> bool {
    let queries = [
        "pacman -Q", "pacman -Ss", "pacman -Si", "pacman -Qi",
        "systemctl status", "systemctl show", "systemctl list",
        "ls", "cat", "head", "tail", "grep", "find",
        "df", "du", "free", "ps", "top", "htop",
        "ip addr", "ip link", "ip route",
        "ss", "netstat",
        "journalctl",
    ];

    queries.iter().any(|q| cmd.contains(q))
}

/// Check if any command has dry-run support
fn has_dry_run(commands: &[String]) -> bool {
    let dry_run_cmds = ["pacman", "yay", "paru", "rsync", "makepkg"];

    for cmd in commands {
        for drc in &dry_run_cmds {
            if cmd.contains(drc) {
                return true;
            }
        }
    }

    false
}

/// Estimate risk for a single command (0.0-1.0)
pub fn estimate_command_risk(cmd: &str) -> f32 {
    let predictions = super::predict_side_effects(&[cmd.to_string()]);
    let score = calculate_experiment_score(&[cmd.to_string()], &predictions, SandboxType::None);
    score.risk_score
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_readonly_low_risk() {
        let risk = estimate_command_risk("ls -la");
        assert!(risk < 0.3);
    }

    #[test]
    fn test_install_moderate_risk() {
        let risk = estimate_command_risk("pacman -S vim");
        assert!(risk > 0.2);
        assert!(risk < 0.7);
    }

    #[test]
    fn test_rm_rf_high_risk() {
        let risk = estimate_command_risk("rm -rf /tmp/test");
        assert!(risk > 0.3);
    }

    #[test]
    fn test_information_gain() {
        let predictions = Vec::new();
        let score = calculate_experiment_score(
            &["pacman -Qi vim".to_string()],
            &predictions,
            SandboxType::None,
        );
        assert!(score.information_gain > 0.5);
    }

    #[test]
    fn test_complexity_risk() {
        let simple_risk = calculate_complexity_risk(&["ls".to_string()]);
        let complex_risk = calculate_complexity_risk(&["cat file | grep foo | sort | uniq".to_string()]);

        assert!(complex_risk > simple_risk);
    }
}
