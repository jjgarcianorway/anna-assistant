//! Answer Formatter - Standard answer format for planner-based responses (6.4.x)
//!
//! Enforces the canonical answer structure:
//! 1. Recap of user's request
//! 2. What Anna detected from telemetry
//! 3. Explanation tied to Arch Wiki
//! 4. Command block ending with "Do you want me to run it for you?? y/N"

use crate::orchestrator::{KnowledgeSourceRef, Plan, PlanStepKind, TelemetrySummary};

/// Context for rendering a human-readable answer
pub struct AnswerContext {
    pub user_goal: String,
    pub telemetry_summary: TelemetrySummary,
    pub plan: Plan,
    pub wiki_sources: Vec<KnowledgeSourceRef>,
}

/// Render a human-readable answer following the standard format
///
/// Format:
/// 1. Recap user's request
/// 2. What was detected from telemetry
/// 3. Explanation with Arch Wiki links
/// 4. Command block with "Do you want me to run it for you?? y/N"
pub fn render_human_answer(ctx: &AnswerContext) -> String {
    let mut answer = String::new();

    // 1. Recap of user's request
    answer.push_str(&format_request_recap(&ctx.user_goal, &ctx.telemetry_summary));
    answer.push_str("\n\n");

    // 2. What Anna detected from telemetry
    answer.push_str(&format_telemetry_detection(&ctx.telemetry_summary));
    answer.push_str("\n\n");

    // 3. Explanation tied to Arch Wiki
    answer.push_str(&format_wiki_explanation(&ctx.wiki_sources, &ctx.plan));
    answer.push_str("\n\n");

    // 4. Command block
    answer.push_str(&format_command_block(&ctx.plan));

    answer
}

/// Format the request recap section
fn format_request_recap(user_goal: &str, telemetry: &TelemetrySummary) -> String {
    // Infer what the user is asking about based on telemetry
    if telemetry.dns_suspected_broken {
        format!(
            "You requested help fixing DNS resolution on your Arch system: \"{}\"",
            user_goal
        )
    } else if !telemetry.failed_services.is_empty() {
        let service_names: Vec<_> = telemetry
            .failed_services
            .iter()
            .map(|s| s.name.as_str())
            .collect();
        format!(
            "You requested help with failed systemd service(s): {}. Request: \"{}\"",
            service_names.join(", "),
            user_goal
        )
    } else {
        format!("You asked: \"{}\"", user_goal)
    }
}

/// Format the telemetry detection section
fn format_telemetry_detection(telemetry: &TelemetrySummary) -> String {
    let mut detections = Vec::new();

    // DNS detection
    if telemetry.dns_suspected_broken {
        if telemetry.network_reachable {
            detections.push("- Network is reachable".to_string());
            detections.push("- DNS resolution suspected broken".to_string());
        } else {
            detections.push("- Network is not reachable".to_string());
        }
    }

    // Service detection
    for service in &telemetry.failed_services {
        if service.is_failed {
            detections.push(format!("- Failed service detected: {}", service.name));
        }
    }

    if detections.is_empty() {
        "Anna detected: System appears healthy.".to_string()
    } else {
        format!("Anna detected:\n{}", detections.join("\n"))
    }
}

/// Format the Arch Wiki explanation section
fn format_wiki_explanation(sources: &[KnowledgeSourceRef], plan: &Plan) -> String {
    let mut explanation = String::from("Based on Arch Wiki guidance:\n");

    // List sources
    for source in sources {
        explanation.push_str(&format!("- {}\n", source.url));
    }

    // Explain the plan structure
    let inspect_count = plan
        .steps
        .iter()
        .filter(|s| s.kind == PlanStepKind::Inspect)
        .count();
    let change_count = plan
        .steps
        .iter()
        .filter(|s| s.kind == PlanStepKind::Change)
        .count();

    explanation.push_str("\n");
    if inspect_count > 0 && change_count > 0 {
        explanation.push_str(&format!(
            "This plan follows the safe pattern: inspect first ({} steps), then propose changes ({} steps).\n",
            inspect_count, change_count
        ));
        explanation
            .push_str("All changes require confirmation and have rollback commands if needed.");
    } else if inspect_count > 0 {
        explanation.push_str(&format!(
            "This plan contains {} inspection steps with no system changes.",
            inspect_count
        ));
    } else {
        explanation.push_str("No actions needed for a healthy system.");
    }

    explanation
}

/// Format the command block section
fn format_command_block(plan: &Plan) -> String {
    if plan.steps.is_empty() {
        return "No commands needed. Your system appears healthy.".to_string();
    }

    let mut block = String::from("This is what we need to run:\n");

    for step in &plan.steps {
        block.push_str(&step.command);
        block.push('\n');
    }

    block.push_str("\nDo you want me to run it for you?? y/N");

    block
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::orchestrator::{
        get_arch_help_dns, plan_dns_fix, ServiceStatus, TelemetrySummary,
    };

    #[test]
    fn test_dns_answer_format() {
        let telemetry = TelemetrySummary::dns_issue();
        let wiki = get_arch_help_dns();
        let plan = plan_dns_fix("fix dns", &telemetry, &wiki);

        let ctx = AnswerContext {
            user_goal: "My DNS is broken".to_string(),
            telemetry_summary: telemetry,
            plan,
            wiki_sources: wiki.sources,
        };

        let answer = render_human_answer(&ctx);

        // Must contain key elements
        assert!(
            answer.contains("DNS resolution"),
            "Answer should mention DNS resolution"
        );
        assert!(
            answer.contains("Anna detected"),
            "Answer should have detection section"
        );
        assert!(
            answer.contains("Arch Wiki"),
            "Answer should reference Arch Wiki"
        );
        assert!(
            answer.contains("This is what we need to run:"),
            "Answer should have command block header"
        );
        assert!(
            answer.contains("Do you want me to run it for you?? y/N"),
            "Answer should end with execution prompt"
        );
        assert!(
            answer.contains("systemctl"),
            "Answer should contain actual commands"
        );
    }

    #[test]
    fn test_service_failure_answer_format() {
        let telemetry = TelemetrySummary::with_failed_service("nginx");
        let answer_ctx = AnswerContext {
            user_goal: "nginx keeps crashing".to_string(),
            telemetry_summary: telemetry.clone(),
            plan: Plan { steps: vec![] }, // Empty plan for simplicity
            wiki_sources: vec![],
        };

        let answer = render_human_answer(&answer_ctx);

        assert!(
            answer.contains("nginx"),
            "Answer should mention the failed service"
        );
        assert!(
            answer.contains("Failed service detected: nginx"),
            "Should detect nginx failure"
        );
    }

    #[test]
    fn test_healthy_system_answer() {
        let telemetry = TelemetrySummary::healthy();
        let ctx = AnswerContext {
            user_goal: "check my system".to_string(),
            telemetry_summary: telemetry,
            plan: Plan { steps: vec![] },
            wiki_sources: vec![],
        };

        let answer = render_human_answer(&ctx);

        assert!(
            answer.contains("healthy"),
            "Answer should mention system is healthy"
        );
        assert!(
            answer.contains("No commands needed"),
            "Should say no commands needed"
        );
    }
}
