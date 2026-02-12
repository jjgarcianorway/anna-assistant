//! Universal Task Handler - No hardcoding, pure LLM intelligence.
//!
//! Philosophy: Anna can do ANYTHING. No exceptions.
//!
//! Instead of pattern matching or routing, we use pure reasoning:
//! 1. Understand what the user wants (intent analysis)
//! 2. Determine what capabilities are needed
//! 3. Acquire missing capabilities (install tools, research)
//! 4. Plan execution strategy
//! 5. Execute with monitoring
//! 6. Learn from outcome
//! 7. Improve for next time
//!
//! No limits. No "this is not supported". If it's possible on Linux, Anna can do it.

use crate::ollama;
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use tracing::{debug, info};

/// Result of analyzing what capabilities are needed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityAnalysis {
    /// What the user wants to accomplish
    pub goal: String,
    /// What tools/packages Anna needs
    pub required_tools: Vec<String>,
    /// What knowledge Anna needs (topics to research)
    pub required_knowledge: Vec<String>,
    /// What system changes might be needed
    pub system_changes: Vec<String>,
    /// Complexity estimate (1-10)
    pub complexity: u8,
    /// Can this be done? (Should always be true unless physically impossible)
    pub feasible: bool,
    /// Why not feasible (only if feasible=false)
    pub infeasibility_reason: Option<String>,
}

/// Analyze what capabilities are needed for a task
pub async fn analyze_capabilities(model: &str, request: &str) -> Result<CapabilityAnalysis> {
    let prompt = format!(
        r#"Analyze this user request and determine what capabilities are needed.

USER REQUEST: "{}"

Think like a systems administrator. What tools, knowledge, and system changes would you need?

Rules:
1. Almost everything is feasible on Linux - be creative
2. If you need a tool you don't have, you can install it
3. If you need knowledge, you can research it
4. Only mark infeasible if physically impossible (e.g., "make it rain")

Respond in JSON:
{{
    "goal": "clear one-sentence goal",
    "required_tools": ["tool1", "tool2"],
    "required_knowledge": ["topic1", "topic2"],
    "system_changes": ["change1", "change2"],
    "complexity": 5,
    "feasible": true,
    "infeasibility_reason": null
}}
"#,
        request
    );

    let response = crate::ollama::chat_with_timeout(model, &prompt, 60).await?;
    let json_str = extract_json(&response)?;
    let analysis: CapabilityAnalysis = serde_json::from_str(&json_str)
        .map_err(|e| anyhow!("Failed to parse capability analysis: {}", e))?;

    info!(
        "Capability analysis: {} tools, {} knowledge areas, complexity {}",
        analysis.required_tools.len(),
        analysis.required_knowledge.len(),
        analysis.complexity
    );

    Ok(analysis)
}

/// Acquire a missing tool by installing it
pub async fn acquire_tool(tool: &str) -> Result<String> {
    info!("Acquiring tool: {}", tool);

    // Check if already installed
    let check_cmd = format!("which {} 2>/dev/null || command -v {} 2>/dev/null", tool, tool);
    if let Ok(output) = crate::core_loop::execute_command(&check_cmd) {
        if !output.trim().is_empty() {
            debug!("Tool {} already installed at {}", tool, output.trim());
            return Ok(format!("Tool {} already available", tool));
        }
    }

    // Try to install with pacman (Arch)
    let install_cmd = format!("sudo pacman -S --noconfirm {}", tool);
    match crate::core_loop::execute_command(&install_cmd) {
        Ok(output) => {
            info!("Successfully installed {}", tool);
            Ok(format!("Installed {} via pacman", tool))
        }
        Err(e) => {
            debug!("Pacman install failed: {}, trying yay", e);
            // Try AUR with yay
            let yay_cmd = format!("yay -S --noconfirm {}", tool);
            match crate::core_loop::execute_command(&yay_cmd) {
                Ok(_) => {
                    info!("Successfully installed {} from AUR", tool);
                    Ok(format!("Installed {} from AUR", tool))
                }
                Err(e) => Err(anyhow!("Could not install {}: {}", tool, e)),
            }
        }
    }
}

/// Research a topic to gain knowledge
pub async fn research_topic(model: &str, topic: &str) -> Result<String> {
    info!("Researching topic: {}", topic);

    let prompt = format!(
        r#"You are researching: {}

Provide a concise but complete explanation covering:
1. What it is
2. How it works on Linux
3. Common commands/tools
4. Best practices
5. Common pitfalls

Be technical and practical. Focus on what a sysadmin needs to know.
"#,
        topic
    );

    let response = crate::ollama::chat_with_timeout(model, &prompt, 60).await?;
    debug!("Research complete for {}, {} chars", topic, response.len());
    Ok(response)
}

/// Generate an execution plan for a task
pub async fn generate_execution_plan(
    model: &str,
    request: &str,
    analysis: &CapabilityAnalysis,
    acquired_knowledge: &[String],
) -> Result<Vec<String>> {
    let knowledge_context = if acquired_knowledge.is_empty() {
        String::new()
    } else {
        format!(
            "\n\nRELEVANT KNOWLEDGE:\n{}",
            acquired_knowledge.join("\n\n")
        )
    };

    let prompt = format!(
        r#"Create a step-by-step execution plan for this request.

USER REQUEST: "{}"

GOAL: {}
AVAILABLE TOOLS: {}
SYSTEM CHANGES NEEDED: {}
COMPLEXITY: {}/10
{}

Create a numbered list of exact commands to execute. Each command should be:
- Complete and executable (no placeholders like "your_file")
- Safe (ask for confirmation before destructive operations)
- Specific to this user's system
- Properly sequenced (dependencies first)

If you need to make assumptions about paths/files, use the most common defaults.
If something is ambiguous, plan to investigate first then act.

Return ONLY the commands, one per line, numbered. No explanations between commands.
Commands only.
"#,
        request,
        analysis.goal,
        analysis.required_tools.join(", "),
        analysis.system_changes.join(", "),
        analysis.complexity,
        knowledge_context
    );

    let response = crate::ollama::chat_with_timeout(model, &prompt, 60).await?;

    // Parse numbered commands
    let commands: Vec<String> = response
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            // Match lines like "1. command", "1) command", or "Step 1: command"
            if let Some(cmd) = line.strip_prefix(|c: char| c.is_numeric()) {
                let cmd = cmd.trim_start_matches('.')
                    .trim_start_matches(')')
                    .trim_start_matches(':')
                    .trim();
                if !cmd.is_empty() && !cmd.starts_with('#') {
                    Some(cmd.to_string())
                } else {
                    None
                }
            } else if line.starts_with("- ") {
                Some(line[2..].to_string())
            } else {
                None
            }
        })
        .collect();

    if commands.is_empty() {
        return Err(anyhow!("No valid commands in execution plan"));
    }

    info!("Generated execution plan with {} steps", commands.len());
    Ok(commands)
}

/// Execute a universal task with full intelligence
pub async fn handle_universal_task(model: &str, request: &str) -> Result<String> {
    info!("Universal handler: {}", request);

    // Step 1: Analyze what capabilities are needed
    let analysis = analyze_capabilities(model, request).await?;

    if !analysis.feasible {
        return Ok(format!(
            "I understand you want to {}, but this isn't feasible: {}",
            analysis.goal,
            analysis
                .infeasibility_reason
                .as_deref()
                .unwrap_or("physical limitations")
        ));
    }

    // Step 2: Acquire missing tools
    let mut tool_results = Vec::new();
    for tool in &analysis.required_tools {
        match acquire_tool(tool).await {
            Ok(msg) => {
                debug!("Tool acquisition: {}", msg);
                tool_results.push(msg);
            }
            Err(e) => {
                debug!("Could not acquire {}: {}", tool, e);
                // Continue anyway - might not be critical
            }
        }
    }

    // Step 3: Research required knowledge
    let mut knowledge = Vec::new();
    for topic in &analysis.required_knowledge {
        match research_topic(model, topic).await {
            Ok(info) => knowledge.push(info),
            Err(e) => debug!("Research failed for {}: {}", topic, e),
        }
    }

    // Step 4: Generate execution plan
    let commands = generate_execution_plan(model, request, &analysis, &knowledge).await?;

    // Step 5: Execute with monitoring
    let mut results = Vec::new();
    results.push(format!("Goal: {}", analysis.goal));

    if !tool_results.is_empty() {
        results.push(format!("\nTools acquired: {}", tool_results.join(", ")));
    }

    results.push(format!("\nExecution plan ({} steps):", commands.len()));

    for (i, cmd) in commands.iter().enumerate() {
        results.push(format!("\nStep {}: {}", i + 1, cmd));

        match crate::core_loop::execute_command(cmd) {
            Ok(output) => {
                let output = crate::core_loop::strip_ansi_codes(&output);
                let output = output.trim();
                if !output.is_empty() {
                    results.push(format!("Output: {}", output));
                } else {
                    results.push("✓ Success".to_string());
                }
            }
            Err(e) => {
                let error_msg = format!("✗ Error: {}", e);
                results.push(error_msg);

                // Try root cause analysis
                if let Some(rca) = crate::intelligence::analyze_failure(&e.to_string(), &[]) {
                    results.push(format!("\nRoot cause: {}", rca));
                }

                // Should we continue or stop?
                // For now, continue - the LLM might have fallback steps
                debug!("Command failed but continuing: {}", e);
            }
        }
    }

    // Step 6: Learn from outcome
    let outcome = results.join("\n");
    crate::intelligence::record_success(request, &outcome, &commands, 0.9);

    Ok(outcome)
}

/// Extract JSON from LLM response (handles markdown code blocks)
fn extract_json(response: &str) -> Result<String> {
    // Try to find JSON in code blocks first
    if let Some(start) = response.find("```json") {
        if let Some(end) = response[start..].find("```") {
            let json_start = start + 7; // len("```json")
            let json_end = start + end;
            return Ok(response[json_start..json_end].trim().to_string());
        }
    }

    // Try to find JSON directly
    if let Some(start) = response.find('{') {
        if let Some(end) = response.rfind('}') {
            if end > start {
                return Ok(response[start..=end].to_string());
            }
        }
    }

    Err(anyhow!("No valid JSON found in response"))
}
