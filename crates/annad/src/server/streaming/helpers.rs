//! Helper functions for streaming request handling.
//! v0.0.993: Added automatic fix detection and offer
//! v0.0.998: Added configuration recipes

use std::collections::HashMap;
use std::sync::RwLock;

use anna_shared::exposure::gate::filter_final_answer_default;
use anna_shared::rpc::{DialogueStep, StepType, StreamingResponse};
use anyhow::Result;

/// Track pending recipe confirmations by session
pub static PENDING_RECIPES: RwLock<Option<HashMap<String, String>>> = RwLock::new(None);

pub fn set_pending_recipe(session_id: &str, recipe_id: &str) {
    if let Ok(mut guard) = PENDING_RECIPES.write() {
        let map = guard.get_or_insert_with(HashMap::new);
        map.insert(session_id.to_string(), recipe_id.to_string());
    }
}

pub fn take_pending_recipe(session_id: &str) -> Option<String> {
    if let Ok(mut guard) = PENDING_RECIPES.write() {
        if let Some(map) = guard.as_mut() {
            return map.remove(session_id);
        }
    }
    None
}

/// Phase 15/22: Send a FinalAnswer with mandatory filtering.
/// All FinalAnswer content MUST go through filter_final_answer_default().
/// Uses ReadOnly intent by default (conservative: no "would you like" offers).
pub async fn send_filtered_final_answer<W: tokio::io::AsyncWriteExt + Unpin>(
    writer: &mut W,
    content: &str,
) -> Result<()> {
    let filtered = filter_final_answer_default(content);
    let step = DialogueStep {
        step_type: StepType::FinalAnswer,
        content: filtered.content,
    };
    let response = StreamingResponse::Step { step };
    let json = serde_json::to_string(&response)?;
    writer.write_all(format!("{}\n", json).as_bytes()).await?;
    Ok(())
}

/// v0.0.997: Check if question is asking about fix history
pub fn is_fix_history_question(question: &str) -> bool {
    let q = question.to_lowercase();

    // Two-word patterns
    let two_word = [
        ("fix", "history"),
        ("what", "fixed"),
        ("fixes", "done"),
        ("show", "fixes"),
        ("list", "fixes"),
        ("recent", "fixes"),
        ("repair", "history"),
        ("auto", "fixes"),
    ];

    for (a, b) in &two_word {
        if q.contains(*a) && q.contains(*b) {
            return true;
        }
    }

    // Three-word patterns
    if (q.contains("what") && q.contains("anna") && q.contains("fix"))
        || (q.contains("what") && q.contains("have") && q.contains("fix"))
    {
        return true;
    }

    false
}

/// v0.0.998: Extract recipe ID from question for pending recipe tracking
pub fn extract_recipe_id(question: &str) -> String {
    let q = question.to_lowercase();

    // Vim recipes
    if q.contains("vim") || q.contains("neovim") {
        if q.contains("dark") {
            return "vim-dark-mode".to_string();
        }
        if q.contains("syntax") {
            return "vim-syntax".to_string();
        }
        if q.contains("line") && q.contains("number") {
            return "vim-line-numbers".to_string();
        }
        if q.contains("mouse") {
            return "vim-mouse".to_string();
        }
        if q.contains("tab") || q.contains("indent") {
            return "vim-tabs".to_string();
        }
    }

    // Git recipes
    if q.contains("git") {
        if q.contains("email") {
            return "git-email".to_string();
        }
        if q.contains("name") {
            return "git-name".to_string();
        }
        if q.contains("alias") {
            return "git-aliases".to_string();
        }
        if q.contains("default") && q.contains("branch") {
            return "git-default-branch".to_string();
        }
    }

    // Shell recipes
    if q.contains("alias") {
        return "shell-alias".to_string();
    }
    if q.contains("path") && (q.contains("add") || q.contains("append")) {
        return "shell-path".to_string();
    }
    if q.contains("export") {
        return "shell-export".to_string();
    }

    // Service recipes
    if q.contains("restart") {
        if let Some(service) = extract_service_from_question(&q) {
            return format!("service-restart-{}", service);
        }
    }
    if q.contains("start") && !q.contains("restart") {
        if let Some(service) = extract_service_from_question(&q) {
            return format!("service-start-{}", service);
        }
    }
    if q.contains("stop") {
        if let Some(service) = extract_service_from_question(&q) {
            return format!("service-stop-{}", service);
        }
    }
    if q.contains("enable") {
        if let Some(service) = extract_service_from_question(&q) {
            return format!("service-enable-{}", service);
        }
    }
    if q.contains("disable") {
        if let Some(service) = extract_service_from_question(&q) {
            return format!("service-disable-{}", service);
        }
    }

    "unknown".to_string()
}

/// Extract service name from question
pub fn extract_service_from_question(q: &str) -> Option<String> {
    let services = [
        "nginx", "apache", "httpd", "mysql", "mariadb", "postgresql", "postgres",
        "docker", "containerd", "redis", "mongodb", "ssh", "sshd", "cups",
        "bluetooth", "networkmanager", "firewalld", "libvirtd", "pipewire",
        "pulseaudio", "avahi", "gdm", "sddm", "lightdm",
    ];

    for service in &services {
        if q.contains(service) {
            return Some(service.to_string());
        }
    }
    None
}

/// v0.0.998: Transform a dialogue step to use team-style messaging
/// This gives the "Hollywood IT teams" experience where users feel like
/// they're watching a team work on their problem.
pub fn team_style_content(step_type: &StepType, content: &str) -> String {
    use crate::team_speak;

    match step_type {
        StepType::IntentClassifying => team_speak::phase_commentary("intent_classify", None),
        StepType::WikiSearch => team_speak::phase_commentary("wiki_search", None),
        StepType::CommandExec => {
            // Transform command into friendly description
            team_speak::describe_command(content)
        }
        StepType::FinalAnswer => content.to_string(), // Keep final answer as-is
        _ => content.to_string(),
    }
}
