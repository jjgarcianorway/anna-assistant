//! Helper functions for query classification
//!
//! Includes intent classification, greeting stripping, and tool name extraction.

use anna_shared::rpc::QueryIntent;

/// v0.0.405: Use new intent types (normalized from legacy)
pub fn classify_intent(q: &str) -> QueryIntent {
    if q.contains("install")
        || q.contains("start")
        || q.contains("stop")
        || q.contains("restart")
        || q.contains("configure")
        || q.contains("enable")
        || q.contains("disable")
        || q.contains("update")
    {
        QueryIntent::Configure // Was Request, now Configure
    } else if q.contains("why")
        || q.contains("debug")
        || q.contains("fix")
        || q.contains("error")
        || q.contains("problem")
        || q.contains("issue")
        || q.contains("not working")
        || q.contains("broken")
    {
        QueryIntent::Diagnose // Was Investigate, now Diagnose
    } else if q.contains("list") || q.contains("show all") || q.contains("what's installed") {
        QueryIntent::List
    } else if q.contains("is running")
        || q.contains("is active")
        || q.contains("is enabled")
        || q.contains("status")
    {
        QueryIntent::CheckStatus
    } else {
        QueryIntent::QueryMetric // Was Question, now QueryMetric
    }
}

pub fn strip_greetings(q: &str) -> String {
    let patterns = [
        "hello",
        "hi ",
        "hey ",
        "good morning",
        "good afternoon",
        "good evening",
        "anna",
        ":)",
        ":(",
        ";)",
        ":d",
        ":p",
        "!",
        "?",
        "…",
        "...",
        "please",
        "can you",
        "could you",
        "would you",
        "tell me",
        "show me",
    ];
    let mut result = q.to_string();
    for p in patterns {
        result = result.replace(p, " ");
    }
    result.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// v0.0.797: Extract tool name from queries like "is nano installed", "do I have vim"
pub fn extract_tool_name_from_query(q: &str) -> Option<String> {
    // Pattern: "is <tool> installed"
    if let Some(pos) = q.find("is ") {
        let rest = &q[pos + 3..];
        if rest.contains(" installed") {
            let tool = rest.split(" installed").next()?.trim();
            if !tool.is_empty() && tool.len() < 30 && !tool.contains(' ') {
                return Some(tool.to_string());
            }
        }
    }

    // Pattern: "do i have <tool>" or "do you have <tool>"
    if q.contains("do i have ") || q.contains("do you have ") {
        let pattern = if q.contains("do i have ") {
            "do i have "
        } else {
            "do you have "
        };
        if let Some(pos) = q.find(pattern) {
            let rest = &q[pos + pattern.len()..];
            let tool = rest.split_whitespace().next()?;
            if !tool.is_empty() && tool.len() < 30 {
                return Some(tool.to_string());
            }
        }
    }

    // Pattern: "have i got <tool>"
    if let Some(pos) = q.find("have i got ") {
        let rest = &q[pos + 11..];
        let tool = rest.split_whitespace().next()?;
        if !tool.is_empty() && tool.len() < 30 {
            return Some(tool.to_string());
        }
    }

    // Pattern: "<tool> installed?"
    if q.ends_with(" installed") || q.ends_with(" installed?") {
        let words: Vec<&str> = q.split_whitespace().collect();
        if words.len() >= 2 {
            let tool_idx = words.len() - 2; // Second to last word (before "installed")
            // Skip "is" if present
            if words.get(tool_idx - 1) != Some(&"is") {
                return Some(words[tool_idx].to_string());
            }
        }
    }

    None
}
