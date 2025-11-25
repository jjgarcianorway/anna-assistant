//! Change Journal - Read-only auditing of Anna's action history
//!
//! v6.51.0: "What did you change?" - Full episode introspection

use crate::action_episodes::{ActionEpisode, ExecutionStatus};
use crate::episode_storage::EpisodeStorage;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use anyhow::Result;

/// Episode domain classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EpisodeDomain {
    Config,      // Configuration file changes
    Services,    // Systemd service operations
    Packages,    // Package install/remove
    Network,     // Network configuration
    General,     // Other/unclassified
}

/// Episode risk level (for journal display)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum EpisodeRisk {
    Safe,
    Moderate,
    High,
}

/// Time window for queries
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeWindow {
    Last24h,
    Last7d,
    Last30d,
    All,
}

impl TimeWindow {
    pub fn since(&self) -> Option<DateTime<Utc>> {
        let now = Utc::now();
        match self {
            TimeWindow::Last24h => Some(now - Duration::hours(24)),
            TimeWindow::Last7d => Some(now - Duration::days(7)),
            TimeWindow::Last30d => Some(now - Duration::days(30)),
            TimeWindow::All => None,
        }
    }
}

/// Risk filter for queries
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EpisodeRiskFilter {
    HighOnly,
    NonSafe,  // Moderate or High
    Any,
}

/// Episode filter for searches
#[derive(Debug, Clone, Default)]
pub struct EpisodeFilter {
    pub time_window: Option<TimeWindow>,
    pub domain: Option<EpisodeDomain>,
    pub risk: Option<EpisodeRiskFilter>,
    pub tags: Vec<String>,
    pub executed_only: bool,
    pub rolled_back_only: bool,
}

/// Compact episode summary for journal lists
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpisodeSummary {
    pub id: i64,
    pub timestamp: DateTime<Utc>,
    pub intent_summary: String,
    pub domain: EpisodeDomain,
    pub risk: EpisodeRisk,
    pub executed: bool,
    pub rolled_back: bool,
    pub validation_score: Option<f32>,
    pub validation_label: Option<String>,
    pub tags: Vec<String>,
}

impl EpisodeSummary {
    /// Create summary from full ActionEpisode
    pub fn from_episode(episode: &ActionEpisode) -> Self {
        // Classify domain from tags
        let domain = classify_episode_domain(episode);

        // Determine risk from rollback capability and execution
        let risk = classify_episode_risk(episode);

        // Check execution status
        let executed = matches!(
            episode.execution_status,
            ExecutionStatus::Executed | ExecutionStatus::PartiallyExecuted
        );

        let rolled_back = episode.execution_status == ExecutionStatus::RolledBack;

        // Extract validation info
        let (validation_score, validation_label) = if let Some(ref v) = episode.post_validation {
            let label = if v.satisfaction_score >= 0.9 {
                "likely satisfied"
            } else if v.satisfaction_score >= 0.7 {
                "mostly satisfied"
            } else if v.satisfaction_score >= 0.5 {
                "uncertain"
            } else {
                "likely unsatisfied"
            };
            (Some(v.satisfaction_score), Some(label.to_string()))
        } else {
            (None, None)
        };

        Self {
            id: episode.episode_id,
            timestamp: episode.created_at,
            intent_summary: episode.user_question.clone(),
            domain,
            risk,
            executed,
            rolled_back,
            validation_score,
            validation_label,
            tags: episode.tags.topics.clone(),
        }
    }
}

/// List recent episodes with optional filtering
pub fn list_recent_episodes(
    storage: &EpisodeStorage,
    limit: usize,
    filter: Option<EpisodeFilter>,
) -> Result<Vec<EpisodeSummary>> {
    // Load recent episodes from storage
    let episodes = storage.list_action_episodes_recent(limit * 2)?; // Get more, we'll filter

    let filter = filter.unwrap_or_default();

    // Apply filters
    let filtered: Vec<EpisodeSummary> = episodes
        .iter()
        .filter_map(|ep| {
            // Time window filter
            if let Some(time_window) = filter.time_window {
                if let Some(since) = time_window.since() {
                    if ep.created_at < since {
                        return None;
                    }
                }
            }

            let summary = EpisodeSummary::from_episode(ep);

            // Domain filter
            if let Some(domain) = filter.domain {
                if summary.domain != domain {
                    return None;
                }
            }

            // Risk filter
            if let Some(risk_filter) = filter.risk {
                match risk_filter {
                    EpisodeRiskFilter::HighOnly => {
                        if summary.risk != EpisodeRisk::High {
                            return None;
                        }
                    }
                    EpisodeRiskFilter::NonSafe => {
                        if summary.risk == EpisodeRisk::Safe {
                            return None;
                        }
                    }
                    EpisodeRiskFilter::Any => {}
                }
            }

            // Tag filter (must match at least one tag)
            if !filter.tags.is_empty() {
                let has_match = filter.tags.iter().any(|tag| {
                    summary.tags.iter().any(|ep_tag| {
                        ep_tag.to_lowercase().contains(&tag.to_lowercase())
                    })
                });
                if !has_match {
                    return None;
                }
            }

            // Executed only filter
            if filter.executed_only && !summary.executed {
                return None;
            }

            // Rolled back only filter
            if filter.rolled_back_only && !summary.rolled_back {
                return None;
            }

            Some(summary)
        })
        .take(limit)
        .collect();

    Ok(filtered)
}

/// Load full episode details by ID
pub fn load_episode_details(storage: &EpisodeStorage, id: i64) -> Result<Option<ActionEpisode>> {
    storage.load_action_episode(id)
}

/// Classify episode domain from tags and actions
fn classify_episode_domain(episode: &ActionEpisode) -> EpisodeDomain {
    // Check tags first
    if let Some(ref domain) = episode.tags.domain {
        match domain.as_str() {
            "editor" | "config" => return EpisodeDomain::Config,
            "packages" => return EpisodeDomain::Packages,
            "system" | "services" => return EpisodeDomain::Services,
            "network" | "ssh" => return EpisodeDomain::Network,
            _ => {}
        }
    }

    // Check topics
    for topic in &episode.tags.topics {
        match topic.as_str() {
            "vim" | "neovim" | "emacs" | "config" | "editor" => return EpisodeDomain::Config,
            "packages" | "install" | "remove" => return EpisodeDomain::Packages,
            "services" | "systemctl" => return EpisodeDomain::Services,
            "ssh" | "network" | "firewall" => return EpisodeDomain::Network,
            _ => {}
        }
    }

    EpisodeDomain::General
}

/// Classify episode risk level
fn classify_episode_risk(episode: &ActionEpisode) -> EpisodeRisk {
    // Check rollback capability as risk indicator
    match episode.rollback_capability {
        crate::action_episodes::RollbackCapability::None => EpisodeRisk::High,
        crate::action_episodes::RollbackCapability::Partial => EpisodeRisk::Moderate,
        crate::action_episodes::RollbackCapability::Full => {
            // Even with full rollback, some domains are moderate risk
            match classify_episode_domain(episode) {
                EpisodeDomain::Services | EpisodeDomain::Packages => EpisodeRisk::Moderate,
                EpisodeDomain::Network => EpisodeRisk::Moderate,
                EpisodeDomain::Config | EpisodeDomain::General => EpisodeRisk::Safe,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::action_episodes::{EpisodeBuilder, EpisodeTags, RollbackCapability, ExecutionStatus};
    use crate::episode_storage::EpisodeStorage;
    use tempfile::tempdir;

    fn make_test_episode(
        question: &str,
        domain: Option<&str>,
        topics: Vec<&str>,
        capability: RollbackCapability,
    ) -> ActionEpisode {
        let tags = EpisodeTags {
            topics: topics.iter().map(|s| s.to_string()).collect(),
            domain: domain.map(|s| s.to_string()),
        };

        let mut episode = EpisodeBuilder::new(question)
            .with_final_answer_summary("Done")
            .with_tags(tags)
            .build();

        episode.rollback_capability = capability;
        episode.execution_status = ExecutionStatus::Executed;
        episode
    }

    #[test]
    fn test_classify_config_domain() {
        let episode = make_test_episode("fix vim", Some("editor"), vec!["vim"], RollbackCapability::Full);
        assert_eq!(classify_episode_domain(&episode), EpisodeDomain::Config);
    }

    #[test]
    fn test_classify_services_domain() {
        let episode = make_test_episode("restart sshd", Some("system"), vec!["services", "ssh"], RollbackCapability::Full);
        assert_eq!(classify_episode_domain(&episode), EpisodeDomain::Services);
    }

    #[test]
    fn test_classify_packages_domain() {
        let episode = make_test_episode("install vim", Some("packages"), vec!["packages", "install"], RollbackCapability::Full);
        assert_eq!(classify_episode_domain(&episode), EpisodeDomain::Packages);
    }

    #[test]
    fn test_classify_safe_risk() {
        let episode = make_test_episode("fix vim", Some("editor"), vec!["vim"], RollbackCapability::Full);
        assert_eq!(classify_episode_risk(&episode), EpisodeRisk::Safe);
    }

    #[test]
    fn test_classify_moderate_risk() {
        let episode = make_test_episode("restart sshd", Some("system"), vec!["services"], RollbackCapability::Full);
        assert_eq!(classify_episode_risk(&episode), EpisodeRisk::Moderate);
    }

    #[test]
    fn test_classify_high_risk() {
        let episode = make_test_episode("partition disk", None, vec![], RollbackCapability::None);
        assert_eq!(classify_episode_risk(&episode), EpisodeRisk::High);
    }

    #[test]
    fn test_episode_summary_from_episode() {
        let episode = make_test_episode("fix vim config", Some("editor"), vec!["vim", "config"], RollbackCapability::Full);
        let summary = EpisodeSummary::from_episode(&episode);

        assert_eq!(summary.intent_summary, "fix vim config");
        assert_eq!(summary.domain, EpisodeDomain::Config);
        assert_eq!(summary.risk, EpisodeRisk::Safe);
        assert!(summary.executed);
        assert!(!summary.rolled_back);
    }

    #[test]
    fn test_list_recent_episodes_ordering() {
        let dir = tempdir().unwrap();
        let storage = EpisodeStorage::new(dir.path().join("test.db")).unwrap();

        // Store episodes in order
        for i in 1..=3 {
            let episode = make_test_episode(&format!("test {}", i), None, vec![], RollbackCapability::Full);
            storage.store_action_episode(&episode).unwrap();
        }

        let summaries = list_recent_episodes(&storage, 10, None).unwrap();
        assert_eq!(summaries.len(), 3);
        // Newest first
        assert!(summaries[0].intent_summary.contains("test 3"));
        assert!(summaries[1].intent_summary.contains("test 2"));
        assert!(summaries[2].intent_summary.contains("test 1"));
    }

    #[test]
    fn test_filter_by_domain() {
        let dir = tempdir().unwrap();
        let storage = EpisodeStorage::new(dir.path().join("test.db")).unwrap();

        storage.store_action_episode(&make_test_episode("vim fix", Some("editor"), vec!["vim"], RollbackCapability::Full)).unwrap();
        storage.store_action_episode(&make_test_episode("install pkg", Some("packages"), vec!["packages"], RollbackCapability::Full)).unwrap();

        let filter = EpisodeFilter {
            domain: Some(EpisodeDomain::Config),
            ..Default::default()
        };

        let summaries = list_recent_episodes(&storage, 10, Some(filter)).unwrap();
        assert_eq!(summaries.len(), 1);
        assert!(summaries[0].intent_summary.contains("vim"));
    }

    #[test]
    fn test_filter_by_tags() {
        let dir = tempdir().unwrap();
        let storage = EpisodeStorage::new(dir.path().join("test.db")).unwrap();

        storage.store_action_episode(&make_test_episode("vim fix", Some("editor"), vec!["vim", "config"], RollbackCapability::Full)).unwrap();
        storage.store_action_episode(&make_test_episode("ssh fix", Some("network"), vec!["ssh"], RollbackCapability::Full)).unwrap();

        let filter = EpisodeFilter {
            tags: vec!["vim".to_string()],
            ..Default::default()
        };

        let summaries = list_recent_episodes(&storage, 10, Some(filter)).unwrap();
        assert_eq!(summaries.len(), 1);
        assert!(summaries[0].intent_summary.contains("vim"));
    }
}
