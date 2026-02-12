//! Root cause analysis types.

use serde::{Deserialize, Serialize};

/// A system event that occurred
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemEvent {
    pub timestamp: String,
    pub event_type: EventType,
    pub component: String, // Service, file, process, etc.
    pub details: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventType {
    ServiceFailed,
    ServiceStarted,
    ServiceStopped,
    HighCpuUsage,
    HighMemoryUsage,
    DiskFull,
    NetworkError,
    PackageInstalled,
    PackageRemoved,
    ConfigChanged,
    LogError,
}

/// A causal relationship between events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CausalLink {
    pub cause: String,      // Event ID or description
    pub effect: String,     // Event ID or description
    pub confidence: f32,    // 0.0 to 1.0
    pub observed_count: u32, // How many times we've seen this pattern
}

/// Root cause analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RootCauseAnalysis {
    pub symptom: String,
    pub root_causes: Vec<RootCause>,
    pub analysis_timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RootCause {
    pub description: String,
    pub confidence: f32,
    pub evidence: Vec<String>,
    pub recommended_action: String,
}

/// Dependency graph of system components
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyGraph {
    pub nodes: Vec<ComponentNode>,
    pub edges: Vec<DependencyEdge>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentNode {
    pub name: String,
    pub component_type: ComponentType,
    pub critical: bool, // Is this component critical to system function?
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComponentType {
    Service,
    Process,
    File,
    Network,
    Disk,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyEdge {
    pub from: String,
    pub to: String,
    pub dependency_type: DependencyType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DependencyType {
    Requires,    // Hard dependency
    Wants,       // Soft dependency
    After,       // Ordering dependency
    Uses,        // Uses a resource
}

impl Default for DependencyGraph {
    fn default() -> Self {
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),
        }
    }
}

impl DependencyGraph {
    /// Find all components that depend on this one
    pub fn find_dependents(&self, component: &str) -> Vec<&ComponentNode> {
        let dependent_names: Vec<&str> = self.edges
            .iter()
            .filter(|e| e.to == component)
            .map(|e| e.from.as_str())
            .collect();

        self.nodes
            .iter()
            .filter(|n| dependent_names.contains(&n.name.as_str()))
            .collect()
    }

    /// Find all components this one depends on
    pub fn find_dependencies(&self, component: &str) -> Vec<&ComponentNode> {
        let dependency_names: Vec<&str> = self.edges
            .iter()
            .filter(|e| e.from == component)
            .map(|e| e.to.as_str())
            .collect();

        self.nodes
            .iter()
            .filter(|n| dependency_names.contains(&n.name.as_str()))
            .collect()
    }

    /// Build dependency graph from systemd units
    pub fn build_from_systemd() -> Self {
        let mut graph = Self::default();

        // Get all active units
        if let Ok(output) = std::process::Command::new("systemctl")
            .args(["list-units", "--all", "--no-pager", "--no-legend"])
            .output()
        {
            let units = String::from_utf8_lossy(&output.stdout);

            for line in units.lines() {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if let Some(unit_name) = parts.first() {
                    // Add node
                    graph.nodes.push(ComponentNode {
                        name: unit_name.to_string(),
                        component_type: ComponentType::Service,
                        critical: false, // Could be determined by checking if it's a target
                    });
                }
            }
        }

        // Get dependencies for each unit
        for node in &graph.nodes.clone() {
            if let Ok(output) = std::process::Command::new("systemctl")
                .args(["show", &node.name, "--property=Requires,Wants,After"])
                .output()
            {
                let deps = String::from_utf8_lossy(&output.stdout);

                for line in deps.lines() {
                    if let Some((key, value)) = line.split_once('=') {
                        if value.is_empty() {
                            continue;
                        }

                        let dep_type = match key {
                            "Requires" => DependencyType::Requires,
                            "Wants" => DependencyType::Wants,
                            "After" => DependencyType::After,
                            _ => continue,
                        };

                        for dep in value.split_whitespace() {
                            graph.edges.push(DependencyEdge {
                                from: node.name.clone(),
                                to: dep.to_string(),
                                dependency_type: dep_type.clone(),
                            });
                        }
                    }
                }
            }
        }

        graph
    }
}
