//! Kubernetes recipe types (v0.0.459).

use serde::{Deserialize, Serialize};

/// Kubernetes recipe features
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum K8sFeature {
    /// List pods in namespace
    ListPods,
    /// Get pod details
    DescribePod,
    /// View pod logs
    PodLogs,
    /// Execute command in pod
    ExecPod,
    /// List deployments
    ListDeployments,
    /// Scale deployment
    ScaleDeployment,
    /// Restart deployment
    RestartDeployment,
    /// Apply manifest
    ApplyManifest,
    /// Delete resource
    DeleteResource,
    /// List services
    ListServices,
    /// Port forward
    PortForward,
    /// Get events
    GetEvents,
    /// Debug pod issues
    DebugPod,
    /// Check cluster health
    ClusterHealth,
    /// List nodes
    ListNodes,
    /// Get resource usage
    ResourceUsage,
    /// List namespaces
    ListNamespaces,
    /// Create namespace
    CreateNamespace,
    /// Get configmaps/secrets
    GetConfig,
}

impl K8sFeature {
    pub fn display_name(&self) -> &'static str {
        match self {
            K8sFeature::ListPods => "list pods",
            K8sFeature::DescribePod => "describe pod",
            K8sFeature::PodLogs => "view pod logs",
            K8sFeature::ExecPod => "exec into pod",
            K8sFeature::ListDeployments => "list deployments",
            K8sFeature::ScaleDeployment => "scale deployment",
            K8sFeature::RestartDeployment => "restart deployment",
            K8sFeature::ApplyManifest => "apply manifest",
            K8sFeature::DeleteResource => "delete resource",
            K8sFeature::ListServices => "list services",
            K8sFeature::PortForward => "port forward",
            K8sFeature::GetEvents => "get events",
            K8sFeature::DebugPod => "debug pod",
            K8sFeature::ClusterHealth => "cluster health",
            K8sFeature::ListNodes => "list nodes",
            K8sFeature::ResourceUsage => "resource usage",
            K8sFeature::ListNamespaces => "list namespaces",
            K8sFeature::CreateNamespace => "create namespace",
            K8sFeature::GetConfig => "get configmaps/secrets",
        }
    }

    /// Keywords that indicate this feature
    pub fn keywords(&self) -> &'static [&'static str] {
        match self {
            K8sFeature::ListPods => &["pods", "list pods", "get pods", "running pods"],
            K8sFeature::DescribePod => &["describe", "pod details", "pod info"],
            K8sFeature::PodLogs => &["logs", "pod logs", "container logs"],
            K8sFeature::ExecPod => &["exec", "shell", "bash", "attach", "connect"],
            K8sFeature::ListDeployments => &["deployments", "list deployments", "get deployments"],
            K8sFeature::ScaleDeployment => &["scale", "replicas", "scale up", "scale down"],
            K8sFeature::RestartDeployment => &["restart", "rollout", "redeploy"],
            K8sFeature::ApplyManifest => &["apply", "deploy", "create", "manifest", "yaml"],
            K8sFeature::DeleteResource => &["delete", "remove", "destroy"],
            K8sFeature::ListServices => &["services", "svc", "list services"],
            K8sFeature::PortForward => &["port-forward", "forward", "tunnel"],
            K8sFeature::GetEvents => &["events", "what happened", "recent events"],
            K8sFeature::DebugPod => &["debug", "troubleshoot", "not working", "failing", "crash"],
            K8sFeature::ClusterHealth => &["health", "cluster status", "cluster health"],
            K8sFeature::ListNodes => &["nodes", "list nodes", "cluster nodes"],
            K8sFeature::ResourceUsage => &["resources", "cpu", "memory", "usage", "top"],
            K8sFeature::ListNamespaces => &["namespaces", "ns", "list namespaces"],
            K8sFeature::CreateNamespace => &["create namespace", "new namespace"],
            K8sFeature::GetConfig => &["configmap", "secret", "config", "secrets"],
        }
    }
}

impl std::fmt::Display for K8sFeature {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

/// A Kubernetes recipe
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct K8sRecipe {
    pub feature: K8sFeature,
    pub description: String,
    pub commands: Vec<String>,
    pub manifest_example: Option<String>,
    pub answer_template: String,
    pub notes: Vec<String>,
    /// Required tools (kubectl, helm, etc.)
    pub requires: Vec<String>,
}

impl K8sRecipe {
    pub fn new(feature: K8sFeature, description: &str) -> Self {
        Self {
            feature,
            description: description.to_string(),
            commands: Vec::new(),
            manifest_example: None,
            answer_template: String::new(),
            notes: Vec::new(),
            requires: vec!["kubectl".to_string()],
        }
    }

    pub fn with_command(mut self, cmd: &str) -> Self {
        self.commands.push(cmd.to_string());
        self
    }

    pub fn with_manifest(mut self, manifest: &str) -> Self {
        self.manifest_example = Some(manifest.to_string());
        self
    }

    pub fn with_answer(mut self, answer: &str) -> Self {
        self.answer_template = answer.to_string();
        self
    }

    pub fn with_note(mut self, note: &str) -> Self {
        self.notes.push(note.to_string());
        self
    }

    pub fn with_requires(mut self, tool: &str) -> Self {
        if !self.requires.contains(&tool.to_string()) {
            self.requires.push(tool.to_string());
        }
        self
    }
}
