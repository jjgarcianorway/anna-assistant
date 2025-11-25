//! Interpreter Core - LLM-driven output interpretation
//!
//! v6.41.0: The Interpreter receives command execution results and uses
//! the LLM to generate human-readable answers.
//!
//! v6.42.0: Real LLM integration with JSON schemas and fallback interpretation.

use crate::executor_core::ExecutionResult;
use crate::llm_client::{LlmClient, LlmError};
use crate::planner_core::Intent;
use serde::{Deserialize, Serialize};

/// Final interpreted answer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterpretedAnswer {
    /// Direct answer to the user's question
    pub answer: String,

    /// Optional detailed summary
    pub details: Option<String>,

    /// Confidence level
    pub confidence: ConfidenceLevel,

    /// LLM reasoning (for trace)
    pub reasoning: String,

    /// Source attribution
    pub source: String,

    /// v6.42.0: Was the goal achieved based on command output?
    pub achieved_goal: bool,

    /// v6.42.0: Validation confidence (0.0-1.0)
    pub validation_confidence: f64,

    /// v6.42.0: Optional follow-up suggestions
    pub followup_suggestions: Vec<String>,

    /// v6.42.0: Short one-line summary
    pub short_summary: Option<String>,
}

/// Confidence level in the answer
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ConfidenceLevel {
    /// High confidence - all commands succeeded
    High,

    /// Medium confidence - some commands failed but we have partial data
    Medium,

    /// Low confidence - most commands failed or data is ambiguous
    Low,
}

/// Build LLM prompt for interpreting execution results
pub fn build_interpreter_prompt(
    intent: &Intent,
    exec_result: &ExecutionResult,
    telemetry_summary: &str,
) -> String {
    let mut prompt = format!(
        "# Output Interpretation Task\n\n\
        Original user query: \"{}\"\n\n",
        intent.query
    );

    prompt.push_str("## Command Execution Results\n\n");

    if exec_result.command_results.is_empty() {
        prompt.push_str("No commands were executed.\n");
    } else {
        for (i, cmd_result) in exec_result.command_results.iter().enumerate() {
            prompt.push_str(&format!(
                "### Command {}: {}\n",
                i + 1,
                cmd_result.full_command
            ));

            prompt.push_str(&format!("Exit code: {}\n", cmd_result.exit_code));

            if !cmd_result.stdout.is_empty() {
                prompt.push_str(&format!(
                    "Output:\n```\n{}\n```\n",
                    cmd_result.stdout.lines().take(50).collect::<Vec<_>>().join("\n")
                ));
            }

            if !cmd_result.stderr.is_empty() && !cmd_result.success {
                prompt.push_str(&format!(
                    "Errors:\n```\n{}\n```\n",
                    cmd_result.stderr.lines().take(20).collect::<Vec<_>>().join("\n")
                ));
            }

            prompt.push('\n');
        }
    }

    prompt.push_str(&format!(
        "\n## System Context\n{}\n\n",
        telemetry_summary
    ));

    prompt.push_str(
        "## Your Task\n\
        Analyze the command outputs and provide a clear, concise answer to the user's question.\n\
        \n\
        Requirements:\n\
        1. Answer directly - start with YES/NO if applicable\n\
        2. Be specific - include actual package names, features, numbers\n\
        3. If commands failed, explain what went wrong honestly\n\
        4. Use system context to enhance your answer\n\
        5. No emojis, no markdown fences in the final answer\n\
        \n\
        Return your interpretation as JSON:\n\
        {\n\
          \"answer\": \"Direct answer text (3-5 sentences max)\",\n\
          \"details\": \"Optional detailed explanation\",\n\
          \"confidence\": \"High\" | \"Medium\" | \"Low\",\n\
          \"reasoning\": \"Brief explanation of how you arrived at this answer\",\n\
          \"source\": \"Command outputs + system telemetry\"\n\
        }\n"
    );

    prompt
}

/// Parse confidence level from string
pub fn parse_confidence(s: &str) -> ConfidenceLevel {
    match s.to_lowercase().as_str() {
        "high" => ConfidenceLevel::High,
        "medium" => ConfidenceLevel::Medium,
        "low" => ConfidenceLevel::Low,
        _ => ConfidenceLevel::Medium, // Default
    }
}

/// Fallback interpreter when LLM is not available
pub fn interpret_without_llm(
    intent: &Intent,
    exec_result: &ExecutionResult,
) -> InterpretedAnswer {
    if !exec_result.success {
        // All commands failed
        let mut error_msg = format!(
            "I tried to answer your question but encountered errors:\n\n"
        );

        for cmd_result in &exec_result.command_results {
            if !cmd_result.success {
                error_msg.push_str(&format!(
                    "  • {}: {}\n",
                    cmd_result.full_command,
                    if cmd_result.stderr.is_empty() {
                        "command failed"
                    } else {
                        cmd_result.stderr.lines().next().unwrap_or("unknown error")
                    }
                ));
            }
        }

        error_msg.push_str("\nPlease ensure required tools are installed.");

        return InterpretedAnswer {
            answer: error_msg,
            details: None,
            confidence: ConfidenceLevel::Low,
            reasoning: "All commands failed".to_string(),
            source: "Execution failure".to_string(),
            achieved_goal: false,
            validation_confidence: 0.0,
            followup_suggestions: vec![],
            short_summary: None,
        };
    }

    // Simple interpretation based on domain
    let answer = match intent.domain {
        crate::planner_core::DomainType::Packages => {
            interpret_package_results(exec_result)
        }
        crate::planner_core::DomainType::Hardware => {
            interpret_hardware_results(exec_result)
        }
        crate::planner_core::DomainType::Gui => {
            interpret_gui_results(exec_result)
        }
        _ => "Results received. See command outputs for details.".to_string(),
    };

    InterpretedAnswer {
        answer,
        details: None,
        confidence: ConfidenceLevel::Medium,
        reasoning: "Fallback interpretation without LLM".to_string(),
        source: "Command execution".to_string(),
        achieved_goal: true, // Assume success if we got results
        validation_confidence: 0.75, // Medium confidence
        followup_suggestions: vec![],
        short_summary: None,
    }
}

fn interpret_package_results(exec_result: &ExecutionResult) -> String {
    let mut packages = Vec::new();

    for cmd_result in &exec_result.command_results {
        if cmd_result.success && !cmd_result.stdout.is_empty() {
            // Extract package names from output
            for line in cmd_result.stdout.lines() {
                if let Some(pkg_name) = line.split_whitespace().next() {
                    if !pkg_name.is_empty() && !packages.contains(&pkg_name.to_string()) {
                        packages.push(pkg_name.to_string());
                    }
                }
            }
        }
    }

    if packages.is_empty() {
        "No matching packages found.".to_string()
    } else {
        format!(
            "Found {} package(s): {}",
            packages.len(),
            packages.join(", ")
        )
    }
}

fn interpret_hardware_results(exec_result: &ExecutionResult) -> String {
    for cmd_result in &exec_result.command_results {
        if cmd_result.success && !cmd_result.stdout.is_empty() {
            // Check if this is CPU flags output
            if cmd_result.stdout.to_lowercase().contains("flags") {
                // Parse CPU flags
                for line in cmd_result.stdout.lines() {
                    if line.to_lowercase().contains("flags") && line.contains(":") {
                        // Extract flags from "Flags: sse sse2 ..." format
                        if let Some(flags_part) = line.split(':').nth(1) {
                            let flags: Vec<&str> = flags_part
                                .split_whitespace()
                                .filter(|f| !f.is_empty())
                                .collect();

                            if flags.is_empty() {
                                return "No CPU flags detected.".to_string();
                            }

                            // Group related flags
                            let sse_flags: Vec<&str> = flags
                                .iter()
                                .filter(|f| f.to_lowercase().starts_with("sse"))
                                .copied()
                                .collect();

                            let avx_flags: Vec<&str> = flags
                                .iter()
                                .filter(|f| f.to_lowercase().starts_with("avx"))
                                .copied()
                                .collect();

                            let mut result = String::new();

                            if !sse_flags.is_empty() {
                                result.push_str(&format!("SSE support: {}\n", sse_flags.join(", ")));
                            }

                            if !avx_flags.is_empty() {
                                result.push_str(&format!("AVX support: {}\n", avx_flags.join(", ")));
                            }

                            if result.is_empty() {
                                // Show sample of available flags
                                let sample: Vec<&str> = flags.iter().take(10).copied().collect();
                                result.push_str(&format!(
                                    "CPU flags available (showing first 10): {}\n",
                                    sample.join(", ")
                                ));
                            }

                            result.push_str(&format!("\nTotal flags: {}", flags.len()));
                            return result;
                        }
                    }
                }
            }
        }
    }

    // Fallback: show first few lines of output
    let mut info_lines = Vec::new();
    for cmd_result in &exec_result.command_results {
        if cmd_result.success && !cmd_result.stdout.is_empty() {
            for line in cmd_result.stdout.lines().take(5) {
                if !line.trim().is_empty() {
                    info_lines.push(line.trim().to_string());
                }
            }
        }
    }

    if info_lines.is_empty() {
        "No hardware information retrieved.".to_string()
    } else {
        info_lines.join("\n")
    }
}

fn interpret_gui_results(exec_result: &ExecutionResult) -> String {
    // Check if this is a DE_WM_DETECTOR special command
    for cmd_result in &exec_result.command_results {
        if cmd_result.stdout.contains("DE_WM_DETECTOR") {
            // Use the actual de_wm_detector module
            let detection = crate::de_wm_detector::detect_de_wm();

            if detection.name == "Unable to detect" || detection.name == "Unknown" {
                return format!(
                    "Could not reliably detect your desktop environment.\n\n\
                    I checked:\n\
                    - Environment variables (XDG_CURRENT_DESKTOP, DESKTOP_SESSION)\n\
                    - Running processes (sway, i3, hyprland, gnome, kde, etc.)\n\
                    - Installed packages\n\
                    - Configuration directories\n\
                    - X11 properties\n\n\
                    You may be in a TTY, SSH session, or using a minimal/custom window manager."
                );
            }

            let de_type_str = match detection.de_type {
                crate::de_wm_detector::DeType::DesktopEnvironment => "Desktop Environment",
                crate::de_wm_detector::DeType::WindowManager => "Window Manager",
                crate::de_wm_detector::DeType::Compositor => "Compositor",
            };

            let confidence_str = match detection.confidence {
                crate::de_wm_detector::Confidence::High => "high",
                crate::de_wm_detector::Confidence::Medium => "medium",
                crate::de_wm_detector::Confidence::Low => "low",
            };

            return format!(
                "You are running: {} ({})\n\
                Detection confidence: {} (via {})",
                detection.name,
                de_type_str,
                confidence_str,
                detection.detection_method
            );
        }
    }

    // Fallback to old method if not using detector
    let mut de_wm_info = Vec::new();
    for cmd_result in &exec_result.command_results {
        if cmd_result.success && !cmd_result.stdout.is_empty() {
            for line in cmd_result.stdout.lines() {
                if !line.trim().is_empty() && !line.contains("DE_WM_DETECTOR") {
                    de_wm_info.push(line.trim().to_string());
                }
            }
        }
    }

    if de_wm_info.is_empty() {
        "Could not detect desktop environment or window manager.".to_string()
    } else {
        format!("Detected: {}", de_wm_info.join(", "))
    }
}

/// v6.42.0: LLM-backed interpreter that uses real LLM for result interpretation
pub struct LlmInterpreter<'a> {
    llm_client: &'a dyn LlmClient,
}

impl<'a> LlmInterpreter<'a> {
    pub fn new(llm_client: &'a dyn LlmClient) -> Self {
        Self { llm_client }
    }

    /// Interpret execution results using LLM, with fallback to deterministic interpretation
    pub fn interpret(
        &self,
        intent: &Intent,
        exec_result: &ExecutionResult,
        system_signals: &serde_json::Value,
    ) -> InterpretedAnswer {
        // Try LLM-backed interpretation first
        match self.interpret_with_llm(intent, exec_result, system_signals) {
            Ok(answer) => {
                tracing::debug!("LLM interpretation succeeded");
                answer
            }
            Err(e) => {
                tracing::debug!("LLM interpretation failed ({}), falling back to deterministic", e);
                // Fall back to deterministic interpretation
                interpret_without_llm(intent, exec_result)
            }
        }
    }

    fn interpret_with_llm(
        &self,
        intent: &Intent,
        exec_result: &ExecutionResult,
        system_signals: &serde_json::Value,
    ) -> Result<InterpretedAnswer, LlmError> {
        let system_prompt = self.build_system_prompt();
        let user_prompt = self.build_user_prompt(intent, exec_result, system_signals);
        let schema = self.get_interpretation_schema();

        let response_json = self.llm_client.call_json(&system_prompt, &user_prompt, &schema)?;

        // Parse response into InterpretedAnswer
        self.parse_interpretation_json(response_json)
    }

    fn build_system_prompt(&self) -> String {
        "You are a result interpreter for Arch Linux system commands. \
        Your job is to analyze command outputs and provide clear, honest answers to the user's question. \
        Be specific and include actual data from the outputs. \
        Do NOT use markdown code fences in your answer. \
        If commands failed, explain what went wrong honestly.".to_string()
    }

    fn build_user_prompt(
        &self,
        intent: &Intent,
        exec_result: &ExecutionResult,
        system_signals: &serde_json::Value,
    ) -> String {
        let mut prompt = format!(
            "User question: \"{}\"\n\n\
            Goal: {:?}\n\
            Domain: {:?}\n\n",
            intent.query, intent.goal, intent.domain
        );

        // Add command execution results
        prompt.push_str("Command execution results:\n");
        if exec_result.command_results.is_empty() {
            prompt.push_str("  (no commands executed)\n");
        } else {
            for (i, cmd_result) in exec_result.command_results.iter().enumerate() {
                prompt.push_str(&format!("\nCommand {}: {}\n", i + 1, cmd_result.full_command));
                prompt.push_str(&format!("Exit code: {}\n", cmd_result.exit_code));

                if !cmd_result.stdout.is_empty() {
                    let output_lines: Vec<&str> = cmd_result.stdout.lines().take(100).collect();
                    prompt.push_str(&format!("Output:\n{}\n", output_lines.join("\n")));
                }

                if !cmd_result.stderr.is_empty() && !cmd_result.success {
                    let error_lines: Vec<&str> = cmd_result.stderr.lines().take(20).collect();
                    prompt.push_str(&format!("Errors:\n{}\n", error_lines.join("\n")));
                }
            }
        }

        // Add system signals
        prompt.push_str("\n\nSystem context:\n");
        prompt.push_str(&serde_json::to_string_pretty(system_signals).unwrap_or_default());

        prompt.push_str("\n\nAnalyze the command outputs and provide a clear answer.");

        prompt
    }

    fn get_interpretation_schema(&self) -> String {
        r#"{
  "final_answer": "string (direct answer to user's question, 3-5 sentences max, NO markdown fences)",
  "achieved_goal": true | false,
  "validation_confidence": 0.0-1.0 (how confident you are this answers the question),
  "followup_suggestions": ["array of string suggestions for related questions"],
  "short_one_line_summary": "string (one-line summary)",
  "confidence": "High | Medium | Low",
  "reasoning": "string (brief explanation of how you arrived at this answer)",
  "source": "string (where the data came from)"
}"#.to_string()
    }

    fn parse_interpretation_json(&self, json: serde_json::Value) -> Result<InterpretedAnswer, LlmError> {
        let answer = json
            .get("final_answer")
            .and_then(|v| v.as_str())
            .ok_or_else(|| LlmError::InvalidJson("Missing 'final_answer' field".to_string()))?
            .to_string();

        let achieved_goal = json
            .get("achieved_goal")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        let validation_confidence = json
            .get("validation_confidence")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.75);

        let followup_suggestions: Vec<String> = json
            .get("followup_suggestions")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        let short_summary = json
            .get("short_one_line_summary")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let confidence = parse_confidence(
            json.get("confidence")
                .and_then(|v| v.as_str())
                .unwrap_or("Medium")
        );

        let reasoning = json
            .get("reasoning")
            .and_then(|v| v.as_str())
            .unwrap_or("LLM-generated interpretation")
            .to_string();

        let source = json
            .get("source")
            .and_then(|v| v.as_str())
            .unwrap_or("Command execution + LLM analysis")
            .to_string();

        Ok(InterpretedAnswer {
            answer,
            details: None,
            confidence,
            reasoning,
            source,
            achieved_goal,
            validation_confidence,
            followup_suggestions,
            short_summary,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::executor_core::{CommandResult, ExecutionResult};
    use crate::planner_core::{CommandPlan, DomainType, GoalType, Intent, SafetyLevel};

    #[test]
    fn test_parse_confidence() {
        assert_eq!(parse_confidence("high"), ConfidenceLevel::High);
        assert_eq!(parse_confidence("Medium"), ConfidenceLevel::Medium);
        assert_eq!(parse_confidence("LOW"), ConfidenceLevel::Low);
        assert_eq!(parse_confidence("unknown"), ConfidenceLevel::Medium);
    }

    #[test]
    fn test_fallback_interpreter_with_failures() {
        let intent = Intent {
            goal: GoalType::Inspect,
            domain: DomainType::Packages,
            constraints: vec![],
            query: "test query".to_string(),
        };

        let exec_result = ExecutionResult {
            plan: CommandPlan {
                commands: vec![],
                safety_level: SafetyLevel::ReadOnly,
                fallbacks: vec![],
                expected_output: String::new(),
                reasoning: String::new(),
            },
            command_results: vec![CommandResult {
                command: "pacman".to_string(),
                full_command: "pacman -Q test".to_string(),
                exit_code: 1,
                stdout: String::new(),
                stderr: "error: package not found".to_string(),
                success: false,
                time_ms: 10,
            }],
            success: false,
            execution_time_ms: 10,
        };

        let answer = interpret_without_llm(&intent, &exec_result);
        assert_eq!(answer.confidence, ConfidenceLevel::Low);
        assert!(answer.answer.contains("encountered errors"));
    }

    #[test]
    fn test_interpret_package_results() {
        let exec_result = ExecutionResult {
            plan: CommandPlan {
                commands: vec![],
                safety_level: SafetyLevel::ReadOnly,
                fallbacks: vec![],
                expected_output: String::new(),
                reasoning: String::new(),
            },
            command_results: vec![CommandResult {
                command: "pacman".to_string(),
                full_command: "pacman -Q steam".to_string(),
                exit_code: 0,
                stdout: "steam 1.0.0.79-2\nheroic-games-launcher 2.9.2-1\n".to_string(),
                stderr: String::new(),
                success: true,
                time_ms: 10,
            }],
            success: true,
            execution_time_ms: 10,
        };

        let result = interpret_package_results(&exec_result);
        assert!(result.contains("Found"));
        assert!(result.contains("steam"));
    }

    // v6.42.0: LLM Interpreter tests
    use crate::llm_client::{FakeLlmClient, LlmError};

    #[test]
    fn test_llm_interpreter_valid_json() {
        let valid_interpretation = serde_json::json!({
            "final_answer": "Yes, you have Steam installed (version 1.0.0.79-2)",
            "achieved_goal": true,
            "validation_confidence": 0.95,
            "followup_suggestions": ["Check for Steam updates"],
            "short_one_line_summary": "Steam is installed",
            "confidence": "High",
            "reasoning": "pacman query returned steam package",
            "source": "pacman -Q command output"
        });

        let fake_client = FakeLlmClient::always_valid(valid_interpretation);
        let interpreter = LlmInterpreter::new(&fake_client);

        let intent = Intent {
            goal: GoalType::Inspect,
            domain: DomainType::Packages,
            constraints: vec![],
            query: "do I have steam?".to_string(),
        };

        let exec_result = ExecutionResult {
            plan: CommandPlan {
                commands: vec![],
                safety_level: SafetyLevel::ReadOnly,
                fallbacks: vec![],
                expected_output: String::new(),
                reasoning: String::new(),
            },
            command_results: vec![CommandResult {
                command: "pacman".to_string(),
                full_command: "pacman -Q steam".to_string(),
                exit_code: 0,
                stdout: "steam 1.0.0.79-2\n".to_string(),
                stderr: String::new(),
                success: true,
                time_ms: 10,
            }],
            success: true,
            execution_time_ms: 10,
        };

        let system_signals = serde_json::json!({});

        let answer = interpreter.interpret(&intent, &exec_result, &system_signals);
        assert_eq!(answer.confidence, ConfidenceLevel::High);
        assert!(answer.achieved_goal);
        assert!(answer.validation_confidence > 0.9);
        assert!(!answer.followup_suggestions.is_empty());
    }

    #[test]
    fn test_llm_interpreter_invalid_json_fallback() {
        let fake_client = FakeLlmClient::always_error(LlmError::InvalidJson(
            "Bad JSON".to_string(),
        ));
        let interpreter = LlmInterpreter::new(&fake_client);

        let intent = Intent {
            goal: GoalType::Inspect,
            domain: DomainType::Packages,
            constraints: vec![],
            query: "do I have games?".to_string(),
        };

        let exec_result = ExecutionResult {
            plan: CommandPlan {
                commands: vec![],
                safety_level: SafetyLevel::ReadOnly,
                fallbacks: vec![],
                expected_output: String::new(),
                reasoning: String::new(),
            },
            command_results: vec![CommandResult {
                command: "sh".to_string(),
                full_command: "sh -c pacman -Qq | grep -Ei steam".to_string(),
                exit_code: 0,
                stdout: "steam\ngamemode\n".to_string(),
                stderr: String::new(),
                success: true,
                time_ms: 10,
            }],
            success: true,
            execution_time_ms: 10,
        };

        let system_signals = serde_json::json!({});

        let answer = interpreter.interpret(&intent, &exec_result, &system_signals);
        // Should fallback to deterministic interpretation
        assert!(answer.reasoning.contains("Fallback interpretation"));
    }

    #[test]
    fn test_llm_interpreter_timeout_fallback() {
        let fake_client = FakeLlmClient::always_error(LlmError::Timeout(30));
        let interpreter = LlmInterpreter::new(&fake_client);

        let intent = Intent {
            goal: GoalType::Inspect,
            domain: DomainType::Hardware,
            constraints: vec![],
            query: "does my CPU have AVX?".to_string(),
        };

        let exec_result = ExecutionResult {
            plan: CommandPlan {
                commands: vec![],
                safety_level: SafetyLevel::ReadOnly,
                fallbacks: vec![],
                expected_output: String::new(),
                reasoning: String::new(),
            },
            command_results: vec![CommandResult {
                command: "lscpu".to_string(),
                full_command: "lscpu".to_string(),
                exit_code: 0,
                stdout: "Flags: sse sse2 avx avx2\n".to_string(),
                stderr: String::new(),
                success: true,
                time_ms: 10,
            }],
            success: true,
            execution_time_ms: 10,
        };

        let system_signals = serde_json::json!({});

        let answer = interpreter.interpret(&intent, &exec_result, &system_signals);
        // Should fallback to deterministic hardware interpretation
        assert!(answer.reasoning.contains("Fallback"));
    }

    #[test]
    fn test_llm_interpreter_low_confidence() {
        let low_conf_interpretation = serde_json::json!({
            "final_answer": "Could not determine with certainty",
            "achieved_goal": false,
            "validation_confidence": 0.3,
            "followup_suggestions": [],
            "short_one_line_summary": "Uncertain result",
            "confidence": "Low",
            "reasoning": "Command output was ambiguous",
            "source": "Ambiguous command output"
        });

        let fake_client = FakeLlmClient::always_valid(low_conf_interpretation);
        let interpreter = LlmInterpreter::new(&fake_client);

        let intent = Intent {
            goal: GoalType::Check,
            domain: DomainType::Packages,
            constraints: vec![],
            query: "test".to_string(),
        };

        let exec_result = ExecutionResult {
            plan: CommandPlan {
                commands: vec![],
                safety_level: SafetyLevel::ReadOnly,
                fallbacks: vec![],
                expected_output: String::new(),
                reasoning: String::new(),
            },
            command_results: vec![],
            success: true,
            execution_time_ms: 0,
        };

        let system_signals = serde_json::json!({});

        let answer = interpreter.interpret(&intent, &exec_result, &system_signals);
        assert_eq!(answer.confidence, ConfidenceLevel::Low);
        assert!(!answer.achieved_goal);
        assert!(answer.validation_confidence < 0.5);
    }
}
