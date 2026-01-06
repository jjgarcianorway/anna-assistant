//! Core request loop - the simple path (v0.0.831).
//!
//! This module implements the VISION.md core loop:
//!
//! ```text
//! User Query → Deterministic Router → Check Known Patterns →
//!   If known: Run probes → Deterministic answer
//!   If not: Translator → Check Recipes →
//!     If found: Execute recipe → Answer
//!     If not: Knowledge lookup → Specialist solves → Anna learns → Answer
//! ```
//!
//! v0.0.812: Added IT Department with named specialists.
//! v0.0.813: Added knowledge lookup (Arch Wiki, man pages, --help).
//! v0.0.815: Added stats tracking for recipe hits vs LLM calls.
//! v0.0.816: Don't learn recipes for dynamic queries (storage, memory, etc).
//! v0.0.826: Integrated deterministic routing - try probes before LLM.
//! v0.0.830: Fixed internal comms, improved recipe matching, better prompts.
//! v0.0.831: Unified staff roster - specialists use anna-shared/roster names.

use anna_shared::doc_fetcher;
use anna_shared::learning_engine::RecipeLibrary;
use anna_shared::rpc::{ProbeResult, RuntimeContext};
use std::time::Instant;
use tracing::{info, warn};

use crate::deterministic;
use crate::probe_registry;
use crate::router::{classify_query, get_route, QueryClass};
use crate::specialists::{ITDepartment, SpecialistRole};
use crate::state::SharedState;

// Submodules
mod types;
mod translator;
mod recipe_handler;
mod specialist_handler;
mod learning;
mod helpers;

// Public exports
pub use types::{AnswerSource, CoreLoopResult, InternalComm, ParsedQuery};

// Internal imports
use types::SpecialistSolution;
use translator::{translate_query, extract_knowledge_tags};
use recipe_handler::{find_matching_recipe, execute_recipe};
use specialist_handler::ask_specialist;
use learning::{create_recipe_from_solution, is_dynamic_query};
use helpers::gather_evidence;

/// The core loop - simple and clear
pub async fn handle_query(state: SharedState, query: &str) -> CoreLoopResult {
    let start = Instant::now();
    let mut comms: Vec<InternalComm> = Vec::new();

    info!("Core loop: query=\"{}\"", query);

    // Step 0: Try deterministic routing FIRST (v0.0.826)
    // This avoids slow LLM calls for known query patterns
    let query_class = classify_query(query);

    if query_class != QueryClass::Unknown {
        info!("Deterministic routing: class={:?}", query_class);

        comms.push(InternalComm {
            from: "Anna".to_string(),
            message: format!("Recognized pattern: {:?}", query_class),
            elapsed_ms: start.elapsed().as_millis() as u64,
        });

        // Get the route with its probes
        let route = get_route(query);

        // Run probes for this route
        let probe_results = run_route_probes(&route.probes).await;

        comms.push(InternalComm {
            from: "Anna".to_string(),
            message: format!("Ran {} probes", probe_results.len()),
            elapsed_ms: start.elapsed().as_millis() as u64,
        });

        // v0.0.829: Handle SystemUpdate specially (needs proposed command)
        if query_class == QueryClass::SystemUpdate {
            if let Some(answer) = handle_system_update_deterministic(&probe_results) {
                info!("Deterministic SystemUpdate answer");

                {
                    let mut s = state.write().await;
                    s.stats.record_request_received();
                    s.stats.record_recipe_hit();
                }

                return CoreLoopResult {
                    answer,
                    source: AnswerSource::Recipe,
                    recipe_id: Some("det_SystemUpdate".to_string()),
                    reliability: 90,
                    elapsed_ms: start.elapsed().as_millis() as u64,
                    internal_comms: comms,
                };
            }
        }

        // Try generic deterministic answer
        if route.can_answer_deterministically() {
            let context = build_runtime_context(&state).await;
            if let Some(det_result) = deterministic::try_answer(query, &context, &probe_results) {
                info!("Deterministic answer for {:?}", query_class);

                // Record stats
                {
                    let mut s = state.write().await;
                    s.stats.record_request_received();
                    s.stats.record_recipe_hit(); // Count deterministic as "fast path"
                }

                // Reliability based on whether we parsed data successfully
                let reliability = if det_result.parsed_data_count > 0 { 90 } else { 70 };

                return CoreLoopResult {
                    answer: det_result.answer,
                    source: AnswerSource::Recipe, // Deterministic = instant
                    recipe_id: Some(format!("det_{:?}", query_class)),
                    reliability,
                    elapsed_ms: start.elapsed().as_millis() as u64,
                    internal_comms: comms,
                };
            }
        }

        // Deterministic routing failed, fall through to LLM path
        info!("Deterministic routing failed, falling back to LLM");
    }

    // Step 1: Translate query to structured form (LLM path)
    let translator_model = {
        let s = state.read().await;
        s.config.llm.translator_model.clone()
    };

    comms.push(InternalComm {
        from: "Anna".to_string(),
        message: "Analyzing your query...".to_string(),
        elapsed_ms: start.elapsed().as_millis() as u64,
    });

    let parsed = match translate_query(&translator_model, query).await {
        Ok(p) => p,
        Err(e) => {
            warn!("Translator failed: {}", e);
            return CoreLoopResult {
                answer: format!("I couldn't understand that query: {}", e),
                source: AnswerSource::Failed,
                recipe_id: None,
                reliability: 0,
                elapsed_ms: start.elapsed().as_millis() as u64,
                internal_comms: comms,
            };
        }
    };

    info!(
        "Translated: intent={}, domain={}, probes={:?}",
        parsed.intent, parsed.domain, parsed.probes
    );

    comms.push(InternalComm {
        from: "Anna".to_string(),
        message: format!(
            "Understood: {} query about {}",
            parsed.intent, parsed.domain
        ),
        elapsed_ms: start.elapsed().as_millis() as u64,
    });

    // Step 2: Check recipes
    let recipe_path = RecipeLibrary::default_path();
    let mut library = RecipeLibrary::load(&recipe_path).unwrap_or_default();

    if let Some(recipe) = find_matching_recipe(&library, &parsed) {
        info!("Recipe found: {}", recipe.id);

        comms.push(InternalComm {
            from: "Anna".to_string(),
            message: format!("Found recipe '{}' - executing", recipe.id),
            elapsed_ms: start.elapsed().as_millis() as u64,
        });

        // Execute recipe
        let answer = execute_recipe(&recipe, &parsed).await;

        // Record success and update stats
        library.record_success(&recipe.id);
        let _ = library.save(&recipe_path);

        // v0.0.815: Record recipe hit in global stats
        {
            let mut s = state.write().await;
            s.stats.record_request_received();
            s.stats.record_recipe_hit();
        }

        return CoreLoopResult {
            answer,
            source: AnswerSource::Recipe,
            recipe_id: Some(recipe.id.clone()),
            reliability: 90,
            elapsed_ms: start.elapsed().as_millis() as u64,
            internal_comms: comms,
        };
    }

    info!("No recipe found, asking specialist");

    // Step 3: Get specialist from IT Department
    let (junior_model, senior_model) = {
        let s = state.read().await;
        (
            s.config.llm.translator_model.clone(), // Junior uses lighter model
            s.config.llm.specialist_model.clone(), // Senior uses deeper model
        )
    };

    let it_dept = ITDepartment::new(&junior_model, &senior_model);
    let team = it_dept.get_team(&parsed.domain);

    // Get the junior specialist first (per VISION.md escalation flow)
    let (specialist, specialist_name) = match team {
        Some(t) => (&t.junior, ITDepartment::display_name(&t.junior)),
        None => {
            // Fallback to system team if domain not found
            let system_team = it_dept.get_team("system").unwrap();
            (&system_team.junior, ITDepartment::display_name(&system_team.junior))
        }
    };

    comms.push(InternalComm {
        from: "Anna".to_string(),
        message: format!(
            "No recipe for '{}', asking {} for help",
            parsed.intent, specialist_name
        ),
        elapsed_ms: start.elapsed().as_millis() as u64,
    });

    // Gather evidence first
    let evidence = gather_evidence(&parsed.probes).await;

    comms.push(InternalComm {
        from: specialist_name.clone(),
        message: format!("Gathered {} pieces of evidence", evidence.len()),
        elapsed_ms: start.elapsed().as_millis() as u64,
    });

    // Step 3.5: Look up knowledge sources (Arch Wiki, man pages, --help)
    let knowledge_tags = extract_knowledge_tags(&parsed);
    let knowledge = doc_fetcher::fetch_docs(&knowledge_tags, 3);

    if !knowledge.is_empty() {
        let sources: Vec<_> = knowledge.iter().map(|k| k.title.clone()).collect();
        comms.push(InternalComm {
            from: specialist_name.clone(),
            message: format!("Found {} knowledge sources: {}", knowledge.len(), sources.join(", ")),
            elapsed_ms: start.elapsed().as_millis() as u64,
        });
    }

    // Junior specialist attempts first
    let mut solution = ask_specialist(&specialist.model, query, &parsed, &evidence, &knowledge, &specialist_name).await;

    // If junior has low confidence, escalate to senior
    let mut final_specialist_name = specialist_name.clone();
    if solution.confidence < 0.7 {
        info!("Junior low confidence ({}), escalating to senior", solution.confidence);

        // Get senior specialist
        let (senior, senior_name) = match team {
            Some(t) => (&t.senior, ITDepartment::display_name(&t.senior)),
            None => {
                let system_team = it_dept.get_team("system").unwrap();
                (&system_team.senior, ITDepartment::display_name(&system_team.senior))
            }
        };

        comms.push(InternalComm {
            from: specialist_name.clone(),
            message: format!(
                "I'm not confident enough ({}%), escalating to {}",
                (solution.confidence * 100.0) as u8,
                senior_name
            ),
            elapsed_ms: start.elapsed().as_millis() as u64,
        });

        // Senior specialist takes over
        solution = ask_specialist(&senior.model, query, &parsed, &evidence, &knowledge, &senior_name).await;
        final_specialist_name = senior_name.clone();

        comms.push(InternalComm {
            from: senior_name,
            message: format!(
                "Analyzed the problem. Confidence: {}%",
                (solution.confidence * 100.0) as u8
            ),
            elapsed_ms: start.elapsed().as_millis() as u64,
        });
    }

    // If still low confidence after senior, return with warning
    if solution.confidence < 0.5 {
        warn!("Even senior has low confidence: {}", solution.confidence);

        comms.push(InternalComm {
            from: final_specialist_name.clone(),
            message: "I'm not confident in this answer, might need more investigation".to_string(),
            elapsed_ms: start.elapsed().as_millis() as u64,
        });

        return CoreLoopResult {
            answer: solution.answer,
            source: AnswerSource::Specialist {
                name: final_specialist_name,
                learned: false,
            },
            recipe_id: None,
            reliability: (solution.confidence * 100.0) as u8,
            elapsed_ms: start.elapsed().as_millis() as u64,
            internal_comms: comms,
        };
    }

    // Step 4: Learn recipe from successful solution
    // v0.0.816: Only learn recipes for NON-DYNAMIC queries
    // Dynamic queries (disk space, memory, processes) return different values each time
    // so storing the literal answer as a recipe is wrong.
    let learned = if solution.confidence >= 0.8 && !is_dynamic_query(&parsed) {
        let recipe = create_recipe_from_solution(&parsed, &evidence, &solution);
        info!("Learning new recipe: {}", recipe.id);

        comms.push(InternalComm {
            from: "Anna".to_string(),
            message: format!("Learning recipe '{}' for next time", recipe.id),
            elapsed_ms: start.elapsed().as_millis() as u64,
        });

        match library.add(recipe.clone()) {
            Ok(_) => {
                let _ = library.save(&recipe_path);
                true
            }
            Err(e) => {
                warn!("Failed to save recipe: {}", e);
                false
            }
        }
    } else {
        if is_dynamic_query(&parsed) {
            info!("Skipping recipe learning for dynamic query (domain={}, probes={:?})",
                  parsed.domain, parsed.probes);
        }
        false
    };

    // v0.0.815: Record LLM usage in global stats (this path used specialist, not recipe)
    {
        let mut s = state.write().await;
        s.stats.record_request_received();
        // Note: NOT a recipe_hit - this went to LLM specialist
    }

    CoreLoopResult {
        answer: solution.answer,
        source: AnswerSource::Specialist {
            name: final_specialist_name,
            learned,
        },
        recipe_id: None,
        reliability: (solution.confidence * 100.0) as u8,
        elapsed_ms: start.elapsed().as_millis() as u64,
        internal_comms: comms,
    }
}

/// Run probes from the route (v0.0.826)
async fn run_route_probes(probe_names: &[String]) -> Vec<ProbeResult> {
    let mut results = Vec::new();

    for probe_name in probe_names {
        // Get the actual command from probe registry
        if let Some(cmd) = probe_registry::probe_id_to_command(probe_name) {
            let start = Instant::now();
            match tokio::process::Command::new("sh")
                .arg("-c")
                .arg(cmd)
                .output()
                .await
            {
                Ok(output) => {
                    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                    results.push(ProbeResult {
                        command: probe_name.clone(),
                        stdout,
                        stderr,
                        exit_code: output.status.code().unwrap_or(-1),
                        timing_ms: start.elapsed().as_millis() as u64,
                    });
                }
                Err(e) => {
                    warn!("Probe {} failed: {}", probe_name, e);
                    results.push(ProbeResult {
                        command: probe_name.clone(),
                        stdout: String::new(),
                        stderr: e.to_string(),
                        exit_code: -1,
                        timing_ms: start.elapsed().as_millis() as u64,
                    });
                }
            }
        }
    }

    results
}

/// v0.0.829: Handle SystemUpdate deterministically
fn handle_system_update_deterministic(probe_results: &[ProbeResult]) -> Option<String> {
    // Detect package manager
    let (pkg_manager, update_cmd) = if std::path::Path::new("/usr/bin/pacman").exists() {
        ("pacman", "sudo pacman -Syu --noconfirm")
    } else if std::path::Path::new("/usr/bin/apt").exists() {
        ("apt", "sudo apt update && sudo apt upgrade -y")
    } else if std::path::Path::new("/usr/bin/dnf").exists() {
        ("dnf", "sudo dnf upgrade -y")
    } else if std::path::Path::new("/usr/bin/zypper").exists() {
        ("zypper", "sudo zypper update -y")
    } else {
        return None; // Can't determine package manager
    };

    // Count available updates from probe results
    let update_count = probe_results.iter()
        .find(|r| r.command == "package_updates" || r.command.contains("checkupdates"))
        .and_then(|r| {
            if r.exit_code == 0 && !r.stdout.trim().is_empty() {
                Some(r.stdout.lines().filter(|l| !l.trim().is_empty()).count())
            } else {
                Some(0)
            }
        });

    match update_count {
        Some(0) => Some("Your system is already up to date! No packages need updating.".to_string()),
        Some(count) => Some(format!(
            "I can update your system ({} package{} available).\n\n\
             To update, run: `{}`\n\n\
             This will download and install all available updates using {}.",
            count,
            if count == 1 { "" } else { "s" },
            update_cmd,
            pkg_manager
        )),
        None => Some(format!(
            "I can update your system.\n\n\
             To update, run: `{}`\n\n\
             This will download and install all available updates using {}.",
            update_cmd,
            pkg_manager
        )),
    }
}

/// Build runtime context from state (v0.0.826)
async fn build_runtime_context(state: &SharedState) -> RuntimeContext {
    use anna_shared::rpc::{Capabilities, HardwareSummary};
    let s = state.read().await;

    // Convert bytes to GB
    let ram_gb = s.hardware.ram_bytes as f64 / (1024.0 * 1024.0 * 1024.0);

    // Get GPU info if available
    let (gpu, gpu_vram_gb) = match &s.hardware.gpu {
        Some(g) => (
            Some(format!("{} {}", g.vendor, g.model)),
            Some(g.vram_bytes as f64 / (1024.0 * 1024.0 * 1024.0)),
        ),
        None => (None, None),
    };

    RuntimeContext {
        version: env!("CARGO_PKG_VERSION").to_string(),
        daemon_running: true,
        capabilities: Capabilities::default(),
        hardware: HardwareSummary {
            cpu_model: s.hardware.cpu_model.clone(),
            cpu_cores: s.hardware.cpu_cores,
            ram_gb,
            gpu,
            gpu_vram_gb,
            os_name: s.hardware.os_name.clone(),
            kernel: s.hardware.kernel.clone(),
            distro: s.hardware.distro.clone(),
        },
        probes: std::collections::HashMap::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_parsed_query() {
        let parsed = ParsedQuery {
            intent: "check_ram".to_string(),
            domain: "system".to_string(),
            probes: vec!["free -h".to_string()],
            entities: HashMap::new(),
        };

        assert_eq!(parsed.intent, "check_ram");
        assert_eq!(parsed.probes.len(), 1);
    }
}
