//! Deterministic routing policy.

use super::domain::QueryDomain;
use super::route::DeterministicRoute;
use std::collections::HashMap;

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
}
