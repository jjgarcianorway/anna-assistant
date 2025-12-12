//! Trace block formatting for debug commands (v0.0.446).
//!
//! Handles formatting of TraceBlock data for the `annactl debug trace` command.

use anna_shared::debug_mode::DebugLevel;
use anna_shared::state_dir;
use anyhow::Result;
use std::fs;
use std::path::PathBuf;

/// Show last canonical TraceBlock.
pub async fn show_last_trace(level: &str) -> Result<()> {
    let path = PathBuf::from(state_dir()).join("trace_log.json");

    if !path.exists() {
        println!("No trace log found at {}", path.display());
        println!("\nTo enable trace logging, set debug.level in Anna's config:");
        println!("  level = \"summary\"  # Basic info: domain, probes, outcome");
        println!("  level = \"trace\"    # Detailed: commands, exit codes, LLM tokens");
        println!("  level = \"full\"     # Everything: raw prompts, raw outputs");
        return Ok(());
    }

    let content = fs::read_to_string(&path)?;

    // Parse last entry
    if let Some(last_line) = content.lines().rev().find(|l| !l.trim().is_empty()) {
        let output_level = match level.to_lowercase().as_str() {
            "summary" | "1" => DebugLevel::Summary,
            "trace" | "2" => DebugLevel::Trace,
            "full" | "3" => DebugLevel::Full,
            _ => DebugLevel::Trace,
        };
        println!("{}", format_trace_block(last_line, output_level)?);
    } else {
        println!("Trace log is empty.");
    }

    Ok(())
}

/// Format a TraceBlock for display.
pub fn format_trace_block(json_line: &str, level: DebugLevel) -> Result<String> {
    let json: serde_json::Value = serde_json::from_str(json_line)?;
    let mut out = String::new();

    // Header based on level
    let header = match level {
        DebugLevel::Summary => "[summary]",
        DebugLevel::Trace => "[trace]",
        DebugLevel::Full => "[full_trace]",
        DebugLevel::Off => return Ok(String::new()),
    };
    out.push_str(&format!("{}\n", header));

    // Always show core info
    format_core_info(&json, &mut out);

    // Summary level: show probe summary and failures
    format_summary_info(&json, &mut out);

    // Trace level: add probe details and LLM info
    if level.includes_trace() {
        format_trace_details(&json, &mut out);
    }

    // Full level: add raw prompts/responses
    if level.includes_full() {
        format_full_details(&json, &mut out);
    }

    // Timeout info (always shown if present)
    format_timeout_info(&json, &mut out);

    Ok(out)
}

/// Format core info fields.
fn format_core_info(json: &serde_json::Value, out: &mut String) {
    if let Some(v) = json.get("request_id").and_then(|v| v.as_str()) {
        out.push_str(&format!("  request_id     {}\n", v));
    }
    if let Some(v) = json.get("query").and_then(|v| v.as_str()) {
        let truncated = if v.len() > 60 { &v[..60] } else { v };
        out.push_str(&format!("  query          {}\n", truncated));
    }
    if let Some(v) = json.get("intent").and_then(|v| v.as_str()) {
        out.push_str(&format!("  intent         {}\n", v));
    }
    if let Some(v) = json.get("domain").and_then(|v| v.as_str()) {
        out.push_str(&format!("  domain         {}\n", v));
    }
    if let Some(v) = json.get("outcome").and_then(|v| v.as_str()) {
        out.push_str(&format!("  outcome        {}\n", v));
    }
}

/// Format summary level info.
fn format_summary_info(json: &serde_json::Value, out: &mut String) {
    if let Some(probes) = json.get("probes").and_then(|v| v.as_array()) {
        let probe_names: Vec<&str> = probes.iter().filter_map(|p| p.as_str()).collect();
        out.push_str(&format!("  probes         [{}]\n", probe_names.join(", ")));
    }

    // Gate result
    if let Some(gate) = json.get("reliability_gate") {
        if let Some(outcome) = gate.get("outcome").and_then(|v| v.as_str()) {
            let score = gate.get("score").and_then(|v| v.as_f64()).unwrap_or(0.0);
            out.push_str(&format!("  gate           {} (score: {:.2})\n", outcome, score));
        }
    }

    // Failures
    if let Some(failures) = json.get("failures").and_then(|v| v.as_array()) {
        if !failures.is_empty() {
            out.push_str("  failures:\n");
            for f in failures {
                if let Some(reason) = f.get("reason").and_then(|v| v.as_str()) {
                    out.push_str(&format!("    - {}\n", reason));
                }
            }
        }
    }
}

/// Format trace level details.
fn format_trace_details(json: &serde_json::Value, out: &mut String) {
    // Probe traces
    if let Some(probe_traces) = json.get("probe_traces").and_then(|v| v.as_array()) {
        if !probe_traces.is_empty() {
            out.push_str("\n  [probe_details]\n");
            for pt in probe_traces {
                let cmd = pt.get("command").and_then(|v| v.as_str()).unwrap_or("");
                let exit = pt.get("exit_code").and_then(|v| v.as_i64()).unwrap_or(-1);
                let dur = pt.get("duration_ms").and_then(|v| v.as_u64()).unwrap_or(0);
                out.push_str(&format!("    {} (exit={}, {}ms)\n", cmd, exit, dur));

                // Parsed values at trace level
                if let Some(parsed) = pt.get("parsed_values").and_then(|v| v.as_str()) {
                    if !parsed.is_empty() {
                        out.push_str(&format!("      parsed: {}\n", parsed));
                    }
                }
            }
        }
    }

    // LLM traces
    if let Some(llm_traces) = json.get("llm_traces").and_then(|v| v.as_array()) {
        if !llm_traces.is_empty() {
            out.push_str("\n  [llm_calls]\n");
            for lt in llm_traces {
                let role = lt.get("role").and_then(|v| v.as_str()).unwrap_or("");
                let model = lt.get("model").and_then(|v| v.as_str()).unwrap_or("");
                let tokens = lt.get("token_count").and_then(|v| v.as_u64()).unwrap_or(0);
                let dur = lt.get("duration_ms").and_then(|v| v.as_u64()).unwrap_or(0);
                out.push_str(&format!("    {}@{} ({}tok, {}ms)\n", role, model, tokens, dur));

                // Prompt digest at trace level
                if let Some(digest) = lt.get("prompt_digest").and_then(|v| v.as_str()) {
                    out.push_str(&format!("      digest: {}\n", digest));
                }
            }
        }
    }

    // Timings
    if let Some(timings) = json.get("timings") {
        let total = timings.get("total_ms").and_then(|v| v.as_u64()).unwrap_or(0);
        let probe = timings.get("probe_ms").and_then(|v| v.as_u64()).unwrap_or(0);
        let llm = timings.get("llm_ms").and_then(|v| v.as_u64()).unwrap_or(0);
        out.push_str(&format!(
            "\n  timings        total:{}ms probe:{}ms llm:{}ms\n",
            total, probe, llm
        ));
    }
}

/// Format full level details.
fn format_full_details(json: &serde_json::Value, out: &mut String) {
    if let Some(llm_traces) = json.get("llm_traces").and_then(|v| v.as_array()) {
        for lt in llm_traces {
            let role = lt.get("role").and_then(|v| v.as_str()).unwrap_or("");

            if let Some(prompt) = lt.get("prompt").and_then(|v| v.as_str()) {
                out.push_str(&format!("\n  [prompt:{}]\n", role));
                for (i, line) in prompt.lines().enumerate() {
                    if i >= 50 {
                        out.push_str("    ...[truncated]\n");
                        break;
                    }
                    out.push_str(&format!("    {}\n", line));
                }
            }

            if let Some(response) = lt.get("response").and_then(|v| v.as_str()) {
                out.push_str(&format!("\n  [response:{}]\n", role));
                for (i, line) in response.lines().enumerate() {
                    if i >= 50 {
                        out.push_str("    ...[truncated]\n");
                        break;
                    }
                    out.push_str(&format!("    {}\n", line));
                }
            }
        }
    }

    // Raw probe output
    if let Some(probe_traces) = json.get("probe_traces").and_then(|v| v.as_array()) {
        for pt in probe_traces {
            let cmd = pt.get("command").and_then(|v| v.as_str()).unwrap_or("");

            if let Some(output) = pt.get("raw_output").and_then(|v| v.as_str()) {
                if !output.is_empty() {
                    out.push_str(&format!("\n  [raw_output:{}]\n", cmd));
                    for (i, line) in output.lines().enumerate() {
                        if i >= 20 {
                            out.push_str("    ...[truncated]\n");
                            break;
                        }
                        out.push_str(&format!("    {}\n", line));
                    }
                }
            }
        }
    }
}

/// Format timeout info.
fn format_timeout_info(json: &serde_json::Value, out: &mut String) {
    if let Some(timeout) = json.get("timeout") {
        let stage = timeout
            .get("stage")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        let configured = timeout
            .get("configured_ms")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        let elapsed = timeout
            .get("elapsed_ms")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        out.push_str(&format!(
            "\n  [TIMEOUT] {} (configured: {}ms, elapsed: {}ms)\n",
            stage, configured, elapsed
        ));
    }
}
