//! Anna Brain v10.0.0 - System Prompt
//!
//! Simplified prompt that local LLMs can actually follow.
//! Key: Be extremely directive with specific examples.

use crate::brain_v10::contracts::{BrainSession, EvidenceItem};

/// The system prompt - simplified for local LLMs
pub const SYSTEM_PROMPT: &str = r#"You are Anna, an Arch Linux system assistant. You answer questions about the user's system.

OUTPUT FORMAT - Always respond with exactly this JSON:
{"step_type":"decide_tool","tool_request":{"tool":"run_shell","arguments":{"command":"YOUR_COMMAND"},"why":"reason"},"answer":null,"evidence_refs":[],"reliability":0.0,"reasoning":"need data","user_question":null}

OR when you have evidence:
{"step_type":"final_answer","tool_request":null,"answer":"Your answer here [E1]","evidence_refs":["E1"],"reliability":0.9,"reasoning":"found in tool output","user_question":null}

COMMAND EXAMPLES FOR COMMON QUESTIONS:
- "RAM/memory" → free -m
- "CPU" → lscpu
- "disk space" → df -h
- "is X installed" → pacman -Qs X
- "GPU/graphics" → lspci | grep -i vga
- "games installed" → pacman -Qs steam && pacman -Qs lutris && pacman -Qs wine
- "file manager" → pacman -Qs thunar && pacman -Qs dolphin && pacman -Qs nautilus
- "orphan packages" → pacman -Qdt
- "desktop/WM" → echo $XDG_CURRENT_DESKTOP && echo $DESKTOP_SESSION
- "network interface" → ip link show
- "wifi status" → nmcli device status
- "DNS" → cat /etc/resolv.conf
- "updates" → checkupdates
- "big folders" → du -sh /home/*/ 2>/dev/null | sort -rh | head -10
- "big files /var" → sudo find /var -type f -exec du -h {} + 2>/dev/null | sort -rh | head -10

RULES:
1. When asked a question, output decide_tool with the appropriate command
2. After seeing tool output (in evidence), output final_answer citing [E1], [E2] etc.
3. If evidence shows empty output for package query, the package is NOT installed
4. Never guess. Use tool output only.

Output ONLY valid JSON. No text outside JSON."#;

/// Build the state message for the LLM - simplified
pub fn build_state_message(session: &BrainSession) -> String {
    let state = serde_json::json!({
        "user_question": session.question,
        "evidence_collected": format_evidence(&session.evidence),
        "iteration": session.iteration,
    });

    serde_json::to_string_pretty(&state).unwrap_or_else(|_| state.to_string())
}

/// Format evidence items for the LLM - simplified
fn format_evidence(evidence: &[EvidenceItem]) -> Vec<serde_json::Value> {
    evidence
        .iter()
        .map(|e| {
            serde_json::json!({
                "id": e.id,
                "command": e.description,
                "output": truncate_content(&e.content, 1500),
                "success": e.is_success()
            })
        })
        .collect()
}

/// Truncate content for LLM context
fn truncate_content(content: &str, max_len: usize) -> String {
    if content.len() <= max_len {
        content.to_string()
    } else {
        format!("{}...[truncated]", &content[..max_len])
    }
}

/// JSON schema for structured output validation
pub const OUTPUT_SCHEMA: &str = r#"{
  "type": "object",
  "properties": {
    "step_type": {
      "type": "string",
      "enum": ["decide_tool", "final_answer", "ask_user"]
    },
    "tool_request": {
      "type": ["object", "null"],
      "properties": {
        "tool": { "type": "string" },
        "arguments": { "type": "object" },
        "why": { "type": "string" }
      }
    },
    "answer": { "type": ["string", "null"] },
    "evidence_refs": {
      "type": "array",
      "items": { "type": "string" }
    },
    "reliability": {
      "type": "number",
      "minimum": 0.0,
      "maximum": 1.0
    },
    "reasoning": { "type": "string" },
    "user_question": { "type": ["string", "null"] }
  },
  "required": ["step_type", "reliability", "reasoning"]
}"#;

/// Map common queries to shell commands (fallback when LLM fails)
pub fn suggest_command_for_query(query: &str) -> Option<&'static str> {
    let q = query.to_lowercase();

    // RAM/Memory
    if q.contains("ram") || q.contains("memory") {
        return Some("free -m");
    }

    // CPU
    if q.contains("cpu") || q.contains("processor") || q.contains("core") || q.contains("thread") {
        return Some("lscpu");
    }

    // GPU
    if q.contains("gpu") || q.contains("graphics") || q.contains("video card") || q.contains("vga") {
        return Some("lspci | grep -i vga");
    }

    // Disk space
    if q.contains("disk") || q.contains("space") || q.contains("storage") || q.contains("filesystem") {
        return Some("df -h");
    }

    // Desktop/WM
    if q.contains("desktop") || q.contains("window manager") || q.contains(" wm ") || q.contains(" de ") {
        return Some("echo $XDG_CURRENT_DESKTOP; echo $XDG_SESSION_TYPE; loginctl show-session $(loginctl | grep $(whoami) | awk '{print $1}') -p Desktop -p Type 2>/dev/null");
    }

    // Network
    if q.contains("wifi") || q.contains("ethernet") || q.contains("network") || q.contains("connected") {
        return Some("ip link show; nmcli device status 2>/dev/null || echo 'nmcli not available'");
    }

    // DNS
    if q.contains("dns") || q.contains("resolver") {
        return Some("cat /etc/resolv.conf");
    }

    // Updates
    if q.contains("update") && !q.contains("upgrade") {
        return Some("checkupdates 2>/dev/null | head -20 || echo 'checkupdates not available or no updates'");
    }

    // Orphan packages
    if q.contains("orphan") {
        return Some("pacman -Qdt");
    }

    // Games
    if q.contains("game") {
        return Some("pacman -Qs 'steam|lutris|wine|proton|gamemode|mangohud|dxvk' 2>/dev/null | head -30");
    }

    // File manager
    if q.contains("file manager") {
        return Some("pacman -Qs 'thunar|dolphin|nautilus|pcmanfm|nemo|caja' 2>/dev/null");
    }

    // Big folders in home
    if q.contains("folder") && q.contains("home") && (q.contains("big") || q.contains("largest") || q.contains("size")) {
        return Some("du -sh ~/*/ 2>/dev/null | sort -rh | head -10");
    }

    // Big files in /var
    if q.contains("file") && q.contains("/var") && (q.contains("big") || q.contains("largest")) {
        return Some("find /var -type f -size +10M -exec ls -lh {} \\; 2>/dev/null | sort -k5 -rh | head -10");
    }

    // SSE/AVX CPU features
    if q.contains("sse") || q.contains("avx") {
        return Some("grep -E 'flags|model name' /proc/cpuinfo | head -2");
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_to_command() {
        assert_eq!(suggest_command_for_query("how much RAM"), Some("free -m"));
        assert_eq!(suggest_command_for_query("what CPU"), Some("lscpu"));
        assert_eq!(suggest_command_for_query("disk space"), Some("df -h"));
        assert!(suggest_command_for_query("random text").is_none());
    }

    #[test]
    fn test_schema_valid() {
        let schema: serde_json::Value = serde_json::from_str(OUTPUT_SCHEMA).unwrap();
        assert!(schema.get("type").is_some());
    }
}
