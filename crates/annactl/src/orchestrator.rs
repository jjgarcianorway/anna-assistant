//! Orchestrator - LLM-A logic for processing questions
//!
//! v0.2.0: Enhanced with tool catalog enforcement and evidence discipline

use crate::client::DaemonClient;
use crate::llm_client::LlmClient;
use anna_common::prompts::{LLM_A_SYSTEM_PROMPT, LLM_B_SYSTEM_PROMPT};
use anna_common::{AnnaResponse, Evidence, ExpertResponse, ModelSelection, ReliabilityScore, Verdict};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use tracing::{debug, info, warn};

/// Tool catalog - the ONLY probes that exist
const TOOL_CATALOG: &[&str] = &["cpu.info", "mem.info", "disk.lsblk"];

/// LLM-A action request
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "action")]
#[serde(rename_all = "snake_case")]
enum OrchestratorAction {
    RequestProbes {
        probes: Vec<String>,
        reason: String,
        #[serde(default)]
        coverage: Option<String>,
    },
    FinalAnswer {
        answer: String,
        confidence: f64,
        sources: Vec<String>,
        #[serde(default)]
        reliability: Option<ReliabilityScoreJson>,
        #[serde(default)]
        limitations: Option<Vec<String>>,
    },
}

/// Reliability score from LLM JSON
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct ReliabilityScoreJson {
    #[serde(default)]
    overall: f64,
    #[serde(default)]
    evidence_quality: f64,
    #[serde(default)]
    reasoning_quality: f64,
    #[serde(default)]
    coverage: f64,
}

/// Validate that probes are in the tool catalog
fn validate_probes(probes: &[String]) -> (Vec<String>, Vec<String>) {
    let catalog: HashSet<&str> = TOOL_CATALOG.iter().copied().collect();
    let mut valid = Vec::new();
    let mut invalid = Vec::new();

    for probe in probes {
        if catalog.contains(probe.as_str()) {
            valid.push(probe.clone());
        } else {
            invalid.push(probe.clone());
        }
    }

    (valid, invalid)
}

/// Process a user question through the two-LLM system
pub async fn process_question(question: &str, daemon: &DaemonClient) -> Result<AnnaResponse> {
    let llm = LlmClient::new();
    let models = ModelSelection::default();

    // Check Ollama availability
    if !llm.is_available().await {
        return Ok(AnnaResponse {
            answer: "Ollama is not running. Please start Ollama first.".to_string(),
            confidence: 0.0,
            sources: vec![],
            warning: Some("LLM backend unavailable".to_string()),
        });
    }

    info!("  📝  Processing question: {}", question);

    // Build tool catalog context for LLM
    let catalog_info = format!(
        "AVAILABLE PROBES (tool catalog):\n{}\n\nUser question: {}\n\nDetermine what probes are needed.",
        TOOL_CATALOG.join(", "),
        question
    );

    let llm_a_response = llm
        .chat(&models.orchestrator, LLM_A_SYSTEM_PROMPT, &catalog_info)
        .await
        .context("Failed to get LLM-A response")?;

    debug!("LLM-A initial response: {}", llm_a_response);

    // Parse LLM-A action
    let action: OrchestratorAction = serde_json::from_str(&llm_a_response).unwrap_or_else(|_| {
        OrchestratorAction::RequestProbes {
            probes: vec!["cpu.info".to_string(), "mem.info".to_string()],
            reason: "Gathering basic system information".to_string(),
            coverage: Some("partial".to_string()),
        }
    });

    match action {
        OrchestratorAction::RequestProbes { probes, reason, .. } => {
            // CRITICAL: Validate probes against tool catalog
            let (valid_probes, invalid_probes) = validate_probes(&probes);

            if !invalid_probes.is_empty() {
                warn!("  ⚠️  LLM-A requested invalid probes: {:?}", invalid_probes);
            }

            if valid_probes.is_empty() {
                return Ok(AnnaResponse {
                    answer: format!(
                        "I cannot answer this question.\n\nReason: No valid probes available.\nRequested probes {:?} do not exist in tool catalog.\n\nAvailable probes: {}",
                        invalid_probes,
                        TOOL_CATALOG.join(", ")
                    ),
                    confidence: 0.0,
                    sources: vec![],
                    warning: Some("No valid probes - possible hallucination".to_string()),
                });
            }

            info!("  🔍  Running probes: {:?} ({})", valid_probes, reason);

            // Run validated probes only
            let probe_refs: Vec<&str> = valid_probes.iter().map(|s| s.as_str()).collect();
            let results = daemon.run_probes(&probe_refs).await?;

            // Build structured evidence
            let evidence_list: Vec<Evidence> = results.iter().map(Evidence::from_probe_result).collect();
            let evidence_json: serde_json::Value = results
                .iter()
                .map(|r| (r.id.clone(), r.data.clone()))
                .collect();

            // Calculate initial reliability based on evidence quality
            let evidence_quality = evidence_list.iter().map(|e| e.reliability).sum::<f64>()
                / evidence_list.len().max(1) as f64;

            // Step 2: Ask LLM-A to answer with evidence
            let evidence_prompt = format!(
                "TOOL CATALOG: {}\n\nUser question: {}\n\nEvidence from probes (ONLY use this data):\n{}\n\nProvide final answer. CITE EVERY CLAIM with [source: probe_id].",
                TOOL_CATALOG.join(", "),
                question,
                serde_json::to_string_pretty(&evidence_json)?
            );

            let llm_a_answer = llm
                .chat(&models.orchestrator, LLM_A_SYSTEM_PROMPT, &evidence_prompt)
                .await
                .context("Failed to get LLM-A answer")?;

            debug!("LLM-A answer: {}", llm_a_answer);

            // Step 3: Validate with LLM-B
            let validation_prompt = format!(
                "TOOL CATALOG (only these probes exist): {}\n\nLLM-A reasoning:\n{}\n\nRaw evidence:\n{}\n\nValidate: Check for hallucinations, verify all claims have [source: probe_id] citations.",
                TOOL_CATALOG.join(", "),
                llm_a_answer,
                serde_json::to_string_pretty(&evidence_json)?
            );

            let llm_b_response = llm
                .chat(&models.expert, LLM_B_SYSTEM_PROMPT, &validation_prompt)
                .await
                .context("Failed to get LLM-B validation")?;

            debug!("LLM-B validation: {}", llm_b_response);

            // Parse LLM-B verdict
            let expert: ExpertResponse =
                serde_json::from_str(&llm_b_response).unwrap_or_else(|_| ExpertResponse {
                    verdict: Verdict::Approved,
                    explanation: "Unable to parse expert response".to_string(),
                    required_probes: vec![],
                    corrected_reasoning: None,
                    confidence: 0.5,
                });

            // Build reliability score
            let mut reliability = ReliabilityScore::calculate(
                evidence_quality,
                expert.confidence,
                valid_probes.len() as f64 / probes.len().max(1) as f64,
            );

            // Deduct for invalid probe requests (LLM hallucination)
            if !invalid_probes.is_empty() {
                reliability.add_deduction(0.2, &format!("Invalid probes requested: {:?}", invalid_probes));
            }

            // Build final response based on verdict
            match expert.verdict {
                Verdict::Approved => {
                    let final_action: OrchestratorAction =
                        serde_json::from_str(&llm_a_answer).unwrap_or_else(|_| {
                            OrchestratorAction::FinalAnswer {
                                answer: llm_a_answer.clone(),
                                confidence: expert.confidence,
                                sources: valid_probes.clone(),
                                reliability: None,
                                limitations: None,
                            }
                        });

                    if let OrchestratorAction::FinalAnswer { answer, sources, limitations, .. } = final_action {
                        let warning = if reliability.overall < 0.6 {
                            Some(format!("Low reliability: {:.0}%", reliability.overall * 100.0))
                        } else {
                            limitations.map(|l| l.join(", "))
                        };

                        Ok(AnnaResponse {
                            answer,
                            confidence: reliability.overall,
                            sources,
                            warning,
                        })
                    } else {
                        Ok(AnnaResponse {
                            answer: llm_a_answer,
                            confidence: reliability.overall,
                            sources: valid_probes,
                            warning: None,
                        })
                    }
                }
                Verdict::Revise => {
                    warn!("  ✏️  LLM-B requested revision: {}", expert.explanation);
                    reliability.add_deduction(0.1, "Required revision");

                    let answer = expert.corrected_reasoning
                        .unwrap_or_else(|| format!("{}\n\n⚠️  Note: {}", llm_a_answer, expert.explanation));

                    Ok(AnnaResponse {
                        answer,
                        confidence: reliability.overall,
                        sources: valid_probes,
                        warning: Some(format!("Revised: {}", expert.explanation)),
                    })
                }
                Verdict::NotPossible => {
                    warn!("  ❌  LLM-B: insufficient evidence");

                    Ok(AnnaResponse {
                        answer: format!(
                            "I cannot provide a reliable answer.\n\n📋  Reason: {}\n🔧  Available probes: {}\n📊  Reliability: {:.0}%",
                            expert.explanation,
                            TOOL_CATALOG.join(", "),
                            expert.confidence * 100.0
                        ),
                        confidence: expert.confidence,
                        sources: valid_probes,
                        warning: Some("Insufficient evidence for reliable answer".to_string()),
                    })
                }
            }
        }
        OrchestratorAction::FinalAnswer { answer, confidence, sources, .. } => {
            // Direct answer without probes - suspicious
            warn!("  ⚠️  LLM-A provided answer without requesting probes");

            Ok(AnnaResponse {
                answer,
                confidence: confidence * 0.5, // Heavily penalize no-evidence answers
                sources,
                warning: Some("⚠️  Answer provided without evidence - reliability reduced".to_string()),
            })
        }
    }
}
