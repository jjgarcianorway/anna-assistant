//! Anna Brain v10.2.0 - System Prompt
//!
//! The key philosophy: Learn from THIS machine, never hardcode.
//! Evidence is the single source of truth.

use crate::brain_v10::contracts::{BrainSession, EvidenceItem};

// Re-export the full system prompt from the dedicated module
pub use crate::brain_v10::system_prompt::SYSTEM_PROMPT_V102 as SYSTEM_PROMPT;

/// Build the state message for the LLM - with clear guidance
pub fn build_state_message(session: &BrainSession) -> String {
    let evidence = format_evidence(&session.evidence);
    let has_data = evidence.iter().any(|e| {
        e.get("output")
            .and_then(|v| v.as_str())
            .map(|s| !s.is_empty() && s != "null")
            .unwrap_or(false)
    });

    let guidance = if has_data {
        "You have evidence. Use final_answer to respond, citing evidence IDs like [E1]."
    } else {
        "No evidence yet. Use decide_tool to run a command."
    };

    let state = serde_json::json!({
        "user_question": session.question,
        "evidence_collected": evidence,
        "iteration": session.iteration,
        "guidance": guidance,
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
/// Returns multiple commands as needed for complex queries
pub fn suggest_command_for_query(query: &str) -> Option<&'static str> {
    let q = query.to_lowercase();

    // RAM/Memory - handles "how much RAM", "free memory", etc.
    if q.contains("ram") || q.contains("memory") || q.contains("free") && !q.contains("disk") {
        return Some("free -h");
    }

    // CPU - handles cores, threads, model, SSE, AVX
    if q.contains("cpu") || q.contains("processor") || q.contains("core") || q.contains("thread") {
        return Some("lscpu");
    }

    // SSE/AVX CPU features - specific query for instruction sets
    if q.contains("sse") || q.contains("avx") {
        return Some("grep -oE '(sse[^ ]*|avx[^ ]*)' /proc/cpuinfo | sort -u; grep 'model name' /proc/cpuinfo | head -1");
    }

    // GPU - handles graphics card, video, nvidia, amd
    if q.contains("gpu") || q.contains("graphics") || q.contains("video") || q.contains("vga") || q.contains("nvidia") || q.contains("amd") && q.contains("card") {
        return Some("lspci | grep -iE 'vga|3d|display'");
    }

    // Disk space - handles "free space", "root filesystem", etc.
    if q.contains("disk") || q.contains("space") || q.contains("storage") || q.contains("filesystem") || q.contains("root") && q.contains("free") {
        return Some("df -h");
    }

    // Big folders in home - handles variations
    if (q.contains("folder") || q.contains("director")) && (q.contains("home") || q.contains("~")) && (q.contains("big") || q.contains("large") || q.contains("size") || q.contains("top")) {
        return Some("du -sh ~/*/ 2>/dev/null | sort -rh | head -10");
    }

    // Big folders under /home (system wide)
    if (q.contains("folder") || q.contains("director")) && q.contains("/home") && (q.contains("big") || q.contains("large") || q.contains("size") || q.contains("top")) {
        return Some("du -sh /home/*/ 2>/dev/null | sort -rh | head -10");
    }

    // Big files in /var
    if q.contains("file") && q.contains("/var") && (q.contains("big") || q.contains("large") || q.contains("size") || q.contains("top")) {
        return Some("find /var -xdev -type f -printf '%s %p\\n' 2>/dev/null | sort -rn | head -10 | awk '{printf \"%.1fM %s\\n\", $1/1048576, $2}'");
    }

    // Desktop/WM - handles "what desktop", "window manager", "DE"
    if q.contains("desktop") || q.contains("window manager") || q.contains(" wm") || q.contains(" de ") || q.contains("environment") {
        return Some("echo \"XDG_CURRENT_DESKTOP=$XDG_CURRENT_DESKTOP\"; echo \"XDG_SESSION_TYPE=$XDG_SESSION_TYPE\"; echo \"DESKTOP_SESSION=$DESKTOP_SESSION\"; ps -e | grep -iE 'gnome-shell|kwin|xfwm|openbox|i3|sway|hyprland|bspwm|awesome' | head -5");
    }

    // Network - wifi, ethernet, interface
    if q.contains("wifi") || q.contains("ethernet") || q.contains("network") || q.contains("connected") || q.contains("interface") {
        return Some("ip link show; echo '---'; nmcli device status 2>/dev/null || echo 'NetworkManager not available'");
    }

    // DNS - resolver configuration
    if q.contains("dns") || q.contains("resolver") || q.contains("nameserver") {
        return Some("cat /etc/resolv.conf; echo '---'; resolvectl status 2>/dev/null | head -20 || echo 'resolvectl not available'");
    }

    // Updates - pacman and AUR
    if q.contains("update") && !q.contains("upgrade") {
        return Some("checkupdates 2>/dev/null | head -20 || echo 'No updates or checkupdates not available'");
    }

    // Orphan packages
    if q.contains("orphan") {
        return Some("pacman -Qdt 2>/dev/null || echo 'No orphans found'");
    }

    // Games - steam, lutris, wine, etc.
    if q.contains("game") {
        return Some("pacman -Qs -q 'steam|lutris|wine|proton|gamemode|mangohud|dxvk|retroarch' 2>/dev/null | head -20");
    }

    // File manager - handles "graphical file manager", "any file manager"
    if q.contains("file manager") || (q.contains("file") && q.contains("manager")) {
        return Some("pacman -Qs -q 'thunar|dolphin|nautilus|pcmanfm|nemo|caja|ranger|mc|lf' 2>/dev/null");
    }

    // Kernel version
    if q.contains("kernel") {
        return Some("uname -r; uname -a");
    }

    // Failed services / systemd issues
    if q.contains("failed") || q.contains("service") && q.contains("issue") || q.contains("systemd") {
        return Some("systemctl --failed; echo '---'; journalctl -p 0..3 -b --no-pager | tail -20");
    }

    // System summary / overview
    if q.contains("summary") || q.contains("overview") || q.contains("know about") || q.contains("how are you") {
        return Some("echo \"Kernel: $(uname -r)\"; free -h | grep Mem; df -h /; ip route | head -1");
    }

    // Issues / problems / worried
    if q.contains("issue") || q.contains("problem") || q.contains("worried") || q.contains("fire") {
        return Some("systemctl --failed 2>/dev/null; journalctl -p 0..3 -b --no-pager 2>/dev/null | tail -10");
    }

    // Pacman cache cleaning
    if q.contains("cache") && q.contains("pacman") || q.contains("clean") && q.contains("cache") {
        return Some("du -sh /var/cache/pacman/pkg; paccache -d 2>/dev/null || echo 'paccache not available (install pacman-contrib)'");
    }

    // Package installed check - generic
    if q.contains("installed") {
        // Try to extract package name - look for common patterns
        if q.contains("steam") {
            return Some("pacman -Qs steam");
        }
        if q.contains("firefox") {
            return Some("pacman -Qs firefox");
        }
        // Generic - will need LLM to figure out the package
        return None;
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_to_command() {
        assert_eq!(suggest_command_for_query("how much RAM"), Some("free -h"));
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
