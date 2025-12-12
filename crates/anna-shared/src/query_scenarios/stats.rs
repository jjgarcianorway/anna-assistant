//! Query scenario statistics collector (v0.0.268).
//!
//! Tracks resolution rates, routing accuracy, recipe learning.

use crate::teams::Team;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Resolution outcome for a query
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResolutionOutcome {
    /// Fast path answered (no LLM)
    FastPath,
    /// Junior resolved successfully
    JuniorResolved,
    /// Senior needed to resolve
    SeniorResolved,
    /// Answer needed revision
    Revised,
    /// Timeout occurred
    Timeout,
    /// User clarification needed
    Clarification,
    /// Failed to resolve
    Failed,
}

/// Stats for a single team
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TeamStats {
    pub team: String,
    pub total_queries: u32,
    pub fast_path: u32,
    pub junior_resolved: u32,
    pub senior_resolved: u32,
    pub revised: u32,
    pub timeouts: u32,
    pub clarifications: u32,
    pub failed: u32,
    pub avg_reliability: f32,
    pub recipes_learned: u32,
    pub recipes_recalled: u32,
    /// How many were routed to wrong team
    pub misrouted: u32,
}

impl TeamStats {
    pub fn new(team: &str) -> Self {
        Self {
            team: team.to_string(),
            ..Default::default()
        }
    }

    pub fn record(&mut self, outcome: ResolutionOutcome, reliability: f32) {
        self.total_queries += 1;

        // Update running average
        let prev_total = (self.total_queries - 1) as f32;
        self.avg_reliability =
            (self.avg_reliability * prev_total + reliability) / self.total_queries as f32;

        match outcome {
            ResolutionOutcome::FastPath => self.fast_path += 1,
            ResolutionOutcome::JuniorResolved => self.junior_resolved += 1,
            ResolutionOutcome::SeniorResolved => self.senior_resolved += 1,
            ResolutionOutcome::Revised => self.revised += 1,
            ResolutionOutcome::Timeout => self.timeouts += 1,
            ResolutionOutcome::Clarification => self.clarifications += 1,
            ResolutionOutcome::Failed => self.failed += 1,
        }
    }

    pub fn success_rate(&self) -> f32 {
        if self.total_queries == 0 {
            return 0.0;
        }
        let successful = self.fast_path + self.junior_resolved + self.senior_resolved;
        successful as f32 / self.total_queries as f32 * 100.0
    }

    pub fn escalation_rate(&self) -> f32 {
        if self.total_queries == 0 {
            return 0.0;
        }
        self.senior_resolved as f32 / self.total_queries as f32 * 100.0
    }
}

/// Comprehensive scenario test statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ScenarioStats {
    pub total_scenarios: u32,
    pub passed: u32,
    pub failed: u32,

    /// Stats per team
    pub by_team: HashMap<String, TeamStats>,

    /// Stats by difficulty
    pub simple_pass_rate: f32,
    pub medium_pass_rate: f32,
    pub complex_pass_rate: f32,

    /// Fast path stats
    pub expected_fast_path: u32,
    pub actual_fast_path: u32,

    /// Recipe learning stats
    pub recipes_should_learn: u32,
    pub recipes_did_learn: u32,
    pub recipe_recalls_tested: u32,
    pub recipe_recalls_succeeded: u32,

    /// Routing accuracy
    pub routing_correct: u32,
    pub routing_incorrect: u32,

    /// Junior vs Senior stats
    pub junior_only_expected: u32,
    pub junior_only_actual: u32,
    pub senior_needed_expected: u32,
    pub senior_needed_actual: u32,
}

impl ScenarioStats {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn overall_pass_rate(&self) -> f32 {
        if self.total_scenarios == 0 {
            return 0.0;
        }
        self.passed as f32 / self.total_scenarios as f32 * 100.0
    }

    pub fn routing_accuracy(&self) -> f32 {
        let total = self.routing_correct + self.routing_incorrect;
        if total == 0 {
            return 0.0;
        }
        self.routing_correct as f32 / total as f32 * 100.0
    }

    pub fn fast_path_accuracy(&self) -> f32 {
        if self.expected_fast_path == 0 {
            return 100.0;
        }
        self.actual_fast_path as f32 / self.expected_fast_path as f32 * 100.0
    }

    pub fn recipe_learning_rate(&self) -> f32 {
        if self.recipes_should_learn == 0 {
            return 100.0;
        }
        self.recipes_did_learn as f32 / self.recipes_should_learn as f32 * 100.0
    }

    pub fn recipe_recall_rate(&self) -> f32 {
        if self.recipe_recalls_tested == 0 {
            return 100.0;
        }
        self.recipe_recalls_succeeded as f32 / self.recipe_recalls_tested as f32 * 100.0
    }

    pub fn get_team_stats(&mut self, team: Team) -> &mut TeamStats {
        let team_name = team.to_string();
        self.by_team
            .entry(team_name.clone())
            .or_insert_with(|| TeamStats::new(&team_name))
    }

    /// Generate summary report
    pub fn summary_report(&self) -> String {
        let mut report = String::new();

        report.push_str("═══════════════════════════════════════════════════════════════\n");
        report.push_str("                    SCENARIO TEST SUMMARY                      \n");
        report.push_str("═══════════════════════════════════════════════════════════════\n\n");

        report.push_str(&format!("Total Scenarios: {}\n", self.total_scenarios));
        report.push_str(&format!(
            "Passed: {} ({:.1}%)\n",
            self.passed,
            self.overall_pass_rate()
        ));
        report.push_str(&format!("Failed: {}\n\n", self.failed));

        report.push_str("─── ROUTING ───\n");
        report.push_str(&format!(
            "Correct: {} | Incorrect: {} | Accuracy: {:.1}%\n\n",
            self.routing_correct,
            self.routing_incorrect,
            self.routing_accuracy()
        ));

        report.push_str("─── FAST PATH ───\n");
        report.push_str(&format!(
            "Expected: {} | Actual: {} | Accuracy: {:.1}%\n\n",
            self.expected_fast_path,
            self.actual_fast_path,
            self.fast_path_accuracy()
        ));

        report.push_str("─── DIFFICULTY ───\n");
        report.push_str(&format!(
            "Simple: {:.1}% | Medium: {:.1}% | Complex: {:.1}%\n\n",
            self.simple_pass_rate, self.medium_pass_rate, self.complex_pass_rate
        ));

        report.push_str("─── RECIPE LEARNING ───\n");
        report.push_str(&format!(
            "Should learn: {} | Did learn: {} | Rate: {:.1}%\n",
            self.recipes_should_learn,
            self.recipes_did_learn,
            self.recipe_learning_rate()
        ));
        report.push_str(&format!(
            "Recalls tested: {} | Succeeded: {} | Rate: {:.1}%\n\n",
            self.recipe_recalls_tested,
            self.recipe_recalls_succeeded,
            self.recipe_recall_rate()
        ));

        report.push_str("─── JUNIOR VS SENIOR ───\n");
        report.push_str(&format!(
            "Junior-only expected: {} | actual: {}\n",
            self.junior_only_expected, self.junior_only_actual
        ));
        report.push_str(&format!(
            "Senior-needed expected: {} | actual: {}\n\n",
            self.senior_needed_expected, self.senior_needed_actual
        ));

        report.push_str("─── BY TEAM ───\n");
        let mut teams: Vec<_> = self.by_team.values().collect();
        teams.sort_by(|a, b| b.total_queries.cmp(&a.total_queries));

        for ts in teams {
            report.push_str(&format!(
                "{:12} | queries: {:>3} | success: {:>5.1}% | escalation: {:>5.1}% | reliability: {:>5.1}\n",
                ts.team,
                ts.total_queries,
                ts.success_rate(),
                ts.escalation_rate(),
                ts.avg_reliability
            ));
        }

        report.push_str("\n═══════════════════════════════════════════════════════════════\n");

        report
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_team_stats_recording() {
        let mut stats = TeamStats::new("storage");
        stats.record(ResolutionOutcome::JuniorResolved, 85.0);
        stats.record(ResolutionOutcome::SeniorResolved, 90.0);
        stats.record(ResolutionOutcome::Failed, 0.0);

        assert_eq!(stats.total_queries, 3);
        assert_eq!(stats.junior_resolved, 1);
        assert_eq!(stats.senior_resolved, 1);
        assert_eq!(stats.failed, 1);
        assert!((stats.success_rate() - 66.67).abs() < 1.0);
    }

    #[test]
    fn test_scenario_stats_summary() {
        let mut stats = ScenarioStats::new();
        stats.total_scenarios = 100;
        stats.passed = 85;
        stats.failed = 15;
        stats.routing_correct = 92;
        stats.routing_incorrect = 8;

        let report = stats.summary_report();
        assert!(report.contains("85.0%"));
        assert!(report.contains("92.0%"));
    }
}
