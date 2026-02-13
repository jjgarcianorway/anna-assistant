//! Systemd timer + service creation from user intent.
//!
//! Generates unit files via wiki research + LLM. No hardcoded templates.
//! All created units are prefixed `anna-` and tracked in ArtifactRegistry.
//!
//! Flow: user intent → wiki research → LLM generates unit files → pkexec install → registry

use anyhow::{anyhow, Result};
use tracing::{debug, info, warn};

use crate::artifact_registry::{ArtifactKind, ArtifactRegistry, CreatedArtifact};

/// Result of a successful automation creation.
pub struct CreatedAutomation {
    pub service_name: String,
    pub description: String,
    pub service_content: String,
    pub timer_content: String,
    pub service_path: String,
    pub timer_path: String,
}

/// Parse LLM output for unit file content.
/// Expects markers: [SERVICE_FILE], [TIMER_FILE], [DESCRIPTION]
fn parse_llm_unit_output(output: &str) -> Option<(String, String, String)> {
    let service = extract_section(output, "[SERVICE_FILE]", &["[TIMER_FILE]", "[DESCRIPTION]"])?;
    let timer = extract_section(output, "[TIMER_FILE]", &["[DESCRIPTION]", "[SERVICE_FILE]"])?;
    let description = extract_section(output, "[DESCRIPTION]", &["[SERVICE_FILE]", "[TIMER_FILE]"])
        .unwrap_or_else(|| "Automated task created by Anna".to_string());

    if service.trim().is_empty() || timer.trim().is_empty() {
        return None;
    }

    Some((service.trim().to_string(), timer.trim().to_string(), description.trim().to_string()))
}

fn extract_section(text: &str, start_marker: &str, end_markers: &[&str]) -> Option<String> {
    let start_idx = text.find(start_marker)? + start_marker.len();
    let remaining = &text[start_idx..];

    let end_idx = end_markers.iter()
        .filter_map(|m| remaining.find(m))
        .min()
        .unwrap_or(remaining.len());

    Some(remaining[..end_idx].trim().to_string())
}

/// Generate a slug-safe unit name from user intent.
fn intent_to_unit_name(intent: &str) -> String {
    let words: Vec<&str> = intent.split_whitespace()
        .filter(|w| !["the", "a", "an", "my", "to", "in", "from", "for", "and", "or"].contains(w))
        .take(4)
        .collect();
    let slug = words.join("-")
        .chars()
        .map(|c| if c.is_alphanumeric() || c == '-' { c.to_ascii_lowercase() } else { '-' })
        .collect::<String>();

    // Trim multiple dashes
    let re_slug: String = slug.split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-");

    format!("anna-{}", &re_slug[..re_slug.len().min(30)])
}

/// Generate unit files for a user intent using wiki + LLM.
pub async fn generate_unit_files(model: &str, intent: &str) -> Result<CreatedAutomation> {
    // Research: Arch Wiki on systemd timers
    let wiki_timers = anna_shared::wiki::search::keyword_search_text("systemd/Timers", 1200)
        .unwrap_or_default();
    let wiki_services = anna_shared::wiki::search::keyword_search_text("systemd", 600)
        .unwrap_or_default();

    // Get system context
    let real_user = crate::user_context::get_real_user()
        .unwrap_or_else(|_| "root".to_string());
    let home_dir = format!("/home/{}", real_user);

    let prompt = format!(
        "You are generating systemd unit files for an Arch Linux system.\n\
        User: {real_user}\n\
        Home: {home_dir}\n\
        \n\
        USER WANTS: {intent}\n\
        \n\
        Arch Wiki - systemd Timers:\n{wiki_timers}\n\
        \n\
        Arch Wiki - systemd:\n{wiki_services}\n\
        \n\
        Generate a systemd .service and .timer that safely implements this automation.\n\
        Use /bin/bash for ExecStart. Use OnCalendar for the timer schedule.\n\
        Make the service run as User={real_user} if working with user files.\n\
        For system-level operations (root required), omit User= line.\n\
        \n\
        Output EXACTLY in this format (no other text):\n\
        [SERVICE_FILE]\n\
        <full .service unit content>\n\
        [TIMER_FILE]\n\
        <full .timer unit content>\n\
        [DESCRIPTION]\n\
        <one sentence description of what this automation does>",
    );

    let response = crate::ollama::chat_with_timeout(model, &prompt, 45).await
        .map_err(|e| anyhow!("LLM error generating unit files: {}", e))?;

    let (service_content, timer_content, description) =
        parse_llm_unit_output(&response)
            .ok_or_else(|| anyhow!("LLM output did not contain valid unit file markers"))?;

    // Validate that output looks like a systemd unit
    if !service_content.contains("[Unit]") || !timer_content.contains("[Timer]") {
        return Err(anyhow!("LLM generated invalid unit file content"));
    }

    let unit_name = intent_to_unit_name(intent);
    let service_name = format!("{}.service", unit_name);
    let timer_name = format!("{}.timer", unit_name);
    let service_path = format!("/etc/systemd/system/{}", service_name);
    let timer_path = format!("/etc/systemd/system/{}", timer_name);

    debug!("Generated unit files for intent '{}': unit={}", intent, unit_name);

    Ok(CreatedAutomation {
        service_name: unit_name,
        description,
        service_content,
        timer_content,
        service_path,
        timer_path,
    })
}

/// Install generated unit files and enable the timer.
/// Uses pkexec for privilege escalation.
pub fn install_automation(automation: &CreatedAutomation) -> Result<()> {
    use std::process::Command;

    // Write service file via pkexec tee
    let status = Command::new("pkexec")
        .args(["tee", &automation.service_path])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::null())
        .spawn()
        .and_then(|mut child| {
            use std::io::Write;
            if let Some(stdin) = child.stdin.as_mut() {
                stdin.write_all(automation.service_content.as_bytes())?;
            }
            child.wait()
        });

    match status {
        Ok(s) if s.success() => debug!("Wrote {}", automation.service_path),
        Ok(_) | Err(_) => return Err(anyhow!("Failed to write {}", automation.service_path)),
    }

    // Write timer file
    let status = Command::new("pkexec")
        .args(["tee", &automation.timer_path])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::null())
        .spawn()
        .and_then(|mut child| {
            use std::io::Write;
            if let Some(stdin) = child.stdin.as_mut() {
                stdin.write_all(automation.timer_content.as_bytes())?;
            }
            child.wait()
        });

    match status {
        Ok(s) if s.success() => debug!("Wrote {}", automation.timer_path),
        Ok(_) | Err(_) => return Err(anyhow!("Failed to write {}", automation.timer_path)),
    }

    // Reload systemd daemon
    let daemon_reload = Command::new("pkexec")
        .args(["systemctl", "daemon-reload"])
        .status();

    if !daemon_reload.map(|s| s.success()).unwrap_or(false) {
        warn!("daemon-reload failed, trying without pkexec");
        Command::new("systemctl").arg("daemon-reload").status().ok();
    }

    // Enable and start the timer
    let timer_unit = format!("{}.timer", automation.service_name);
    let enable = Command::new("pkexec")
        .args(["systemctl", "enable", "--now", &timer_unit])
        .status();

    if !enable.map(|s| s.success()).unwrap_or(false) {
        return Err(anyhow!("Failed to enable timer {}", timer_unit));
    }

    info!("Installed and enabled automation: {}", automation.service_name);
    Ok(())
}

/// Full flow: generate → install → register.
/// Returns user-readable confirmation message.
pub async fn create_and_register_automation(model: &str, intent: &str) -> Result<String> {
    info!("Creating automation for: {}", intent);

    let automation = generate_unit_files(model, intent).await?;

    // Show the plan before applying
    let plan_summary = format!(
        "Creating automation:\n\
        Name: {}\n\
        Description: {}\n\
        Service: {}\n\
        Timer: {}",
        automation.service_name,
        automation.description,
        automation.service_path,
        automation.timer_path
    );
    debug!("{}", plan_summary);

    // Install (low risk — auto-apply)
    install_automation(&automation)?;

    // Register in artifact registry
    let timer_unit = format!("{}.timer", automation.service_name);
    let service_unit = format!("{}.service", automation.service_name);
    let artifact = CreatedArtifact::new(
        ArtifactKind::SystemdTimer,
        automation.service_name.trim_start_matches("anna-"),
        &automation.description,
        vec![automation.service_path.clone(), automation.timer_path.clone()],
        vec![
            format!("pkexec systemctl disable --now {}", timer_unit),
            format!("pkexec systemctl disable --now {}", service_unit),
            format!("pkexec rm -f /etc/systemd/system/{} /etc/systemd/system/{}", timer_unit, service_unit),
            "pkexec systemctl daemon-reload".into(),
        ],
    ).with_unit(timer_unit);

    let mut registry = ArtifactRegistry::load();
    registry.add(artifact);

    Ok(format!(
        "Automation created and running!\n\n\
        Name: {}\n\
        Description: {}\n\n\
        The timer is now active. Ask me \"what automations have you created?\" to see all active automations.",
        automation.service_name.trim_start_matches("anna-"),
        automation.description,
    ))
}

/// Remove an automation by name query.
pub fn remove_automation(query: &str) -> String {
    use std::process::Command;

    let mut registry = ArtifactRegistry::load();
    let result = registry.remove_by_name(query);

    match result {
        None => format!("No active automation matching '{}' found. Check with \"what automations have you created?\"", query),
        Some((artifact, remove_cmds)) => {
            let mut errors = Vec::new();
            for cmd in &remove_cmds {
                let parts: Vec<&str> = cmd.split_whitespace().collect();
                if parts.is_empty() { continue; }
                let status = Command::new(parts[0]).args(&parts[1..]).status();
                if !status.map(|s| s.success()).unwrap_or(false) {
                    errors.push(cmd.clone());
                }
            }
            if errors.is_empty() {
                format!("Removed '{}' successfully. The automation is no longer running.", artifact.name)
            } else {
                format!(
                    "Removed '{}' from registry, but some cleanup commands failed:\n{}",
                    artifact.name,
                    errors.join("\n")
                )
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_intent_to_unit_name() {
        let name = intent_to_unit_name("delete files in downloads older than 30 days");
        assert!(name.starts_with("anna-"));
        assert!(name.contains("delete") || name.contains("files"));
    }

    #[test]
    fn test_parse_llm_unit_output() {
        let output = "[SERVICE_FILE]\n[Unit]\nDescription=Test\n[Service]\nExecStart=/bin/true\n[TIMER_FILE]\n[Unit]\nDescription=Test timer\n[Timer]\nOnCalendar=daily\n[DESCRIPTION]\nRuns daily cleanup";
        let result = parse_llm_unit_output(output);
        assert!(result.is_some());
        let (service, timer, desc) = result.unwrap();
        assert!(service.contains("[Unit]"));
        assert!(timer.contains("[Timer]"));
        assert!(desc.contains("cleanup"));
    }

    #[test]
    fn test_parse_llm_unit_output_invalid() {
        let result = parse_llm_unit_output("no markers here");
        assert!(result.is_none());
    }
}
