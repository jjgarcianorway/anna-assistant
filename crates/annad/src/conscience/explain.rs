//! Explanation and reasoning system for conscience layer
//!
//! Phase 1.1: Generate human-readable explanations for decisions
//! Citation: [archwiki:System_maintenance]

use super::types::{ConscienceDecision, DecisionOutcome, ReasoningNode, ReasoningTree};
use chrono::{DateTime, Utc};

/// Format reasoning tree as human-readable text
pub fn format_reasoning_tree(tree: &ReasoningTree, indent: usize) -> String {
    format_reasoning_node(&tree.root, indent)
}

/// Format single reasoning node with children
fn format_reasoning_node(node: &ReasoningNode, indent: usize) -> String {
    let mut output = String::new();
    let prefix = "  ".repeat(indent);

    // Format statement with confidence indicator
    let confidence_emoji = confidence_indicator(node.confidence);
    output.push_str(&format!(
        "{}{} {} (confidence: {:.0}%)\n",
        prefix,
        confidence_emoji,
        node.statement,
        node.confidence * 100.0
    ));

    // Format evidence
    for evidence in &node.evidence {
        output.push_str(&format!("{}  • {}\n", prefix, evidence));
    }

    // Format children
    for child in &node.children {
        output.push('\n');
        output.push_str(&format_reasoning_node(child, indent + 1));
    }

    output
}

/// Get confidence indicator emoji
fn confidence_indicator(confidence: f64) -> &'static str {
    if confidence >= 0.9 {
        "✓"
    } else if confidence >= 0.7 {
        "○"
    } else if confidence >= 0.5 {
        "△"
    } else {
        "!"
    }
}

/// Format conscience decision as detailed report
pub fn explain_decision(decision: &ConscienceDecision) -> String {
    let mut output = String::new();

    output.push_str(&format!(
        "╔═══════════════════════════════════════════════════════════╗\n"
    ));
    output.push_str(&format!(
        "║ CONSCIENCE DECISION REPORT                                ║\n"
    ));
    output.push_str(&format!(
        "╚═══════════════════════════════════════════════════════════╝\n\n"
    ));

    // Decision metadata
    output.push_str(&format!("Decision ID: {}\n", decision.id));
    output.push_str(&format!(
        "Timestamp:   {}\n",
        format_timestamp(&decision.timestamp)
    ));
    output.push_str(&format!("Action:      {:?}\n\n", decision.action));

    // Outcome
    output.push_str(&format!("OUTCOME:\n"));
    match &decision.outcome {
        DecisionOutcome::Approved { execution_result } => {
            output.push_str(&format!("  ✓ APPROVED and executed\n"));
            output.push_str(&format!("  Result: {}\n", execution_result));
        }
        DecisionOutcome::Rejected { reason } => {
            output.push_str(&format!("  ✗ REJECTED\n"));
            output.push_str(&format!("  Reason: {}\n", reason));
        }
        DecisionOutcome::Flagged { reason } => {
            output.push_str(&format!("  ⚠ FLAGGED for manual review\n"));
            output.push_str(&format!("  Reason: {}\n", reason));
        }
        DecisionOutcome::Pending => {
            output.push_str(&format!("  ⏳ PENDING decision\n"));
        }
    }
    output.push('\n');

    // Ethical evaluation
    output.push_str(&format!("ETHICAL EVALUATION:\n"));
    output.push_str(&format!(
        "  Overall Score: {:.1}%\n",
        decision.ethical_score.overall() * 100.0
    ));
    output.push_str(&format!(
        "  Safety:        {:.1}%\n",
        decision.ethical_score.safety * 100.0
    ));
    output.push_str(&format!(
        "  Privacy:       {:.1}%\n",
        decision.ethical_score.privacy * 100.0
    ));
    output.push_str(&format!(
        "  Integrity:     {:.1}%\n",
        decision.ethical_score.integrity * 100.0
    ));
    output.push_str(&format!(
        "  Autonomy:      {:.1}%\n\n",
        decision.ethical_score.autonomy * 100.0
    ));

    // Confidence
    output.push_str(&format!("CONFIDENCE:\n"));
    output.push_str(&format!(
        "  Decision Confidence: {:.1}%\n",
        decision.confidence * 100.0
    ));
    output.push_str(&format!(
        "  Uncertainty:         {:.1}%\n\n",
        (1.0 - decision.confidence) * 100.0
    ));

    // Reasoning tree
    output.push_str(&format!("REASONING:\n"));
    output.push_str(&format_reasoning_tree(&decision.reasoning, 1));
    output.push('\n');

    // Rollback plan if present
    if let Some(plan) = &decision.rollback_plan {
        output.push_str(&format!("ROLLBACK PLAN:\n"));
        output.push_str(&format!("  Plan ID:     {}\n", plan.id));
        output.push_str(&format!("  Description: {}\n", plan.description));
        output.push_str(&format!("  Est. Time:   {}s\n", plan.estimated_time));
        output.push_str(&format!(
            "  Backups:     {} paths\n",
            plan.backup_paths.len()
        ));
        output.push_str(&format!(
            "  Checksums:   {} files\n\n",
            plan.checksums.len()
        ));
    }

    output.push_str(&format!(
        "───────────────────────────────────────────────────────────\n"
    ));

    output
}

/// Format timestamp in human-readable format
fn format_timestamp(timestamp: &DateTime<Utc>) -> String {
    timestamp.format("%Y-%m-%d %H:%M:%S UTC").to_string()
}

/// Create summary of decision for display
pub fn summarize_decision(decision: &ConscienceDecision) -> String {
    let outcome_str = match &decision.outcome {
        DecisionOutcome::Approved { .. } => "✓ Approved",
        DecisionOutcome::Rejected { .. } => "✗ Rejected",
        DecisionOutcome::Flagged { .. } => "⚠ Flagged",
        DecisionOutcome::Pending => "⏳ Pending",
    };

    format!(
        "{} | {} | Ethical: {:.0}% | Confidence: {:.0}% | {:?}",
        decision.id[..8].to_string(),
        outcome_str,
        decision.ethical_score.overall() * 100.0,
        decision.confidence * 100.0,
        decision.action
    )
}

/// Generate ASCII tree visualization of reasoning
pub fn visualize_reasoning_tree(tree: &ReasoningTree) -> String {
    visualize_node(&tree.root, "", true)
}

fn visualize_node(node: &ReasoningNode, prefix: &str, is_last: bool) -> String {
    let mut output = String::new();

    let connector = if is_last { "└─ " } else { "├─ " };
    let confidence_bar = confidence_bar(node.confidence);

    output.push_str(&format!(
        "{}{}{} {} [{}]\n",
        prefix, connector, node.statement, confidence_bar, node.confidence
    ));

    // Add evidence
    let new_prefix = if is_last {
        format!("{}   ", prefix)
    } else {
        format!("{}│  ", prefix)
    };

    for (i, evidence) in node.evidence.iter().enumerate() {
        let is_last_evidence = i == node.evidence.len() - 1 && node.children.is_empty();
        let ev_connector = if is_last_evidence {
            "└─ "
        } else {
            "├─ "
        };
        output.push_str(&format!("{}{}• {}\n", new_prefix, ev_connector, evidence));
    }

    // Add children
    for (i, child) in node.children.iter().enumerate() {
        let is_last_child = i == node.children.len() - 1;
        output.push_str(&visualize_node(child, &new_prefix, is_last_child));
    }

    output
}

/// Create confidence bar visualization
fn confidence_bar(confidence: f64) -> String {
    let filled = (confidence * 10.0) as usize;
    let empty = 10 - filled;

    format!("{}{}", "█".repeat(filled), "░".repeat(empty))
}

/// Format list of pending actions
pub fn format_pending_actions(actions: &[super::types::PendingAction]) -> String {
    if actions.is_empty() {
        return "No pending actions requiring review.\n".to_string();
    }

    let mut output = String::new();

    output.push_str(&format!(
        "╔═══════════════════════════════════════════════════════════╗\n"
    ));
    output.push_str(&format!(
        "║ PENDING ACTIONS REQUIRING REVIEW                          ║\n"
    ));
    output.push_str(&format!(
        "╚═══════════════════════════════════════════════════════════╝\n\n"
    ));

    for (i, action) in actions.iter().enumerate() {
        output.push_str(&format!("{}. ID: {}\n", i + 1, action.id));
        output.push_str(&format!(
            "   Time:        {}\n",
            format_timestamp(&action.timestamp)
        ));
        output.push_str(&format!("   Action:      {:?}\n", action.action));
        output.push_str(&format!(
            "   Uncertainty: {:.1}%\n",
            action.uncertainty * 100.0
        ));
        output.push_str(&format!(
            "   Ethical:     {:.1}%\n",
            action.ethical_score.overall() * 100.0
        ));
        output.push_str(&format!("   Reason:      {}\n", action.flag_reason));
        output.push_str(&format!(
            "   Weakest Dim: {} ({:.1}%)\n\n",
            action.ethical_score.weakest_dimension(),
            action.ethical_score.min_score() * 100.0
        ));
    }

    output.push_str(&format!(
        "Use 'annactl conscience explain <id>' for detailed reasoning.\n"
    ));
    output.push_str(&format!(
        "Use 'annactl conscience approve <id>' to approve an action.\n"
    ));
    output.push_str(&format!(
        "Use 'annactl conscience reject <id>' to reject an action.\n"
    ));

    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::conscience::types::EthicalScore;
    use crate::sentinel::SentinelAction;

    #[test]
    fn test_confidence_indicator() {
        assert_eq!(confidence_indicator(0.95), "✓");
        assert_eq!(confidence_indicator(0.75), "○");
        assert_eq!(confidence_indicator(0.55), "△");
        assert_eq!(confidence_indicator(0.35), "!");
    }

    #[test]
    fn test_format_reasoning_tree() {
        let mut tree = ReasoningTree::new("Test decision".to_string());
        tree.root.evidence.push("Evidence 1".to_string());
        tree.root.children.push(ReasoningNode {
            statement: "Child reasoning".to_string(),
            evidence: vec!["Child evidence".to_string()],
            confidence: 0.8,
            children: vec![],
        });

        let formatted = format_reasoning_tree(&tree, 0);
        assert!(formatted.contains("Test decision"));
        assert!(formatted.contains("Evidence 1"));
        assert!(formatted.contains("Child reasoning"));
    }

    #[test]
    fn test_summarize_decision() {
        let decision = ConscienceDecision {
            id: "test-id-12345678".to_string(),
            timestamp: Utc::now(),
            action: SentinelAction::None,
            reasoning: ReasoningTree::new("Test".to_string()),
            ethical_score: EthicalScore {
                safety: 0.9,
                privacy: 0.9,
                integrity: 0.9,
                autonomy: 0.9,
            },
            confidence: 0.85,
            outcome: DecisionOutcome::Approved {
                execution_result: "Success".to_string(),
            },
            rollback_plan: None,
        };

        let summary = summarize_decision(&decision);
        assert!(summary.contains("test-id-"));
        assert!(summary.contains("Approved"));
    }
}
