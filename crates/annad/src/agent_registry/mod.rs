//! Agent Registry - Central registry for all agents.
//!
//! The registry manages agent discovery, routing, and lifecycle.

use anna_shared::agent::{
    Agent, AgentCapability, AgentDomain, AgentTask, ModelTier,
};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, info};

/// Central registry for all agents.
pub struct AgentRegistry {
    /// All registered agents by ID.
    agents: HashMap<String, Arc<dyn Agent>>,
    /// Index: domain -> agent IDs.
    domain_index: HashMap<AgentDomain, Vec<String>>,
    /// Index: capability name -> agent IDs.
    capability_index: HashMap<String, Vec<String>>,
}

impl AgentRegistry {
    /// Create a new empty registry.
    pub fn new() -> Self {
        Self {
            agents: HashMap::new(),
            domain_index: HashMap::new(),
            capability_index: HashMap::new(),
        }
    }

    /// Register an agent.
    pub fn register(&mut self, agent: Arc<dyn Agent>) {
        let id = agent.id().to_string();
        info!("Registering agent: {} ({})", agent.name(), id);

        // Index by domain
        self.domain_index
            .entry(agent.domain())
            .or_default()
            .push(id.clone());

        // Index by capabilities
        for cap in agent.capabilities() {
            self.capability_index
                .entry(cap.name.clone())
                .or_default()
                .push(id.clone());
        }

        self.agents.insert(id, agent);
    }

    /// Get agent by ID.
    pub fn get(&self, id: &str) -> Option<Arc<dyn Agent>> {
        self.agents.get(id).cloned()
    }

    /// Get all agents in a domain.
    pub fn by_domain(&self, domain: AgentDomain) -> Vec<Arc<dyn Agent>> {
        self.domain_index
            .get(&domain)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| self.agents.get(id).cloned())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get all agents with a capability.
    pub fn by_capability(&self, capability: &str) -> Vec<Arc<dyn Agent>> {
        self.capability_index
            .get(capability)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| self.agents.get(id).cloned())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Find agents that can handle a task, ranked by confidence.
    pub fn find_capable_agents(&self, task: &AgentTask) -> Vec<(Arc<dyn Agent>, f32)> {
        let mut candidates: Vec<(Arc<dyn Agent>, f32)> = self.agents
            .values()
            .map(|agent| {
                let confidence = agent.can_handle(task);
                (agent.clone(), confidence)
            })
            .filter(|(_, confidence)| *confidence > 0.1)
            .collect();

        // Sort by confidence (highest first)
        candidates.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        debug!(
            "Found {} capable agents for task: {}",
            candidates.len(),
            task.question.chars().take(50).collect::<String>()
        );

        candidates
    }

    /// Get the best agent for a task.
    pub fn get_best_agent(&self, task: &AgentTask) -> Option<Arc<dyn Agent>> {
        self.find_capable_agents(task)
            .into_iter()
            .next()
            .map(|(agent, _)| agent)
    }

    /// Get all registered agents.
    pub fn all_agents(&self) -> Vec<Arc<dyn Agent>> {
        self.agents.values().cloned().collect()
    }

    /// Get agent count.
    pub fn count(&self) -> usize {
        self.agents.len()
    }

    /// Get agents by model tier.
    pub fn by_model_tier(&self, tier: ModelTier) -> Vec<Arc<dyn Agent>> {
        self.agents
            .values()
            .filter(|a| a.model_tier() == tier)
            .cloned()
            .collect()
    }
}

impl Default for AgentRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Candidate agent with routing score.
#[derive(Debug, Clone)]
pub struct AgentCandidate {
    pub agent_id: String,
    pub agent_name: String,
    pub domain: AgentDomain,
    pub confidence: f32,
    pub model_tier: ModelTier,
}

impl AgentCandidate {
    pub fn from_agent(agent: &Arc<dyn Agent>, confidence: f32) -> Self {
        Self {
            agent_id: agent.id().to_string(),
            agent_name: agent.name().to_string(),
            domain: agent.domain(),
            confidence,
            model_tier: agent.model_tier(),
        }
    }
}

/// Route a task to the best agents.
pub fn route_task(registry: &AgentRegistry, task: &AgentTask) -> Vec<AgentCandidate> {
    registry
        .find_capable_agents(task)
        .into_iter()
        .map(|(agent, confidence)| AgentCandidate::from_agent(&agent, confidence))
        .collect()
}

/// Check if a question spans multiple domains.
pub fn is_multi_domain(task: &AgentTask) -> bool {
    task.domains.len() > 1
}

/// Get domains from a task for parallel routing.
pub fn get_parallel_domains(task: &AgentTask) -> Vec<AgentDomain> {
    if task.domains.len() > 1 {
        task.domains.clone()
    } else {
        vec![]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anna_shared::agent::{AgentContext, AgentResult, AgentMemory};
    use async_trait::async_trait;

    struct TestAgent {
        id: String,
        domain: AgentDomain,
    }

    #[async_trait]
    impl Agent for TestAgent {
        fn id(&self) -> &str { &self.id }
        fn name(&self) -> &str { "Test Agent" }
        fn domain(&self) -> AgentDomain { self.domain }
        fn capabilities(&self) -> Vec<AgentCapability> { vec![] }
        fn model_tier(&self) -> ModelTier { ModelTier::Fast }
        fn can_handle(&self, _task: &AgentTask) -> f32 { 0.5 }
        async fn execute(&self, task: AgentTask, _ctx: &AgentContext) -> AgentResult {
            AgentResult::success(&task.id, &self.id, "test", 0.8)
        }
        fn memory(&self) -> &AgentMemory {
            static MEMORY: std::sync::OnceLock<AgentMemory> = std::sync::OnceLock::new();
            MEMORY.get_or_init(AgentMemory::default)
        }
        fn learn(&mut self, _task: &AgentTask, _result: &AgentResult) {}
    }

    #[test]
    fn test_registry_register() {
        let mut registry = AgentRegistry::new();
        let agent = Arc::new(TestAgent {
            id: "test-1".to_string(),
            domain: AgentDomain::Network,
        });
        registry.register(agent);
        assert_eq!(registry.count(), 1);
    }

    #[test]
    fn test_registry_by_domain() {
        let mut registry = AgentRegistry::new();
        registry.register(Arc::new(TestAgent {
            id: "net-1".to_string(),
            domain: AgentDomain::Network,
        }));
        registry.register(Arc::new(TestAgent {
            id: "sys-1".to_string(),
            domain: AgentDomain::System,
        }));

        let net_agents = registry.by_domain(AgentDomain::Network);
        assert_eq!(net_agents.len(), 1);
        assert_eq!(net_agents[0].id(), "net-1");
    }
}
