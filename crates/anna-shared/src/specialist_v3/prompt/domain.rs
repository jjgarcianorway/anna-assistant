//! Domain-specific prompt additions.

/// Domain-specific prompt additions.
pub struct DomainPrompt {
    pub domain: String,
    pub focus_areas: Vec<String>,
    pub common_probes: Vec<String>,
}

impl DomainPrompt {
    /// Desktop specialist prompt additions.
    pub fn desktop() -> Self {
        Self {
            domain: "desktop".to_string(),
            focus_areas: vec![
                "Memory and swap usage".to_string(),
                "Disk space and I/O".to_string(),
                "Display and GPU status".to_string(),
                "Audio/PipeWire/PulseAudio".to_string(),
                "Desktop environment issues".to_string(),
            ],
            common_probes: vec![
                "probe:free".to_string(),
                "probe:df".to_string(),
                "probe:top".to_string(),
                "probe:systemctl_user".to_string(),
            ],
        }
    }

    /// Server specialist prompt additions.
    pub fn server() -> Self {
        Self {
            domain: "server".to_string(),
            focus_areas: vec![
                "Service health and status".to_string(),
                "Resource utilization".to_string(),
                "Log analysis".to_string(),
                "Container status".to_string(),
            ],
            common_probes: vec![
                "probe:systemctl".to_string(),
                "probe:journalctl".to_string(),
                "probe:docker_ps".to_string(),
                "probe:ss".to_string(),
            ],
        }
    }

    /// Network specialist prompt additions.
    pub fn network() -> Self {
        Self {
            domain: "network".to_string(),
            focus_areas: vec![
                "Network interfaces".to_string(),
                "DNS resolution".to_string(),
                "Firewall rules".to_string(),
                "Connection status".to_string(),
            ],
            common_probes: vec![
                "probe:ip_addr".to_string(),
                "probe:ss".to_string(),
                "probe:resolvectl".to_string(),
                "probe:ping".to_string(),
            ],
        }
    }

    /// Security specialist prompt additions.
    pub fn security() -> Self {
        Self {
            domain: "security".to_string(),
            focus_areas: vec![
                "Authentication logs".to_string(),
                "Failed login attempts".to_string(),
                "Firewall status".to_string(),
                "Package integrity".to_string(),
            ],
            common_probes: vec![
                "probe:lastlog".to_string(),
                "probe:journalctl_auth".to_string(),
                "probe:iptables".to_string(),
                "probe:pacman_check".to_string(),
            ],
        }
    }

    /// Get domain prompt by name.
    pub fn for_domain(domain: &str) -> Self {
        match domain {
            "desktop" => Self::desktop(),
            "server" => Self::server(),
            "network" => Self::network(),
            "security" => Self::security(),
            _ => Self::desktop(), // Default to desktop
        }
    }

    /// Add domain context to prompt.
    pub fn augment_prompt(&self, base_prompt: &str) -> String {
        format!(
            "{}\n\nDOMAIN FOCUS: {}\nKEY AREAS: {}\nCOMMON PROBES: {}",
            base_prompt,
            self.domain,
            self.focus_areas.join(", "),
            self.common_probes.join(", ")
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_domain_prompts() {
        let desktop = DomainPrompt::desktop();
        assert_eq!(desktop.domain, "desktop");
        assert!(!desktop.focus_areas.is_empty());

        let server = DomainPrompt::server();
        assert_eq!(server.domain, "server");
    }
}
