//! Service Graph - Systemd units, dependencies, failure states, log signatures.
//!
//! Models the service dependency graph for understanding:
//! - What depends on what
//! - Failure cascades
//! - Log patterns for diagnosis

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Complete service dependency graph
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ServiceGraph {
    /// All known services
    pub services: HashMap<String, ServiceNode>,
    /// Dependency edges (service -> depends_on)
    pub dependencies: HashMap<String, Vec<String>>,
    /// Reverse dependencies (service -> required_by)
    pub reverse_deps: HashMap<String, Vec<String>>,
    /// Known failure patterns
    pub failure_patterns: Vec<FailurePattern>,
}

/// A node in the service graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceNode {
    /// Unit name (e.g., "nginx.service")
    pub name: String,
    /// Unit type
    pub unit_type: UnitType,
    /// Current state
    pub state: ServiceState,
    /// Last known active state timestamp
    pub last_active: Option<String>,
    /// Last failure timestamp
    pub last_failure: Option<String>,
    /// Failure count in current window
    pub failure_count: u32,
    /// Log signature patterns for this service
    pub log_signatures: Vec<LogSignature>,
    /// Is this a critical system service?
    pub critical: bool,
}

/// Systemd unit types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UnitType {
    Service,
    Socket,
    Target,
    Mount,
    Automount,
    Device,
    Swap,
    Path,
    Timer,
    Slice,
    Scope,
}

impl UnitType {
    pub fn from_extension(ext: &str) -> Self {
        match ext {
            "service" => UnitType::Service,
            "socket" => UnitType::Socket,
            "target" => UnitType::Target,
            "mount" => UnitType::Mount,
            "automount" => UnitType::Automount,
            "device" => UnitType::Device,
            "swap" => UnitType::Swap,
            "path" => UnitType::Path,
            "timer" => UnitType::Timer,
            "slice" => UnitType::Slice,
            "scope" => UnitType::Scope,
            _ => UnitType::Service,
        }
    }
}

/// Service state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ServiceState {
    Active,
    Inactive,
    Failed,
    Activating,
    Deactivating,
    Reloading,
    Unknown,
}

impl ServiceState {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "active" => ServiceState::Active,
            "inactive" => ServiceState::Inactive,
            "failed" => ServiceState::Failed,
            "activating" => ServiceState::Activating,
            "deactivating" => ServiceState::Deactivating,
            "reloading" => ServiceState::Reloading,
            _ => ServiceState::Unknown,
        }
    }

    pub fn is_problematic(&self) -> bool {
        matches!(self, ServiceState::Failed | ServiceState::Unknown)
    }
}

/// Log signature for pattern matching
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogSignature {
    /// Regex pattern to match
    pub pattern: String,
    /// What this pattern indicates
    pub meaning: String,
    /// Severity level
    pub severity: LogSeverity,
    /// Suggested actions when seen
    pub suggested_actions: Vec<String>,
}

/// Log severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LogSeverity {
    Debug,
    Info,
    Warning,
    Error,
    Critical,
}

/// A known failure pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailurePattern {
    /// Pattern name
    pub name: String,
    /// Services involved
    pub services: Vec<String>,
    /// Log patterns that indicate this failure
    pub log_patterns: Vec<String>,
    /// Root cause description
    pub root_cause: String,
    /// Resolution steps
    pub resolution: Vec<String>,
}

impl ServiceGraph {
    /// Create new empty service graph
    pub fn new() -> Self {
        Self::default()
    }

    /// Add or update a service
    pub fn upsert_service(&mut self, service: ServiceNode) {
        self.services.insert(service.name.clone(), service);
    }

    /// Add a dependency relationship
    pub fn add_dependency(&mut self, service: &str, depends_on: &str) {
        self.dependencies
            .entry(service.to_string())
            .or_default()
            .push(depends_on.to_string());

        self.reverse_deps
            .entry(depends_on.to_string())
            .or_default()
            .push(service.to_string());
    }

    /// Get all services that would be affected if a service fails
    pub fn failure_cascade(&self, service: &str) -> Vec<String> {
        let mut affected = HashSet::new();
        let mut queue = vec![service.to_string()];

        while let Some(current) = queue.pop() {
            if affected.contains(&current) {
                continue;
            }
            affected.insert(current.clone());

            if let Some(dependents) = self.reverse_deps.get(&current) {
                for dep in dependents {
                    if !affected.contains(dep) {
                        queue.push(dep.clone());
                    }
                }
            }
        }

        affected.remove(service);
        affected.into_iter().collect()
    }

    /// Get all failed services
    pub fn get_failed(&self) -> Vec<&ServiceNode> {
        self.services
            .values()
            .filter(|s| s.state == ServiceState::Failed)
            .collect()
    }

    /// Count failed services
    pub fn count_failed(&self) -> usize {
        self.services
            .values()
            .filter(|s| s.state.is_problematic())
            .count()
    }

    /// Find services matching a log pattern
    pub fn match_log_pattern(&self, log_line: &str) -> Vec<(&ServiceNode, &LogSignature)> {
        let mut matches = Vec::new();

        for service in self.services.values() {
            for sig in &service.log_signatures {
                if let Ok(re) = regex::Regex::new(&sig.pattern) {
                    if re.is_match(log_line) {
                        matches.push((service, sig));
                    }
                }
            }
        }

        matches
    }

    /// Get startup order (topological sort)
    pub fn startup_order(&self) -> Vec<String> {
        let mut visited = HashSet::new();
        let mut order = Vec::new();

        fn visit(
            service: &str,
            deps: &HashMap<String, Vec<String>>,
            visited: &mut HashSet<String>,
            order: &mut Vec<String>,
        ) {
            if visited.contains(service) {
                return;
            }
            visited.insert(service.to_string());

            if let Some(dependencies) = deps.get(service) {
                for dep in dependencies {
                    visit(dep, deps, visited, order);
                }
            }

            order.push(service.to_string());
        }

        for service in self.services.keys() {
            visit(service, &self.dependencies, &mut visited, &mut order);
        }

        order
    }
}

impl ServiceNode {
    /// Create a new service node
    pub fn new(name: &str, unit_type: UnitType) -> Self {
        Self {
            name: name.to_string(),
            unit_type,
            state: ServiceState::Unknown,
            last_active: None,
            last_failure: None,
            failure_count: 0,
            log_signatures: Vec::new(),
            critical: false,
        }
    }

    /// Mark as failed
    pub fn mark_failed(&mut self) {
        self.state = ServiceState::Failed;
        self.last_failure = Some(chrono::Utc::now().to_rfc3339());
        self.failure_count += 1;
    }

    /// Mark as active
    pub fn mark_active(&mut self) {
        self.state = ServiceState::Active;
        self.last_active = Some(chrono::Utc::now().to_rfc3339());
    }

    /// Add a log signature
    pub fn add_signature(&mut self, pattern: &str, meaning: &str, severity: LogSeverity) {
        self.log_signatures.push(LogSignature {
            pattern: pattern.to_string(),
            meaning: meaning.to_string(),
            severity,
            suggested_actions: Vec::new(),
        });
    }
}

/// Common service failure patterns for Arch Linux
pub fn common_failure_patterns() -> Vec<FailurePattern> {
    vec![
        FailurePattern {
            name: "NetworkManager DNS Failure".to_string(),
            services: vec!["NetworkManager.service".to_string()],
            log_patterns: vec![
                r"DNS resolution failed".to_string(),
                r"nameserver.*unreachable".to_string(),
            ],
            root_cause: "DNS servers unreachable or misconfigured".to_string(),
            resolution: vec![
                "Check /etc/resolv.conf".to_string(),
                "Verify network connectivity".to_string(),
                "systemctl restart NetworkManager".to_string(),
            ],
        },
        FailurePattern {
            name: "Socket Activation Failure".to_string(),
            services: vec!["*.socket".to_string()],
            log_patterns: vec![r"Failed to listen on.*socket".to_string()],
            root_cause: "Port already in use or permission denied".to_string(),
            resolution: vec![
                "ss -tlnp | grep PORT".to_string(),
                "Check socket file permissions".to_string(),
                "Kill conflicting process".to_string(),
            ],
        },
        FailurePattern {
            name: "Mount Failure".to_string(),
            services: vec!["*.mount".to_string()],
            log_patterns: vec![
                r"mount point.*does not exist".to_string(),
                r"wrong fs type".to_string(),
            ],
            root_cause: "Mount point missing or filesystem type mismatch".to_string(),
            resolution: vec![
                "mkdir -p /mount/point".to_string(),
                "Check fstab entry".to_string(),
                "Verify filesystem type".to_string(),
            ],
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dependency_cascade() {
        let mut graph = ServiceGraph::new();

        graph.upsert_service(ServiceNode::new("nginx.service", UnitType::Service));
        graph.upsert_service(ServiceNode::new("php-fpm.service", UnitType::Service));
        graph.upsert_service(ServiceNode::new("mysql.service", UnitType::Service));

        // nginx depends on php-fpm
        graph.add_dependency("nginx.service", "php-fpm.service");
        // php-fpm depends on mysql
        graph.add_dependency("php-fpm.service", "mysql.service");

        let affected = graph.failure_cascade("mysql.service");
        assert!(affected.contains(&"php-fpm.service".to_string()));
        assert!(affected.contains(&"nginx.service".to_string()));
    }

    #[test]
    fn test_startup_order() {
        let mut graph = ServiceGraph::new();

        graph.upsert_service(ServiceNode::new("a", UnitType::Service));
        graph.upsert_service(ServiceNode::new("b", UnitType::Service));
        graph.upsert_service(ServiceNode::new("c", UnitType::Service));

        graph.add_dependency("a", "b");
        graph.add_dependency("b", "c");

        let order = graph.startup_order();
        let pos_a = order.iter().position(|x| x == "a").unwrap();
        let pos_b = order.iter().position(|x| x == "b").unwrap();
        let pos_c = order.iter().position(|x| x == "c").unwrap();

        // c should come before b, b before a
        assert!(pos_c < pos_b);
        assert!(pos_b < pos_a);
    }

    #[test]
    fn test_service_state() {
        assert!(ServiceState::Failed.is_problematic());
        assert!(!ServiceState::Active.is_problematic());
    }
}
