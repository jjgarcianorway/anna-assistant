//! Tests for specialists registry.

use anna_shared::specialists::{SpecialistProfile, SpecialistRole, SpecialistsRegistry};
use anna_shared::teams::Team;

#[test]
fn test_specialist_role_display() {
    assert_eq!(SpecialistRole::Translator.to_string(), "translator");
    assert_eq!(SpecialistRole::Junior.to_string(), "junior");
    assert_eq!(SpecialistRole::Senior.to_string(), "senior");
}

#[test]
fn test_specialist_profile_new() {
    let profile = SpecialistProfile::new(Team::Storage, SpecialistRole::Junior);

    assert_eq!(profile.team, Team::Storage);
    assert_eq!(profile.role, SpecialistRole::Junior);
    assert_eq!(profile.model_id, "local-default");
    assert_eq!(profile.max_rounds, 3);
    assert_eq!(profile.escalation_threshold, 60);
    assert_eq!(profile.style_id, "storage-junior");
}

#[test]
fn test_specialist_profile_builders() {
    let profile = SpecialistProfile::new(Team::Network, SpecialistRole::Senior)
        .with_model("llama3.2")
        .with_max_rounds(5)
        .with_escalation_threshold(70);

    assert_eq!(profile.model_id, "llama3.2");
    assert_eq!(profile.max_rounds, 5);
    assert_eq!(profile.escalation_threshold, 70);
}

#[test]
fn test_registry_with_defaults() {
    let registry = SpecialistsRegistry::with_defaults();

    // 8 teams × 3 roles = 24 profiles
    assert_eq!(registry.len(), 24);
    assert!(registry.is_complete());
}

#[test]
fn test_registry_lookup() {
    let registry = SpecialistsRegistry::with_defaults();

    let junior = registry.get(Team::Storage, SpecialistRole::Junior);
    assert!(junior.is_some());
    assert_eq!(junior.unwrap().team, Team::Storage);
    assert_eq!(junior.unwrap().role, SpecialistRole::Junior);

    let senior = registry.get(Team::Performance, SpecialistRole::Senior);
    assert!(senior.is_some());
    assert_eq!(senior.unwrap().team, Team::Performance);
}

#[test]
fn test_registry_is_complete() {
    let registry = SpecialistsRegistry::with_defaults();
    assert!(registry.is_complete());

    let empty = SpecialistsRegistry::new();
    assert!(!empty.is_complete());
}

#[test]
fn test_deterministic_defaults_stable() {
    let registry1 = SpecialistsRegistry::with_defaults();
    let registry2 = SpecialistsRegistry::with_defaults();

    // Same number of profiles
    assert_eq!(registry1.len(), registry2.len());

    // Same profiles in same order
    for (p1, p2) in registry1.profiles().iter().zip(registry2.profiles().iter()) {
        assert_eq!(p1.team, p2.team);
        assert_eq!(p1.role, p2.role);
        assert_eq!(p1.model_id, p2.model_id);
        assert_eq!(p1.max_rounds, p2.max_rounds);
        assert_eq!(p1.escalation_threshold, p2.escalation_threshold);
    }
}

#[test]
fn test_all_teams_have_junior_and_senior() {
    let registry = SpecialistsRegistry::with_defaults();

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
        assert!(
            registry.get(team, SpecialistRole::Junior).is_some(),
            "Missing junior for {:?}",
            team
        );
        assert!(
            registry.get(team, SpecialistRole::Senior).is_some(),
            "Missing senior for {:?}",
            team
        );
    }
}

#[test]
fn test_registry_set() {
    let mut registry = SpecialistsRegistry::with_defaults();

    let custom = SpecialistProfile::new(Team::Security, SpecialistRole::Junior)
        .with_model("security-model");

    registry.set(custom);

    let profile = registry.get(Team::Security, SpecialistRole::Junior).unwrap();
    assert_eq!(profile.model_id, "security-model");

    // Count should not change (updated existing)
    assert_eq!(registry.len(), 24);
}

// v0.0.28 tests

#[test]
fn test_profile_prompt_junior() {
    let profile = SpecialistProfile::new(Team::Storage, SpecialistRole::Junior);
    let prompt = profile.prompt();
    assert!(prompt.contains("Storage"));
    assert!(prompt.contains("Junior"));
}

#[test]
fn test_profile_prompt_senior() {
    let profile = SpecialistProfile::new(Team::Network, SpecialistRole::Senior);
    let prompt = profile.prompt();
    assert!(prompt.contains("Network"));
    assert!(prompt.contains("Senior"));
}

#[test]
fn test_registry_junior_prompt() {
    let registry = SpecialistsRegistry::with_defaults();
    let prompt = registry.junior_prompt(Team::Storage);
    assert!(prompt.contains("Storage"));
    assert!(prompt.contains("Junior"));
}

#[test]
fn test_registry_senior_prompt() {
    let registry = SpecialistsRegistry::with_defaults();
    let prompt = registry.senior_prompt(Team::Network);
    assert!(prompt.contains("Network"));
    assert!(prompt.contains("Senior"));
}

#[test]
fn test_registry_junior_model() {
    let registry = SpecialistsRegistry::with_defaults();
    let model = registry.junior_model(Team::Storage);
    assert_eq!(model, Some("local-default"));
}

#[test]
fn test_registry_escalation_threshold() {
    let registry = SpecialistsRegistry::with_defaults();
    let threshold = registry.escalation_threshold(Team::Storage);
    assert_eq!(threshold, 60);
}
