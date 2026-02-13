//! Temporal task handling: background monitoring for a specified duration.
//! v0.3.162: Enables "capture network traffic for 20 minutes" type requests.

use anna_shared::rpc::{AskResult, DialogueStep, StepType};
use anyhow::Result;
use tracing::info;

/// Handle temporal tasks (background monitoring for X duration).
/// v0.3.162: Enables "capture network traffic for 20 minutes" type requests.
pub async fn handle_temporal_task(model: &str, question: &str, duration_secs: u64) -> Result<AskResult> {
    info!("Handling temporal task: {} for {}s", question, duration_secs);

    let mut dialogue = vec![
        DialogueStep {
            step_type: StepType::UserQuestion,
            content: question.to_string(),
        },
    ];

    // Use universal handler to figure out HOW to do the monitoring
    let monitoring_setup = crate::universal_handler::handle_universal_task(model, question).await?;

    dialogue.push(DialogueStep {
        step_type: StepType::InvestigationProbe,
        content: format!("Setting up {} minute monitoring...", duration_secs / 60),
    });

    // Extract the commands from universal handler output
    // Parse the execution plan to get start/stop commands
    let start_cmd = extract_monitoring_command(&monitoring_setup);

    // Start the temporal task
    let task = crate::temporal_tasks::start_temporal_task(
        question.to_string(),
        start_cmd.clone(),
        None, // Stop command if needed
        duration_secs,
    )
    .await?;

    let answer = format!(
        "Started monitoring task (ID: {}). Will run for {} minutes and report back.\n\nTo check progress: annactl \"check task {}\"",
        task.id,
        duration_secs / 60,
        task.id
    );

    dialogue.push(DialogueStep {
        step_type: StepType::FinalAnswer,
        content: answer.clone(),
    });

    Ok(AskResult {
        answer,
        success: true,
        iterations: 1,
        commands_executed: vec![start_cmd],
        dialogue,
        needs_clarification: false,
        clarification_question: None,
        cached: false,
        citations: vec![],
        abstained: false,
        final_confidence: Some(0.8),
    })
}

/// Extract monitoring command from universal handler output.
pub fn extract_monitoring_command(output: &str) -> String {
    // Look for "Step 1:" or first command in output
    for line in output.lines() {
        if line.contains("Step 1:") || line.starts_with("1.") {
            // Extract command after colon or number
            let cmd = line
                .split_once(':')
                .map(|(_, cmd)| cmd.trim())
                .unwrap_or(line.trim());
            return cmd.to_string();
        }
    }
    // Fallback: use whole output as command (probably wrong but better than nothing)
    output.lines().next().unwrap_or("echo 'monitoring'").to_string()
}

/// Extract task type from question for strategy learning.
pub fn extract_task_type(question: &str) -> String {
    let q = question.to_lowercase();

    if q.contains("install") || q.contains("package") || q.contains("pacman") || q.contains("yay") {
        "package_management".to_string()
    } else if q.contains("network") || q.contains("wifi") || q.contains("ethernet") || q.contains("ip") {
        "network_configuration".to_string()
    } else if q.contains("service") || q.contains("systemctl") || q.contains("daemon") {
        "service_management".to_string()
    } else if q.contains("disk") || q.contains("partition") || q.contains("mount") || q.contains("filesystem") {
        "disk_management".to_string()
    } else if q.contains("user") || q.contains("permission") || q.contains("sudo") || q.contains("group") {
        "user_management".to_string()
    } else if q.contains("config") || q.contains("configure") || q.contains("setting") {
        "system_configuration".to_string()
    } else if q.contains("error") || q.contains("fix") || q.contains("broken") || q.contains("fail") {
        "troubleshooting".to_string()
    } else if q.contains("monitor") || q.contains("status") || q.contains("check") {
        "monitoring".to_string()
    } else {
        "general_task".to_string()
    }
}
