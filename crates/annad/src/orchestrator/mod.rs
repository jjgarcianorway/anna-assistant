//! Agent Orchestrator - Coordinates multiple agents to solve tasks.
//!
//! The orchestrator is the brain of the multi-agent system:
//! - Routes tasks to capable agents
//! - Handles parallel execution for multi-domain tasks
//! - Synthesizes results from multiple agents
//! - Manages agent learning

mod synthesis;

pub use synthesis::synthesize_results;

use crate::agent_registry::{AgentRegistry, is_multi_domain, get_parallel_domains};
use crate::model_router::ModelRouter;
use anna_shared::agent::{
    Agent, AgentContext, AgentDomain, AgentResult, AgentTask, ExecutionBudget,
    SystemProfile,
};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Multi-agent orchestrator configuration.
#[derive(Debug, Clone)]
pub struct OrchestratorConfig {
    /// Enable multi-agent mode
    pub multi_agent_mode: bool,
    /// Enable parallel investigation
    pub parallel_investigation: bool,
    /// Maximum parallel agents
    pub max_parallel_agents: u32,
    /// Default execution budget
    pub default_budget: ExecutionBudget,
}

impl Default for OrchestratorConfig {
    fn default() -> Self {
        Self {
            multi_agent_mode: true,
            parallel_investigation: true,
            max_parallel_agents: 3,
            default_budget: ExecutionBudget::default(),
        }
    }
}

/// Multi-agent orchestrator.
pub struct AgentOrchestrator {
    /// Agent registry
    registry: Arc<RwLock<AgentRegistry>>,
    /// Model router
    model_router: Arc<ModelRouter>,
    /// Configuration
    config: OrchestratorConfig,
}

impl AgentOrchestrator {
    /// Create a new orchestrator.
    pub fn new(
        registry: Arc<RwLock<AgentRegistry>>,
        model_router: Arc<ModelRouter>,
        config: OrchestratorConfig,
    ) -> Self {
        Self {
            registry,
            model_router,
            config,
        }
    }

    /// Create with default configuration.
    pub fn with_defaults(registry: Arc<RwLock<AgentRegistry>>) -> Self {
        Self::new(
            registry,
            Arc::new(ModelRouter::new()),
            OrchestratorConfig::default(),
        )
    }

    /// Main entry point - solve a task using available agents.
    pub async fn solve(&self, question: &str) -> AgentResult {
        let task = AgentTask::new(question);
        info!("Orchestrator: solving task {}", task.id);

        // Build context
        let ctx = self.build_context(&task);

        // Find capable agents
        let registry = self.registry.read().await;
        let candidates = registry.find_capable_agents(&task);

        if candidates.is_empty() {
            warn!("No capable agents found for task: {}", question);
            return AgentResult::failure(&task.id, "orchestrator", "No capable agents found");
        }

        // Get primary agent (highest confidence)
        let (primary_agent, primary_confidence) = candidates.first().unwrap().clone();
        debug!(
            "Primary agent: {} (confidence: {:.2})",
            primary_agent.name(),
            primary_confidence
        );

        // Check for parallel opportunities
        if self.config.parallel_investigation && is_multi_domain(&task) {
            let parallel_domains = get_parallel_domains(&task);
            if parallel_domains.len() > 1 {
                drop(registry); // Release read lock before parallel execution
                return self.execute_parallel(task, ctx, parallel_domains).await;
            }
        }

        // Single agent execution
        drop(registry); // Release read lock
        self.execute_single(primary_agent, task, ctx).await
    }

    /// Execute task with a single agent.
    async fn execute_single(
        &self,
        agent: Arc<dyn Agent>,
        task: AgentTask,
        ctx: AgentContext,
    ) -> AgentResult {
        info!("Executing task {} with agent {}", task.id, agent.name());

        let result = agent.execute(task.clone(), &ctx).await;

        // Update agent learning (need write lock)
        if result.success {
            let mut registry = self.registry.write().await;
            if let Some(agent) = registry.get(&agent.id().to_string()) {
                // Note: In production, we'd want to update the agent's memory
                // through the registry. For now, just log.
                debug!("Would update learning for agent {}", agent.id());
            }
        }

        result
    }

    /// Execute task with parallel agents for multi-domain questions.
    async fn execute_parallel(
        &self,
        task: AgentTask,
        ctx: AgentContext,
        domains: Vec<AgentDomain>,
    ) -> AgentResult {
        info!(
            "Executing parallel investigation for {} domains",
            domains.len()
        );

        let mut handles = Vec::new();
        let registry = self.registry.read().await;

        // Spawn tasks for each domain
        for domain in domains.iter().take(self.config.max_parallel_agents as usize) {
            let agents = registry.by_domain(*domain);
            if let Some(agent) = agents.first().cloned() {
                let subtask = task.create_subtask(&format!(
                    "{} (focus: {})",
                    task.question,
                    domain.as_str()
                ));
                let ctx_clone = ctx.clone();

                let handle = tokio::spawn(async move {
                    agent.execute(subtask, &ctx_clone).await
                });
                handles.push((*domain, handle));
            }
        }

        drop(registry); // Release read lock

        // Collect results
        let mut results = Vec::new();
        for (domain, handle) in handles {
            match handle.await {
                Ok(result) => {
                    debug!("Got result from {} domain agent", domain.as_str());
                    results.push(result);
                }
                Err(e) => {
                    warn!("Parallel task failed for domain {:?}: {}", domain, e);
                }
            }
        }

        // Synthesize results
        synthesize_results(&task, results)
    }

    /// Build execution context for a task.
    fn build_context(&self, task: &AgentTask) -> AgentContext {
        // Get system profile (simplified)
        let system_profile = SystemProfile {
            hostname: hostname::get()
                .map(|h| h.to_string_lossy().to_string())
                .unwrap_or_else(|_| "unknown".to_string()),
            os_name: "Linux".to_string(),
            kernel: get_kernel_version(),
            distro: get_distro(),
            cpu_cores: num_cpus::get() as u32,
            memory_gb: get_memory_gb(),
            gpu: detect_gpu(),
        };

        // Select model based on task complexity
        let complexity = self.model_router.classify_complexity(&task.question);
        let model_name = match complexity {
            crate::model_router::Complexity::Simple => {
                self.model_router.model_name_for_tier(anna_shared::agent::ModelTier::Fast)
            }
            crate::model_router::Complexity::Standard => {
                self.model_router.model_name_for_tier(anna_shared::agent::ModelTier::Standard)
            }
            _ => {
                self.model_router.model_name_for_tier(anna_shared::agent::ModelTier::Deep)
            }
        };

        AgentContext {
            session_id: uuid::Uuid::new_v4().to_string(),
            system_profile,
            execution_budget: self.config.default_budget.clone(),
            model_name: model_name.to_string(),
        }
    }
}

// Helper functions for system info

fn get_kernel_version() -> String {
    std::fs::read_to_string("/proc/version")
        .ok()
        .and_then(|v| v.split_whitespace().nth(2).map(|s| s.to_string()))
        .unwrap_or_else(|| "unknown".to_string())
}

fn get_distro() -> String {
    std::fs::read_to_string("/etc/os-release")
        .ok()
        .and_then(|content| {
            content.lines()
                .find(|l| l.starts_with("PRETTY_NAME="))
                .map(|l| l.trim_start_matches("PRETTY_NAME=").trim_matches('"').to_string())
        })
        .unwrap_or_else(|| "Linux".to_string())
}

fn get_memory_gb() -> f32 {
    std::fs::read_to_string("/proc/meminfo")
        .ok()
        .and_then(|content| {
            content.lines()
                .find(|l| l.starts_with("MemTotal:"))
                .and_then(|l| {
                    l.split_whitespace()
                        .nth(1)
                        .and_then(|v| v.parse::<u64>().ok())
                })
        })
        .map(|kb| kb as f32 / 1024.0 / 1024.0)
        .unwrap_or(0.0)
}

fn detect_gpu() -> Option<String> {
    std::process::Command::new("lspci")
        .output()
        .ok()
        .and_then(|output| {
            let stdout = String::from_utf8_lossy(&output.stdout);
            stdout.lines()
                .find(|l| l.contains("VGA") || l.contains("3D"))
                .map(|l| l.to_string())
        })
}
