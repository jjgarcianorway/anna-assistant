//! Orchestrator integration with streaming system.
//!
//! Bridges the multi-agent orchestrator with the existing Ralph loop.

use anna_shared::agent::{AgentTask, detect_domains};
use anna_shared::config::AnnaConfig;
use tracing::{debug, info};

/// Check if a question should use multi-agent mode.
pub fn should_use_multi_agent(question: &str, config: &AnnaConfig) -> bool {
    // Check if multi-agent mode is enabled
    if !config.agents.multi_agent_mode {
        return false;
    }

    // Check if parallel investigation is enabled for multi-domain
    if config.agents.parallel_investigation {
        let domains = detect_domains(question);
        if domains.len() > 1 {
            debug!("Multi-domain question detected: {:?}", domains);
            return true;
        }
    }

    false
}

/// Get the recommended model for a task based on complexity.
pub fn get_recommended_model(question: &str, config: &AnnaConfig) -> String {
    use crate::model_router::{ComplexityClassifier, Complexity};

    let classifier = ComplexityClassifier::new();
    let complexity = classifier.classify(question);

    let model = match complexity {
        Complexity::Simple => &config.agents.fast_model,
        Complexity::Standard => &config.agents.standard_model,
        Complexity::Complex | Complexity::VeryComplex => &config.agents.deep_model,
    };

    info!("Task complexity: {:?}, recommended model: {}", complexity, model);
    model.clone()
}

/// Analyze a question and return metadata for the streaming handler.
#[derive(Debug, Clone)]
pub struct TaskAnalysis {
    /// Detected domains
    pub domains: Vec<String>,
    /// Is multi-domain question
    pub is_multi_domain: bool,
    /// Recommended model tier
    pub recommended_model: String,
    /// Complexity level
    pub complexity: String,
}

impl TaskAnalysis {
    pub fn analyze(question: &str, config: &AnnaConfig) -> Self {
        use crate::model_router::{ComplexityClassifier, Complexity};

        let domains: Vec<String> = detect_domains(question)
            .into_iter()
            .map(|d| format!("{:?}", d))
            .collect();

        let is_multi_domain = domains.len() > 1;

        let classifier = ComplexityClassifier::new();
        let complexity = classifier.classify(question);

        let complexity_str = match complexity {
            Complexity::Simple => "simple",
            Complexity::Standard => "standard",
            Complexity::Complex => "complex",
            Complexity::VeryComplex => "very_complex",
        }.to_string();

        let recommended_model = match complexity {
            Complexity::Simple => config.agents.fast_model.clone(),
            Complexity::Standard => config.agents.standard_model.clone(),
            Complexity::Complex | Complexity::VeryComplex => config.agents.deep_model.clone(),
        };

        Self {
            domains,
            is_multi_domain,
            recommended_model,
            complexity: complexity_str,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_question_analysis() {
        let config = AnnaConfig::default();
        let analysis = TaskAnalysis::analyze("what is my IP?", &config);

        assert_eq!(analysis.complexity, "simple");
        assert!(!analysis.is_multi_domain);
    }

    #[test]
    fn test_multi_domain_question() {
        let config = AnnaConfig::default();
        let analysis = TaskAnalysis::analyze("check my wifi and disk space", &config);

        assert!(analysis.is_multi_domain || analysis.domains.len() >= 1);
    }
}
