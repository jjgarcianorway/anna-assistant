//! Plan extraction and policy logic.

use crate::recipe_schema::{ConfirmationPolicy, PlanStep, SuccessCriteria};
use regex::Regex;
use super::types::{FileEditType, TicketData};

/// Extract plan steps from ticket data.
pub fn extract_plan(data: &TicketData) -> Vec<PlanStep> {
    let mut plan = Vec::new();

    // Add explanation if we have a summary
    if let Some(summary) = &data.eligibility.specialist_summary {
        if !summary.is_empty() {
            plan.push(PlanStep::Explain {
                message: summary.clone(),
            });
        }
    }

    // Add file edits with backups
    for edit in &data.file_edits {
        // Backup first for mutating edits
        if edit.edit_type != FileEditType::WriteFile {
            plan.push(PlanStep::BackupFile {
                path: edit.path.clone(),
            });
        }

        match edit.edit_type {
            FileEditType::AppendLine => {
                if let Some(content) = &edit.content {
                    plan.push(PlanStep::AppendLine {
                        path: edit.path.clone(),
                        line: content.clone(),
                    });
                }
            }
            FileEditType::PrependLine => {
                if let Some(content) = &edit.content {
                    plan.push(PlanStep::PrependLine {
                        path: edit.path.clone(),
                        line: content.clone(),
                    });
                }
            }
            FileEditType::ReplaceLine => {
                if let (Some(pattern), Some(content)) = (&edit.pattern, &edit.content) {
                    plan.push(PlanStep::ReplaceLine {
                        path: edit.path.clone(),
                        pattern: pattern.clone(),
                        replacement: content.clone(),
                    });
                }
            }
            FileEditType::EnsureLine => {
                if let Some(content) = &edit.content {
                    plan.push(PlanStep::EnsureLine {
                        path: edit.path.clone(),
                        line: content.clone(),
                    });
                }
            }
            FileEditType::RemoveLines => {
                if let Some(pattern) = &edit.pattern {
                    plan.push(PlanStep::RemoveLines {
                        path: edit.path.clone(),
                        pattern: pattern.clone(),
                    });
                }
            }
            FileEditType::WriteFile => {
                if let Some(content) = &edit.content {
                    plan.push(PlanStep::WriteFile {
                        path: edit.path.clone(),
                        content: content.clone(),
                        mode: None,
                    });
                }
            }
        }
    }

    // Add commands
    for cmd in &data.commands {
        if cmd.is_verification {
            plan.push(PlanStep::VerifyCommand {
                command: cmd.command.clone(),
                expect_success: true,
            });
        } else if let Some(service) = extract_systemctl_service(&cmd.command) {
            // Convert systemctl commands to service steps
            if cmd.command.contains("enable") {
                plan.push(PlanStep::EnableService {
                    service,
                    start: cmd.command.contains("--now"),
                });
            } else if cmd.command.contains("disable") {
                plan.push(PlanStep::DisableService {
                    service,
                    stop: cmd.command.contains("--now"),
                });
            } else if cmd.command.contains("restart") {
                plan.push(PlanStep::RestartService { service });
            }
        } else {
            plan.push(PlanStep::RunCommand {
                command: cmd.command.clone(),
                description: cmd.description.clone().unwrap_or_default(),
                rollback_command: None,
            });
        }
    }

    plan
}

fn extract_systemctl_service(command: &str) -> Option<String> {
    let re = Regex::new(r"systemctl\s+(?:enable|disable|start|stop|restart)\s+(\S+)").ok()?;
    re.captures(command)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().to_string())
}

/// Determine confirmation policy based on plan steps.
pub fn determine_confirmation_policy(plan: &[PlanStep]) -> ConfirmationPolicy {
    let has_mutating = plan.iter().any(|s| s.is_mutating());
    if has_mutating {
        ConfirmationPolicy::MutatingOnly
    } else {
        ConfirmationPolicy::Never
    }
}

/// Build success criteria from plan.
pub fn build_success_criteria(plan: &[PlanStep]) -> SuccessCriteria {
    let must_succeed: Vec<String> = plan
        .iter()
        .filter(|s| s.is_mutating())
        .map(|s| s.type_name().to_string())
        .collect();

    SuccessCriteria {
        must_succeed,
        rollback_on_failure: true,
        post_verification: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_systemctl_extraction() {
        let service = extract_systemctl_service("systemctl enable sshd");
        assert_eq!(service, Some("sshd".into()));

        let service = extract_systemctl_service("systemctl restart nginx.service");
        assert_eq!(service, Some("nginx.service".into()));
    }
}
