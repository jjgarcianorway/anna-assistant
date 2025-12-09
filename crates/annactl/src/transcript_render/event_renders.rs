//! Individual event rendering helpers (v0.0.179).

use anna_shared::ui::colors;

use super::helpers::truncate;

pub fn render_probe_end(probe_id: &str, exit_code: i32, timing_ms: u64, stdout_preview: Option<&str>) {
    let status = if exit_code == 0 {
        format!("{}ok{}", colors::OK, colors::RESET)
    } else {
        format!("{}exit {}{}", colors::WARN, exit_code, colors::RESET)
    };
    let preview = stdout_preview
        .map(|s| format!(" \"{}\"", truncate(s, 40)))
        .unwrap_or_default();
    println!("{} {} ({}ms){}", probe_id, status, timing_ms, preview);
}

pub fn render_ticket_created(ticket_id: &str, domain: &str, intent: &str, evidence_required: bool) {
    println!(
        "\n{}[ticket]{} {} created (domain={}, intent={}, evidence={})",
        colors::CYAN,
        colors::RESET,
        &ticket_id[..8.min(ticket_id.len())],
        domain,
        intent,
        if evidence_required { "required" } else { "optional" }
    );
}

pub fn render_junior_review(attempt: u8, score: u8, verified: bool, issues: &[String]) {
    let status = if verified {
        format!("{}verified{}", colors::OK, colors::RESET)
    } else {
        format!("{}needs revision{}", colors::WARN, colors::RESET)
    };
    println!(
        "\n{}[junior]{} attempt {} -> {} (score={})",
        colors::CYAN,
        colors::RESET,
        attempt,
        status,
        score
    );
    if !issues.is_empty() && !verified {
        println!(
            "{}  issues: {}{}",
            colors::DIM,
            issues.join(", "),
            colors::RESET
        );
    }
}

pub fn render_senior_escalation(successful: bool, reason: Option<&str>) {
    let status = if successful {
        format!("{}provided guidance{}", colors::OK, colors::RESET)
    } else {
        format!("{}could not help{}", colors::WARN, colors::RESET)
    };
    println!(
        "\n{}[senior]{} escalation -> {}",
        colors::WARN,
        colors::RESET,
        status
    );
    if let Some(r) = reason {
        println!("{}  reason: {}{}", colors::DIM, r, colors::RESET);
    }
}

pub fn render_revision_applied(changes_made: &[String]) {
    if !changes_made.is_empty() {
        println!(
            "{}[revision]{} {} change{}",
            colors::DIM,
            colors::RESET,
            changes_made.len(),
            if changes_made.len() == 1 { "" } else { "s" }
        );
        for change in changes_made {
            println!("{}  - {}{}", colors::DIM, change, colors::RESET);
        }
    }
}

pub fn render_review_gate(decision: &str, score: u8, requires_llm: bool) {
    let llm_tag = if requires_llm { " [needs LLM]" } else { "" };
    println!(
        "\n{}[gate]{} {} (score={}){}",
        colors::CYAN,
        colors::RESET,
        decision,
        score,
        llm_tag
    );
}

pub fn render_team_review(team: &str, reviewer: &str, decision: &str, issues_count: usize) {
    let issues_str = if issues_count > 0 {
        format!(
            ", {} issue{}",
            issues_count,
            if issues_count == 1 { "" } else { "s" }
        )
    } else {
        String::new()
    };
    println!(
        "{}[{}/{}]{} {}{}",
        colors::CYAN,
        team,
        reviewer,
        colors::RESET,
        decision,
        issues_str
    );
}

pub fn render_clarification_asked(prompt: &str, choices: &[String], reason: &str) {
    println!("\n{}[clarify]{} {}", colors::WARN, colors::RESET, prompt);
    if !choices.is_empty() {
        println!(
            "{}  options: {}{}",
            colors::DIM,
            choices.join(", "),
            colors::RESET
        );
    }
    println!("{}  ({}){}", colors::DIM, reason, colors::RESET);
}

pub fn render_clarification_verified(verified: bool, source: &str, alternatives: &[String]) {
    if verified {
        println!(
            "{}[verify]{} {}confirmed{} ({})",
            colors::DIM,
            colors::RESET,
            colors::OK,
            colors::RESET,
            source
        );
    } else {
        println!(
            "{}[verify]{} {}not found{} ({})",
            colors::DIM,
            colors::RESET,
            colors::WARN,
            colors::RESET,
            source
        );
        if !alternatives.is_empty() {
            println!(
                "{}  alternatives: {}{}",
                colors::DIM,
                alternatives.join(", "),
                colors::RESET
            );
        }
    }
}

pub fn render_fast_path(handled: bool, class: &str, reason: &str, probes_needed: bool) {
    if handled {
        println!(
            "{}[fast]{} {} {} (no LLM needed)",
            colors::OK,
            colors::RESET,
            class,
            if probes_needed { "(probes run)" } else { "(cached)" }
        );
    } else {
        println!("{}[fast]{} skipped: {}", colors::DIM, colors::RESET, reason);
    }
}

pub fn render_evidence_summary(evidence_kinds: &[String], probe_count: usize, key_findings: &[String]) {
    println!(
        "{}[evidence]{} {} probe{}, kinds: {:?}",
        colors::DIM,
        colors::RESET,
        probe_count,
        if probe_count == 1 { "" } else { "s" },
        evidence_kinds
    );
    if !key_findings.is_empty() {
        for finding in key_findings {
            println!("{}  - {}{}", colors::DIM, finding, colors::RESET);
        }
    }
}

pub fn render_proposed_action(action_id: &str, description: &str, risk_level: &str, rollback_available: bool) {
    let risk_color = match risk_level {
        "high" => colors::ERR,
        "medium" => colors::WARN,
        _ => colors::OK,
    };
    println!(
        "\n{}[action]{} {} (risk: {}{}{})",
        colors::WARN,
        colors::RESET,
        &action_id[..8.min(action_id.len())],
        risk_color,
        risk_level,
        colors::RESET
    );
    println!("{}  {}{}", colors::DIM, description, colors::RESET);
    if rollback_available {
        println!("{}  rollback: available{}", colors::DIM, colors::RESET);
    }
}

pub fn render_action_confirmation(prompt: &str, options: &[String]) {
    println!("{}[confirm]{} {}", colors::WARN, colors::RESET, prompt);
    if !options.is_empty() {
        println!(
            "{}  options: {}{}",
            colors::DIM,
            options.join(", "),
            colors::RESET
        );
    }
}
