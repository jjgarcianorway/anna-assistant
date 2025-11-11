//! Conscience CLI commands
//!
//! Phase 1.1: Ethical governance and self-reflection
//! Citation: [archwiki:System_maintenance]

use anna_common::ipc::ResponseData;
use anyhow::{anyhow, Result};
use std::time::Instant;

use crate::errors::*;
use crate::rpc_client::RpcClient;

/// Execute conscience review command
pub async fn execute_conscience_review_command(
    req_id: &str,
    state: &str,
    start_time: Instant,
) -> Result<()> {
    let mut client = RpcClient::connect().await?;

    match client.conscience_review().await {
        Ok(ResponseData::ConsciencePending(data)) => {
            if data.pending_actions.is_empty() {
                println!("╔═══════════════════════════════════════════════════════════╗");
                println!("║ NO PENDING ACTIONS REQUIRING REVIEW                       ║");
                println!("╚═══════════════════════════════════════════════════════════╝");
                println!();
                println!("Conscience layer is operating normally.");
                println!("All actions passed ethical evaluation.");
            } else {
                println!("╔═══════════════════════════════════════════════════════════╗");
                println!("║ PENDING ACTIONS REQUIRING REVIEW                          ║");
                println!("╚═══════════════════════════════════════════════════════════╝");
                println!();

                for (i, action) in data.pending_actions.iter().enumerate() {
                    println!("{}. ID: {}", i + 1, action.id);
                    println!("   Time:        {}", action.timestamp);
                    println!("   Action:      {}", action.action);
                    println!("   Uncertainty: {:.1}%", action.uncertainty * 100.0);
                    println!("   Ethical:     {:.1}%", action.ethical_score * 100.0);
                    println!("   Reason:      {}", action.flag_reason);
                    println!("   Weakest Dim: {} ({:.1}%)", action.weakest_dimension, action.ethical_score * 100.0);
                    println!();
                }

                println!("Use 'annactl conscience explain <id>' for detailed reasoning.");
                println!("Use 'annactl conscience approve <id>' to approve an action.");
                println!("Use 'annactl conscience reject <id>' to reject an action.");
            }

            Ok(())
        }
        Ok(_) => Err(anyhow!("Expected ConsciencePending response")),
        Err(e) => Err(e),
    }
}

/// Execute conscience explain command
pub async fn execute_conscience_explain_command(
    decision_id: &str,
    req_id: &str,
    state: &str,
    start_time: Instant,
) -> Result<()> {
    let mut client = RpcClient::connect().await?;

    match client.conscience_explain(decision_id.to_string()).await {
        Ok(ResponseData::ConscienceDecision(data)) => {
            println!("╔═══════════════════════════════════════════════════════════╗");
            println!("║ CONSCIENCE DECISION REPORT                                ║");
            println!("╚═══════════════════════════════════════════════════════════╝");
            println!();

            println!("Decision ID: {}", data.id);
            println!("Timestamp:   {}", data.timestamp);
            println!("Action:      {}", data.action);
            println!();

            println!("OUTCOME:");
            println!("  {}", data.outcome);
            println!();

            println!("ETHICAL EVALUATION:");
            println!("  Overall Score: {:.1}%", data.ethical_score * 100.0);
            println!("  Safety:        {:.1}%", data.safety * 100.0);
            println!("  Privacy:       {:.1}%", data.privacy * 100.0);
            println!("  Integrity:     {:.1}%", data.integrity * 100.0);
            println!("  Autonomy:      {:.1}%", data.autonomy * 100.0);
            println!();

            println!("CONFIDENCE:");
            println!("  Decision Confidence: {:.1}%", data.confidence * 100.0);
            println!("  Uncertainty:         {:.1}%", (1.0 - data.confidence) * 100.0);
            println!();

            println!("REASONING:");
            println!("{}", data.reasoning);

            if data.has_rollback_plan {
                println!("ROLLBACK PLAN: Available");
            }

            println!("───────────────────────────────────────────────────────────");

            Ok(())
        }
        Ok(_) => Err(anyhow!("Expected ConscienceDecision response")),
        Err(e) => Err(e),
    }
}

/// Execute conscience approve command
pub async fn execute_conscience_approve_command(
    decision_id: &str,
    req_id: &str,
    state: &str,
    start_time: Instant,
) -> Result<()> {
    let mut client = RpcClient::connect().await?;

    match client.conscience_approve(decision_id.to_string()).await {
        Ok(ResponseData::ConscienceActionResult(msg)) => {
            println!("✓ Action approved: {}", msg);
            println!();
            println!("The flagged action has been manually approved and will be executed.");
            Ok(())
        }
        Ok(_) => Err(anyhow!("Expected ConscienceActionResult response")),
        Err(e) => Err(e),
    }
}

/// Execute conscience reject command
pub async fn execute_conscience_reject_command(
    decision_id: &str,
    req_id: &str,
    state: &str,
    start_time: Instant,
) -> Result<()> {
    let mut client = RpcClient::connect().await?;

    match client.conscience_reject(decision_id.to_string()).await {
        Ok(ResponseData::ConscienceActionResult(msg)) => {
            println!("✗ Action rejected: {}", msg);
            println!();
            println!("The flagged action has been manually rejected and will not be executed.");
            Ok(())
        }
        Ok(_) => Err(anyhow!("Expected ConscienceActionResult response")),
        Err(e) => Err(e),
    }
}

/// Execute conscience introspect command
pub async fn execute_conscience_introspect_command(
    req_id: &str,
    state: &str,
    start_time: Instant,
) -> Result<()> {
    let mut client = RpcClient::connect().await?;

    println!("Running conscience introspection...");
    println!();

    match client.conscience_introspect().await {
        Ok(ResponseData::ConscienceIntrospection(data)) => {
            println!("╔═══════════════════════════════════════════════════════════╗");
            println!("║ CONSCIENCE INTROSPECTION REPORT                           ║");
            println!("╚═══════════════════════════════════════════════════════════╝");
            println!();

            println!("Report Generated: {}", data.timestamp);
            println!("Analysis Period:  {}", data.period);
            println!();

            println!("DECISION SUMMARY:");
            println!("  Total Reviewed: {}", data.decisions_reviewed);
            println!("  Approved:       {}", data.approved_count);
            println!("  Rejected:       {}", data.rejected_count);
            println!("  Flagged:        {}", data.flagged_count);
            println!();

            println!("QUALITY METRICS:");
            println!("  Avg Ethical Score: {:.1}%", data.avg_ethical_score * 100.0);
            println!("  Avg Confidence:    {:.1}%", data.avg_confidence * 100.0);
            println!();

            if data.violations_count > 0 {
                println!("VIOLATIONS DETECTED: {}", data.violations_count);
                println!("  Review logs for details");
                println!();
            }

            println!("RECOMMENDATIONS:");
            for (i, rec) in data.recommendations.iter().enumerate() {
                println!("  {}. {}", i + 1, rec);
            }

            println!();
            println!("───────────────────────────────────────────────────────────");

            Ok(())
        }
        Ok(_) => Err(anyhow!("Expected ConscienceIntrospection response")),
        Err(e) => Err(e),
    }
}
