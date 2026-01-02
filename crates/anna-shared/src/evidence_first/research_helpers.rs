//! Quick research helpers for common patterns.

use super::research::{ResearchLoop, ResearchPlan, ResearchResult};

/// Quick research helper for common patterns.
pub struct QuickResearch;

impl QuickResearch {
    /// Research a boot-related issue.
    pub fn boot_issue(ticket_id: &str) -> ResearchResult {
        let mut plan = ResearchPlan::new(ticket_id)
            .with_keywords(vec![
                "boot".to_string(),
                "slow".to_string(),
                "startup".to_string(),
            ])
            .with_commands(vec!["systemd-analyze".to_string()]);

        let loop_ = ResearchLoop::new();
        loop_.execute(&mut plan)
    }

    /// Research a service issue.
    pub fn service_issue(ticket_id: &str, service: &str) -> ResearchResult {
        let mut plan = ResearchPlan::new(ticket_id)
            .with_keywords(vec![
                service.to_string(),
                "service".to_string(),
                "failed".to_string(),
            ])
            .with_commands(vec!["systemctl".to_string(), "journalctl".to_string()]);

        let loop_ = ResearchLoop::new();
        loop_.execute(&mut plan)
    }

    /// Research a memory issue.
    pub fn memory_issue(ticket_id: &str) -> ResearchResult {
        let mut plan = ResearchPlan::new(ticket_id)
            .with_keywords(vec![
                "memory".to_string(),
                "ram".to_string(),
                "swap".to_string(),
            ])
            .with_commands(vec!["free".to_string()]);

        let loop_ = ResearchLoop::new();
        loop_.execute(&mut plan)
    }
}
