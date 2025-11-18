//! Empathy Kernel CLI commands
//!
//! Phase 1.2: CLI interface for empathy operations

use crate::rpc_client::RpcClient;
use anna_common::ipc::{Method, ResponseData};
use anyhow::{anyhow, Result};

/// Execute `annactl empathy pulse` command
pub async fn execute_empathy_pulse_command() -> Result<()> {
    let mut client = RpcClient::connect().await?;

    match client.call(Method::EmpathyPulse).await? {
        ResponseData::EmpathyPulse(data) => {
            println!("\n=== Anna Empathy Kernel - Current Pulse ===");
            println!("\nTimestamp: {}", data.timestamp);

            println!("\nIndices:");
            println!("  Empathy Index:  {:.1}%", data.empathy_index * 100.0);
            println!("  Strain Index:   {:.1}%", data.strain_index * 100.0);

            println!("\nResonance Map:");
            println!(
                "  User:         {:.1}%",
                data.resonance_map.user_resonance * 100.0
            );
            println!(
                "  System:       {:.1}%",
                data.resonance_map.system_resonance * 100.0
            );
            println!(
                "  Environment:  {:.1}%",
                data.resonance_map.environment_resonance * 100.0
            );

            if !data.resonance_map.recent_adjustments.is_empty() {
                println!(
                    "\n  Recent Adjustments: {} in tracking",
                    data.resonance_map.recent_adjustments.len()
                );
            }

            println!("\nContext Summary:");
            println!("  {}", data.context_summary);

            if !data.recent_perceptions.is_empty() {
                println!(
                    "\nRecent Perceptions (last {}):",
                    data.recent_perceptions.len()
                );
                for (i, perception) in data.recent_perceptions.iter().enumerate() {
                    println!(
                        "\n  {}. {} - {}",
                        i + 1,
                        perception.timestamp,
                        perception.action
                    );

                    println!("     Impacts:");
                    println!(
                        "       User:   {:.0}% - {}",
                        perception.stakeholder_impacts.user.score * 100.0,
                        perception.stakeholder_impacts.user.impact_type
                    );
                    println!(
                        "       System: {:.0}% - {}",
                        perception.stakeholder_impacts.system.score * 100.0,
                        perception.stakeholder_impacts.system.impact_type
                    );
                    println!(
                        "       Env:    {:.0}% - {}",
                        perception.stakeholder_impacts.environment.score * 100.0,
                        perception.stakeholder_impacts.environment.impact_type
                    );

                    if let Some(ref adaptation) = perception.adaptation {
                        println!("     Adaptation: {}", adaptation);
                    }
                }
            }

            println!();
            Ok(())
        }
        _ => Err(anyhow!("Expected EmpathyPulse response")),
    }
}

/// Execute `annactl empathy simulate <action>` command
pub async fn execute_empathy_simulate_command(action_str: &str) -> Result<()> {
    let mut client = RpcClient::connect().await?;

    match client
        .call(Method::EmpathySimulate {
            action: action_str.to_string(),
        })
        .await?
    {
        ResponseData::EmpathySimulation(data) => {
            println!("\n=== Empathy Simulation ===");
            println!("\nAction: {}", data.action);

            println!("\nAnalysis:");
            println!("{}", data.reasoning);

            println!("\nEvaluation:");
            if data.would_proceed {
                println!("  Decision: PROCEED (action would proceed)");
            } else {
                println!("  Decision: DEFER (action would be deferred)");
            }

            if let Some(ref reason) = data.evaluation.deferral_reason {
                println!("  Reason: {}", reason);
            }

            if data.evaluation.recommended_delay > 0 {
                println!(
                    "  Recommended Delay: {} seconds",
                    data.evaluation.recommended_delay
                );
            }

            if let Some(ref tone) = data.evaluation.tone_adaptation {
                println!("\nTone Adaptation:");
                println!("  {}", tone);
            }

            println!();
            Ok(())
        }
        _ => Err(anyhow!("Expected EmpathySimulation response")),
    }
}
