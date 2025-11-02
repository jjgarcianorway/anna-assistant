//! Autonomy Command Interface for Anna v0.14.0 "Orion III" Phase 2.3
//!
//! CLI commands for managing autonomy tiers and action confidence

use anyhow::Result;
use clap::Parser;

use crate::autonomy::{AutonomyManager, AutonomyTier};

/// Autonomy command arguments
#[derive(Parser, Debug)]
pub struct AutonomyArgs {
    /// Show current autonomy status
    #[arg(long)]
    pub status: bool,

    /// Manually promote to next tier
    #[arg(long)]
    pub promote: bool,

    /// Manually demote to previous tier
    #[arg(long)]
    pub demote: bool,

    /// Set specific tier (observer, assisted, autonomous)
    #[arg(long)]
    pub set_tier: Option<String>,

    /// Show confidence levels for all actions
    #[arg(long)]
    pub confidence: bool,

    /// Check if ready for tier promotion
    #[arg(long)]
    pub check_promotion: bool,
}

/// Execute autonomy command
pub fn execute(args: &AutonomyArgs) -> Result<()> {
    let mut manager = AutonomyManager::new()?;

    if args.status {
        show_status(&manager)?;
    } else if args.promote {
        promote_tier(&mut manager)?;
    } else if args.demote {
        demote_tier(&mut manager)?;
    } else if let Some(ref tier_str) = args.set_tier {
        set_tier(&mut manager, tier_str)?;
    } else if args.confidence {
        show_confidence(&manager)?;
    } else if args.check_promotion {
        check_promotion_readiness(&manager)?;
    } else {
        // Default: show status
        show_status(&manager)?;
    }

    Ok(())
}

/// Show current autonomy status
fn show_status(manager: &AutonomyManager) -> Result<()> {
    let tier = manager.get_tier();
    let emoji = tier_emoji(&tier);
    let description = tier_description(&tier);

    println!("╭─ Autonomy Status ────────────────────────────────────");
    println!("│");
    println!("│  Current Tier: {} {}", emoji, tier_name(&tier));
    println!("│  Description: {}", description);
    println!("│");

    // Show tier capabilities
    println!("│  Capabilities:");
    match tier {
        AutonomyTier::Observer => {
            println!("│    • Log recommended actions only");
            println!("│    • No automatic execution");
            println!("│    • Full user control");
        }
        AutonomyTier::Assisted => {
            println!("│    • Prompt for confirmation (≥50% confidence)");
            println!("│    • Auto-run high confidence actions (≥80%)");
            println!("│    • 24-hour cooldown between runs");
        }
        AutonomyTier::Autonomous => {
            println!("│    • Auto-run all confident actions (≥50%)");
            println!("│    • Intelligent cooldown management");
            println!("│    • Self-escalation and demotion");
        }
    }

    println!("│");

    // Show action confidence count
    let confidence_map = manager.get_all_confidence();
    let high_confidence = confidence_map.values().filter(|c| c.confidence >= 0.8).count();
    let medium_confidence = confidence_map.values().filter(|c| c.confidence >= 0.5 && c.confidence < 0.8).count();
    let low_confidence = confidence_map.values().filter(|c| c.confidence < 0.5).count();

    println!("│  Action Confidence:");
    println!("│    High (≥80%):   {} actions", high_confidence);
    println!("│    Medium (50-80%): {} actions", medium_confidence);
    println!("│    Low (<50%):    {} actions", low_confidence);
    println!("│");

    println!("╰──────────────────────────────────────────────────────");

    Ok(())
}

/// Promote to next autonomy tier
fn promote_tier(manager: &mut AutonomyManager) -> Result<()> {
    let old_tier = manager.get_tier();

    match manager.promote() {
        Ok(new_tier) => {
            println!("╭─ Autonomy Promotion ─────────────────────────────────");
            println!("│");
            println!("│  ✅ Promoted: {} {} → {} {}",
                tier_emoji(&old_tier), tier_name(&old_tier),
                tier_emoji(&new_tier), tier_name(&new_tier));
            println!("│");
            println!("│  New Capabilities:");
            match new_tier {
                AutonomyTier::Assisted => {
                    println!("│    • Anna will now prompt for action confirmation");
                    println!("│    • High-confidence actions (≥80%) auto-run");
                }
                AutonomyTier::Autonomous => {
                    println!("│    • Anna now operates autonomously");
                    println!("│    • All confident actions (≥50%) auto-run");
                    println!("│    • Self-healing and adaptation enabled");
                }
                _ => {}
            }
            println!("│");
            println!("╰──────────────────────────────────────────────────────");
        }
        Err(e) => {
            println!("❌ Cannot promote: {}", e);
            if matches!(old_tier, AutonomyTier::Autonomous) {
                println!("   Already at highest tier (Autonomous)");
            }
        }
    }

    Ok(())
}

/// Demote to previous autonomy tier
fn demote_tier(manager: &mut AutonomyManager) -> Result<()> {
    let old_tier = manager.get_tier();

    match manager.demote() {
        Ok(new_tier) => {
            println!("╭─ Autonomy Demotion ──────────────────────────────────");
            println!("│");
            println!("│  ⚠ Demoted: {} {} → {} {}",
                tier_emoji(&old_tier), tier_name(&old_tier),
                tier_emoji(&new_tier), tier_name(&new_tier));
            println!("│");
            println!("│  Reduced Capabilities:");
            match new_tier {
                AutonomyTier::Observer => {
                    println!("│    • Anna will only log recommendations");
                    println!("│    • No automatic execution");
                }
                AutonomyTier::Assisted => {
                    println!("│    • Anna will prompt for confirmation");
                    println!("│    • Limited auto-execution (≥80% confidence)");
                }
                _ => {}
            }
            println!("│");
            println!("╰──────────────────────────────────────────────────────");
        }
        Err(e) => {
            println!("❌ Cannot demote: {}", e);
            if matches!(old_tier, AutonomyTier::Observer) {
                println!("   Already at lowest tier (Observer)");
            }
        }
    }

    Ok(())
}

/// Set specific autonomy tier
fn set_tier(manager: &mut AutonomyManager, tier_str: &str) -> Result<()> {
    let tier = match tier_str.to_lowercase().as_str() {
        "observer" => AutonomyTier::Observer,
        "assisted" => AutonomyTier::Assisted,
        "autonomous" => AutonomyTier::Autonomous,
        _ => {
            println!("❌ Invalid tier: {}", tier_str);
            println!("   Valid tiers: observer, assisted, autonomous");
            return Ok(());
        }
    };

    manager.set_tier(tier)?;

    println!("╭─ Autonomy Tier Change ───────────────────────────────");
    println!("│");
    println!("│  ✅ Tier set to: {} {}", tier_emoji(&tier), tier_name(&tier));
    println!("│");
    println!("│  {}", tier_description(&tier));
    println!("│");
    println!("╰──────────────────────────────────────────────────────");

    Ok(())
}

/// Show confidence levels for all actions
fn show_confidence(manager: &AutonomyManager) -> Result<()> {
    let confidence_map = manager.get_all_confidence();

    if confidence_map.is_empty() {
        println!("No action confidence data available yet.");
        return Ok(());
    }

    // Sort by confidence (descending)
    let mut sorted: Vec<_> = confidence_map.iter().collect();
    sorted.sort_by(|a, b| b.1.confidence.partial_cmp(&a.1.confidence).unwrap());

    println!("╭─ Action Confidence Levels ───────────────────────────");
    println!("│");
    println!("│  Action                        Confidence  Runs  Success%");
    println!("│  ─────────────────────────────────────────────────────");

    for (action_id, conf) in sorted.iter().take(15) {
        let success_rate = if conf.total_runs > 0 {
            ((conf.successful_runs as f32 / conf.total_runs as f32) * 100.0) as u32
        } else {
            0
        };

        let conf_emoji = if conf.confidence >= 0.8 {
            "🟢"
        } else if conf.confidence >= 0.5 {
            "🟡"
        } else {
            "🔴"
        };

        println!(
            "│  {} {:<25} {:>5.1}%  {:>4}  {:>7}%",
            conf_emoji,
            truncate(action_id, 25),
            conf.confidence * 100.0,
            conf.total_runs,
            success_rate
        );
    }

    if sorted.len() > 15 {
        println!("│");
        println!("│  ... and {} more actions", sorted.len() - 15);
    }

    println!("│");
    println!("╰──────────────────────────────────────────────────────");

    Ok(())
}

/// Check if ready for tier promotion
fn check_promotion_readiness(manager: &AutonomyManager) -> Result<()> {
    let current_tier = manager.get_tier();

    // Simulate readiness check (in real implementation, this would check actual metrics)
    let overall_health = 85; // Example value
    let critical_anomalies = false;
    let perf_drift = 8.5; // Example: 8.5% performance drift

    println!("╭─ Promotion Readiness Check ──────────────────────────");
    println!("│");
    println!("│  Current Tier: {} {}", tier_emoji(&current_tier), tier_name(&current_tier));
    println!("│");

    // Check promotion criteria
    let health_ok = overall_health >= 80;
    let no_critical = !critical_anomalies;
    let perf_ok = perf_drift < 15.0;

    println!("│  System Health Criteria:");
    println!("│    {} Overall health: {}%",
        if health_ok { "✅" } else { "❌" }, overall_health);
    println!("│    {} No critical anomalies",
        if no_critical { "✅" } else { "❌" });
    println!("│    {} Performance drift: {:.1}%",
        if perf_ok { "✅" } else { "❌" }, perf_drift);
    println!("│");

    // Check confidence criteria
    let confidence_map = manager.get_all_confidence();
    let high_conf_count = confidence_map.values()
        .filter(|c| c.ready_for_escalation())
        .count();

    let conf_criteria_met = high_conf_count >= 3;

    println!("│  Action Confidence Criteria:");
    println!("│    {} High-confidence actions: {} (need ≥3)",
        if conf_criteria_met { "✅" } else { "❌" }, high_conf_count);
    println!("│");

    // Overall readiness
    let ready = health_ok && no_critical && perf_ok && conf_criteria_met;

    if ready {
        println!("│  ✅ Ready for promotion to next tier!");
        println!("│");
        println!("│     Run: annactl autonomy --promote");
    } else {
        println!("│  ⏳ Not yet ready for promotion");
        println!("│");
        println!("│     Continue learning and build confidence.");
    }

    println!("│");
    println!("╰──────────────────────────────────────────────────────");

    Ok(())
}

/// Get tier emoji
fn tier_emoji(tier: &AutonomyTier) -> &str {
    match tier {
        AutonomyTier::Observer => "👀",
        AutonomyTier::Assisted => "🤝",
        AutonomyTier::Autonomous => "🤖",
    }
}

/// Get tier name
fn tier_name(tier: &AutonomyTier) -> &str {
    match tier {
        AutonomyTier::Observer => "Observer",
        AutonomyTier::Assisted => "Assisted",
        AutonomyTier::Autonomous => "Autonomous",
    }
}

/// Get tier description
fn tier_description(tier: &AutonomyTier) -> &str {
    match tier {
        AutonomyTier::Observer => "Logs recommended actions only, no execution",
        AutonomyTier::Assisted => "Prompts for confirmation, auto-runs high-confidence (≥80%)",
        AutonomyTier::Autonomous => "Auto-runs confident actions (≥50%), self-healing enabled",
    }
}

/// Truncate string to max length
fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}
