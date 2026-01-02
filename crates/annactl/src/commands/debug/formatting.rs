//! Debug output formatting utilities (v0.0.446).

use anyhow::Result;

/// Format a debug block for display.
pub fn format_debug_block(json_line: &str) -> Result<String> {
    let json: serde_json::Value = serde_json::from_str(json_line)?;
    let mut out = String::new();

    out.push_str("[debug]\n");

    if let Some(v) = json.get("request_id").and_then(|v| v.as_str()) {
        out.push_str(&format!("  request_id          {}\n", v));
    }
    if let Some(v) = json.get("outcome").and_then(|v| v.as_str()) {
        out.push_str(&format!("  outcome             {}\n", v));
    }
    if let Some(v) = json.get("routed_topic").and_then(|v| v.as_str()) {
        out.push_str(&format!("  routed_topic        {}\n", v));
    }

    // Models
    if let Some(models) = json.get("models") {
        let mut parts = Vec::new();
        if let Some(t) = models.get("translator").and_then(|v| v.as_str()) {
            parts.push(format!("translator:{}", t));
        }
        if let Some(s) = models.get("specialist").and_then(|v| v.as_str()) {
            parts.push(format!("specialist:{}", s));
        }
        if !parts.is_empty() {
            out.push_str(&format!("  models_used         {}\n", parts.join(", ")));
        }
    }

    // Timings
    if let Some(timings) = json.get("timings") {
        let total = timings
            .get("total_ms")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        let probe = timings
            .get("probe_ms")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        let llm = timings.get("llm_ms").and_then(|v| v.as_u64()).unwrap_or(0);
        out.push_str(&format!(
            "  timings             total:{}ms probe:{}ms llm:{}ms\n",
            total, probe, llm
        ));
    }

    // Reason codes
    if let Some(codes) = json.get("reason_codes").and_then(|v| v.as_array()) {
        let code_strs: Vec<&str> = codes.iter().filter_map(|c| c.as_str()).collect();
        out.push_str(&format!(
            "  reason_codes        [{}]\n",
            code_strs.join(", ")
        ));
    }

    // Timeout
    if let Some(timeout) = json.get("timeout") {
        let stage = timeout
            .get("stage")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        let configured = timeout
            .get("timeout_ms")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        let elapsed = timeout
            .get("elapsed_ms")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        out.push_str(&format!(
            "  timeout             {} (configured: {}ms, elapsed: {}ms)\n",
            stage, configured, elapsed
        ));
    }

    Ok(out)
}
