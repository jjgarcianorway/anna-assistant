//! Plan command - Arch Wiki-based execution planner (6.3.0)
//!
//! Fetches telemetry and generates actionable plans using the orchestrator.
//! Read-only - displays plans but does not execute them.
//!
//! Commands:
//! - `annactl plan` → Human-readable plan output
//! - `annactl plan --json` → Machine-readable JSON output

use anna_common::terminal_format as fmt;
use anna_common::orchestrator::{
    get_arch_help_dns, get_arch_help_service_failure,
    plan_dns_fix, plan_service_failure_fix,
    Plan, TelemetrySummary, ServiceStatus,
};
use anyhow::Result;
use std::time::Instant;

use crate::logging::LogEntry;
use crate::system_query;

/// Execute 'annactl plan' command - generate execution plan
pub async fn execute_plan_command(
    json: bool,
    _req_id: &str,
    _start_time: Instant,
) -> Result<()> {
    // Fetch system telemetry
    let system_telemetry = system_query::query_system_telemetry()?;

    // Convert to TelemetrySummary for planner
    let telemetry = convert_to_planner_summary(&system_telemetry);

    // Run planner slices
    let mut all_plans = Vec::new();

    // DNS slice
    if telemetry.dns_suspected_broken && telemetry.network_reachable {
        let wiki = get_arch_help_dns();
        let plan = plan_dns_fix("DNS resolution issue detected", &telemetry, &wiki);

        if !plan.steps.is_empty() {
            all_plans.push((
                "DNS Resolution Fix".to_string(),
                plan,
                wiki.sources,
            ));
        }
    }

    // Service failure slice
    for service in &telemetry.failed_services {
        if service.is_failed {
            let service_name = service.name.trim_end_matches(".service");
            let wiki = get_arch_help_service_failure(service_name);
            let plan = plan_service_failure_fix(
                &format!("Service {} is failed", service_name),
                &telemetry,
                &wiki,
            );

            if !plan.steps.is_empty() {
                all_plans.push((
                    format!("Service Failure Fix: {}", service_name),
                    plan,
                    wiki.sources,
                ));
            }
        }
    }

    // Output
    if json {
        output_json(&all_plans)?;
    } else {
        output_human_readable(&all_plans)?;
    }

    Ok(())
}

/// Convert SystemTelemetry to TelemetrySummary for planner
fn convert_to_planner_summary(
    system_telemetry: &anna_common::telemetry::SystemTelemetry,
) -> TelemetrySummary {
    // Network is reachable if we have an active connection
    let network_reachable = system_telemetry.network.is_connected;

    // For now, simple heuristic: assume DNS is healthy unless network is down
    // TODO: Add DNS-specific checks to NetworkInfo in future
    let dns_suspected_broken = false;

    // Collect failed services from failed_units
    let failed_services: Vec<ServiceStatus> = system_telemetry
        .services
        .failed_units
        .iter()
        .filter(|unit| unit.unit_type == "service")
        .map(|unit| ServiceStatus {
            name: unit.name.clone(),
            is_failed: true,
        })
        .collect();

    TelemetrySummary {
        dns_suspected_broken,
        network_reachable,
        failed_services,
    }
}

/// Output plans in JSON format
fn output_json(
    plans: &[(String, Plan, Vec<anna_common::orchestrator::KnowledgeSourceRef>)],
) -> Result<()> {
    use serde_json::json;

    let plans_json: Vec<_> = plans
        .iter()
        .map(|(desc, plan, _sources)| {
            let fix = plan.to_suggested_fix(desc.clone());
            json!({
                "description": fix.description,
                "steps": fix.steps.iter().map(|s| {
                    json!({
                        "kind": s.kind,
                        "command": s.command,
                        "requires_confirmation": s.requires_confirmation,
                        "rollback_command": s.rollback_command,
                    })
                }).collect::<Vec<_>>(),
                "knowledge_sources": fix.knowledge_sources.iter().map(|ks| {
                    json!({
                        "url": ks.url,
                        "kind": ks.kind,
                    })
                }).collect::<Vec<_>>(),
            })
        })
        .collect();

    let output = json!({
        "plans": plans_json,
    });

    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}

/// Output plans in human-readable format
fn output_human_readable(
    plans: &[(String, Plan, Vec<anna_common::orchestrator::KnowledgeSourceRef>)],
) -> Result<()> {
    if plans.is_empty() {
        println!("{}", fmt::bold("No Issues Detected"));
        println!();
        println!("Your system appears healthy. No execution plans needed.");
        return Ok(());
    }

    println!("{}", fmt::bold("Anna Execution Plans"));
    println!("{}", "=".repeat(50));
    println!();
    println!(
        "{}",
        fmt::dimmed("Based on Arch Wiki guidance. Read-only - no changes will be made.")
    );
    println!();

    for (i, (description, plan, sources)) in plans.iter().enumerate() {
        println!(
            "{} {}",
            fmt::bold(&format!("Plan {}:", i + 1)),
            fmt::bold(description)
        );
        println!();

        // Knowledge sources
        if !sources.is_empty() {
            println!("{}", fmt::dimmed("References:"));
            for source in sources {
                println!("  {}", fmt::dimmed(&format!("• {}", source.url)));
            }
            println!();
        }

        // Steps
        println!("{}", fmt::bold("Steps:"));
        for (step_idx, step) in plan.steps.iter().enumerate() {
            let step_num = step_idx + 1;
            let kind_label = match step.kind {
                anna_common::orchestrator::PlanStepKind::Inspect => {
                    fmt::dimmed("[INSPECT]")
                }
                anna_common::orchestrator::PlanStepKind::Change => {
                    fmt::bold("[CHANGE]")
                }
            };

            println!("  {}. {} {}", step_num, kind_label, step.command);

            if step.requires_confirmation {
                println!(
                    "     {}",
                    fmt::dimmed("⚠ Requires confirmation before execution")
                );
            }

            if let Some(rollback) = &step.rollback_command {
                println!("     {}", fmt::dimmed(&format!("Rollback: {}", rollback)));
            }
        }

        println!();
    }

    Ok(())
}
