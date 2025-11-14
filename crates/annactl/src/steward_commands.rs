//! Status command - system health with REAL analysis
//!
//! Phase 4.3: Enhanced with caretaker brain intelligence
//! Phase 4.7: Profile-aware with noise control tracking
//! Citation: [archwiki:System_maintenance]

use anna_common::caretaker_brain::{CaretakerBrain, IssueSeverity, IssueVisibility};
use anna_common::context::{self, apply_issue_decisions, apply_visibility_hints, NoiseControlConfig};
use anna_common::disk_analysis::DiskAnalysis;
use anna_common::display::*;
use anna_common::ipc::ResponseData;
use anna_common::profile::MachineProfile;
use anyhow::{Context, Result};
use chrono::Utc;
use std::time::Instant;

use crate::context_detection;
use crate::errors::*;
use crate::json_types::{profile_to_string, DiskSummaryJson, HealthSummaryJson, IssueJson, StatusJson};
use crate::logging::LogEntry;
use crate::rpc_client::RpcClient;

/// Execute 'status' command - show system health with intelligent analysis
pub async fn execute_status_command(
    json: bool,
    req_id: &str,
    state: &str,
    start_time: Instant,
) -> Result<()> {
    let mut client = RpcClient::connect().await?;
    let use_color = context_detection::should_use_color();

    // Run comprehensive health probes
    let probes = vec![
        "pacman-db".to_string(),
        "systemd-units".to_string(),
        "journal-errors".to_string(),
        "services-failed".to_string(),
        "tlp-config".to_string(),
        "bluetooth-service".to_string(),
        "missing-firmware".to_string(),
    ];

    let health_response = client.health_run(15000, probes).await?;
    let health_data = match health_response {
        ResponseData::HealthRun(data) => data,
        _ => anyhow::bail!("Invalid health response from daemon"),
    };

    // Do REAL disk analysis
    let disk_analysis = DiskAnalysis::analyze_root()?;

    // Detect machine profile (Phase 4.6)
    let profile = MachineProfile::detect();

    // Phase 4.7: Initialize context database (idempotent, safe to call every time)
    let _ = context::ensure_initialized().await;

    // Use caretaker brain for intelligent analysis (profile-aware)
    let mut caretaker_analysis = CaretakerBrain::analyze(
        Some(&health_data.results),
        Some(&disk_analysis),
        profile
    );

    // Phase 4.7: Apply visibility hints for noise control tracking
    // Note: Status shows ALL issues regardless of visibility (unlike daily)
    if let Some(db) = context::db() {
        let config = NoiseControlConfig::default();
        let issues_clone = caretaker_analysis.issues.clone();
        if let Ok(issues_with_hints) = db.execute(move |conn| {
            apply_visibility_hints(conn, issues_clone, &config)
        }).await {
            caretaker_analysis.issues = issues_with_hints;
        }
    }

    // Phase 4.9: Apply user decisions (acknowledge/snooze) on top of noise control
    // Status shows ALL issues, but tracks decisions for display markers
    if let Some(db) = context::db() {
        let issues_clone = caretaker_analysis.issues.clone();
        if let Ok(issues_with_decisions) = db.execute(move |conn| {
            apply_issue_decisions(conn, issues_clone)
        }).await {
            caretaker_analysis.issues = issues_with_decisions;
        }
    }

    // Phase 5.2: Record observations for behavioral analysis
    // This happens AFTER all visibility hints and decisions are applied
    let profile_str = profile_to_string(profile);
    for issue in &caretaker_analysis.issues {
        // Use repair_action_id as stable key, fallback to title if not present
        let issue_key = issue.repair_action_id.clone()
            .unwrap_or_else(|| issue.title.clone());

        let severity_int = severity_to_int(&issue.severity);
        let visible = issue.visibility != IssueVisibility::Deemphasized;
        let decision = issue.decision_info.as_ref().map(|(d, _)| d.clone());

        // Silently record observation - no error handling needed (fire and forget)
        let _ = context::record_observation(
            issue_key,
            severity_int,
            profile_str.clone(),
            visible,
            decision,
        ).await;
    }

    // Determine status level from caretaker analysis
    let status_level = match caretaker_analysis.overall_status.as_str() {
        "critical" => StatusLevel::Critical,
        "needs-attention" => StatusLevel::Warning,
        _ => StatusLevel::Success,
    };

    // Phase 4.9: JSON output mode
    if json {
        let json_output = StatusJson {
            schema_version: "v1".to_string(),  // Phase 5.1: Stable JSON schema versioning
            profile: profile_to_string(profile),
            timestamp: Utc::now().to_rfc3339(),
            health: HealthSummaryJson {
                ok: health_data.summary.ok,
                warnings: health_data.summary.warn,
                failures: health_data.summary.fail,
            },
            disk: Some(DiskSummaryJson {
                used_percent: disk_analysis.usage_percent,
                total_bytes: disk_analysis.total_bytes,
                available_bytes: disk_analysis.available_bytes,
            }),
            // Status shows ALL issues including deemphasized
            issues: caretaker_analysis.issues.iter()
                .map(|issue| IssueJson::from_caretaker_issue(issue))
                .collect(),
        };

        println!("{}", serde_json::to_string_pretty(&json_output)?);

        // Log command
        let duration_ms = start_time.elapsed().as_millis() as u64;
        let exit_code = if caretaker_analysis.overall_status == "critical" {
            1
        } else {
            EXIT_SUCCESS
        };
        let log_entry = LogEntry {
            ts: LogEntry::now(),
            req_id: req_id.to_string(),
            state: state.to_string(),
            command: "status".to_string(),
            allowed: Some(true),
            args: vec!["--json".to_string()],
            exit_code,
            citation: "[archwiki:System_maintenance]".to_string(),
            duration_ms,
            ok: exit_code == EXIT_SUCCESS,
            error: None,
        };
        let _ = log_entry.write();

        std::process::exit(exit_code);
    }

    // Main status section with profile information (Phase 4.7)
    let profile_name = match profile {
        MachineProfile::Laptop => "Laptop",
        MachineProfile::Desktop => "Desktop",
        MachineProfile::ServerLike => "Server-Like",
        MachineProfile::Unknown => "Unknown",
    };

    let mut section = Section::new(
        format!("System Status ({}) - {}", profile_name, caretaker_analysis.overall_status),
        status_level,
        use_color,
    );

    section.add_line(format!("State: {}", state));
    section.add_line(&caretaker_analysis.summary);
    section.add_blank();

    // Disk status
    let disk_status = format!(
        "Disk: {:.1}% used ({} available)",
        disk_analysis.usage_percent,
        format_bytes(disk_analysis.available_bytes)
    );
    section.add_line(disk_status);

    // Health summary
    section.add_line(format!(
        "Health: {} ok, {} warnings, {} failures",
        health_data.summary.ok,
        health_data.summary.warn,
        health_data.summary.fail
    ));

    print!("{}", section.render());

    // Show detailed analysis from caretaker brain
    if !caretaker_analysis.issues.is_empty() {
        println!("\n📊 Detailed Analysis:\n");

        // Group issues by severity
        let critical: Vec<_> = caretaker_analysis.issues.iter()
            .filter(|i| i.severity == IssueSeverity::Critical)
            .collect();
        let warnings: Vec<_> = caretaker_analysis.issues.iter()
            .filter(|i| i.severity == IssueSeverity::Warning)
            .collect();
        let info: Vec<_> = caretaker_analysis.issues.iter()
            .filter(|i| i.severity == IssueSeverity::Info)
            .collect();

        // Show critical issues first
        if !critical.is_empty() {
            println!("🔴 Critical Issues:\n");
            for issue in critical {
                // Phase 4.9: Show decision marker if present
                let decision_marker = if let Some((decision_type, snooze_date)) = &issue.decision_info {
                    if decision_type == "acknowledged" {
                        " [acknowledged]"
                    } else if decision_type == "snoozed" {
                        if let Some(date) = snooze_date {
                            &format!(" [snoozed until {}]", date)
                        } else {
                            " [snoozed]"
                        }
                    } else {
                        ""
                    }
                } else {
                    ""
                };
                println!("  • {}{}", issue.title, decision_marker);
                println!("    {}", issue.explanation);
                println!("    💡 {}", issue.recommended_action);
                if let Some(impact) = &issue.estimated_impact {
                    println!("    📊 {}", impact);
                }
                if let Some(reference) = &issue.reference {
                    println!("    📚 {}", reference);
                }
                println!();
            }
        }

        // Then warnings
        if !warnings.is_empty() {
            println!("⚠️  Warnings:\n");
            for issue in warnings {
                // Phase 4.9: Show decision marker if present
                let decision_marker = if let Some((decision_type, snooze_date)) = &issue.decision_info {
                    if decision_type == "acknowledged" {
                        " [acknowledged]"
                    } else if decision_type == "snoozed" {
                        if let Some(date) = snooze_date {
                            &format!(" [snoozed until {}]", date)
                        } else {
                            " [snoozed]"
                        }
                    } else {
                        ""
                    }
                } else {
                    ""
                };
                println!("  • {}{}", issue.title, decision_marker);
                println!("    {}", issue.explanation);
                println!("    💡 {}", issue.recommended_action);
                if let Some(reference) = &issue.reference {
                    println!("    📚 {}", reference);
                }
                println!();
            }
        }

        // Then info
        if !info.is_empty() {
            println!("ℹ️  Recommendations:\n");
            for issue in info {
                // Phase 4.9: Show decision marker if present
                let decision_marker = if let Some((decision_type, snooze_date)) = &issue.decision_info {
                    if decision_type == "acknowledged" {
                        " [acknowledged]"
                    } else if decision_type == "snoozed" {
                        if let Some(date) = snooze_date {
                            &format!(" [snoozed until {}]", date)
                        } else {
                            " [snoozed]"
                        }
                    } else {
                        ""
                    }
                } else {
                    ""
                };
                println!("  • {}{}", issue.title, decision_marker);
                println!("    {}", issue.explanation);
                println!("    💡 {}", issue.recommended_action);
                if let Some(reference) = &issue.reference {
                    println!("    📚 {}", reference);
                }
                println!();
            }
        }

        println!("💡 Next Steps:");
        let has_critical = caretaker_analysis.issues.iter()
            .any(|i| i.severity == IssueSeverity::Critical);
        if has_critical {
            println!("   🚨 Run 'sudo annactl repair' to fix critical issues");
        } else {
            println!("   Run 'sudo annactl repair' to attempt automatic fixes");
        }
        println!("   Or review each issue and fix manually");
    } else {
        println!("\n✅ All systems healthy\n");
        println!("💡 Your system is running smoothly. No action needed.");
    }

    // Phase 5.3: Show insights hint if patterns exist (once per day)
    if should_show_insights_hint().await {
        println!("💡 Insight: Recurring patterns detected. For details run 'annactl insights'.");
    }

    println!();

    // Log command
    let duration_ms = start_time.elapsed().as_millis() as u64;
    let exit_code = if caretaker_analysis.overall_status == "critical" {
        1
    } else {
        EXIT_SUCCESS
    };

    let log_entry = LogEntry {
        ts: LogEntry::now(),
        req_id: req_id.to_string(),
        state: state.to_string(),
        command: "status".to_string(),
        allowed: Some(true),
        args: vec![],
        exit_code,
        citation: "[archwiki:System_maintenance]".to_string(),
        duration_ms,
        ok: exit_code == EXIT_SUCCESS,
        error: None,
    };
    let _ = log_entry.write();

    std::process::exit(exit_code);
}

fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    const TB: u64 = GB * 1024;

    if bytes >= TB {
        format!("{:.1}TB", bytes as f64 / TB as f64)
    } else if bytes >= GB {
        format!("{:.1}GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1}MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1}KB", bytes as f64 / KB as f64)
    } else {
        format!("{}B", bytes)
    }
}

// Phase 5.2: Helper to convert severity to integer for observation recording
fn severity_to_int(severity: &IssueSeverity) -> i32 {
    match severity {
        IssueSeverity::Info => 0,
        IssueSeverity::Warning => 1,
        IssueSeverity::Critical => 2,
    }
}

// Phase 5.3: Check if insights hint should be shown (once per day)
async fn should_show_insights_hint() -> bool {
    // Try to generate insights for last 30 days
    let insights_result = anna_common::insights::generate_insights(30).await;

    if let Ok(report) = insights_result {
        // Only show if there are actual patterns
        if !report.patterns.is_empty() {
            // Check if we've shown the hint today already
            // Use a simple file-based flag in context directory
            if let Ok(context_dir) = std::env::var("XDG_DATA_HOME")
                .map(|d| std::path::PathBuf::from(d).join("anna"))
                .or_else(|_| {
                    std::env::var("HOME").map(|h| std::path::PathBuf::from(h).join(".local/share/anna"))
                }) {
                let hint_file = context_dir.join(".insights_hint_shown");

                // Check if file exists and was modified today
                if let Ok(metadata) = std::fs::metadata(&hint_file) {
                    if let Ok(modified) = metadata.modified() {
                        let now = std::time::SystemTime::now();
                        if let Ok(duration) = now.duration_since(modified) {
                            // If modified within last 24 hours, don't show again
                            if duration.as_secs() < 86400 {
                                return false;
                            }
                        }
                    }
                }

                // Create or update the hint file
                let _ = std::fs::create_dir_all(&context_dir);
                let _ = std::fs::write(&hint_file, "");

                return true;
            }
        }
    }

    false
}
