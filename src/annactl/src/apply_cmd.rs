//! Apply command for Anna v1.1 "Advisor Intelligence"
//!
//! Safely execute low-risk advisor recommendations with rollback support

use anyhow::{Context, Result};
use anna_common::{header, section, status, Level, TermCaps};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::advisor_cmd::Advice;
use crate::audit_log::{AuditEntry, AuditLog};

/// Rollback token for safe action reversal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackToken {
    pub advice_id: String,
    pub executed_at: u64,
    pub command: String,
    pub success: bool,
    pub output: String,
    pub state_snapshot: Option<StateSnapshot>,
}

/// System state snapshot for rollback
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateSnapshot {
    pub packages_before: Vec<String>,
    pub files_modified: Vec<String>,
}

/// Apply mode
pub enum ApplyMode {
    Interactive,  // Ask for each recommendation
    DryRun,       // Show what would happen
    Auto,         // Apply all low-risk automatically
    Specific(String), // Apply specific advice by ID
}

/// Apply recommendations from advisor
pub async fn run_apply(mode: ApplyMode, dry_run: bool, yes: bool) -> Result<()> {
    let caps = TermCaps::detect();

    println!("{}", header(&caps, "Apply Recommendations"));
    println!();

    // Fetch advisor recommendations
    let advice_list = crate::advisor_cmd::fetch_advisor_results().await?;

    if advice_list.is_empty() {
        println!("{}", status(&caps, Level::Ok, "No recommendations to apply"));
        return Ok(());
    }

    // Filter to only actionable advice (has fix_cmd)
    let actionable: Vec<&Advice> = advice_list.iter()
        .filter(|a| a.fix_cmd.is_some())
        .collect();

    if actionable.is_empty() {
        println!("{}", status(&caps, Level::Warn, "No actionable recommendations found"));
        println!();
        println!("  All current recommendations require manual intervention.");
        return Ok(());
    }

    println!("Found {} actionable recommendations", actionable.len());
    println!();

    match mode {
        ApplyMode::Interactive => apply_interactive(&actionable, dry_run).await?,
        ApplyMode::DryRun => apply_dry_run(&actionable).await?,
        ApplyMode::Auto => apply_auto(&actionable, yes).await?,
        ApplyMode::Specific(id) => apply_specific(&actionable, &id, dry_run).await?,
    }

    Ok(())
}

/// Apply interactively (ask for each)
async fn apply_interactive(actionable: &[&Advice], dry_run: bool) -> Result<()> {
    let caps = TermCaps::detect();

    for advice in actionable {
        println!("{}", section(&caps, &advice.title));
        println!();

        print_advice_details(advice)?;

        if dry_run {
            println!("  [DRY RUN] Would execute: {}", advice.fix_cmd.as_ref().unwrap());
            println!();
            continue;
        }

        // Ask user
        print!("Apply this recommendation? [y/N]: ");
        std::io::stdout().flush()?;

        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;

        if input.trim().to_lowercase() != "y" {
            println!("  Skipped");
            println!();
            continue;
        }

        // Execute
        match execute_advice(advice).await {
            Ok(token) => {
                if token.success {
                    println!("{}", status(&caps, Level::Ok, "Applied successfully"));
                    save_rollback_token(&token)?;
                } else {
                    println!("{}", status(&caps, Level::Err, "Failed to apply"));
                    println!("  Output: {}", token.output);
                }
            }
            Err(e) => {
                println!("{}", status(&caps, Level::Err, &format!("Error: {}", e)));
            }
        }
        println!();
    }

    Ok(())
}

/// Show what would be applied (dry run)
async fn apply_dry_run(actionable: &[&Advice]) -> Result<()> {
    let caps = TermCaps::detect();

    println!("{}", section(&caps, "Dry Run - Actions Preview"));
    println!();

    for (i, advice) in actionable.iter().enumerate() {
        println!("{}. {}", i + 1, advice.title);
        println!("   Risk: {}", advice.fix_risk.as_ref().unwrap_or(&"Unknown".to_string()));
        println!("   Command: {}", advice.fix_cmd.as_ref().unwrap());
        println!();
    }

    println!("Run without --dry-run to apply these recommendations");
    Ok(())
}

/// Apply all low-risk recommendations automatically
async fn apply_auto(actionable: &[&Advice], skip_confirm: bool) -> Result<()> {
    let caps = TermCaps::detect();

    // Filter to low-risk only
    let low_risk: Vec<&Advice> = actionable.iter()
        .filter(|a| is_low_risk(a))
        .copied()
        .collect();

    if low_risk.is_empty() {
        println!("{}", status(&caps, Level::Warn, "No low-risk recommendations found"));
        println!();
        println!("  All recommendations require manual review.");
        return Ok(());
    }

    println!("{}", section(&caps, "Auto-Apply Low-Risk Recommendations"));
    println!();
    println!("Found {} low-risk recommendations:", low_risk.len());
    println!();

    for advice in &low_risk {
        println!("  • {}", advice.title);
    }
    println!();

    if !skip_confirm {
        print!("Apply all low-risk recommendations? [y/N]: ");
        std::io::stdout().flush()?;

        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;

        if input.trim().to_lowercase() != "y" {
            println!("Cancelled");
            return Ok(());
        }
    }

    // Execute each
    let mut success_count = 0;
    let mut fail_count = 0;

    for advice in low_risk {
        println!("Applying: {}", advice.title);

        match execute_advice(advice).await {
            Ok(token) => {
                if token.success {
                    success_count += 1;
                    println!("{}", status(&caps, Level::Ok, "Success"));
                    save_rollback_token(&token)?;
                } else {
                    fail_count += 1;
                    println!("{}", status(&caps, Level::Err, "Failed"));
                }
            }
            Err(e) => {
                fail_count += 1;
                println!("{}", status(&caps, Level::Err, &format!("Error: {}", e)));
            }
        }
        println!();
    }

    println!("{}", section(&caps, "Summary"));
    println!();
    println!("  ✓ Applied: {}", success_count);
    println!("  ✗ Failed:  {}", fail_count);
    println!();

    Ok(())
}

/// Apply specific recommendation by ID
async fn apply_specific(actionable: &[&Advice], id: &str, dry_run: bool) -> Result<()> {
    let caps = TermCaps::detect();

    let advice = actionable.iter()
        .find(|a| a.id == id)
        .ok_or_else(|| anyhow::anyhow!("Recommendation not found: {}", id))?;

    println!("{}", section(&caps, &advice.title));
    println!();
    print_advice_details(advice)?;

    if dry_run {
        println!("  [DRY RUN] Would execute: {}", advice.fix_cmd.as_ref().unwrap());
        return Ok(());
    }

    // Execute
    match execute_advice(advice).await {
        Ok(token) => {
            if token.success {
                println!("{}", status(&caps, Level::Ok, "Applied successfully"));
                save_rollback_token(&token)?;
            } else {
                println!("{}", status(&caps, Level::Err, "Failed to apply"));
                println!("  Output: {}", token.output);
            }
        }
        Err(e) => {
            println!("{}", status(&caps, Level::Err, &format!("Error: {}", e)));
        }
    }

    Ok(())
}

/// Check if advice is low-risk
fn is_low_risk(advice: &Advice) -> bool {
    if let Some(risk) = &advice.fix_risk {
        let risk_lower = risk.to_lowercase();
        risk_lower.starts_with("low") || risk_lower.contains("safe")
    } else {
        false
    }
}

/// Print advice details
fn print_advice_details(advice: &Advice) -> Result<()> {
    println!("  Category: {}", advice.category);
    println!("  Reason:   {}", advice.reason);
    println!("  Action:   {}", advice.action);

    if let Some(risk) = &advice.fix_risk {
        println!("  Risk:     {}", risk);
    }

    if let Some(cmd) = &advice.fix_cmd {
        println!("  Command:  {}", cmd);
    }

    if !advice.refs.is_empty() {
        println!("  Wiki:     {}", advice.refs[0]);
    }

    println!();
    Ok(())
}

/// Execute advice and create rollback token
async fn execute_advice(advice: &Advice) -> Result<RollbackToken> {
    let cmd = advice.fix_cmd.as_ref()
        .context("No fix command available")?;

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    // Execute command
    let output = Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .output()?;

    let success = output.status.success();
    let output_str = String::from_utf8_lossy(&output.stdout).to_string()
        + &String::from_utf8_lossy(&output.stderr).to_string();

    // Create rollback token
    let token = RollbackToken {
        advice_id: advice.id.clone(),
        executed_at: now,
        command: cmd.clone(),
        success,
        output: output_str,
        state_snapshot: None, // TODO: Implement state snapshots
    };

    // Log to audit
    if success {
        let audit = AuditLog::new()?;
        let entry = AuditEntry::new("advisor", &format!("Applied: {}", advice.title), "apply")
            .with_action_id(advice.id.clone())
            .with_result("success".to_string())
            .with_details(format!("Command: {}", cmd));

        audit.log(entry)?;
    }

    Ok(token)
}

/// Save rollback token to disk
fn save_rollback_token(token: &RollbackToken) -> Result<()> {
    let state_dir = get_state_dir()?;
    fs::create_dir_all(&state_dir)?;

    let rollback_file = state_dir.join("rollback_tokens.jsonl");

    let json = serde_json::to_string(token)?;
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(rollback_file)?;

    writeln!(file, "{}", json)?;

    Ok(())
}

/// Get state directory
fn get_state_dir() -> Result<PathBuf> {
    let home = std::env::var("HOME").context("HOME not set")?;
    Ok(PathBuf::from(home).join(".local/state/anna"))
}
