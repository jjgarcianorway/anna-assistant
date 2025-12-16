//! Core request loop - the simple path (v0.0.812).
//!
//! This module implements the VISION.md core loop:
//!
//! ```text
//! User Query → Translator → Check Recipes →
//!   If found: Execute recipe → Answer
//!   If not: Specialist solves → Anna learns → Answer
//! ```
//!
//! This is intentionally simple. No 10 special-case handlers.
//! The translator understands the query, recipes handle known patterns,
//! specialists handle new problems and teach Anna.
//!
//! v0.0.812: Added IT Department with named specialists.

use anna_shared::learning_engine::{
    AnswerKind, AnswerTemplate, LearnedRecipe, LogicType, RecipeLibrary, RecipeLogic,
    RecipeOrigin, RecipePattern, RecipeProbe,
};
use std::collections::HashMap;
use std::time::Instant;
use tracing::{info, warn};

use crate::ollama;
use crate::probes;
use crate::specialists::{ITDepartment, SpecialistRole};
use crate::state::SharedState;

/// Result of the core loop
#[derive(Debug)]
pub struct CoreLoopResult {
    pub answer: String,
    pub source: AnswerSource,
    pub recipe_id: Option<String>,
    pub reliability: u8,
    pub elapsed_ms: u64,
    pub internal_comms: Vec<InternalComm>,
}

/// Where the answer came from
#[derive(Debug, Clone, PartialEq)]
pub enum AnswerSource {
    /// Answered from a learned recipe (instant)
    Recipe,
    /// Answered by specialist (LLM), now learned
    Specialist { name: String, learned: bool },
    /// Failed to get an answer
    Failed,
}

/// Internal communication entry (fly-on-wall experience)
#[derive(Debug, Clone)]
pub struct InternalComm {
    pub from: String,
    pub message: String,
    pub elapsed_ms: u64,
}

/// The core loop - simple and clear
pub async fn handle_query(state: SharedState, query: &str) -> CoreLoopResult {
    let start = Instant::now();
    let mut comms: Vec<InternalComm> = Vec::new();

    info!("Core loop: query=\"{}\"", query);

    // Step 1: Translate query to structured form
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

        // Record success
        library.record_success(&recipe.id);
        let _ = library.save(&recipe_path);

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
        message: format!("Gathered {} pieces of evidence, analyzing...", evidence.len()),
        elapsed_ms: start.elapsed().as_millis() as u64,
    });

    // Junior specialist attempts first
    let mut solution = ask_specialist(&specialist.model, query, &parsed, &evidence, &specialist_name).await;

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
        solution = ask_specialist(&senior.model, query, &parsed, &evidence, &senior_name).await;
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
    let learned = if solution.confidence >= 0.8 {
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
        false
    };

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

/// Parsed query from translator
#[derive(Debug, Clone)]
pub struct ParsedQuery {
    pub intent: String,
    pub domain: String,
    pub probes: Vec<String>,
    pub entities: HashMap<String, String>,
}

/// Translate natural language query to structured form
async fn translate_query(model: &str, query: &str) -> Result<ParsedQuery, String> {
    let prompt = format!(
        r#"Analyze this Linux system query and extract:
1. intent: what the user wants (e.g., "check_ram", "list_services", "configure_vim")
2. domain: category (system, network, storage, services, desktop, security)
3. probes: shell commands needed (e.g., ["free -h", "cat /proc/meminfo"])

Query: "{}"

Respond ONLY with valid JSON, no other text:
{{"intent": "...", "domain": "...", "probes": ["...", "..."]}}"#,
        query
    );

    let response = ollama::chat_with_timeout(model, &prompt, 10)
        .await
        .map_err(|e| format!("Translator failed: {}", e))?;

    // Parse JSON response
    let json: serde_json::Value = serde_json::from_str(&response)
        .or_else(|_| {
            // Try to extract JSON from response
            if let Some(start) = response.find('{') {
                if let Some(end) = response.rfind('}') {
                    return serde_json::from_str(&response[start..=end]);
                }
            }
            Err(serde_json::Error::io(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "No JSON found",
            )))
        })
        .map_err(|e| format!("Failed to parse translator response: {} - Raw: {}", e, response))?;

    Ok(ParsedQuery {
        intent: json["intent"].as_str().unwrap_or("unknown").to_string(),
        domain: json["domain"].as_str().unwrap_or("system").to_string(),
        probes: json["probes"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default(),
        entities: HashMap::new(),
    })
}

/// Find a recipe that matches the parsed query
fn find_matching_recipe(library: &RecipeLibrary, parsed: &ParsedQuery) -> Option<LearnedRecipe> {
    // First try exact intent match
    let by_intent = library.by_intent(&parsed.intent);
    if !by_intent.is_empty() {
        for recipe in by_intent {
            if recipe.enabled {
                return Some(recipe.clone());
            }
        }
    }

    // Then try domain match with keyword scoring
    let by_domain = library.by_domain(&parsed.domain);
    let mut best_match: Option<(LearnedRecipe, u32)> = None;

    for recipe in by_domain {
        if !recipe.enabled {
            continue;
        }

        // Score by keyword overlap
        let query_words: std::collections::HashSet<_> = parsed
            .intent
            .split(|c: char| !c.is_alphanumeric())
            .filter(|s| !s.is_empty())
            .collect();

        let mut score = 0u32;
        for keyword in &recipe.pattern.keywords {
            if query_words.contains(keyword.as_str()) {
                score += 10;
            }
        }

        if score > 0 && (best_match.is_none() || score > best_match.as_ref().unwrap().1) {
            best_match = Some((recipe.clone(), score));
        }
    }

    best_match.map(|(r, _)| r)
}

/// Execute a recipe and return the answer
async fn execute_recipe(recipe: &LearnedRecipe, parsed: &ParsedQuery) -> String {
    let mut values: HashMap<String, String> = HashMap::new();

    for probe in &recipe.probes {
        match probes::run_command(&probe.tool) {
            Ok(output) => {
                values.insert(probe.id.clone(), output.trim().to_string());
            }
            Err(e) => {
                warn!("Probe {} failed: {}", probe.id, e);
                values.insert(probe.id.clone(), format!("(error: {})", e));
            }
        }
    }

    for (k, v) in &parsed.entities {
        values.insert(k.clone(), v.clone());
    }

    recipe.answer_template.render_detailed(&values)
}

/// Gather evidence by running probes
async fn gather_evidence(probe_cmds: &[String]) -> HashMap<String, String> {
    let mut evidence = HashMap::new();

    for probe in probe_cmds {
        match probes::run_command(probe) {
            Ok(output) => {
                evidence.insert(probe.clone(), output);
            }
            Err(e) => {
                warn!("Evidence probe failed: {} - {}", probe, e);
            }
        }
    }

    evidence
}

/// Solution from specialist
#[derive(Debug)]
struct SpecialistSolution {
    answer: String,
    confidence: f32,
    explanation: String,
}

/// Ask specialist to solve the problem
async fn ask_specialist(
    model: &str,
    query: &str,
    parsed: &ParsedQuery,
    evidence: &HashMap<String, String>,
    specialist_name: &str,
) -> SpecialistSolution {
    let evidence_str = evidence
        .iter()
        .map(|(k, v)| format!("## {}\n```\n{}\n```", k, v))
        .collect::<Vec<_>>()
        .join("\n\n");

    let prompt = format!(
        r#"You are {}, a Linux {} specialist. Answer this query using the evidence provided.

Query: "{}"

Evidence:
{}

Provide a clear, direct answer to the user's question.
Then respond with JSON containing your answer and confidence:
{{"answer": "your answer here", "confidence": 0.9, "explanation": "brief reasoning"}}"#,
        specialist_name, parsed.domain, query, evidence_str
    );

    let response = match ollama::chat_with_timeout(model, &prompt, 30).await {
        Ok(r) => r,
        Err(e) => {
            return SpecialistSolution {
                answer: format!("Specialist error: {}", e),
                confidence: 0.0,
                explanation: "Failed to get specialist response".to_string(),
            };
        }
    };

    // Try to parse JSON from response
    let json: serde_json::Value = serde_json::from_str(&response)
        .or_else(|_| {
            if let Some(start) = response.find('{') {
                if let Some(end) = response.rfind('}') {
                    return serde_json::from_str(&response[start..=end]);
                }
            }
            Err(serde_json::Error::io(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "No JSON found",
            )))
        })
        .unwrap_or_else(|_| {
            // If JSON parsing fails, use the response as-is
            serde_json::json!({
                "answer": response,
                "confidence": 0.6,
                "explanation": "Raw response (JSON parsing failed)"
            })
        });

    SpecialistSolution {
        answer: json["answer"].as_str().unwrap_or(&response).to_string(),
        confidence: json["confidence"].as_f64().unwrap_or(0.6) as f32,
        explanation: json["explanation"].as_str().unwrap_or("").to_string(),
    }
}

/// Create a recipe from a successful specialist solution
fn create_recipe_from_solution(
    parsed: &ParsedQuery,
    _evidence: &HashMap<String, String>,
    solution: &SpecialistSolution,
) -> LearnedRecipe {
    let id = format!("{}-{}", parsed.intent, chrono::Utc::now().timestamp());

    let mut recipe = LearnedRecipe::new(&id, &parsed.domain);

    // Set pattern from intent keywords
    let keywords: Vec<&str> = parsed
        .intent
        .split('_')
        .filter(|s| !s.is_empty())
        .collect();
    recipe.pattern = RecipePattern::new(&parsed.intent).with_keywords(&keywords);

    // Set probes from what we used
    for probe_cmd in &parsed.probes {
        recipe
            .probes
            .push(RecipeProbe::new(probe_cmd, probe_cmd));
    }

    // Set answer template
    recipe.answer_template = AnswerTemplate::new(
        &solution.answer,
        &format!("{}\n\n(Learned from specialist)", solution.answer),
    );

    // Set logic
    recipe.logic = RecipeLogic {
        logic_type: LogicType::Template,
        answer_kind: AnswerKind::Diagnostic,
        steps: vec![solution.explanation.clone()],
        conditionals: HashMap::new(),
    };

    // Set origin
    recipe.origin =
        RecipeOrigin::learned("specialist", &format!("{} Specialist", parsed.domain));

    recipe
}

/// Capitalize first letter
fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn test_capitalize() {
        assert_eq!(capitalize("system"), "System");
        assert_eq!(capitalize("network"), "Network");
        assert_eq!(capitalize(""), "");
    }
}
