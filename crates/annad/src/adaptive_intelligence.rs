//! Adaptive Intelligence - Anna learns, adapts, and never gives up.
//! Pure intelligence: try multiple strategies until one succeeds.

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

/// A strategy for solving a problem.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Strategy {
    pub name: String,
    pub description: String,
    pub commands: Vec<String>,
    pub confidence: f32,
    pub complexity: u8, // 1-10, start with low complexity
}

/// Result of attempting a strategy.
#[derive(Debug, Clone)]
pub struct StrategyResult {
    pub strategy: Strategy,
    pub success: bool,
    pub output: String,
    pub error: Option<String>,
    pub duration_ms: u64,
}

/// Generate multiple strategies for a task, ordered by simplicity.
pub async fn generate_strategies(model: &str, task: &str) -> Result<Vec<Strategy>> {
    info!("Generating adaptive strategies for: {}", task);

    let prompt = format!(
        r#"Generate 3-5 different strategies to accomplish this task, ordered from simplest to most complex.

TASK: "{}"

RULES:
1. Start with simplest approach first
2. Each strategy should be DIFFERENT (different tools, different approach)
3. Order by: simple → moderate → complex → creative
4. Be specific - actual commands, not generic descriptions
5. Think creatively - combine tools in novel ways

For each strategy, provide:
- Name (2-4 words)
- Description (one sentence)
- Commands (actual shell commands to run)
- Confidence (0.0-1.0 - how likely this will work)
- Complexity (1-10)

Respond in JSON array:
[
  {{
    "name": "Strategy name",
    "description": "What this does",
    "commands": ["cmd1", "cmd2", "cmd3"],
    "confidence": 0.8,
    "complexity": 3
  }},
  ...
]

Think step by step. Be creative. Try different tools. Combine approaches.
"#,
        task
    );

    let response = crate::ollama::chat_with_timeout(model, &prompt, 90).await?;
    let json_str = extract_json(&response)?;

    let strategies: Vec<Strategy> = serde_json::from_str(&json_str)
        .map_err(|e| anyhow!("Failed to parse strategies: {}", e))?;

    // Sort by complexity (simplest first)
    let mut sorted = strategies;
    sorted.sort_by_key(|s| s.complexity);

    info!("Generated {} strategies", sorted.len());
    for (i, strategy) in sorted.iter().enumerate() {
        debug!(
            "Strategy {}: {} (complexity {}, confidence {:.0}%)",
            i + 1,
            strategy.name,
            strategy.complexity,
            strategy.confidence * 100.0
        );
    }

    Ok(sorted)
}

/// Try strategies progressively until one succeeds.
pub async fn execute_with_adaptation(
    model: &str,
    task: &str,
    strategies: Vec<Strategy>,
) -> Result<StrategyResult> {
    info!("Executing strategies with adaptation");

    for (i, strategy) in strategies.iter().enumerate() {
        info!(
            "Attempting strategy {} of {}: {}",
            i + 1,
            strategies.len(),
            strategy.name
        );

        let start = std::time::Instant::now();
        let mut output = String::new();
        let mut had_error = false;
        let mut error_msg = None;

        // Execute each command in the strategy
        for (cmd_idx, cmd) in strategy.commands.iter().enumerate() {
            debug!("Executing command {} of {}: {}", cmd_idx + 1, strategy.commands.len(), cmd);

            match crate::core_loop::execute_command(cmd) {
                Ok(cmd_output) => {
                    let clean = crate::core_loop::strip_ansi_codes(&cmd_output);
                    output.push_str(&clean);
                    output.push('\n');
                }
                Err(e) => {
                    warn!("Command failed: {}", e);
                    had_error = true;
                    error_msg = Some(e.to_string());

                    // Try to analyze failure and adapt
                    if let Some(adapted_cmd) = analyze_and_adapt(model, cmd, &e.to_string()).await {
                        info!("Adapted command, retrying");
                        match crate::core_loop::execute_command(&adapted_cmd) {
                            Ok(adapted_output) => {
                                let clean = crate::core_loop::strip_ansi_codes(&adapted_output);
                                output.push_str(&clean);
                                output.push('\n');
                                had_error = false;
                                error_msg = None;
                            }
                            Err(e2) => {
                                warn!("Adapted command also failed: {}", e2);
                                // Continue to next strategy
                            }
                        }
                    }

                    if had_error {
                        break; // Skip rest of commands in this strategy
                    }
                }
            }
        }

        let duration_ms = start.elapsed().as_millis() as u64;

        if !had_error && !output.trim().is_empty() {
            info!("Strategy '{}' succeeded in {}ms", strategy.name, duration_ms);
            return Ok(StrategyResult {
                strategy: strategy.clone(),
                success: true,
                output,
                error: None,
                duration_ms,
            });
        } else {
            info!(
                "Strategy '{}' failed after {}ms: {:?}",
                strategy.name, duration_ms, error_msg
            );

            // Don't give up yet - try next strategy
            if i + 1 < strategies.len() {
                info!("Trying next strategy...");
            }
        }
    }

    // All strategies exhausted
    Err(anyhow!("All {} strategies exhausted without success", strategies.len()))
}

/// Analyze a command failure and suggest an adapted version.
async fn analyze_and_adapt(model: &str, failed_cmd: &str, error: &str) -> Option<String> {
    debug!("Analyzing failure to adapt command");

    let prompt = format!(
        r#"This command failed. Suggest ONE alternative command that might work.

FAILED COMMAND: {}
ERROR: {}

Analyze the error and suggest a different approach:
- If tool missing: suggest installing it first, or use alternative tool
- If permission denied: add sudo or change approach
- If path not found: try common alternative paths
- If syntax error: fix the syntax

Respond with ONLY the adapted command, nothing else. One line.
If no good alternative exists, respond with: NONE
"#,
        failed_cmd, error
    );

    match crate::ollama::chat_with_timeout(model, &prompt, 30).await {
        Ok(response) => {
            let adapted = response.trim();
            if adapted == "NONE" || adapted.is_empty() {
                None
            } else {
                debug!("Adapted command: {}", adapted);
                Some(adapted.to_string())
            }
        }
        Err(e) => {
            warn!("Failed to generate adapted command: {}", e);
            None
        }
    }
}

/// Discover tools that could solve a problem.
pub async fn discover_tools_for_task(model: &str, task: &str) -> Result<Vec<String>> {
    info!("Discovering tools for task: {}", task);

    let prompt = format!(
        r#"What Linux tools/commands could accomplish this task?

TASK: "{}"

List 3-5 specific tools/commands that could help, ordered by how well-suited they are.
Include both common and specialized tools.

Think creatively:
- Standard tools (find, grep, awk, sed, etc.)
- Specialized tools (fd, rg, jq, etc.)
- Scripts/combinations
- Novel approaches

Respond in JSON array of tool names:
["tool1", "tool2", "tool3", ...]

Only tool names, no descriptions.
"#,
        task
    );

    let response = crate::ollama::chat_with_timeout(model, &prompt, 60).await?;
    let json_str = extract_json(&response)?;

    let tools: Vec<String> = serde_json::from_str(&json_str)
        .map_err(|e| anyhow!("Failed to parse tool list: {}", e))?;

    info!("Discovered {} potential tools", tools.len());
    Ok(tools)
}

/// Check which discovered tools are available (installed).
pub fn check_tools_available(tools: &[String]) -> Vec<String> {
    let mut available = Vec::new();

    for tool in tools {
        let check = format!("command -v {} 2>/dev/null", tool);
        if crate::core_loop::execute_command(&check).is_ok() {
            available.push(tool.clone());
        }
    }

    info!("{} of {} tools available", available.len(), tools.len());
    available
}

/// Install missing tools that could help.
pub async fn install_helpful_tools(tools: &[String]) -> Vec<String> {
    let mut installed = Vec::new();

    for tool in tools {
        // Check if already available
        let check = format!("command -v {} 2>/dev/null", tool);
        if crate::core_loop::execute_command(&check).is_ok() {
            continue;
        }

        // Try to install
        info!("Installing potentially helpful tool: {}", tool);
        match crate::universal_handler::acquire_tool(tool).await {
            Ok(_) => {
                installed.push(tool.clone());
            }
            Err(e) => {
                debug!("Could not install {}: {}", tool, e);
            }
        }
    }

    info!("Installed {} helpful tools", installed.len());
    installed
}

/// Creative synthesis: combine multiple approaches.
pub async fn synthesize_creative_solution(
    model: &str,
    task: &str,
    available_tools: &[String],
) -> Result<Strategy> {
    info!("Synthesizing creative solution");

    let prompt = format!(
        r#"Create a CREATIVE solution to this task by combining tools in novel ways.

TASK: "{}"
AVAILABLE TOOLS: {}

Think outside the box:
- Pipe commands together creatively
- Use tools in unexpected ways
- Combine multiple approaches
- Be clever and efficient

Generate ONE creative strategy:
{{
  "name": "Creative strategy name",
  "description": "What makes this approach clever",
  "commands": ["cmd1 | cmd2", "cmd3 && cmd4", ...],
  "confidence": 0.7,
  "complexity": 7
}}

Be bold. Be creative. Make it work.
"#,
        task,
        available_tools.join(", ")
    );

    let response = crate::ollama::chat_with_timeout(model, &prompt, 60).await?;
    let json_str = extract_json(&response)?;

    let strategy: Strategy = serde_json::from_str(&json_str)
        .map_err(|e| anyhow!("Failed to parse creative strategy: {}", e))?;

    info!("Creative strategy: {}", strategy.name);
    Ok(strategy)
}

/// Full adaptive problem solving workflow.
pub async fn solve_adaptively(model: &str, task: &str) -> Result<String> {
    info!("🧠 ADAPTIVE INTELLIGENCE ACTIVATED for: {}", task);

    // Phase 1: Generate multiple standard strategies
    let mut strategies = generate_strategies(model, task).await?;

    // Phase 2: Discover potentially helpful tools
    let discovered_tools = discover_tools_for_task(model, task).await?;
    let available_tools = check_tools_available(&discovered_tools);

    // Phase 3: Install missing tools that could help (first 2-3)
    let to_install: Vec<String> = discovered_tools
        .iter()
        .filter(|t| !available_tools.contains(t))
        .take(3)
        .cloned()
        .collect();

    if !to_install.is_empty() {
        info!("Installing {} potentially helpful tools", to_install.len());
        install_helpful_tools(&to_install).await;
    }

    // Phase 4: Add a creative synthesis strategy at the end
    let updated_available: Vec<String> = check_tools_available(&discovered_tools);
    if let Ok(creative_strategy) = synthesize_creative_solution(model, task, &updated_available).await {
        strategies.push(creative_strategy);
    }

    // Phase 5: Execute strategies with adaptation
    match execute_with_adaptation(model, task, strategies).await {
        Ok(result) => {
            info!("✓ SOLVED with strategy: {}", result.strategy.name);
            Ok(format!(
                "Solved using '{}' strategy:\n{}",
                result.strategy.name, result.output
            ))
        }
        Err(e) => {
            warn!("All strategies exhausted: {}", e);
            Err(e)
        }
    }
}

/// Extract JSON from LLM response.
fn extract_json(response: &str) -> Result<String> {
    // Try code block first
    if let Some(start) = response.find("```json") {
        if let Some(end) = response[start..].find("```") {
            let json_start = start + 7;
            let json_end = start + end;
            return Ok(response[json_start..json_end].trim().to_string());
        }
    }

    // Try array/object directly
    if let Some(start) = response.find('[') {
        if let Some(end) = response.rfind(']') {
            if end > start {
                return Ok(response[start..=end].to_string());
            }
        }
    }

    if let Some(start) = response.find('{') {
        if let Some(end) = response.rfind('}') {
            if end > start {
                return Ok(response[start..=end].to_string());
            }
        }
    }

    Err(anyhow!("No valid JSON found in response"))
}
