//! Orchestrator - LLM-A logic for processing questions

use crate::client::DaemonClient;
use crate::llm_client::LlmClient;
use anna_common::prompts::{LLM_A_SYSTEM_PROMPT, LLM_B_SYSTEM_PROMPT};
use anna_common::{AnnaResponse, ExpertResponse, ModelSelection, Verdict};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

/// LLM-A action request
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "action")]
#[serde(rename_all = "snake_case")]
enum OrchestratorAction {
    RequestProbes {
        probes: Vec<String>,
        reason: String,
    },
    FinalAnswer {
        answer: String,
        confidence: f64,
        sources: Vec<String>,
    },
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

    info!("  Processing question: {}", question);

    // Step 1: Ask LLM-A what probes are needed
    let initial_prompt = format!(
        "User question: {}\n\nAnalyze this question and determine what probes are needed.",
        question
    );

    let llm_a_response = llm
        .chat(&models.orchestrator, LLM_A_SYSTEM_PROMPT, &initial_prompt)
        .await
        .context("Failed to get LLM-A response")?;

    debug!("LLM-A initial response: {}", llm_a_response);

    // Parse LLM-A action
    let action: OrchestratorAction = serde_json::from_str(&llm_a_response).unwrap_or_else(|_| {
        // Default to requesting basic probes if parsing fails
        OrchestratorAction::RequestProbes {
            probes: vec!["cpu.info".to_string(), "mem.info".to_string()],
            reason: "Gathering basic system information".to_string(),
        }
    });

    match action {
        OrchestratorAction::RequestProbes { probes, reason } => {
            info!("  Requesting probes: {:?} ({})", probes, reason);

            // Run requested probes
            let probe_refs: Vec<&str> = probes.iter().map(|s| s.as_str()).collect();
            let results = daemon.run_probes(&probe_refs).await?;

            // Build evidence context
            let evidence: serde_json::Value = results
                .iter()
                .map(|r| (r.id.clone(), r.data.clone()))
                .collect();

            // Step 2: Ask LLM-A to answer with evidence
            let evidence_prompt = format!(
                "User question: {}\n\nEvidence from probes:\n{}\n\nProvide final answer based on this evidence.",
                question,
                serde_json::to_string_pretty(&evidence)?
            );

            let llm_a_answer = llm
                .chat(&models.orchestrator, LLM_A_SYSTEM_PROMPT, &evidence_prompt)
                .await
                .context("Failed to get LLM-A answer")?;

            debug!("LLM-A answer: {}", llm_a_answer);

            // Step 3: Validate with LLM-B
            let validation_prompt = format!(
                "LLM-A reasoning:\n{}\n\nEvidence:\n{}\n\nValidate this reasoning.",
                llm_a_answer,
                serde_json::to_string_pretty(&evidence)?
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

            // Build final response based on verdict
            match expert.verdict {
                Verdict::Approved => {
                    // Parse the approved answer
                    let final_action: OrchestratorAction =
                        serde_json::from_str(&llm_a_answer).unwrap_or_else(|_| {
                            OrchestratorAction::FinalAnswer {
                                answer: llm_a_answer.clone(),
                                confidence: expert.confidence,
                                sources: probes.clone(),
                            }
                        });

                    if let OrchestratorAction::FinalAnswer {
                        answer,
                        confidence: _,
                        sources,
                    } = final_action
                    {
                        Ok(AnnaResponse {
                            answer,
                            confidence: expert.confidence,
                            sources,
                            warning: None,
                        })
                    } else {
                        Ok(AnnaResponse {
                            answer: llm_a_answer,
                            confidence: expert.confidence,
                            sources: probes,
                            warning: None,
                        })
                    }
                }
                Verdict::Revise => {
                    warn!("LLM-B requested revision: {}", expert.explanation);

                    // Use corrected reasoning if available
                    let answer = expert
                        .corrected_reasoning
                        .unwrap_or_else(|| format!("{}\n\nNote: {}", llm_a_answer, expert.explanation));

                    Ok(AnnaResponse {
                        answer,
                        confidence: expert.confidence,
                        sources: probes,
                        warning: Some(format!("Revised: {}", expert.explanation)),
                    })
                }
                Verdict::NotPossible => {
                    warn!(
                        "LLM-B: insufficient evidence, needs: {:?}",
                        expert.required_probes
                    );

                    Ok(AnnaResponse {
                        answer: format!(
                            "Unable to answer with confidence.\nReason: {}\nNeeded: {:?}",
                            expert.explanation, expert.required_probes
                        ),
                        confidence: expert.confidence,
                        sources: probes,
                        warning: Some("Insufficient evidence".to_string()),
                    })
                }
            }
        }
        OrchestratorAction::FinalAnswer {
            answer,
            confidence,
            sources,
        } => {
            // Direct answer without probes (unlikely but handle it)
            Ok(AnnaResponse {
                answer,
                confidence,
                sources,
                warning: Some("Answer provided without evidence verification".to_string()),
            })
        }
    }
}
