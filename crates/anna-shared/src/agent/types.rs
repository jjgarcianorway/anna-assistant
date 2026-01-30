//! Agent types and data structures.

use serde::{Deserialize, Serialize};
use std::time::Instant;

/// A task to be executed by an agent.
#[derive(Debug, Clone)]
pub struct AgentTask {
    /// Unique task identifier.
    pub id: String,
    /// The question or request to handle.
    pub question: String,
    /// Task context (system info, session state).
    pub context: TaskContext,
    /// Parent task ID if this is a subtask.
    pub parent_task_id: Option<String>,
    /// Deadline for task completion.
    pub deadline: Option<Instant>,
    /// Detected domains in the question.
    pub domains: Vec<super::traits::AgentDomain>,
}

impl AgentTask {
    pub fn new(question: &str) -> Self {
        let domains = detect_domains(question);
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            question: question.to_string(),
            context: TaskContext::default(),
            parent_task_id: None,
            deadline: None,
            domains,
        }
    }

    pub fn with_context(mut self, ctx: TaskContext) -> Self {
        self.context = ctx;
        self
    }

    pub fn create_subtask(&self, question: &str) -> Self {
        let mut subtask = Self::new(question);
        subtask.parent_task_id = Some(self.id.clone());
        subtask.context = self.context.clone();
        subtask
    }
}

/// Context provided to agents during execution.
#[derive(Debug, Clone, Default)]
pub struct TaskContext {
    pub session_id: String,
    pub system_profile: SystemProfile,
    pub execution_budget: ExecutionBudget,
}

/// System profile information.
#[derive(Debug, Clone, Default)]
pub struct SystemProfile {
    pub hostname: String,
    pub os_name: String,
    pub kernel: String,
    pub distro: String,
    pub cpu_cores: u32,
    pub memory_gb: f32,
    pub gpu: Option<String>,
}

/// Execution budget for resource management.
#[derive(Debug, Clone)]
pub struct ExecutionBudget {
    pub max_iterations: u32,
    pub max_probes: u32,
    pub timeout_secs: u64,
    pub parallel_allowed: bool,
}

impl Default for ExecutionBudget {
    fn default() -> Self {
        Self {
            max_iterations: 5,
            max_probes: 10,
            timeout_secs: 60,
            parallel_allowed: true,
        }
    }
}

/// Result from agent execution.
#[derive(Debug, Clone)]
pub struct AgentResult {
    /// Task ID this result is for.
    pub task_id: String,
    /// Agent ID that produced this result.
    pub agent_id: String,
    /// Whether execution succeeded.
    pub success: bool,
    /// The answer/response (if successful).
    pub answer: Option<String>,
    /// Evidence collected during investigation.
    pub evidence: Vec<Evidence>,
    /// Confidence in the answer (0.0-1.0).
    pub confidence: f32,
    /// Subtasks for delegation.
    pub subtasks: Vec<AgentTask>,
    /// Learning data from this execution.
    pub learning: Option<Learning>,
}

impl AgentResult {
    pub fn success(task_id: &str, agent_id: &str, answer: &str, confidence: f32) -> Self {
        Self {
            task_id: task_id.to_string(),
            agent_id: agent_id.to_string(),
            success: true,
            answer: Some(answer.to_string()),
            evidence: vec![],
            confidence,
            subtasks: vec![],
            learning: None,
        }
    }

    pub fn failure(task_id: &str, agent_id: &str, reason: &str) -> Self {
        Self {
            task_id: task_id.to_string(),
            agent_id: agent_id.to_string(),
            success: false,
            answer: Some(reason.to_string()),
            evidence: vec![],
            confidence: 0.0,
            subtasks: vec![],
            learning: None,
        }
    }

    pub fn with_evidence(mut self, evidence: Vec<Evidence>) -> Self {
        self.evidence = evidence;
        self
    }
}

/// Evidence collected during investigation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Evidence {
    pub source: EvidenceSource,
    pub command: Option<String>,
    pub output: String,
    pub timestamp: String,
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EvidenceSource {
    Command,
    Wiki,
    ManPage,
    Config,
    Memory,
    Recipe,
}

/// Learning data from a task execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Learning {
    pub keywords: Vec<String>,
    pub successful_probes: Vec<String>,
    pub answer_pattern: Option<String>,
}

/// Agent capability descriptor.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AgentCapability {
    pub name: String,
    pub keywords: Vec<String>,
}

impl AgentCapability {
    pub fn new(name: &str, keywords: &[&str]) -> Self {
        Self {
            name: name.to_string(),
            keywords: keywords.iter().map(|s| s.to_string()).collect(),
        }
    }

    pub fn from_keyword(keyword: &str) -> Self {
        Self {
            name: keyword.to_string(),
            keywords: vec![keyword.to_string()],
        }
    }
}

/// Context for agent execution (passed to execute()).
#[derive(Debug, Clone)]
pub struct AgentContext {
    pub session_id: String,
    pub system_profile: SystemProfile,
    pub execution_budget: ExecutionBudget,
    pub model_name: String,
}

impl Default for AgentContext {
    fn default() -> Self {
        Self {
            session_id: String::new(),
            system_profile: SystemProfile::default(),
            execution_budget: ExecutionBudget::default(),
            model_name: "qwen2.5:14b".to_string(),
        }
    }
}

/// Detect domains mentioned in a question.
pub fn detect_domains(question: &str) -> Vec<super::traits::AgentDomain> {
    use super::traits::AgentDomain;

    let q_lower = question.to_lowercase();
    let mut domains = Vec::new();

    for domain in [
        AgentDomain::Network,
        AgentDomain::Desktop,
        AgentDomain::System,
        AgentDomain::Packages,
        AgentDomain::Hardware,
        AgentDomain::Audio,
        AgentDomain::Storage,
        AgentDomain::Security,
    ] {
        for keyword in domain.keywords() {
            if q_lower.contains(keyword) {
                domains.push(domain);
                break;
            }
        }
    }

    if domains.is_empty() {
        domains.push(AgentDomain::General);
    }

    domains
}
