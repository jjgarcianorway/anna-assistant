//! Internal Dialogue System for v5.7.0-beta.55
//!
//! Implements telemetry-first two-round dialogue:
//! 1. Planning round: Analyzes question, identifies missing info, sketches structure
//! 2. Answer round: Generates final structured output with all sections

use anna_common::historian::SystemSummary;
use anna_common::llm::{ChatMessage, LlmConfig};
use anna_common::personality::PersonalityConfig;
use anna_common::types::SystemFacts;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

/// Compressed telemetry payload for LLM context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryPayload {
    /// Hardware summary
    pub hardware: HardwareSummary,

    /// OS information
    pub os: OsSummary,

    /// Current resource usage
    pub resources: ResourceSummary,

    /// Recent errors and warnings (last 5)
    pub recent_errors: Vec<String>,

    /// Historian trends (if available)
    pub trends: Option<TrendsSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareSummary {
    pub cpu_model: String,
    pub cpu_cores: u32,
    pub total_ram_gb: f64,
    pub gpu_model: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OsSummary {
    pub hostname: String,
    pub kernel: String,
    pub arch_status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceSummary {
    pub load_avg: (f64, f64, f64),
    pub ram_used_percent: f64,
    pub disk_usage: Vec<DiskUsage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskUsage {
    pub mount: String,
    pub used_percent: f64,
    pub available_gb: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendsSummary {
    pub avg_boot_time_s: f64,
    pub avg_cpu_percent: f64,
    pub stability_score: u8,
    pub performance_score: u8,
    pub days_analyzed: u32,
}

impl TelemetryPayload {
    /// Compress SystemFacts and Historian data into compact payload
    pub fn compress(facts: &SystemFacts, historian: Option<&SystemSummary>) -> Self {
        let hardware = HardwareSummary {
            cpu_model: facts.cpu_model.clone(),
            cpu_cores: facts.cpu_cores as u32,
            total_ram_gb: facts.total_memory_gb,
            gpu_model: facts.gpu_model.clone(),
        };

        let os = OsSummary {
            hostname: facts.hostname.clone(),
            kernel: facts.kernel.clone(),
            arch_status: "rolling".to_string(),
        };

        let load_avg = facts.system_health
            .as_ref()
            .map(|h| (
                h.load_averages.one_minute,
                h.load_averages.five_minutes,
                h.load_averages.fifteen_minutes,
            ))
            .unwrap_or((0.0, 0.0, 0.0));

        // Beta.77: Get actual RAM usage from memory_usage_info
        let ram_used_percent = facts.memory_usage_info
            .as_ref()
            .map(|m| m.ram_usage_percent as f64)
            .unwrap_or(0.0);

        // Beta.77: Get disk usage from df command for major mount points
        let disk_usage: Vec<DiskUsage> = Self::get_disk_usage();

        let resources = ResourceSummary {
            load_avg,
            ram_used_percent,
            disk_usage,
        };

        let recent_errors = facts.failed_services.iter()
            .take(5)
            .map(|s| format!("Failed service: {}", s))
            .collect();

        let trends = historian.map(|h| TrendsSummary {
            avg_boot_time_s: h.boot_trends.avg_boot_time_ms as f64 / 1000.0,
            avg_cpu_percent: h.cpu_trends.avg_utilization_percent,
            stability_score: h.health_summary.avg_stability_score,
            performance_score: h.health_summary.avg_performance_score,
            days_analyzed: h.health_summary.days_analyzed,
        });

        TelemetryPayload {
            hardware,
            os,
            resources,
            recent_errors,
            trends,
        }
    }

    /// Get disk usage for major mount points
    fn get_disk_usage() -> Vec<DiskUsage> {
        use std::process::Command;

        let output = match Command::new("df")
            .arg("-h")
            .arg("--output=target,pcent,avail")
            .output()
        {
            Ok(o) => o,
            Err(_) => return vec![],
        };

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut disk_usage = Vec::new();

        // Parse df output, skip header line
        for line in stdout.lines().skip(1) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 3 {
                let mount = parts[0];
                let used_str = parts[1].trim_end_matches('%');
                let avail_str = parts[2];

                // Only include major mount points (/, /home, /boot, etc.)
                if mount.starts_with('/') && (
                    mount == "/" ||
                    mount == "/home" ||
                    mount == "/boot" ||
                    mount.starts_with("/mnt") ||
                    mount.starts_with("/media")
                ) {
                    if let Ok(used_percent) = used_str.parse::<f64>() {
                        // Parse available GB (remove 'G' suffix if present)
                        let available_gb = avail_str
                            .trim_end_matches('G')
                            .trim_end_matches('M')
                            .trim_end_matches('K')
                            .parse::<f64>()
                            .unwrap_or(0.0);

                        disk_usage.push(DiskUsage {
                            mount: mount.to_string(),
                            used_percent,
                            available_gb,
                        });
                    }
                }
            }
        }

        disk_usage
    }

    /// Render as concise text for LLM context
    pub fn render(&self) -> String {
        let mut text = String::from("[ANNA_TELEMETRY_PAYLOAD]\n");

        // Hardware
        text.push_str("Hardware:\n");
        text.push_str(&format!("  CPU: {} ({} cores)\n", self.hardware.cpu_model, self.hardware.cpu_cores));
        text.push_str(&format!("  RAM: {:.1} GB\n", self.hardware.total_ram_gb));
        if let Some(ref gpu) = self.hardware.gpu_model {
            text.push_str(&format!("  GPU: {}\n", gpu));
        }
        text.push('\n');

        // OS
        text.push_str("OS:\n");
        text.push_str(&format!("  Hostname: {}\n", self.os.hostname));
        text.push_str(&format!("  Kernel: {}\n", self.os.kernel));
        text.push_str(&format!("  Arch: {} release\n", self.os.arch_status));
        text.push('\n');

        // Resources
        text.push_str("Current Resources:\n");
        text.push_str(&format!("  Load: {:.2}, {:.2}, {:.2}\n",
            self.resources.load_avg.0,
            self.resources.load_avg.1,
            self.resources.load_avg.2,
        ));
        text.push_str(&format!("  RAM usage: {:.1}%\n", self.resources.ram_used_percent));

        for disk in &self.resources.disk_usage {
            text.push_str(&format!("  Disk {}: {:.1}% ({:.1} GB free)\n",
                disk.mount,
                disk.used_percent,
                disk.available_gb,
            ));
        }
        text.push('\n');

        // Errors
        if !self.recent_errors.is_empty() {
            text.push_str("Recent Errors:\n");
            for err in &self.recent_errors {
                text.push_str(&format!("  • {}\n", err));
            }
            text.push('\n');
        }

        // Trends
        if let Some(ref trends) = self.trends {
            text.push_str("30-Day Trends:\n");
            text.push_str(&format!("  Boot time: {:.1}s avg\n", trends.avg_boot_time_s));
            text.push_str(&format!("  CPU usage: {:.1}% avg\n", trends.avg_cpu_percent));
            text.push_str(&format!("  Stability: {}/100\n", trends.stability_score));
            text.push_str(&format!("  Performance: {}/100\n", trends.performance_score));
            text.push_str(&format!("  Days analyzed: {}\n", trends.days_analyzed));
            text.push('\n');
        }

        text.push_str("[/ANNA_TELEMETRY_PAYLOAD]\n");
        text
    }
}

/// Result of internal dialogue process
#[derive(Debug, Clone)]
pub struct InternalDialogueResult {
    /// Final answer from the answer round
    pub answer: String,

    /// Internal trace (only populated if ANNA_INTERNAL_TRACE=1)
    pub trace: Option<InternalTrace>,
}

/// Internal trace for debugging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InternalTrace {
    /// Summary of internal process
    pub internal_summary: String,

    /// Planner prompt excerpt (first 500 chars)
    pub planner_prompt_excerpt: String,

    /// Planner response excerpt (first 500 chars)
    pub planner_response_excerpt: String,

    /// Answer prompt excerpt (first 500 chars)
    pub answer_prompt_excerpt: String,
}

impl InternalTrace {
    /// Render as section for output
    pub fn render(&self) -> String {
        format!(
            "[ANNA_INTERNAL_TRACE]\n\
            internal_summary: {}\n\n\
            planner_prompt_excerpt: {}\n\n\
            planner_response_excerpt: {}\n\n\
            answer_prompt_excerpt: {}\n\
            [/ANNA_INTERNAL_TRACE]\n",
            self.internal_summary,
            self.planner_prompt_excerpt,
            self.planner_response_excerpt,
            self.answer_prompt_excerpt,
        )
    }
}

/// Detect if model is "small" (1b or 3b parameters)
/// Small models struggle with two-round dialogue due to limited context
fn is_small_model(model_name: &str) -> bool {
    let model_lower = model_name.to_lowercase();
    model_lower.contains(":1b") || model_lower.contains(":3b")
}

/// Run internal dialogue with telemetry-first approach
/// Uses simplified single-round for small models (1b, 3b)
/// Uses two-round planning+answer for larger models (8b+)
pub async fn run_internal_dialogue(
    user_message: &str,
    payload: &TelemetryPayload,
    personality: &PersonalityConfig,
    current_model: &str,
    llm_config: &LlmConfig,
) -> Result<InternalDialogueResult> {
    let trace_enabled = std::env::var("ANNA_INTERNAL_TRACE").is_ok();

    // Detect model size and choose appropriate strategy
    let use_simple_mode = is_small_model(current_model);

    if use_simple_mode {
        // Simple mode: Single-round direct answer for small models
        let simple_prompt = build_simple_prompt(user_message, payload, personality, current_model);
        let answer = query_llm(llm_config, &simple_prompt).await?;

        let trace = if trace_enabled {
            Some(InternalTrace {
                internal_summary: "Simple mode: single-round (small model)".to_string(),
                planner_prompt_excerpt: "".to_string(),
                planner_response_excerpt: "".to_string(),
                answer_prompt_excerpt: simple_prompt.chars().take(500).collect(),
            })
        } else {
            None
        };

        Ok(InternalDialogueResult { answer, trace })
    } else {
        // Advanced mode: Two-round dialogue for larger models
        // Step 1: Planning round
        let planner_prompt = build_planner_prompt(user_message, payload, personality, current_model);
        let planner_response = query_llm(llm_config, &planner_prompt).await?;

        // Step 2: Answer round
        let answer_prompt = build_answer_prompt(user_message, payload, personality, current_model, &planner_response);
        let answer = query_llm(llm_config, &answer_prompt).await?;

        // Build trace if enabled
        let trace = if trace_enabled {
            Some(InternalTrace {
                internal_summary: "Two-round dialogue: planning + answer".to_string(),
                planner_prompt_excerpt: planner_prompt.chars().take(500).collect(),
                planner_response_excerpt: planner_response.chars().take(500).collect(),
                answer_prompt_excerpt: answer_prompt.chars().take(500).collect(),
            })
        } else {
            None
        };

        Ok(InternalDialogueResult { answer, trace })
    }
}

/// Build simple prompt for small models (1b, 3b)
/// Simplified single-round with minimal context and anti-hallucination rules
fn build_simple_prompt(
    user_message: &str,
    payload: &TelemetryPayload,
    _personality: &PersonalityConfig,
    _current_model: &str,
) -> String {
    let mut prompt = String::new();

    prompt.push_str("You are Anna, an Arch Linux system administrator.\n\n");

    // Only include system info if question seems hardware-related
    let question_lower = user_message.to_lowercase();
    let needs_hardware = question_lower.contains("computer")
        || question_lower.contains("system")
        || question_lower.contains("hardware")
        || question_lower.contains("specs")
        || question_lower.contains("cpu")
        || question_lower.contains("ram")
        || question_lower.contains("gpu");

    if needs_hardware {
        prompt.push_str("SYSTEM INFO:\n");
        prompt.push_str(&format!("CPU: {} ({} cores)\n", payload.hardware.cpu_model, payload.hardware.cpu_cores));
        prompt.push_str(&format!("RAM: {:.1} GB\n", payload.hardware.total_ram_gb));
        if let Some(ref gpu) = payload.hardware.gpu_model {
            prompt.push_str(&format!("GPU: {}\n", gpu));
        }
        prompt.push_str(&format!("Kernel: {}\n", payload.os.kernel));
        prompt.push('\n');
    }

    // User question
    prompt.push_str("QUESTION:\n");
    prompt.push_str(user_message);
    prompt.push_str("\n\n");

    // Anti-hallucination instructions (Beta.89: Added telemetry-first rule)
    prompt.push_str("CRITICAL RULES:\n");
    prompt.push_str("1. CHECK SYSTEM INFO ABOVE - use it to answer, don't guess\n");
    prompt.push_str("2. Answer ONLY what was asked - don't add extra information\n");
    prompt.push_str("3. If you don't know something, say \"I don't have that information\"\n");
    prompt.push_str("4. ONLY suggest real Arch Linux commands (pacman, systemctl, vim, etc.)\n");
    prompt.push_str("5. NEVER invent commands or tools that don't exist\n");
    prompt.push_str("6. If suggesting config files, check they actually exist on Arch\n");
    prompt.push_str("7. Keep answer under 150 words\n");
    prompt.push_str("8. Link to Arch Wiki ONLY if directly relevant\n\n");

    prompt.push_str("ANSWER:\n");

    prompt
}

/// Build planner prompt (Step 1) - for larger models only
fn build_planner_prompt(
    user_message: &str,
    payload: &TelemetryPayload,
    personality: &PersonalityConfig,
    _current_model: &str,
) -> String {
    let mut prompt = String::new();

    prompt.push_str("You are Anna's planning system. Analyze the user's question and telemetry.\n\n");

    // Telemetry
    prompt.push_str(&payload.render());
    prompt.push('\n');

    // Personality
    prompt.push_str(&personality.render_personality_view());
    prompt.push('\n');

    // User question
    prompt.push_str("[USER_QUESTION]\n");
    prompt.push_str(user_message);
    prompt.push_str("\n[/USER_QUESTION]\n\n");

    // Instructions
    prompt.push_str("[PLANNER_TASK]\n");
    prompt.push_str("Analyze this question and provide:\n\n");
    prompt.push_str("1. CLASSIFICATION: What type of request is this?\n");
    prompt.push_str("   - Hardware/system info query?\n");
    prompt.push_str("   - Configuration change request?\n");
    prompt.push_str("   - Troubleshooting/diagnostic?\n");
    prompt.push_str("   - Personality/meta request?\n\n");
    prompt.push_str("2. TELEMETRY_CHECK: Do we have enough data to answer?\n");
    prompt.push_str("   - List what data we have from [ANNA_TELEMETRY_PAYLOAD]\n");
    prompt.push_str("   - List what data is missing (if any)\n\n");
    prompt.push_str("3. ANSWER_STRUCTURE: Sketch the final answer structure:\n");
    prompt.push_str("   - What should [ANNA_SUMMARY] say?\n");
    prompt.push_str("   - What steps go in [ANNA_ACTION_PLAN]?\n");
    prompt.push_str("   - What Arch Wiki pages are relevant?\n");
    prompt.push_str("   - What backup/restore commands are needed?\n\n");
    prompt.push_str("Keep this concise (300 words max).\n");
    prompt.push_str("[/PLANNER_TASK]\n");

    prompt
}

/// Build answer prompt (Step 2)
fn build_answer_prompt(
    user_message: &str,
    payload: &TelemetryPayload,
    personality: &PersonalityConfig,
    current_model: &str,
    planner_response: &str,
) -> String {
    let mut prompt = String::new();

    prompt.push_str("You are Anna, Arch Linux system administrator. Generate the final answer.\n\n");

    // Include planner's analysis
    prompt.push_str("[PLANNER_ANALYSIS]\n");
    prompt.push_str(planner_response);
    prompt.push_str("\n[/PLANNER_ANALYSIS]\n\n");

    // Telemetry
    prompt.push_str(&payload.render());
    prompt.push('\n');

    // Personality
    prompt.push_str(&personality.render_personality_view());
    prompt.push('\n');

    // User question
    prompt.push_str("[USER_QUESTION]\n");
    prompt.push_str(user_message);
    prompt.push_str("\n[/USER_QUESTION]\n\n");

    // Final instructions
    prompt.push_str(&build_answer_instructions(current_model));

    prompt
}

/// Build answer round instructions
fn build_answer_instructions(_current_model: &str) -> String {
    let mut instr = String::new();

    instr.push_str("[ANSWER_TASK]\n");
    instr.push_str("Generate the final answer with these EXACT sections:\n\n");

    instr.push_str("1. [ANNA_TUI_HEADER]\n");
    instr.push_str("   status: OK | WARN | CRIT\n");
    instr.push_str("   focus: <topic>\n");
    instr.push_str("   mode: <model name>\n");
    instr.push_str("   model_hint: <suggestion or 'current ok'>\n");
    instr.push_str("   [/ANNA_TUI_HEADER]\n\n");

    instr.push_str("2. [ANNA_SUMMARY]\n");
    instr.push_str("   2-4 line summary of the question and answer.\n");
    instr.push_str("   [/ANNA_SUMMARY]\n\n");

    instr.push_str("3. [ANNA_ACTION_PLAN]\n");
    instr.push_str("   JSON array of action steps (if applicable).\n");
    instr.push_str("   [/ANNA_ACTION_PLAN]\n\n");

    instr.push_str("4. [ANNA_HUMAN_OUTPUT]\n");
    instr.push_str("   Markdown formatted answer with:\n");
    instr.push_str("   - Direct answer (1-3 sentences at top)\n");
    instr.push_str("   - Commands section (```bash blocks)\n");
    instr.push_str("   - Backup and restore commands with ANNA_BACKUP.YYYYMMDD-HHMMSS suffix\n");
    instr.push_str("   - References section with Arch Wiki links\n");
    instr.push_str("   [/ANNA_HUMAN_OUTPUT]\n\n");

    instr.push_str("CRITICAL RULES:\n");
    instr.push_str("- Phase 1 mode: ANSWERS ONLY. NO EXECUTION.\n");
    instr.push_str("- Always check telemetry FIRST before answering\n");
    instr.push_str("- NEVER guess hardware, service names, or file paths\n");
    instr.push_str("- ONLY suggest real Arch Linux commands - NEVER invent tools\n");
    instr.push_str("- If data is missing, say so and propose commands to get it\n");
    instr.push_str("- Every file modification needs backup + restore commands\n");
    instr.push_str("- Always include Arch Wiki references when relevant\n");
    instr.push_str("- Answer ONLY what was asked - don't dump irrelevant information\n");
    instr.push_str("- Respect personality traits in tone and verbosity\n");
    instr.push_str("[/ANSWER_TASK]\n");

    instr
}

/// Query LLM with a prepared prompt
async fn query_llm(config: &LlmConfig, prompt: &str) -> Result<String> {
    let base_url = config.base_url.as_ref()
        .context("LLM base_url not configured")?;

    let model = config.model.as_ref()
        .context("LLM model not configured")?;

    let endpoint = format!("{}/chat/completions", base_url.trim_end_matches('/'));

    let messages = vec![ChatMessage {
        role: "user".to_string(),
        content: prompt.to_string(),
    }];

    #[derive(Serialize)]
    struct ChatCompletionRequest {
        model: String,
        messages: Vec<ChatMessage>,
        max_tokens: Option<u32>,
        temperature: f64,
        stream: bool,
    }

    #[derive(Deserialize)]
    struct ChatCompletionResponse {
        choices: Vec<ChatChoice>,
    }

    #[derive(Deserialize)]
    struct ChatChoice {
        message: ChatMessage,
    }

    let request = ChatCompletionRequest {
        model: model.clone(),
        messages,
        max_tokens: config.max_tokens,
        temperature: 0.7,
        stream: false,  // TODO beta.58: Enable streaming with futures dependency
    };

    let client = reqwest::Client::new();
    let mut req_builder = client.post(&endpoint).json(&request);

    if let Some(api_key_env) = &config.api_key_env {
        if let Ok(api_key) = std::env::var(api_key_env) {
            req_builder = req_builder.bearer_auth(api_key);
        }
    }

    let response = req_builder.send().await.context("Failed to send LLM request")?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_default();
        anyhow::bail!("LLM API error {}: {}", status, error_text);
    }

    let completion: ChatCompletionResponse = response.json().await.context("Failed to parse LLM response")?;

    let answer = completion
        .choices
        .first()
        .map(|c| c.message.content.clone())
        .unwrap_or_else(|| "No response from LLM".to_string());

    Ok(answer)
}
