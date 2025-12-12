//! Deterministic-First Policy (v0.0.445).
//!
//! Before involving ANY LLM, check if deterministic probes can answer.
//!
//! LLMs are ONLY allowed for:
//! - Interpretation
//! - Diagnosis
//! - Explanation
//! - Multi-step reasoning
//!
//! This is a HARD routing rule.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Deterministic route types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeterministicRoute {
    /// Pure deterministic - probes only, no LLM
    ProbesOnly,
    /// Deterministic with formatting - probes + LLM for presentation
    ProbesWithFormat,
    /// Requires LLM - interpretation, diagnosis, or explanation
    RequiresLlm,
}

/// Query domain for deterministic routing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QueryDomain {
    Ram,
    Cpu,
    Uptime,
    Disk,
    Services,
    Packages,
    Swap,
    Kernel,
    Desktop,
    Network,
    Users,
    Processes,
}

impl QueryDomain {
    /// Parse domain from query.
    pub fn from_query(query: &str) -> Option<Self> {
        let q = query.to_lowercase();

        // RAM/Memory
        if q.contains("ram")
            || q.contains("memory")
            || q.contains("free mem")
            || q.contains("available mem")
        {
            return Some(Self::Ram);
        }

        // CPU
        if q.contains("cpu") || q.contains("processor") || q.contains("cores") {
            return Some(Self::Cpu);
        }

        // Uptime
        if q.contains("uptime") || q.contains("how long") && q.contains("running") {
            return Some(Self::Uptime);
        }

        // Disk
        if q.contains("disk")
            || q.contains("storage")
            || q.contains("filesystem")
            || q.contains("mount")
            || q.contains("partition")
        {
            return Some(Self::Disk);
        }

        // Services
        if q.contains("service")
            || q.contains("daemon")
            || q.contains("systemd")
            || q.contains("unit")
        {
            return Some(Self::Services);
        }

        // Packages
        if q.contains("package")
            || q.contains("installed")
            || q.contains("install")
            || q.contains("pacman")
            || q.contains("apt")
            || q.contains("yum")
        {
            return Some(Self::Packages);
        }

        // Swap
        if q.contains("swap") {
            return Some(Self::Swap);
        }

        // Kernel
        if q.contains("kernel") || q.contains("linux version") || q.contains("uname") {
            return Some(Self::Kernel);
        }

        // Desktop
        if q.contains("desktop")
            || q.contains("de")
            || q.contains("window manager")
            || q.contains("wm")
            || q.contains("gnome")
            || q.contains("kde")
            || q.contains("i3")
            || q.contains("sway")
        {
            return Some(Self::Desktop);
        }

        // Network
        if q.contains("network")
            || q.contains("ip address")
            || q.contains("interface")
            || q.contains("ethernet")
            || q.contains("wifi")
        {
            return Some(Self::Network);
        }

        // Users
        if q.contains("user") || q.contains("logged in") || q.contains("who") {
            return Some(Self::Users);
        }

        // Processes
        if q.contains("process") || q.contains("pid") || q.contains("running program") {
            return Some(Self::Processes);
        }

        None
    }

    /// Get probes for this domain.
    pub fn probes(&self) -> Vec<&'static str> {
        match self {
            Self::Ram => vec!["free -h", "cat /proc/meminfo | head -5"],
            Self::Cpu => vec!["lscpu | head -20", "nproc"],
            Self::Uptime => vec!["uptime -p", "uptime"],
            Self::Disk => vec!["df -h", "lsblk -f"],
            Self::Services => vec!["systemctl list-units --failed", "systemctl --user list-units --failed"],
            Self::Packages => vec!["pacman -Q 2>/dev/null | wc -l || dpkg -l 2>/dev/null | wc -l"],
            Self::Swap => vec!["swapon --show", "free -h | grep Swap"],
            Self::Kernel => vec!["uname -r", "uname -a"],
            Self::Desktop => vec!["echo $XDG_CURRENT_DESKTOP", "echo $DESKTOP_SESSION"],
            Self::Network => vec!["ip -br addr", "ip route | head -5"],
            Self::Users => vec!["who", "users"],
            Self::Processes => vec!["ps aux --sort=-%mem | head -10"],
        }
    }
}

/// Deterministic routing policy.
#[derive(Debug, Clone, Default)]
pub struct DeterministicPolicy {
    /// Domain to route mapping
    routes: HashMap<String, DeterministicRoute>,
}

impl DeterministicPolicy {
    /// Create policy with default routes.
    pub fn new() -> Self {
        let mut routes = HashMap::new();

        // Pure deterministic queries (no LLM needed)
        let pure_deterministic = [
            "how much ram",
            "how much memory",
            "free memory",
            "available memory",
            "uptime",
            "how long running",
            "kernel version",
            "linux version",
            "do i have swap",
            "is swap enabled",
            "swap size",
            "cpu cores",
            "how many cores",
            "disk usage",
            "disk space",
            "current desktop",
            "what de",
            "my ip",
            "ip address",
            "logged in users",
        ];

        for pattern in pure_deterministic {
            routes.insert(pattern.to_string(), DeterministicRoute::ProbesOnly);
        }

        // Probes with formatting (need LLM to present nicely)
        let probes_with_format = [
            "show services",
            "list failed services",
            "top processes",
            "memory usage by process",
            "installed packages",
        ];

        for pattern in probes_with_format {
            routes.insert(pattern.to_string(), DeterministicRoute::ProbesWithFormat);
        }

        Self { routes }
    }

    /// Determine route for a query.
    pub fn route(&self, query: &str) -> DeterministicRoute {
        let q = query.to_lowercase();

        // FIRST: Check for explanation/diagnosis keywords (need LLM)
        // This takes priority over domain detection
        // Note: "what is my X" is a simple query, "what is X" (conceptual) needs LLM
        let needs_explanation = q.contains("why")
            || q.contains("how do i")
            || q.contains("explain")
            || q.contains("help me")
            || q.starts_with("fix ")
            || q.contains("troubleshoot")
            || q.contains("diagnose")
            || q.contains("how does")
            || (q.contains("what is ") && !q.contains("what is my") && !q.contains("what is the"));

        if needs_explanation {
            return DeterministicRoute::RequiresLlm;
        }

        // Check for exact pattern matches
        for (pattern, route) in &self.routes {
            if q.contains(pattern) {
                return *route;
            }
        }

        // Check query domain
        if let Some(domain) = QueryDomain::from_query(query) {
            // These domains are fully deterministic for simple queries
            match domain {
                QueryDomain::Ram
                | QueryDomain::Uptime
                | QueryDomain::Swap
                | QueryDomain::Kernel
                | QueryDomain::Desktop => return DeterministicRoute::ProbesOnly,

                // These need formatting
                QueryDomain::Cpu
                | QueryDomain::Disk
                | QueryDomain::Network
                | QueryDomain::Users => return DeterministicRoute::ProbesWithFormat,

                // These often need interpretation
                QueryDomain::Services | QueryDomain::Packages | QueryDomain::Processes => {
                    // Simple checks are deterministic
                    if is_simple_check(&q) {
                        return DeterministicRoute::ProbesOnly;
                    }
                    return DeterministicRoute::ProbesWithFormat;
                }
            }
        }

        // Default: requires LLM for safety
        DeterministicRoute::RequiresLlm
    }

    /// Get probes for a deterministic route.
    pub fn get_probes(&self, query: &str) -> Option<Vec<String>> {
        let route = self.route(query);

        if route == DeterministicRoute::RequiresLlm {
            return None;
        }

        if let Some(domain) = QueryDomain::from_query(query) {
            return Some(domain.probes().iter().map(|s| s.to_string()).collect());
        }

        None
    }

    /// Check if query can be answered without LLM.
    pub fn can_skip_llm(&self, query: &str) -> bool {
        self.route(query) == DeterministicRoute::ProbesOnly
    }
}

/// Check if query is a simple yes/no check.
fn is_simple_check(query: &str) -> bool {
    query.starts_with("is ")
        || query.starts_with("do i have")
        || query.starts_with("does ")
        || query.contains(" running")
        || query.contains(" installed")
        || query.contains(" enabled")
}

/// Format deterministic answer from probe output.
pub fn format_deterministic_answer(domain: QueryDomain, probe_output: &str) -> String {
    match domain {
        QueryDomain::Ram => format_ram_answer(probe_output),
        QueryDomain::Swap => format_swap_answer(probe_output),
        QueryDomain::Uptime => format_uptime_answer(probe_output),
        QueryDomain::Kernel => format_kernel_answer(probe_output),
        QueryDomain::Desktop => format_desktop_answer(probe_output),
        _ => probe_output.trim().to_string(),
    }
}

fn format_ram_answer(output: &str) -> String {
    // Parse free -h output
    for line in output.lines() {
        if line.starts_with("Mem:") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 4 {
                return format!("{} available", parts[6]);
            }
        }
    }
    output.trim().to_string()
}

fn format_swap_answer(output: &str) -> String {
    let trimmed = output.trim();
    if trimmed.is_empty() || trimmed.contains("no swap") {
        "No, swap is not enabled.".to_string()
    } else {
        let first_line = trimmed.lines().next().unwrap_or("");
        if first_line.contains("NAME") {
            // swapon --show header, get next line
            if let Some(data) = trimmed.lines().nth(1) {
                format!("Yes, swap is enabled: {}", data)
            } else {
                "Yes, swap is enabled.".to_string()
            }
        } else {
            format!("Yes, swap is enabled: {}", first_line)
        }
    }
}

fn format_uptime_answer(output: &str) -> String {
    // uptime -p output is already human-readable
    output.trim().to_string()
}

fn format_kernel_answer(output: &str) -> String {
    output.trim().to_string()
}

fn format_desktop_answer(output: &str) -> String {
    let trimmed = output.trim();
    if trimmed.is_empty() {
        "No desktop environment detected (might be running in TTY).".to_string()
    } else {
        trimmed.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pure_deterministic_queries() {
        let policy = DeterministicPolicy::new();

        assert_eq!(
            policy.route("how much ram do I have"),
            DeterministicRoute::ProbesOnly
        );
        assert_eq!(
            policy.route("do I have swap"),
            DeterministicRoute::ProbesOnly
        );
        assert_eq!(
            policy.route("what is my uptime"),
            DeterministicRoute::ProbesOnly
        );
        assert_eq!(
            policy.route("kernel version"),
            DeterministicRoute::ProbesOnly
        );
    }

    #[test]
    fn test_llm_required_queries() {
        let policy = DeterministicPolicy::new();

        assert_eq!(
            policy.route("why is my system slow"),
            DeterministicRoute::RequiresLlm
        );
        assert_eq!(
            policy.route("how do I fix nginx"),
            DeterministicRoute::RequiresLlm
        );
        assert_eq!(
            policy.route("explain systemd"),
            DeterministicRoute::RequiresLlm
        );
    }

    #[test]
    fn test_can_skip_llm() {
        let policy = DeterministicPolicy::new();

        assert!(policy.can_skip_llm("how much free memory"));
        assert!(policy.can_skip_llm("is swap enabled"));
        assert!(!policy.can_skip_llm("why is nginx failing"));
    }

    #[test]
    fn test_get_probes() {
        let policy = DeterministicPolicy::new();

        let probes = policy.get_probes("how much ram");
        assert!(probes.is_some());
        assert!(!probes.unwrap().is_empty());

        let probes = policy.get_probes("explain systemd");
        assert!(probes.is_none());
    }

    #[test]
    fn test_format_swap_answer() {
        assert_eq!(
            format_swap_answer(""),
            "No, swap is not enabled."
        );
        assert!(format_swap_answer("NAME TYPE SIZE\n/dev/sda2 partition 8G")
            .contains("Yes, swap is enabled"));
    }
}
