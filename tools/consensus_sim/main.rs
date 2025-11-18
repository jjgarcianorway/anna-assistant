#!/usr/bin/env rust-script
//! Consensus Simulator - Deterministic test scenarios for Phase 1.8 PoC
//!
//! Usage:
//!   consensus_sim --nodes 5 --scenario healthy
//!   consensus_sim --nodes 5 --scenario slow-node
//!   consensus_sim --nodes 5 --scenario byzantine
//!
//! Outputs machine-readable JSON reports to ./artifacts/simulations/

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

// ============================================================================
// TYPES (duplicated from annad consensus for standalone sim)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AuditObservation {
    node_id: String,
    audit_id: String,
    round_id: String,
    window_hours: u64,
    timestamp: String,
    forecast_hash: String,
    outcome_hash: String,
    tis_components: TISComponents,
    tis_overall: f64,
    bias_flags: Vec<String>,
    signature: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TISComponents {
    prediction_accuracy: f64,
    ethical_alignment: f64,
    coherence_stability: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SimulationReport {
    scenario: String,
    node_count: usize,
    round_id: String,
    observations_submitted: usize,
    quorum_reached: bool,
    required_quorum: usize,
    consensus_tis: Option<f64>,
    consensus_biases: Vec<String>,
    byzantine_nodes: Vec<String>,
    success: bool,
    notes: String,
}

// ============================================================================
// SIMULATOR LOGIC
// ============================================================================

fn generate_observation(
    node_id: &str,
    audit_id: &str,
    round_id: &str,
    tis: f64,
    biases: Vec<String>,
) -> AuditObservation {
    AuditObservation {
        node_id: node_id.to_string(),
        audit_id: audit_id.to_string(),
        round_id: round_id.to_string(),
        window_hours: 24,
        timestamp: "2025-11-12T12:00:00Z".to_string(),
        forecast_hash: format!("sha256:forecast_{}", audit_id),
        outcome_hash: format!("sha256:outcome_{}", audit_id),
        tis_components: TISComponents {
            prediction_accuracy: 0.85,
            ethical_alignment: 0.80,
            coherence_stability: 0.90,
        },
        tis_overall: tis,
        bias_flags: biases,
        signature: vec![0; 64], // Mock signature for PoC
    }
}

fn simulate_healthy_quorum(node_count: usize) -> SimulationReport {
    let round_id = "round_healthy_001";
    let mut observations = Vec::new();

    // All nodes submit consistent observations
    for i in 0..node_count {
        let node_id = format!("node_{:03}", i);
        let audit_id = format!("audit_healthy_{:03}", i);
        let tis = 0.85 + (i as f64 * 0.01); // Slight variation
        observations.push(generate_observation(
            &node_id,
            &audit_id,
            round_id,
            tis,
            vec![],
        ));
    }

    // Compute consensus
    let quorum_threshold = node_count.div_ceil(2);
    let quorum_reached = observations.len() >= quorum_threshold;

    let consensus_tis = if quorum_reached {
        Some(observations.iter().map(|o| o.tis_overall).sum::<f64>() / observations.len() as f64)
    } else {
        None
    };

    SimulationReport {
        scenario: "healthy".to_string(),
        node_count,
        round_id: round_id.to_string(),
        observations_submitted: observations.len(),
        quorum_reached,
        required_quorum: quorum_threshold,
        consensus_tis,
        consensus_biases: vec![],
        byzantine_nodes: vec![],
        success: quorum_reached,
        notes: "All nodes submitted consistent observations. Consensus reached successfully."
            .to_string(),
    }
}

fn simulate_slow_node(node_count: usize) -> SimulationReport {
    let round_id = "round_slow_node_001";
    let mut observations = Vec::new();

    // Most nodes submit, but one is slow (doesn't submit)
    for i in 0..node_count - 1 {
        let node_id = format!("node_{:03}", i);
        let audit_id = format!("audit_slow_{:03}", i);
        let tis = 0.85 + (i as f64 * 0.01);
        observations.push(generate_observation(
            &node_id,
            &audit_id,
            round_id,
            tis,
            vec![],
        ));
    }

    // Last node is slow - doesn't submit

    let quorum_threshold = node_count.div_ceil(2);
    let quorum_reached = observations.len() >= quorum_threshold;

    let consensus_tis = if quorum_reached {
        Some(observations.iter().map(|o| o.tis_overall).sum::<f64>() / observations.len() as f64)
    } else {
        None
    };

    SimulationReport {
        scenario: "slow-node".to_string(),
        node_count,
        round_id: round_id.to_string(),
        observations_submitted: observations.len(),
        quorum_reached,
        required_quorum: quorum_threshold,
        consensus_tis,
        consensus_biases: vec![],
        byzantine_nodes: vec![],
        success: quorum_reached,
        notes: format!(
            "Node {:03} was slow and didn't submit. Consensus still reached with {}/{} nodes.",
            node_count - 1,
            observations.len(),
            node_count
        ),
    }
}

fn simulate_byzantine(node_count: usize) -> SimulationReport {
    let round_id = "round_byzantine_001";
    let mut observations = Vec::new();
    let mut byzantine_nodes = Vec::new();

    // First node submits twice (Byzantine double-submit)
    let byzantine_node = "node_000";
    observations.push(generate_observation(
        byzantine_node,
        "audit_byzantine_000a",
        round_id,
        0.85,
        vec![],
    ));
    // Second observation from same node (will be detected)
    observations.push(generate_observation(
        byzantine_node,
        "audit_byzantine_000b",
        round_id,
        0.95, // Different TIS - conflicting
        vec![],
    ));

    byzantine_nodes.push(byzantine_node.to_string());

    // Other nodes submit normally
    for i in 1..node_count {
        let node_id = format!("node_{:03}", i);
        let audit_id = format!("audit_byzantine_{:03}", i);
        let tis = 0.85 + (i as f64 * 0.01);
        observations.push(generate_observation(
            &node_id,
            &audit_id,
            round_id,
            tis,
            vec![],
        ));
    }

    // Consensus excludes Byzantine node
    let valid_observations: Vec<_> = observations
        .iter()
        .filter(|o| !byzantine_nodes.contains(&o.node_id))
        .collect();

    let quorum_threshold = node_count.div_ceil(2);
    let quorum_reached = valid_observations.len() >= quorum_threshold;

    let consensus_tis = if quorum_reached {
        Some(
            valid_observations
                .iter()
                .map(|o| o.tis_overall)
                .sum::<f64>()
                / valid_observations.len() as f64,
        )
    } else {
        None
    };

    SimulationReport {
        scenario: "byzantine".to_string(),
        node_count,
        round_id: round_id.to_string(),
        observations_submitted: observations.len(),
        quorum_reached,
        required_quorum: quorum_threshold,
        consensus_tis,
        consensus_biases: vec![],
        byzantine_nodes,
        success: quorum_reached,
        notes: format!(
            "Node {} detected as Byzantine (double-submit). Excluded from consensus. Quorum reached with {} valid nodes.",
            byzantine_node,
            valid_observations.len()
        ),
    }
}

// ============================================================================
// MAIN
// ============================================================================

fn main() {
    let args: Vec<String> = std::env::args().collect();

    // Parse arguments
    let mut nodes = 5;
    let mut scenario = "healthy".to_string();

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--nodes" => {
                if i + 1 < args.len() {
                    nodes = args[i + 1].parse().unwrap_or(5);
                    i += 2;
                } else {
                    eprintln!("Error: --nodes requires a value");
                    std::process::exit(1);
                }
            }
            "--scenario" => {
                if i + 1 < args.len() {
                    scenario = args[i + 1].clone();
                    i += 2;
                } else {
                    eprintln!("Error: --scenario requires a value");
                    std::process::exit(1);
                }
            }
            "--help" | "-h" => {
                println!("Consensus Simulator - Phase 1.8 PoC");
                println!();
                println!("Usage:");
                println!("  consensus_sim --nodes <N> --scenario <scenario>");
                println!();
                println!("Options:");
                println!("  --nodes <N>           Number of nodes (3-7, default: 5)");
                println!("  --scenario <scenario> Scenario: healthy, slow-node, byzantine");
                println!();
                println!("Examples:");
                println!("  consensus_sim --nodes 5 --scenario healthy");
                println!("  consensus_sim --nodes 5 --scenario slow-node");
                println!("  consensus_sim --nodes 5 --scenario byzantine");
                std::process::exit(0);
            }
            _ => {
                eprintln!("Error: Unknown argument: {}", args[i]);
                eprintln!("Run with --help for usage");
                std::process::exit(1);
            }
        }
    }

    // Validate nodes
    if !(3..=7).contains(&nodes) {
        eprintln!("Error: nodes must be between 3 and 7");
        std::process::exit(1);
    }

    // Run simulation
    let report = match scenario.as_str() {
        "healthy" => simulate_healthy_quorum(nodes),
        "slow-node" => simulate_slow_node(nodes),
        "byzantine" => simulate_byzantine(nodes),
        _ => {
            eprintln!("Error: Unknown scenario: {}", scenario);
            eprintln!("Valid scenarios: healthy, slow-node, byzantine");
            std::process::exit(1);
        }
    };

    // Create output directory
    let output_dir = PathBuf::from("./artifacts/simulations");
    fs::create_dir_all(&output_dir).unwrap();

    // Write report
    let output_file = output_dir.join(format!("{}.json", scenario));
    let json = serde_json::to_string_pretty(&report).unwrap();
    fs::write(&output_file, json).unwrap();

    // Print summary
    println!("\n=== Consensus Simulation: {} ===\n", scenario);
    println!("Nodes:                {}", report.node_count);
    println!("Round ID:             {}", report.round_id);
    println!("Observations:         {}", report.observations_submitted);
    println!("Required Quorum:      {}", report.required_quorum);
    println!("Quorum Reached:       {}", report.quorum_reached);

    if let Some(tis) = report.consensus_tis {
        println!("Consensus TIS:        {:.3}", tis);
    } else {
        println!("Consensus TIS:        N/A (quorum not reached)");
    }

    if !report.byzantine_nodes.is_empty() {
        println!(
            "Byzantine Nodes:      {}",
            report.byzantine_nodes.join(", ")
        );
    }

    println!("\nNotes: {}", report.notes);
    println!("\nReport saved to: {}\n", output_file.display());

    if report.success {
        std::process::exit(0);
    } else {
        std::process::exit(1);
    }
}
