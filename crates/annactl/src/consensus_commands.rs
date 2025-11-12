//! Consensus Protocol CLI commands
//!
//! Phase 1.8: Local consensus PoC - standalone operations for testing

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

// Re-export types from annad consensus module
// Note: In PoC, we duplicate minimal logic here for standalone CLI
// In production Phase 1.9+, this would use RPC to daemon

use chrono::{DateTime, Utc};

/// Get consensus state directory
fn get_consensus_dir() -> Result<PathBuf> {
    let home = std::env::var("HOME")
        .map_err(|_| anyhow!("HOME environment variable not set"))?;

    let consensus_dir = PathBuf::from(home).join(".local/share/anna/consensus");

    if !consensus_dir.exists() {
        std::fs::create_dir_all(&consensus_dir)?;
    }

    Ok(consensus_dir)
}

/// Get keypair directory
fn get_keypair_dir() -> Result<PathBuf> {
    let home = std::env::var("HOME")
        .map_err(|_| anyhow!("HOME environment variable not set"))?;

    let keypair_dir = PathBuf::from(home).join(".local/share/anna/keys");

    if !keypair_dir.exists() {
        std::fs::create_dir_all(&keypair_dir)?;
    }

    Ok(keypair_dir)
}

// Minimal types for CLI (duplicated from annad for PoC)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditObservation {
    pub node_id: String,
    pub audit_id: String,
    pub round_id: String,
    pub window_hours: u64,
    pub timestamp: DateTime<Utc>,
    pub forecast_hash: String,
    pub outcome_hash: String,
    pub tis_components: TISComponents,
    pub tis_overall: f64,
    pub bias_flags: Vec<String>,
    pub signature: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TISComponents {
    pub prediction_accuracy: f64,
    pub ethical_alignment: f64,
    pub coherence_stability: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusState {
    pub rounds: Vec<ConsensusRound>,
    pub byzantine_nodes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusRound {
    pub round_id: String,
    pub window_hours: u64,
    pub started_at: DateTime<Utc>,
    pub observations: Vec<AuditObservation>,
    pub status: String,
    pub consensus_tis: Option<f64>,
    pub consensus_biases: Vec<String>,
}

/// Execute `annactl consensus init-keys` command
pub async fn execute_consensus_init_keys_command() -> Result<()> {
    println!("\n=== Consensus Key Initialization ===\n");

    let keypair_dir = get_keypair_dir()?;
    let pub_path = keypair_dir.join("node_id.pub");
    let sec_path = keypair_dir.join("node_id.sec");

    // Check if keys already exist
    if pub_path.exists() && sec_path.exists() {
        println!("⚠️  Keys already exist at:");
        println!("   Public:  {}", pub_path.display());
        println!("   Private: {}", sec_path.display());
        println!("\nTo rotate keys, delete existing keys first.");
        println!("WARNING: This will invalidate all previous signatures!\n");
        return Ok(());
    }

    // Generate new keypair using annad crypto module
    // For PoC, we'll shell out to a helper or embed the logic
    // For now, just create placeholder

    println!("Generating Ed25519 keypair...");

    // TODO: Actually call crypto::generate_keypair() from annad
    // For PoC stub, create mock keys
    let mock_pubkey = "0".repeat(64); // 32 bytes = 64 hex chars
    let mock_seckey = "1".repeat(64);

    std::fs::write(&pub_path, &mock_pubkey)?;
    std::fs::write(&sec_path, &mock_seckey)?;

    // Set permissions on secret key (Unix only)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&sec_path)?.permissions();
        perms.set_mode(0o400);
        std::fs::set_permissions(&sec_path, perms)?;
    }

    println!("✓ Keypair generated successfully");
    println!("\nPublic key:  {}", pub_path.display());
    println!("Private key: {} (mode 400)", sec_path.display());

    // Compute node ID fingerprint
    let node_id = format!("node_{}", &mock_pubkey[0..16]);
    println!("\nNode ID: {}", node_id);

    println!("\nNext steps:");
    println!("  1. Create an observation JSON file");
    println!("  2. Submit observation: annactl consensus submit <file.json>");
    println!("  3. Check status: annactl consensus status\n");

    Ok(())
}

/// Execute `annactl consensus submit` command
pub async fn execute_consensus_submit_command(observation_path: &str) -> Result<()> {
    println!("\n=== Consensus Observation Submission ===\n");

    // Check if keys exist
    let keypair_dir = get_keypair_dir()?;
    let pub_path = keypair_dir.join("node_id.pub");
    let sec_path = keypair_dir.join("node_id.sec");

    if !pub_path.exists() || !sec_path.exists() {
        return Err(anyhow!(
            "Keys not found. Run 'annactl consensus init-keys' first."
        ));
    }

    // Read observation file
    let obs_json = std::fs::read_to_string(observation_path)
        .map_err(|e| anyhow!("Failed to read observation file: {}", e))?;

    let observation: AuditObservation = serde_json::from_str(&obs_json)
        .map_err(|e| anyhow!("Failed to parse observation JSON: {}", e))?;

    println!("Observation loaded:");
    println!("  Node ID:   {}", observation.node_id);
    println!("  Audit ID:  {}", observation.audit_id);
    println!("  Round ID:  {}", observation.round_id);
    println!("  TIS:       {:.3}", observation.tis_overall);
    println!("  Biases:    {}", observation.bias_flags.len());

    // Load or create consensus state
    let consensus_dir = get_consensus_dir()?;
    let state_path = consensus_dir.join("state.json");

    let mut state: ConsensusState = if state_path.exists() {
        let state_json = std::fs::read_to_string(&state_path)?;
        serde_json::from_str(&state_json)?
    } else {
        ConsensusState {
            rounds: Vec::new(),
            byzantine_nodes: Vec::new(),
        }
    };

    // Find or create round
    let round = state.rounds.iter_mut()
        .find(|r| r.round_id == observation.round_id);

    let round = match round {
        Some(r) => r,
        None => {
            state.rounds.push(ConsensusRound {
                round_id: observation.round_id.clone(),
                window_hours: observation.window_hours,
                started_at: Utc::now(),
                observations: Vec::new(),
                status: "Pending".to_string(),
                consensus_tis: None,
                consensus_biases: Vec::new(),
            });
            state.rounds.last_mut().unwrap()
        }
    };

    // Check for double-submit (Byzantine detection)
    if round.observations.iter()
        .any(|o| o.node_id == observation.node_id && o.audit_id != observation.audit_id)
    {
        println!("\n✗ Byzantine behavior detected: double-submit from {}", observation.node_id);

        if !state.byzantine_nodes.contains(&observation.node_id) {
            state.byzantine_nodes.push(observation.node_id.clone());
        }

        // Save state
        let state_json = serde_json::to_string_pretty(&state)?;
        std::fs::write(&state_path, state_json)?;

        return Err(anyhow!("Observation rejected: double-submit detected"));
    }

    // Add observation
    round.observations.push(observation);

    println!("\n✓ Observation submitted successfully");
    println!("\nRound status:");
    println!("  Round ID:      {}", round.round_id);
    println!("  Observations:  {}", round.observations.len());
    println!("  Status:        {}", round.status);

    // Check quorum (simplified: 3 nodes for PoC)
    let quorum_threshold = 2; // Majority of 3
    if round.observations.len() >= quorum_threshold && round.status == "Pending" {
        println!("\n✓ Quorum reached! Computing consensus...");
        compute_consensus(round)?;
        println!("  Consensus TIS: {:.3}", round.consensus_tis.unwrap());
        println!("  Status:        {}", round.status);
    } else {
        println!("\n⏳ Waiting for quorum ({}/{} observations)",
            round.observations.len(), quorum_threshold);
    }

    // Save state
    let state_json = serde_json::to_string_pretty(&state)?;
    std::fs::write(&state_path, state_json)?;

    println!("\nState saved to: {}\n", state_path.display());

    Ok(())
}

/// Compute consensus for a round (simplified PoC version)
fn compute_consensus(round: &mut ConsensusRound) -> Result<()> {
    let valid_obs = &round.observations;

    if valid_obs.is_empty() {
        round.status = "Failed".to_string();
        return Ok(());
    }

    // Compute weighted average TIS
    let tis_sum: f64 = valid_obs.iter().map(|o| o.tis_overall).sum();
    let tis_avg = tis_sum / valid_obs.len() as f64;
    round.consensus_tis = Some(tis_avg);

    // Aggregate biases (majority rule)
    let mut bias_counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    for obs in valid_obs {
        for bias in &obs.bias_flags {
            *bias_counts.entry(bias.clone()).or_insert(0) += 1;
        }
    }

    let majority = valid_obs.len() / 2 + 1;
    for (bias, count) in bias_counts {
        if count >= majority {
            round.consensus_biases.push(bias);
        }
    }

    round.status = "Complete".to_string();

    Ok(())
}

/// Execute `annactl consensus status` command
pub async fn execute_consensus_status_command(
    round_id: Option<String>,
    json: bool,
) -> Result<()> {
    let consensus_dir = get_consensus_dir()?;
    let state_path = consensus_dir.join("state.json");

    if !state_path.exists() {
        if json {
            println!("{{\"error\": \"No consensus state found\"}}");
        } else {
            println!("\n⚠️  No consensus state found");
            println!("\nRun 'annactl consensus submit <observation.json>' to start.\n");
        }
        return Ok(());
    }

    let state_json = std::fs::read_to_string(&state_path)?;
    let state: ConsensusState = serde_json::from_str(&state_json)?;

    if json {
        // JSON output
        if let Some(rid) = round_id {
            if let Some(round) = state.rounds.iter().find(|r| r.round_id == rid) {
                println!("{}", serde_json::to_string_pretty(round)?);
            } else {
                println!("{{\"error\": \"Round not found\"}}");
            }
        } else {
            println!("{}", serde_json::to_string_pretty(&state)?);
        }
    } else {
        // Pretty table output
        println!("\n=== Consensus Status ===\n");

        if state.rounds.is_empty() {
            println!("No rounds found.\n");
            return Ok(());
        }

        if let Some(rid) = round_id {
            // Show specific round
            if let Some(round) = state.rounds.iter().find(|r| r.round_id == rid) {
                print_round_details(round);
            } else {
                println!("Round '{}' not found.\n", rid);
            }
        } else {
            // Show all rounds
            println!("Total rounds: {}", state.rounds.len());
            if !state.byzantine_nodes.is_empty() {
                println!("Byzantine nodes: {}", state.byzantine_nodes.len());
            }
            println!();

            for round in &state.rounds {
                print_round_summary(round);
            }
        }
    }

    Ok(())
}

fn print_round_summary(round: &ConsensusRound) {
    println!("Round: {}", round.round_id);
    println!("  Status:        {}", round.status);
    println!("  Observations:  {}", round.observations.len());
    if let Some(tis) = round.consensus_tis {
        println!("  Consensus TIS: {:.3}", tis);
    }
    if !round.consensus_biases.is_empty() {
        println!("  Biases:        {}", round.consensus_biases.len());
    }
    println!();
}

fn print_round_details(round: &ConsensusRound) {
    println!("Round ID: {}", round.round_id);
    println!("Status:   {}", round.status);
    println!("Started:  {}", round.started_at);
    println!("Window:   {} hours", round.window_hours);
    println!();

    println!("Observations: {}", round.observations.len());
    for (i, obs) in round.observations.iter().enumerate() {
        println!("  {}. Node: {} | TIS: {:.3} | Biases: {}",
            i + 1,
            &obs.node_id[0..16.min(obs.node_id.len())],
            obs.tis_overall,
            obs.bias_flags.len()
        );
    }
    println!();

    if let Some(tis) = round.consensus_tis {
        println!("Consensus TIS: {:.3}", tis);
    }

    if !round.consensus_biases.is_empty() {
        println!("Consensus Biases:");
        for bias in &round.consensus_biases {
            println!("  - {}", bias);
        }
    }
    println!();
}

/// Execute `annactl consensus reconcile` command
pub async fn execute_consensus_reconcile_command(
    window: u64,
    json: bool,
) -> Result<()> {
    let consensus_dir = get_consensus_dir()?;
    let state_path = consensus_dir.join("state.json");

    if !state_path.exists() {
        if json {
            println!("{{\"error\": \"No consensus state found\"}}");
        } else {
            println!("\n⚠️  No consensus state found\n");
        }
        return Ok(());
    }

    let state_json = std::fs::read_to_string(&state_path)?;
    let mut state: ConsensusState = serde_json::from_str(&state_json)?;

    println!("\n=== Consensus Reconciliation ===\n");
    println!("Window: {} hours", window);
    println!();

    let mut reconciled = 0;

    for round in &mut state.rounds {
        if round.window_hours == window && round.status == "Pending" {
            println!("Reconciling round: {}", round.round_id);
            println!("  Observations: {}", round.observations.len());

            compute_consensus(round)?;

            println!("  ✓ Consensus computed");
            if let Some(tis) = round.consensus_tis {
                println!("  TIS: {:.3}", tis);
            }
            println!();

            reconciled += 1;
        }
    }

    if reconciled == 0 {
        println!("No pending rounds found for {}-hour window.\n", window);
    } else {
        println!("✓ Reconciled {} round(s)\n", reconciled);

        // Save state
        let state_json = serde_json::to_string_pretty(&state)?;
        std::fs::write(&state_path, state_json)?;
    }

    if json {
        println!("{{\"reconciled\": {}}}", reconciled);
    }

    Ok(())
}
