//! Core query classification patterns (v0.0.174).
//!
//! Help, meta/small-talk, system triage, health summary.

use crate::router::QueryClass;

/// Classify core queries: help, meta, triage, health summary.
/// Returns Some if matched, None otherwise.
pub fn classify_core(q: &str, stripped: &str) -> Option<QueryClass> {
    // Help request (check first as it's specific)
    if q.trim() == "help" || q.contains("what can you do") || q.contains("how do i use") {
        return Some(QueryClass::Help);
    }

    // v0.0.77: Meta/small-talk - bypass LLM entirely
    if q == "how are you"
        || q == "how are you?"
        || q.starts_with("how are you ")
        || q.contains("what is your name")
        || q.contains("what's your name")
        || q.contains("who are you")
        || q.contains("are you ok")
        || q.contains("are you okay")
        || q.contains("are you using llm")
        || q.contains("are you an ai")
        || q.contains("are you a bot")
        || q.contains("are you human")
        || q.contains("are you real")
        || q == "hello"
        || q == "hi"
        || q == "hey"
        || q == "thanks"
        || q == "thank you"
        || q == "good morning"
        || q == "good afternoon"
        || q == "good evening"
    {
        return Some(QueryClass::MetaSmallTalk);
    }

    // v0.0.77: Kernel version - "kernel version", "uname", "linux version"
    if q.contains("kernel version")
        || q.contains("kernel release")
        || q == "uname"
        || q == "uname -a"
        || q.contains("linux version")
        || q.contains("what kernel")
    {
        return Some(QueryClass::KernelVersion);
    }

    // v0.0.77: Config file location - "where is vim config", "hyprland config path"
    let config_location_query = (q.contains("where is")
        || q.contains("where's")
        || q.contains("path to")
        || q.contains("location of")
        || q.contains("find the"))
        && q.contains("config");
    let specific_config_query = (q.contains("vim")
        || q.contains("nvim")
        || q.contains("hyprland")
        || q.contains("sway")
        || q.contains("alacritty")
        || q.contains("kitty")
        || q.contains("bash")
        || q.contains("zsh")
        || q.contains("fish"))
        && (q.contains("config") || q.contains("rc file") || q.contains("dotfile"));
    if config_location_query || specific_config_query {
        return Some(QueryClass::ConfigFileLocation);
    }

    // SystemTriage (FAST PATH): error/warning focused queries
    if stripped.contains("any errors")
        || stripped.contains("any problems")
        || stripped.contains("any issues")
        || stripped.contains("any warnings")
        || stripped.contains("errors so far")
        || stripped.contains("problems so far")
        || stripped.contains("what's wrong")
        || stripped.contains("whats wrong")
        || stripped.contains("is everything ok")
        || stripped.contains("is everything okay")
        || stripped.contains("how is my computer")
        || stripped.contains("how's my computer")
        || stripped.contains("computer doing")
        || q.contains("health")
        || q.trim() == "errors"
        || q.trim() == "warnings"
        || q.trim() == "problems"
        || q.trim() == "status"
        || q.trim() == "health"
    {
        return Some(QueryClass::SystemTriage);
    }

    // System health summary: FULL system overview
    if q.contains("summary")
        || q.contains("status report")
        || q.contains("overview")
        || q.contains("full report")
        || q.contains("system status")
        || stripped.contains("how is the system")
        || stripped.contains("how's the system")
        || stripped.contains("check my system")
        || stripped.contains("check the system")
        || stripped.contains("system check")
        || q.trim() == "report"
    {
        return Some(QueryClass::SystemHealthSummary);
    }

    // System slow (multi-probe diagnostic)
    // v0.0.799: Exclude boot-related slow queries - handled by BootBlame in classify_system
    let is_boot_slow = q.contains("boot") || q.contains("startup") || q.contains("bootup");
    if (q.contains("slow") || q.contains("sluggish") || q.contains("laggy")) && !is_boot_slow {
        return Some(QueryClass::SystemSlow);
    }

    None
}
