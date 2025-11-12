//! Chronos Loop CLI commands
//!
//! Phase 1.5: Temporal reasoning and predictive ethics
//! Citation: [archwiki:System_maintenance]

use crate::rpc_client::RpcClient;
use anna_common::ipc::{Method, ResponseData};
use anyhow::{anyhow, Result};

/// Execute `annactl chronos forecast <window>` command
pub async fn execute_chronos_forecast_command(window_hours: u64) -> Result<()> {
    let mut client = RpcClient::connect().await?;

    match client
        .call(Method::ChronosForecast { window_hours })
        .await?
    {
        ResponseData::ChronosForecast(data) => {
            println!("\n=== Anna Chronos Loop - Forecast Generated ===");
            println!("\nForecast ID: {}", data.forecast_id);
            println!("Generated: {}", data.generated_at);
            println!("Horizon: {} hours", data.horizon_hours);
            println!("Confidence: {:.1}%", data.confidence * 100.0);

            println!("\n=== Projected Final State ({}h ahead) ===", data.horizon_hours);
            println!("Health Score:      {:.1}%", data.final_health * 100.0);
            println!("Empathy Index:     {:.1}%", data.final_empathy * 100.0);
            println!("Strain Index:      {:.1}%", data.final_strain * 100.0);
            println!("Network Coherence: {:.1}%", data.final_coherence * 100.0);

            println!("\n=== Ethics Projection ===");
            println!(
                "Temporal Empathy Index: {:.1}%",
                data.temporal_empathy_index * 100.0
            );
            println!("Moral Cost:             {:.2}", data.moral_cost);
            println!("Ethical Trajectory:     {}", data.ethical_trajectory);

            if !data.stakeholder_impacts.is_empty() {
                println!("\nStakeholder Impacts:");
                for (stakeholder, impact) in &data.stakeholder_impacts {
                    let impact_sign = if *impact >= 0.0 { "+" } else { "" };
                    println!("  {:<12} {}{:.2}", stakeholder, impact_sign, impact);
                }
            }

            if !data.divergence_warnings.is_empty() {
                println!("\n⚠️  Divergence Warnings:");
                for (i, warning) in data.divergence_warnings.iter().enumerate() {
                    println!("  {}. {}", i + 1, warning);
                }
            }

            if !data.recommendations.is_empty() {
                println!("\nRecommendations:");
                for (i, rec) in data.recommendations.iter().enumerate() {
                    println!("  {}. {}", i + 1, rec);
                }
            }

            println!("\nArchive Hash: {}", data.archive_hash);
            println!("\nCitation: [archwiki:System_maintenance]");
            println!();
            Ok(())
        }
        _ => Err(anyhow!("Expected ChronosForecast response")),
    }
}

/// Execute `annactl chronos audit` command
pub async fn execute_chronos_audit_command() -> Result<()> {
    let mut client = RpcClient::connect().await?;

    match client.call(Method::ChronosAudit).await? {
        ResponseData::ChronosAudit(data) => {
            println!("\n=== Anna Chronos Loop - Audit Summary ===");
            println!("\nTotal Archived Forecasts: {}", data.total_archived);

            if !data.recent_forecasts.is_empty() {
                println!("\nRecent Forecasts:");
                for (i, forecast) in data.recent_forecasts.iter().enumerate() {
                    println!("\n  {}. Forecast ID: {}", i + 1, forecast.forecast_id);
                    println!("     Generated:    {}", forecast.generated_at);
                    println!("     Horizon:      {} hours", forecast.horizon_hours);
                    println!("     Confidence:   {:.1}%", forecast.confidence * 100.0);
                    println!("     Warnings:     {}", forecast.warnings_count);
                    println!("     Moral Cost:   {:.2}", forecast.moral_cost);
                }
            } else {
                println!("\nNo forecasts archived yet");
            }

            println!("\nCitation: [archwiki:System_maintenance]");
            println!();
            Ok(())
        }
        _ => Err(anyhow!("Expected ChronosAudit response")),
    }
}

/// Execute `annactl chronos align` command
pub async fn execute_chronos_align_command() -> Result<()> {
    let mut client = RpcClient::connect().await?;

    match client.call(Method::ChronosAlign).await? {
        ResponseData::ChronosAlign(data) => {
            println!("\n=== Anna Chronos Loop - Parameter Alignment ===");
            println!("\nAlignment Status: {}", data.status);
            println!("Aligned Parameters: {}", data.parameters_aligned);

            if !data.parameter_changes.is_empty() {
                println!("\nParameter Changes:");
                for (param, value) in &data.parameter_changes {
                    println!("  {} → {}", param, value);
                }
            } else {
                println!("\nNo parameter changes required");
            }

            println!("\nCitation: [archwiki:System_maintenance]");
            println!();
            Ok(())
        }
        _ => Err(anyhow!("Expected ChronosAlign response")),
    }
}
