//! Docker Compose recipe types (v0.0.235).

use serde::{Deserialize, Serialize};

/// Docker Compose recipe features
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DockerFeature {
    /// Create a docker-compose.yml file
    CreateCompose,
    /// Start services with docker compose
    StartServices,
    /// Stop services
    StopServices,
    /// View logs
    ViewLogs,
    /// List running containers
    ListContainers,
    /// Build images
    BuildImages,
    /// Pull images
    PullImages,
    /// Execute command in container
    ExecContainer,
    /// Remove containers/volumes
    Cleanup,
    /// Debug issues
    Debug,
}

impl DockerFeature {
    pub fn display_name(&self) -> &'static str {
        match self {
            DockerFeature::CreateCompose => "create docker-compose.yml",
            DockerFeature::StartServices => "start docker compose",
            DockerFeature::StopServices => "stop docker compose",
            DockerFeature::ViewLogs => "view docker logs",
            DockerFeature::ListContainers => "list containers",
            DockerFeature::BuildImages => "build images",
            DockerFeature::PullImages => "pull images",
            DockerFeature::ExecContainer => "exec in container",
            DockerFeature::Cleanup => "cleanup docker",
            DockerFeature::Debug => "debug docker",
        }
    }

    /// Keywords that indicate this feature
    pub fn keywords(&self) -> &'static [&'static str] {
        match self {
            DockerFeature::CreateCompose => &["create", "write", "new", "compose file", "yml"],
            DockerFeature::StartServices => &["start", "up", "run", "launch"],
            DockerFeature::StopServices => &["stop", "down", "shutdown"],
            DockerFeature::ViewLogs => &["logs", "log", "output"],
            DockerFeature::ListContainers => &["list", "ps", "running", "containers"],
            DockerFeature::BuildImages => &["build", "rebuild"],
            DockerFeature::PullImages => &["pull", "download", "update"],
            DockerFeature::ExecContainer => &["exec", "execute", "shell", "bash", "attach"],
            DockerFeature::Cleanup => &["cleanup", "clean", "prune", "remove", "delete"],
            DockerFeature::Debug => &["debug", "troubleshoot", "failing", "not working"],
        }
    }
}

impl std::fmt::Display for DockerFeature {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

/// A Docker Compose recipe
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockerRecipe {
    pub feature: DockerFeature,
    pub description: String,
    pub commands: Vec<String>,
    pub compose_example: Option<String>,
    pub answer_template: String,
    pub notes: Vec<String>,
}

impl DockerRecipe {
    pub fn new(feature: DockerFeature, description: &str) -> Self {
        Self {
            feature,
            description: description.to_string(),
            commands: Vec::new(),
            compose_example: None,
            answer_template: String::new(),
            notes: Vec::new(),
        }
    }

    pub fn with_command(mut self, cmd: &str) -> Self {
        self.commands.push(cmd.to_string());
        self
    }

    pub fn with_compose(mut self, compose: &str) -> Self {
        self.compose_example = Some(compose.to_string());
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
}
