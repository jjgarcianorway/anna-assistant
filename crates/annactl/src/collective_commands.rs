//! Collective Mind CLI commands
//!
//! Phase 1.3: CLI interface for distributed cooperation

use crate::rpc_client::RpcClient;
use anna_common::ipc::{Method, ResponseData};
use anyhow::{anyhow, Result};

/// Execute `annactl collective status` command
pub async fn execute_collective_status_command() -> Result<()> {
    let mut client = RpcClient::connect().await?;

    match client.call(Method::CollectiveStatus).await? {
        ResponseData::CollectiveStatus(data) => {
            println!("\n=== Anna Collective Mind - Network Status ===");

            if !data.enabled {
                println!("\n⚠️  Collective Mind is disabled");
                println!("\nTo enable distributed cooperation, configure /etc/anna/collective.yml");
                println!("Citation: [archwiki:System_maintenance]");
                return Ok(());
            }

            println!("\nNode ID: {}", data.node_id);
            println!("\nNetwork:");
            println!("  Connected Peers:  {}/{}", data.connected_peers, data.total_peers);
            println!("  Network Health:   {:.1}%", data.network_health * 100.0);

            println!("\nEmpathy Metrics:");
            println!("  Avg Network Empathy: {:.1}%", data.avg_network_empathy * 100.0);
            println!("  Avg Network Strain:  {:.1}%", data.avg_network_strain * 100.0);

            println!("\nConsensus:");
            println!("  Recent Decisions: {}", data.recent_decisions);

            println!("\nCitation: [archwiki:System_maintenance]");
            println!();
            Ok(())
        }
        _ => Err(anyhow!("Expected CollectiveStatus response")),
    }
}

/// Execute `annactl collective trust <peer_id>` command
pub async fn execute_collective_trust_command(peer_id: &str) -> Result<()> {
    let mut client = RpcClient::connect().await?;

    match client
        .call(Method::CollectiveTrust {
            peer_id: peer_id.to_string(),
        })
        .await?
    {
        ResponseData::CollectiveTrust(data) => {
            println!("\n=== Trust Details for Peer ===");
            println!("\nPeer: {} ({})", data.peer_name, data.peer_id);
            println!("Address: {}", data.peer_address);
            println!(
                "Status: {}",
                if data.connected { "Connected" } else { "Disconnected" }
            );

            println!("\nTrust Scores:");
            println!("  Overall:            {:.1}%", data.overall_trust * 100.0);
            println!("  Honesty:            {:.1}%", data.honesty * 100.0);
            println!("  Reliability:        {:.1}%", data.reliability * 100.0);
            println!("  Ethical Alignment:  {:.1}%", data.ethical_alignment * 100.0);

            println!("\nInteraction History:");
            println!("  Messages Received:  {}", data.messages_received);
            println!("  Messages Validated: {}", data.messages_validated);
            if data.messages_received > 0 {
                let validation_rate =
                    (data.messages_validated as f64 / data.messages_received as f64) * 100.0;
                println!("  Validation Rate:    {:.1}%", validation_rate);
            }
            println!("  Last Interaction:   {}", data.last_interaction);

            println!("\nCitation: [archwiki:System_maintenance]");
            println!();
            Ok(())
        }
        _ => Err(anyhow!("Expected CollectiveTrust response")),
    }
}

/// Execute `annactl collective explain <consensus_id>` command
pub async fn execute_collective_explain_command(consensus_id: &str) -> Result<()> {
    let mut client = RpcClient::connect().await?;

    match client
        .call(Method::CollectiveExplain {
            consensus_id: consensus_id.to_string(),
        })
        .await?
    {
        ResponseData::CollectiveExplanation(data) => {
            println!("\n=== Consensus Decision Explanation ===");
            println!("\nID: {}", data.consensus_id);
            println!("Action: {}", data.action);
            println!("Decision: {}", data.decision);
            println!("Timestamp: {}", data.timestamp);

            println!("\nVoting Summary:");
            println!("  Total Participants:     {}", data.total_participants);
            println!("  Approval Rate:          {:.1}%", data.approval_percentage);
            println!("  Weighted Approval:      {:.1}%", data.weighted_approval);

            if !data.votes.is_empty() {
                println!("\nVote Breakdown:");
                for (i, vote) in data.votes.iter().enumerate() {
                    println!("\n  {}. Peer: {}", i + 1, vote.peer_id);
                    println!("     Vote:    {}", vote.vote);
                    println!("     Weight:  {:.2}", vote.weight);
                    println!("     Trust:   {:.1}%", vote.trust_score * 100.0);
                    println!("     Ethical: {:.1}%", vote.ethical_score * 100.0);
                    println!("     Reason:  {}", vote.reasoning);
                }
            }

            println!("\nReasoning:");
            println!("{}", data.reasoning);

            println!("\nCitation: [archwiki:System_maintenance]");
            println!();
            Ok(())
        }
        _ => Err(anyhow!("Expected CollectiveExplanation response")),
    }
}
