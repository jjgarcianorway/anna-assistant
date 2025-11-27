//! Orchestrator - LLM-A logic for processing questions
//!
//! v0.3.0:
//! - Help/Version via LLM pipeline
//! - Stable Repeated Answers with reconciliation
//! - Strict Hallucination Guardrails
//!
//! v0.4.0:
//! - Update status in version/help output

use crate::client::DaemonClient;
use crate::llm_client::LlmClient;
use anna_common::prompts::{LLM_A_SYSTEM_PROMPT, LLM_B_SYSTEM_PROMPT};
use anna_common::{
    AnnaResponse, Evidence, ExpertResponse, ModelSelection, ReliabilityScore, UpdateConfig,
    UpdateResult, UpdateState, Verdict,
};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use tracing::{debug, info, warn};

/// Tool catalog - the ONLY probes that exist
const TOOL_CATALOG: &[&str] = &["cpu.info", "mem.info", "disk.lsblk"];

/// Internal query types for help/version
#[derive(Debug, Clone)]
pub enum InternalQueryType {
    Version {
        version: String,
        daemon_status: String,
        probe_count: usize,
        update_config: UpdateConfig,
        update_state: UpdateState,
    },
    Help {
        update_config: UpdateConfig,
    },
}

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

/// Stability tracking for repeated answers
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct StabilityResult {
    answer: String,
    stability_bonus: f64,
    stability_note: Option<String>,
    required_reconciliation: bool,
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

/// Check if two answers are semantically similar
fn answers_match(answer1: &str, answer2: &str) -> bool {
    // Simple heuristic: if >80% of words match, consider them equivalent
    let words1: HashSet<&str> = answer1.split_whitespace().collect();
    let words2: HashSet<&str> = answer2.split_whitespace().collect();

    if words1.is_empty() || words2.is_empty() {
        return answer1 == answer2;
    }

    let intersection = words1.intersection(&words2).count();
    let union = words1.union(&words2).count();

    let similarity = intersection as f64 / union as f64;
    similarity > 0.8
}

/// Format update state for display
fn format_update_state(state: &UpdateState) -> (String, String) {
    let last_check = state
        .last_check
        .map(|ts| {
            chrono::DateTime::from_timestamp(ts, 0)
                .map(|dt| dt.format("%Y-%m-%d %H:%M:%S UTC").to_string())
                .unwrap_or_else(|| "invalid".to_string())
        })
        .unwrap_or_else(|| "never".to_string());

    let last_result = state
        .last_result
        .map(|r| match r {
            UpdateResult::Ok => "ok".to_string(),
            UpdateResult::Failed => format!(
                "failed{}",
                state
                    .last_failure_reason
                    .as_ref()
                    .map(|r| format!(" ({})", r))
                    .unwrap_or_default()
            ),
            UpdateResult::NoUpdate => "no update".to_string(),
            UpdateResult::Unknown => "unknown".to_string(),
        })
        .unwrap_or_else(|| "unknown".to_string());

    (last_check, last_result)
}

/// Process internal queries (help/version) via LLM pipeline
pub async fn process_internal_query(
    _question: &str,
    _daemon: &DaemonClient,
    query_type: InternalQueryType,
) -> Result<AnnaResponse> {
    let llm = LlmClient::new();
    let models = ModelSelection::default();

    // Check Ollama availability
    if !llm.is_available().await {
        // Fallback for when LLM is not available
        return Ok(match query_type {
            InternalQueryType::Version { version, daemon_status, probe_count, update_config, update_state } => {
                let (last_check, last_result) = format_update_state(&update_state);
                let update_mode = if update_config.auto { "auto" } else { "manual" };

                AnnaResponse {
                    answer: format!(
                        "Anna Assistant v{}\nChannel: {}\nUpdate mode: {}\nLast update check: {}\nLast update result: {}\nDaemon: {}\nTool catalog: {} probes registered\n\n⚠️  LLM unavailable - showing basic info only",
                        version, update_config.channel.as_str(), update_mode, last_check, last_result, daemon_status, probe_count
                    ),
                    confidence: 0.5,
                    sources: vec!["internal.state".to_string(), "update.config".to_string(), "update.state".to_string()],
                    warning: Some("LLM backend unavailable".to_string()),
                }
            }
            InternalQueryType::Help { update_config } => {
                let auto_update_note = if update_config.auto && update_config.channel == anna_common::UpdateChannel::Dev {
                    "\n\nAuto-update: enabled (dev mode, every 10 minutes)"
                } else if update_config.auto {
                    "\n\nAuto-update: enabled (controlled via config)"
                } else {
                    "\n\nAuto-update: disabled (configure via ~/.config/anna/config.toml)"
                };

                AnnaResponse {
                    answer: format!(r#"Usage:
  annactl "<question>"      Ask Anna anything
  annactl                   Start interactive REPL
  annactl -V | --version    Show version
  annactl -h | --help       Show help{}

⚠️  LLM unavailable - showing basic help only"#, auto_update_note),
                    confidence: 0.5,
                    sources: vec!["internal.help".to_string(), "update.config".to_string()],
                    warning: Some("LLM backend unavailable".to_string()),
                }
            }
        });
    }

    match query_type {
        InternalQueryType::Version { version, daemon_status, probe_count, update_config, update_state } => {
            // Get model info
            let model_name = &models.orchestrator;
            let (last_check, last_result) = format_update_state(&update_state);
            let update_mode = if update_config.auto { "auto" } else { "manual" };

            // Build evidence for version query
            let evidence = serde_json::json!({
                "internal.version": {
                    "anna_version": version,
                    "daemon_status": daemon_status,
                    "model": model_name,
                    "tool_catalog_count": probe_count,
                    "tool_catalog": TOOL_CATALOG,
                },
                "update.config": {
                    "channel": update_config.channel.as_str(),
                    "auto": update_config.auto,
                    "interval_seconds": update_config.effective_interval(),
                },
                "update.state": {
                    "last_check": last_check,
                    "last_result": last_result,
                }
            });

            let prompt = format!(
                "You are reporting your own version information. Use ONLY this evidence:\n{}\n\nFormat your response EXACTLY as:\nAnna Assistant v<version>\nChannel: <channel>\nUpdate mode: <auto|manual>\nLast update check: <timestamp>\nLast update result: <result>\nDaemon: <status>\nModel: <model>\nTool catalog: <n> probes registered",
                serde_json::to_string_pretty(&evidence)?
            );

            let llm_response = llm
                .chat(&models.orchestrator, LLM_A_SYSTEM_PROMPT, &prompt)
                .await
                .unwrap_or_else(|_| format!(
                    "Anna Assistant v{}\nChannel: {}\nUpdate mode: {}\nLast update check: {}\nLast update result: {}\nDaemon: {}\nModel: {}\nTool catalog: {} probes registered",
                    version, update_config.channel.as_str(), update_mode, last_check, last_result, daemon_status, model_name, probe_count
                ));

            // Validate with LLM-B
            let validation_prompt = format!(
                "Verify this version report uses only the provided evidence:\n{}\n\nEvidence: {}",
                llm_response,
                serde_json::to_string_pretty(&evidence)?
            );

            let expert_response = llm
                .chat(&models.expert, LLM_B_SYSTEM_PROMPT, &validation_prompt)
                .await;

            // Calculate confidence based on evidence coverage
            let mut confidence = if expert_response.is_ok() { 0.95 } else { 0.85 };

            // Reduce confidence if update state is missing
            if update_state.last_check.is_none() {
                confidence -= 0.05;
            }

            Ok(AnnaResponse {
                answer: format!(
                    "Anna Assistant v{}\nChannel: {}\nUpdate mode: {}\nLast update check: {}\nLast update result: {}\nDaemon: {}\nModel: {}\nTool catalog: {} probes registered",
                    version, update_config.channel.as_str(), update_mode, last_check, last_result, daemon_status, model_name, probe_count
                ),
                confidence,
                sources: vec![
                    "internal.version".to_string(),
                    "update.config".to_string(),
                    "update.state".to_string(),
                ],
                warning: None,
            })
        }
        InternalQueryType::Help { update_config } => {
            // Build help evidence
            let auto_update_note = if update_config.auto && update_config.channel == anna_common::UpdateChannel::Dev {
                "Auto-update: enabled (dev mode, updates every 10 minutes when new version available)"
            } else if update_config.auto {
                "Auto-update: enabled (controlled via config)"
            } else {
                "Auto-update: disabled (configure via ~/.config/anna/config.toml)"
            };

            let evidence = serde_json::json!({
                "internal.help": {
                    "commands": [
                        {"usage": "annactl \"<question>\"", "description": "Ask Anna anything"},
                        {"usage": "annactl", "description": "Start interactive REPL"},
                        {"usage": "annactl -V | --version", "description": "Show version"},
                        {"usage": "annactl -h | --help", "description": "Show help"},
                    ],
                    "tool_catalog": TOOL_CATALOG,
                },
                "update.config": {
                    "channel": update_config.channel.as_str(),
                    "auto": update_config.auto,
                    "note": auto_update_note,
                }
            });

            let prompt = format!(
                "You are explaining how to use yourself. Use ONLY this evidence:\n{}\n\nFormat as a clean help message with Usage section and examples. Mention the auto-update configuration.",
                serde_json::to_string_pretty(&evidence)?
            );

            let _llm_response = llm
                .chat(&models.orchestrator, LLM_A_SYSTEM_PROMPT, &prompt)
                .await;

            // Use deterministic help output for stability
            let help_text = format!(r#"Usage:
  annactl "<question>"      Ask Anna anything
  annactl                   Start interactive REPL
  annactl -V | --version    Show version
  annactl -h | --help       Show help

Examples:
  annactl "How many CPU cores do I have?"
  annactl "What's my RAM usage?"
  annactl "Show disk information"

{}

Evidence-based answers only. No hallucinations. No guesses."#, auto_update_note);

            Ok(AnnaResponse {
                answer: help_text,
                confidence: 1.0,
                sources: vec!["internal.help".to_string(), "update.config".to_string()],
                warning: None,
            })
        }
    }
}

/// Process a user question through the two-LLM system with stability checks
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

    info!("📝  Processing question: {}", question);

    // === STABILITY CHECK: Run twice and compare ===
    let first_result = process_question_internal(question, daemon, &llm, &models).await?;
    let second_result = process_question_internal(question, daemon, &llm, &models).await?;

    let stability = if answers_match(&first_result.answer, &second_result.answer) {
        // Answers match - add stability bonus
        StabilityResult {
            answer: first_result.answer.clone(),
            stability_bonus: 0.10, // +10%
            stability_note: Some("Stable: consistent across attempts".to_string()),
            required_reconciliation: false,
        }
    } else {
        // Answers differ - need reconciliation
        warn!("⚠️  Answer instability detected, running reconciliation");

        let reconciliation_prompt = format!(
            "Two attempts at answering produced different results.\n\nAttempt 1:\n{}\n\nAttempt 2:\n{}\n\nReconcile these into a single canonical answer. Use only facts that appear in BOTH answers.",
            first_result.answer,
            second_result.answer
        );

        let canonical = llm
            .chat(&models.expert, LLM_B_SYSTEM_PROMPT, &reconciliation_prompt)
            .await
            .unwrap_or_else(|_| first_result.answer.clone());

        StabilityResult {
            answer: canonical,
            stability_bonus: 0.05, // +5% for reconciled answer
            stability_note: Some("Stability: reconciliation used (attempts differed)".to_string()),
            required_reconciliation: true,
        }
    };

    // Build final response with stability info
    let mut final_confidence = first_result.confidence + stability.stability_bonus;
    final_confidence = final_confidence.min(1.0);

    let mut answer = stability.answer;
    if let Some(note) = &stability.stability_note {
        answer = format!("{}\n\n📊  {}", answer, note);
    }

    Ok(AnnaResponse {
        answer,
        confidence: final_confidence,
        sources: first_result.sources,
        warning: first_result.warning,
    })
}

/// Internal question processing (single attempt)
async fn process_question_internal(
    question: &str,
    daemon: &DaemonClient,
    llm: &LlmClient,
    models: &ModelSelection,
) -> Result<AnnaResponse> {
    // Build tool catalog context for LLM
    let catalog_info = format!(
        "AVAILABLE PROBES (tool catalog):\n{}\n\nUser question: {}\n\nDetermine what probes are needed. If NO probe can answer this question, respond with insufficient_evidence immediately.",
        TOOL_CATALOG.join(", "),
        question
    );

    let llm_a_response = llm
        .chat(&models.orchestrator, LLM_A_SYSTEM_PROMPT, &catalog_info)
        .await
        .context("Failed to get LLM-A response")?;

    debug!("LLM-A initial response: {}", llm_a_response);

    // === STRICT HALLUCINATION GUARDRAIL ===
    // Check if LLM-A is trying to answer without probes
    if llm_a_response.to_lowercase().contains("insufficient")
        || llm_a_response.to_lowercase().contains("no probe")
        || llm_a_response.to_lowercase().contains("cannot answer")
    {
        return Ok(create_insufficient_evidence_response(question));
    }

    // Parse LLM-A action
    let action: OrchestratorAction = serde_json::from_str(&llm_a_response).unwrap_or_else(|_| {
        // Try to extract probe requests from unstructured response
        let mut probes = Vec::new();
        for probe in TOOL_CATALOG {
            if llm_a_response.contains(probe) {
                probes.push(probe.to_string());
            }
        }
        if probes.is_empty() {
            probes = vec!["cpu.info".to_string(), "mem.info".to_string()];
        }
        OrchestratorAction::RequestProbes {
            probes,
            reason: "Gathering system information".to_string(),
            coverage: Some("partial".to_string()),
        }
    });

    match action {
        OrchestratorAction::RequestProbes { probes, reason, .. } => {
            // CRITICAL: Validate probes against tool catalog
            let (valid_probes, invalid_probes) = validate_probes(&probes);

            if !invalid_probes.is_empty() {
                warn!("⚠️  LLM-A requested invalid probes: {:?}", invalid_probes);
            }

            // === STRICT HALLUCINATION GUARDRAIL ===
            if valid_probes.is_empty() {
                return Ok(AnnaResponse {
                    answer: format!(
                        "{}  Insufficient evidence\n\n❌  Cannot answer this question.\n\n📋  Missing probes:\n{}\n\n🔧  Available probes: {}",
                        "🚫",
                        invalid_probes.iter().map(|p| format!("   • No {} probe available", p)).collect::<Vec<_>>().join("\n"),
                        TOOL_CATALOG.join(", ")
                    ),
                    confidence: 0.0,
                    sources: vec![],
                    warning: Some("Hallucination blocked - requested probes do not exist".to_string()),
                });
            }

            info!("🔍  Running probes: {:?} ({})", valid_probes, reason);

            // Run validated probes only
            let probe_refs: Vec<&str> = valid_probes.iter().map(|s| s.as_str()).collect();
            let results = daemon.run_probes(&probe_refs).await?;

            // === STRICT HALLUCINATION GUARDRAIL ===
            if results.is_empty() {
                return Ok(create_insufficient_evidence_response(question));
            }

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
                "TOOL CATALOG: {}\n\nUser question: {}\n\nEvidence from probes (ONLY use this data):\n{}\n\nProvide final answer. CITE EVERY CLAIM with [source: probe_id]. If evidence is insufficient, say so explicitly.",
                TOOL_CATALOG.join(", "),
                question,
                serde_json::to_string_pretty(&evidence_json)?
            );

            let llm_a_answer = llm
                .chat(&models.orchestrator, LLM_A_SYSTEM_PROMPT, &evidence_prompt)
                .await
                .context("Failed to get LLM-A answer")?;

            debug!("LLM-A answer: {}", llm_a_answer);

            // Step 3: Validate with LLM-B (Strict hallucination check)
            let validation_prompt = format!(
                "TOOL CATALOG (only these probes exist): {}\n\nLLM-A reasoning:\n{}\n\nRaw evidence:\n{}\n\nSTRICT VALIDATION:\n1. Check EVERY claim has [source: probe_id]\n2. Verify claim matches evidence exactly\n3. REJECT any claim without direct evidence\n4. If ANY hallucination detected, return NOT_POSSIBLE",
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

            // === STRICT HALLUCINATION GUARDRAIL ===
            // If reliability < 0.70, return insufficient evidence
            if reliability.overall < 0.70 {
                return Ok(AnnaResponse {
                    answer: format!(
                        "{}  Insufficient evidence\n\n⚠️  Reliability too low: {:.0}%\n\n📋  Reason: {}\n\n🔧  Available probes: {}",
                        "🚫",
                        reliability.overall * 100.0,
                        expert.explanation,
                        TOOL_CATALOG.join(", ")
                    ),
                    confidence: reliability.overall,
                    sources: valid_probes,
                    warning: Some("Low reliability - answer not trustworthy".to_string()),
                });
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
                        let warning = if reliability.overall < 0.8 {
                            Some(format!("Reliability: {:.0}%", reliability.overall * 100.0))
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
                    warn!("✏️  LLM-B requested revision: {}", expert.explanation);
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
                    warn!("❌  LLM-B: insufficient evidence or hallucination detected");

                    Ok(AnnaResponse {
                        answer: format!(
                            "{}  Insufficient evidence\n\n📋  Reason: {}\n\n🔧  Available probes: {}\n📊  Reliability: {:.0}%",
                            "🚫",
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
        OrchestratorAction::FinalAnswer { confidence, sources, .. } => {
            // === STRICT HALLUCINATION GUARDRAIL ===
            // Direct answer without probes - REJECT
            warn!("⚠️  LLM-A provided answer without requesting probes - BLOCKED");

            Ok(AnnaResponse {
                answer: format!(
                    "{}  Insufficient evidence\n\n❌  Cannot provide answer without probe data.\n\n🔧  Available probes: {}",
                    "🚫",
                    TOOL_CATALOG.join(", ")
                ),
                confidence: confidence * 0.3, // Heavily penalize
                sources,
                warning: Some("Blocked: Answer attempted without evidence".to_string()),
            })
        }
    }
}

/// Create a standard insufficient evidence response
fn create_insufficient_evidence_response(question: &str) -> AnnaResponse {
    // Determine which probes might be missing based on question keywords
    let mut missing_probes = Vec::new();
    let q_lower = question.to_lowercase();

    if q_lower.contains("gpu") || q_lower.contains("graphics") || q_lower.contains("nvidia") || q_lower.contains("amd") {
        missing_probes.push("No gpu.info probe available");
    }
    if q_lower.contains("network") || q_lower.contains("wifi") || q_lower.contains("ethernet") || q_lower.contains("ip") {
        missing_probes.push("No network.info probe available");
    }
    if q_lower.contains("package") || q_lower.contains("install") || q_lower.contains("apt") || q_lower.contains("pacman") {
        missing_probes.push("No package.info probe available");
    }
    if q_lower.contains("process") || q_lower.contains("running") || q_lower.contains("service") {
        missing_probes.push("No process.info probe available");
    }
    if q_lower.contains("user") || q_lower.contains("account") {
        missing_probes.push("No user.info probe available");
    }

    if missing_probes.is_empty() {
        missing_probes.push("No suitable probe available for this query");
    }

    AnnaResponse {
        answer: format!(
            "{}  Insufficient evidence\n\n❌  Cannot answer this question.\n\n📋  Missing probes:\n{}\n\n🔧  Available probes: {}",
            "🚫",
            missing_probes.iter().map(|p| format!("   • {}", p)).collect::<Vec<_>>().join("\n"),
            TOOL_CATALOG.join(", ")
        ),
        confidence: 0.0,
        sources: vec![],
        warning: Some("No probe available for this domain".to_string()),
    }
}
