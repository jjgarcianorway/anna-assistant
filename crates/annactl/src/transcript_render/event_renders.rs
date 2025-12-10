//! Individual event rendering helpers (v0.0.337).
//! v0.0.179: Initial implementation.
//! v0.0.303: Removed truncation - show full output in debug mode.
//! v0.0.337: Use centralized UI printing for consistency.

use anna_shared::ui::{colors, print_hint, print_label};

pub fn render_probe_end(probe_id: &str, exit_code: i32, timing_ms: u64, stdout_preview: Option<&str>) {
    let status = if exit_code == 0 {
        format!("{}ok{}", colors::OK, colors::RESET)
    } else {
        format!("{}exit {}{}", colors::WARN, exit_code, colors::RESET)
    };
    // v0.0.303: Show full output - no truncation in debug mode
    let preview = stdout_preview
        .map(|s| format!(" \"{}\"", s))
        .unwrap_or_default();
    println!("{} {} ({}ms){}", probe_id, status, timing_ms, preview);
}

pub fn render_ticket_created(ticket_id: &str, domain: &str, intent: &str, evidence_required: bool) {
    println!();
    print_label(
        "ticket",
        &format!(
            "{} created (domain={}, intent={}, evidence={})",
            &ticket_id[..8.min(ticket_id.len())],
            domain,
            intent,
            if evidence_required { "required" } else { "optional" }
        ),
        colors::CYAN,
    );
}

pub fn render_junior_review(attempt: u8, score: u8, verified: bool, issues: &[String]) {
    let status = if verified {
        format!("{}verified{}", colors::OK, colors::RESET)
    } else {
        format!("{}needs revision{}", colors::WARN, colors::RESET)
    };
    println!();
    print_label("junior", &format!("attempt {} -> {} (score={})", attempt, status, score), colors::CYAN);
    if !issues.is_empty() && !verified {
        print_hint(&format!("issues: {}", issues.join(", ")));
    }
}

pub fn render_senior_escalation(successful: bool, reason: Option<&str>) {
    let status = if successful {
        format!("{}provided guidance{}", colors::OK, colors::RESET)
    } else {
        format!("{}could not help{}", colors::WARN, colors::RESET)
    };
    println!();
    print_label("senior", &format!("escalation -> {}", status), colors::WARN);
    if let Some(r) = reason {
        print_hint(&format!("reason: {}", r));
    }
}

pub fn render_revision_applied(changes_made: &[String]) {
    if !changes_made.is_empty() {
        let plural = if changes_made.len() == 1 { "" } else { "s" };
        print_label("revision", &format!("{} change{}", changes_made.len(), plural), colors::DIM);
        for change in changes_made {
            print_hint(&format!("- {}", change));
        }
    }
}

pub fn render_review_gate(decision: &str, score: u8, requires_llm: bool) {
    let llm_tag = if requires_llm { " [needs LLM]" } else { "" };
    println!();
    print_label("gate", &format!("{} (score={}){}", decision, score, llm_tag), colors::CYAN);
}

pub fn render_team_review(team: &str, reviewer: &str, decision: &str, issues_count: usize) {
    let issues_str = if issues_count > 0 {
        let plural = if issues_count == 1 { "" } else { "s" };
        format!(", {} issue{}", issues_count, plural)
    } else {
        String::new()
    };
    // Use a combined label for team/reviewer
    print_label(&format!("{}/{}", team, reviewer), &format!("{}{}", decision, issues_str), colors::CYAN);
}

pub fn render_clarification_asked(prompt: &str, choices: &[String], reason: &str) {
    println!();
    print_label("clarify", prompt, colors::WARN);
    if !choices.is_empty() {
        print_hint(&format!("options: {}", choices.join(", ")));
    }
    print_hint(&format!("({})", reason));
}

pub fn render_clarification_verified(verified: bool, source: &str, alternatives: &[String]) {
    if verified {
        print_label("verify", &format!("{}confirmed{} ({})", colors::OK, colors::RESET, source), colors::DIM);
    } else {
        print_label("verify", &format!("{}not found{} ({})", colors::WARN, colors::RESET, source), colors::DIM);
        if !alternatives.is_empty() {
            print_hint(&format!("alternatives: {}", alternatives.join(", ")));
        }
    }
}

pub fn render_fast_path(handled: bool, class: &str, reason: &str, probes_needed: bool) {
    if handled {
        let cache_status = if probes_needed { "(probes run)" } else { "(cached)" };
        print_label("fast", &format!("{} {} (no LLM needed)", class, cache_status), colors::OK);
    } else {
        print_label("fast", &format!("skipped: {}", reason), colors::DIM);
    }
}

pub fn render_evidence_summary(evidence_kinds: &[String], probe_count: usize, key_findings: &[String]) {
    let plural = if probe_count == 1 { "" } else { "s" };
    print_label("evidence", &format!("{} probe{}, kinds: {:?}", probe_count, plural, evidence_kinds), colors::DIM);
    for finding in key_findings {
        print_hint(&format!("- {}", finding));
    }
}

pub fn render_proposed_action(action_id: &str, description: &str, risk_level: &str, rollback_available: bool) {
    let risk_color = match risk_level {
        "high" => colors::ERR,
        "medium" => colors::WARN,
        _ => colors::OK,
    };
    println!();
    print_label(
        "action",
        &format!("{} (risk: {}{}{})", &action_id[..8.min(action_id.len())], risk_color, risk_level, colors::RESET),
        colors::WARN,
    );
    print_hint(description);
    if rollback_available {
        print_hint("rollback: available");
    }
}

pub fn render_action_confirmation(prompt: &str, options: &[String]) {
    print_label("confirm", prompt, colors::WARN);
    if !options.is_empty() {
        print_hint(&format!("options: {}", options.join(", ")));
    }
}
