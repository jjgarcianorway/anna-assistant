//! Mirror Protocol CLI commands
//!
//! Phase 1.4: CLI interface for recursive introspection

use crate::rpc_client::RpcClient;
use anna_common::ipc::{Method, ResponseData};
use anyhow::{anyhow, Result};

/// Execute `annactl mirror reflect` command
pub async fn execute_mirror_reflect_command() -> Result<()> {
    let mut client = RpcClient::connect().await?;

    match client.call(Method::MirrorReflect).await? {
        ResponseData::MirrorReflection(data) => {
            println!("\n=== Anna Mirror Protocol - Reflection Generated ===");
            println!("\nReflection ID: {}", data.reflection_id);
            println!("Timestamp: {}", data.timestamp);
            println!("Period: {} to {}", data.period_start, data.period_end);

            println!("\nSelf-Assessment:");
            println!("  Coherence Score:     {:.1}%", data.self_coherence * 100.0);
            println!("  Ethical Decisions:   {}", data.ethical_decisions_count);
            println!("  Conscience Actions:  {}", data.conscience_actions_count);

            println!("\nEmpathy Summary:");
            println!(
                "  Avg Empathy Index:   {:.1}%",
                data.avg_empathy_index * 100.0
            );
            println!(
                "  Avg Strain Index:    {:.1}%",
                data.avg_strain_index * 100.0
            );
            println!("  Empathy Trend:       {}", data.empathy_trend);
            println!("  Adaptations Made:    {}", data.adaptations_count);

            if !data.self_identified_biases.is_empty() {
                println!("\n⚠️  Self-Identified Biases:");
                for (i, bias) in data.self_identified_biases.iter().enumerate() {
                    println!("  {}. {}", i + 1, bias);
                }
            } else {
                println!("\n✓ No biases self-identified");
            }

            println!("\nCitation: [archwiki:System_maintenance]");
            println!();
            Ok(())
        }
        _ => Err(anyhow!("Expected MirrorReflection response")),
    }
}

/// Execute `annactl mirror audit` command
pub async fn execute_mirror_audit_command() -> Result<()> {
    let mut client = RpcClient::connect().await?;

    match client.call(Method::MirrorAudit).await? {
        ResponseData::MirrorAudit(data) => {
            println!("\n=== Anna Mirror Protocol - Audit Summary ===");

            if !data.enabled {
                println!("\n⚠️  Mirror Protocol is disabled");
                println!("\nTo enable metacognition, configure /etc/anna/mirror.yml");
                println!("Citation: [archwiki:System_maintenance]");
                return Ok(());
            }

            println!("\nCurrent Status:");
            println!(
                "  Network Coherence:   {:.1}%",
                data.current_coherence * 100.0
            );
            println!(
                "  Last Reflection:     {}",
                data.last_reflection
                    .as_ref()
                    .unwrap_or(&"Never".to_string())
            );
            println!(
                "  Last Consensus:      {}",
                data.last_consensus.as_ref().unwrap_or(&"Never".to_string())
            );

            println!("\nRecent Activity:");
            println!("  Reflections:         {}", data.recent_reflections_count);
            println!("  Peer Critiques:      {}", data.received_critiques_count);
            println!("  Active Remediations: {}", data.active_remediations_count);

            if let Some(network_coherence) = data.network_coherence {
                println!("\nNetwork Metrics:");
                println!("  Network Coherence:   {:.1}%", network_coherence * 100.0);
            }

            if !data.recent_critiques.is_empty() {
                println!("\nRecent Peer Critiques:");
                for (i, critique) in data.recent_critiques.iter().enumerate().take(5) {
                    println!("\n  {}. From: {}", i + 1, critique.critic_id);
                    println!(
                        "     Coherence Assessment: {:.1}%",
                        critique.coherence_assessment * 100.0
                    );
                    println!("     Inconsistencies: {}", critique.inconsistencies_count);
                    println!("     Biases Detected: {}", critique.biases_count);
                    if !critique.recommendations.is_empty() {
                        println!("     Recommendation: {}", critique.recommendations[0]);
                    }
                }
            }

            println!("\nCitation: [archwiki:System_maintenance]");
            println!();
            Ok(())
        }
        _ => Err(anyhow!("Expected MirrorAudit response")),
    }
}

/// Execute `annactl mirror repair` command
pub async fn execute_mirror_repair_command() -> Result<()> {
    let mut client = RpcClient::connect().await?;

    match client.call(Method::MirrorRepair).await? {
        ResponseData::MirrorRepair(data) => {
            println!("\n=== Anna Mirror Protocol - Remediation Report ===");
            println!("\nTimestamp: {}", data.timestamp);
            println!("Total Remediations: {}", data.total_remediations);
            println!("Successful: {}", data.successful_remediations);
            println!("Failed/Skipped: {}", data.failed_remediations);

            println!("\nSummary: {}", data.summary);

            if !data.applied_remediations.is_empty() {
                println!("\nApplied Remediations:");
                for (i, remediation) in data.applied_remediations.iter().enumerate() {
                    println!("\n  {}. {}", i + 1, remediation.description);
                    println!("     Type: {}", remediation.remediation_type);
                    println!("     Expected Impact: {}", remediation.expected_impact);

                    if !remediation.parameter_adjustments.is_empty() {
                        println!("     Adjustments:");
                        for (param, value) in &remediation.parameter_adjustments {
                            println!("       {} → {:.2}", param, value);
                        }
                    }
                }
            }

            if data.successful_remediations > 0 {
                println!("\n✓ Bias corrections applied successfully");
            }

            println!("\nCitation: [archwiki:System_maintenance]");
            println!();
            Ok(())
        }
        _ => Err(anyhow!("Expected MirrorRepair response")),
    }
}

/// Execute `annactl mirror audit-forecast` command (Phase 1.6)
pub async fn execute_mirror_audit_forecast_command(window_hours: u64, json: bool) -> Result<()> {
    let mut client = RpcClient::connect().await?;

    match client
        .call(Method::MirrorAuditForecast {
            window_hours: Some(window_hours),
        })
        .await?
    {
        ResponseData::MirrorAuditForecast(data) => {
            if json {
                println!("{}", serde_json::to_string_pretty(&data)?);
                return Ok(());
            }

            println!("\n=== Anna Mirror Audit - Temporal Forecast Verification ===");
            println!("\nTotal Audits: {}", data.total_audits);

            if let Some(last_audit) = data.last_audit_at {
                println!("Last Audit: {}", last_audit);
            }

            if let Some(avg_integrity) = data.average_temporal_integrity {
                println!("Average Temporal Integrity: {:.1}%", avg_integrity * 100.0);
            }

            if !data.active_biases.is_empty() {
                println!("\n⚠️  Active Biases Detected:");
                for (i, bias) in data.active_biases.iter().enumerate() {
                    println!(
                        "\n  {}. {} (confidence: {:.1}%)",
                        i + 1,
                        bias.kind,
                        bias.confidence * 100.0
                    );
                    println!("     Evidence: {}", bias.evidence);
                    println!(
                        "     Magnitude: {:.1}% (sample size: {})",
                        bias.magnitude * 100.0,
                        bias.sample_size
                    );
                }
            } else {
                println!("\n✓ No systematic biases detected");
            }

            if !data.pending_adjustments.is_empty() {
                println!("\n📋 Pending Adjustment Plans:");
                for (i, plan) in data.pending_adjustments.iter().enumerate() {
                    println!(
                        "\n  {}. Plan ID: {} (created: {})",
                        i + 1,
                        plan.plan_id,
                        plan.created_at
                    );
                    println!("     Target: {}", plan.target);
                    println!(
                        "     Expected Improvement: +{:.1}%",
                        plan.expected_improvement * 100.0
                    );
                    println!("     Rationale: {}", plan.rationale);

                    if !plan.adjustments.is_empty() {
                        println!("     Recommended Adjustments:");
                        for adj in &plan.adjustments {
                            print!("       • {}: ", adj.parameter);
                            if let Some(current) = adj.current_value {
                                print!("{:.2} → ", current);
                            }
                            println!("{:.2}", adj.recommended_value);
                            println!("         Reason: {}", adj.reason);
                        }
                    }
                }
                println!("\n⚠️  Advisory Only: Adjustments require manual approval");
            } else {
                println!("\n✓ No adjustments recommended");
            }

            println!("\nCitation: [archwiki:System_maintenance]");
            println!();
            Ok(())
        }
        _ => Err(anyhow!("Expected MirrorAuditForecast response")),
    }
}

/// Execute `annactl mirror reflect-temporal` command (Phase 1.6)
pub async fn execute_mirror_reflect_temporal_command(window_hours: u64, json: bool) -> Result<()> {
    let mut client = RpcClient::connect().await?;

    match client
        .call(Method::MirrorReflectTemporal {
            window_hours: Some(window_hours),
        })
        .await?
    {
        ResponseData::MirrorReflectTemporal(data) => {
            if json {
                println!("{}", serde_json::to_string_pretty(&data)?);
                return Ok(());
            }

            println!("\n=== Anna Mirror Protocol - Temporal Self-Reflection ===");
            println!("\nReflection ID: {}", data.reflection_id);
            println!("Generated: {}", data.generated_at);
            println!("Window: {} hours", data.window_hours);

            println!(
                "\nTemporal Integrity Score: {:.1}%",
                data.temporal_integrity_score * 100.0
            );

            println!("\nSummary:");
            println!("  {}", data.summary);

            if !data.biases_detected.is_empty() {
                println!("\n⚠️  Biases Detected:");
                for (i, bias) in data.biases_detected.iter().enumerate() {
                    println!(
                        "\n  {}. {} (confidence: {:.1}%)",
                        i + 1,
                        bias.kind,
                        bias.confidence * 100.0
                    );
                    println!("     {}", bias.evidence);
                }
            } else {
                println!("\n✓ No biases detected");
            }

            if let Some(plan) = &data.recommended_adjustments {
                println!("\n📋 Recommended Adjustments:");
                println!("  Target: {}", plan.target);
                println!(
                    "  Expected Improvement: +{:.1}%",
                    plan.expected_improvement * 100.0
                );
                println!("  Rationale: {}", plan.rationale);

                if !plan.adjustments.is_empty() {
                    println!("\n  Parameters:");
                    for adj in &plan.adjustments {
                        print!("    • {}: ", adj.parameter);
                        if let Some(current) = adj.current_value {
                            print!("{:.2} → ", current);
                        }
                        println!("{:.2}", adj.recommended_value);
                        println!("      {}", adj.reason);
                    }
                }
                println!("\n  ⚠️  Advisory Only: Manual approval required");
            } else {
                println!("\n✓ No adjustments recommended");
            }

            println!("\nCitation: [archwiki:System_maintenance]");
            println!();
            Ok(())
        }
        _ => Err(anyhow!("Expected MirrorReflectTemporal response")),
    }
}
