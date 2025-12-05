//! Team-specific LLM review prompts.
//!
//! Prompt templates for Junior and Senior reviewers when LLM review is needed.
//! Each prompt is deterministic and outputs structured JSON.
//!
//! Prompts are split into separate modules to keep file sizes under 400 lines.

mod review_prompts_junior;
mod review_prompts_senior;

use crate::teams::Team;
use review_prompts_junior::*;
use review_prompts_senior::*;

/// Get junior review prompt for a team
pub fn junior_prompt(team: Team) -> &'static str {
    match team {
        Team::Desktop => DESKTOP_JUNIOR_PROMPT,
        Team::Storage => STORAGE_JUNIOR_PROMPT,
        Team::Network => NETWORK_JUNIOR_PROMPT,
        Team::Performance => PERFORMANCE_JUNIOR_PROMPT,
        Team::Services => SERVICES_JUNIOR_PROMPT,
        Team::Security => SECURITY_JUNIOR_PROMPT,
        Team::Hardware => HARDWARE_JUNIOR_PROMPT,
        Team::General => GENERAL_JUNIOR_PROMPT,
    }
}

/// Get senior review prompt for a team
pub fn senior_prompt(team: Team) -> &'static str {
    match team {
        Team::Desktop => DESKTOP_SENIOR_PROMPT,
        Team::Storage => STORAGE_SENIOR_PROMPT,
        Team::Network => NETWORK_SENIOR_PROMPT,
        Team::Performance => PERFORMANCE_SENIOR_PROMPT,
        Team::Services => SERVICES_SENIOR_PROMPT,
        Team::Security => SECURITY_SENIOR_PROMPT,
        Team::Hardware => HARDWARE_SENIOR_PROMPT,
        Team::General => GENERAL_SENIOR_PROMPT,
    }
}

/// Check if prompt contains required sections
pub fn prompt_has_required_sections(prompt: &str) -> bool {
    prompt.contains("## Role")
        && prompt.contains("## Inputs")
        && prompt.contains("## Output")
        && prompt.contains("## Rules")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prompts_exist_for_all_teams() {
        let teams = [
            Team::Desktop,
            Team::Storage,
            Team::Network,
            Team::Performance,
            Team::Services,
            Team::Security,
            Team::Hardware,
            Team::General,
        ];

        for team in teams {
            let junior = junior_prompt(team);
            let senior = senior_prompt(team);

            assert!(!junior.is_empty(), "Missing junior prompt for {:?}", team);
            assert!(!senior.is_empty(), "Missing senior prompt for {:?}", team);
        }
    }

    #[test]
    fn test_prompts_are_deterministic_strings() {
        let p1 = junior_prompt(Team::Storage);
        let p2 = junior_prompt(Team::Storage);

        assert_eq!(p1, p2);
        assert!(p1.contains("Storage"));
    }

    #[test]
    fn test_prompts_contain_required_sections() {
        let teams = [
            Team::Desktop,
            Team::Storage,
            Team::Network,
            Team::Performance,
            Team::Services,
            Team::Security,
            Team::Hardware,
            Team::General,
        ];

        for team in teams {
            let junior = junior_prompt(team);
            let senior = senior_prompt(team);

            assert!(
                prompt_has_required_sections(junior),
                "Junior prompt for {:?} missing required sections",
                team
            );
            assert!(
                prompt_has_required_sections(senior),
                "Senior prompt for {:?} missing required sections",
                team
            );
        }
    }

    #[test]
    fn test_senior_prompts_have_override_capability() {
        let teams = [Team::Storage, Team::Network, Team::General];

        for team in teams {
            let senior = senior_prompt(team);
            assert!(
                senior.contains("override"),
                "Senior prompt for {:?} should mention override capability",
                team
            );
        }
    }
}
