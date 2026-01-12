//! Service management recipes (systemd).
//! v0.0.998: Initial implementation

use crate::changes::run_sudo_command;
use regex::Regex;
use std::collections::HashMap;
use std::sync::RwLock;
use tracing::info;

use super::RecipeResult;

/// Pending service recipes awaiting confirmation
static PENDING_SERVICE: RwLock<Option<HashMap<String, PendingServiceChange>>> = RwLock::new(None);

#[derive(Clone)]
struct PendingServiceChange {
    command: String,
    description: String,
}

/// Try to match a service-related recipe
pub fn try_recipe(q: &str) -> Option<RecipeResult> {
    // Restart service
    if q.contains("restart") {
        if let Some(service) = extract_service_name(q) {
            return Some(offer_restart_service(&service));
        }
    }

    // Start service
    if q.contains("start") && !q.contains("restart") {
        if let Some(service) = extract_service_name(q) {
            return Some(offer_start_service(&service));
        }
    }

    // Stop service
    if q.contains("stop") {
        if let Some(service) = extract_service_name(q) {
            return Some(offer_stop_service(&service));
        }
    }

    // Enable service at startup
    if q.contains("enable") && (q.contains("startup") || q.contains("boot") || q.contains("service")) {
        if let Some(service) = extract_service_name(q) {
            return Some(offer_enable_service(&service));
        }
    }

    // Disable service
    if q.contains("disable") {
        if let Some(service) = extract_service_name(q) {
            return Some(offer_disable_service(&service));
        }
    }

    None
}

fn extract_service_name(text: &str) -> Option<String> {
    // Common service names to look for
    let common_services = [
        "nginx", "apache", "httpd", "mysql", "mariadb", "postgresql", "postgres",
        "docker", "containerd", "redis", "mongodb", "ssh", "sshd", "cups",
        "bluetooth", "networkmanager", "network-manager", "firewalld", "ufw",
        "libvirtd", "pipewire", "pulseaudio", "avahi", "cups", "gdm", "sddm",
        "lightdm", "systemd-resolved", "systemd-timesyncd", "cronie", "cron",
    ];

    let text_lower = text.to_lowercase();

    // First check for common service names
    for service in &common_services {
        if text_lower.contains(service) {
            return Some(service.to_string());
        }
    }

    // Try to extract a service name pattern (word followed by optional .service)
    let re = Regex::new(r"\b([a-z][a-z0-9_-]+)(?:\.service)?\b").ok()?;
    for cap in re.captures_iter(&text_lower) {
        let name = &cap[1];
        // Skip common words that aren't services
        if !["restart", "start", "stop", "enable", "disable", "service", "the", "my", "please"].contains(&name) {
            return Some(name.to_string());
        }
    }

    None
}

fn offer_restart_service(service: &str) -> RecipeResult {
    let cmd = format!("systemctl restart {}", service);

    store_pending(&format!("service-restart-{}", service), PendingServiceChange {
        command: cmd.clone(),
        description: format!("Restart {} service", service),
    });

    RecipeResult {
        success: true,
        message: format!(
            "I'll restart the {} service:\n  sudo {}\n\nThis may briefly interrupt the service.",
            service, cmd
        ),
        needs_confirmation: true,
        confirmation_prompt: Some(format!("Restart {}?", service)),
    }
}

fn offer_start_service(service: &str) -> RecipeResult {
    let cmd = format!("systemctl start {}", service);

    store_pending(&format!("service-start-{}", service), PendingServiceChange {
        command: cmd.clone(),
        description: format!("Start {} service", service),
    });

    RecipeResult {
        success: true,
        message: format!("I'll start the {} service:\n  sudo {}", service, cmd),
        needs_confirmation: true,
        confirmation_prompt: Some(format!("Start {}?", service)),
    }
}

fn offer_stop_service(service: &str) -> RecipeResult {
    let cmd = format!("systemctl stop {}", service);

    store_pending(&format!("service-stop-{}", service), PendingServiceChange {
        command: cmd.clone(),
        description: format!("Stop {} service", service),
    });

    RecipeResult {
        success: true,
        message: format!(
            "I'll stop the {} service:\n  sudo {}\n\nThe service will not restart until you start it again.",
            service, cmd
        ),
        needs_confirmation: true,
        confirmation_prompt: Some(format!("Stop {}?", service)),
    }
}

fn offer_enable_service(service: &str) -> RecipeResult {
    let cmd = format!("systemctl enable {}", service);

    store_pending(&format!("service-enable-{}", service), PendingServiceChange {
        command: cmd.clone(),
        description: format!("Enable {} at startup", service),
    });

    RecipeResult {
        success: true,
        message: format!(
            "I'll enable {} to start automatically at boot:\n  sudo {}\n\nThe service will start on next boot (or you can start it now).",
            service, cmd
        ),
        needs_confirmation: true,
        confirmation_prompt: Some(format!("Enable {} at startup?", service)),
    }
}

fn offer_disable_service(service: &str) -> RecipeResult {
    let cmd = format!("systemctl disable {}", service);

    store_pending(&format!("service-disable-{}", service), PendingServiceChange {
        command: cmd.clone(),
        description: format!("Disable {} from startup", service),
    });

    RecipeResult {
        success: true,
        message: format!(
            "I'll disable {} from starting at boot:\n  sudo {}\n\nThe service will keep running until you stop it.",
            service, cmd
        ),
        needs_confirmation: true,
        confirmation_prompt: Some(format!("Disable {}?", service)),
    }
}

fn store_pending(id: &str, change: PendingServiceChange) {
    if let Ok(mut guard) = PENDING_SERVICE.write() {
        let map = guard.get_or_insert_with(HashMap::new);
        map.insert(id.to_string(), change);
    }
}

fn take_pending(id: &str) -> Option<PendingServiceChange> {
    if let Ok(mut guard) = PENDING_SERVICE.write() {
        if let Some(map) = guard.as_mut() {
            return map.remove(id);
        }
    }
    None
}

/// Execute a confirmed service recipe
pub fn execute_confirmed(recipe_id: &str) -> RecipeResult {
    let pending = match take_pending(recipe_id) {
        Some(p) => p,
        None => {
            return RecipeResult {
                success: false,
                message: "Recipe expired or not found. Please try again.".to_string(),
                needs_confirmation: false,
                confirmation_prompt: None,
            };
        }
    };

    match run_sudo_command(&pending.command) {
        Ok(output) => {
            info!("Executed service recipe: {}", recipe_id);
            let msg = if output.is_empty() {
                format!("Done! {}", pending.description)
            } else {
                format!("Done! {}\n\n{}", pending.description, output)
            };
            RecipeResult {
                success: true,
                message: msg,
                needs_confirmation: false,
                confirmation_prompt: None,
            }
        }
        Err(e) => RecipeResult {
            success: false,
            message: format!("Failed: {}\n\nYou might need to check if the service exists with: systemctl status {}", e, recipe_id.split('-').last().unwrap_or("service")),
            needs_confirmation: false,
            confirmation_prompt: None,
        },
    }
}
