//! Tests for facts store and lifecycle management.

use anna_shared::facts::*;
use tempfile::tempdir;

#[test]
fn test_fact_key_display() {
    assert_eq!(FactKey::PreferredEditor.to_string(), "preferred_editor");
    assert_eq!(FactKey::EditorInstalled("vim".to_string()).to_string(), "editor_installed:vim");
    assert_eq!(FactKey::BinaryAvailable("nvim".to_string()).to_string(), "binary_available:nvim");
}

#[test]
fn test_fact_creation() {
    let fact = Fact::verified(FactKey::PreferredEditor, "vim".to_string(), "probe:which vim".to_string());
    assert!(fact.verified);
    assert_eq!(fact.value, "vim");
    assert_eq!(fact.source, "probe:which vim");
    assert_eq!(fact.lifecycle, FactLifecycle::Active);
}

#[test]
fn test_facts_store_set_get() {
    let mut store = FactsStore::new();
    store.set_verified(FactKey::PreferredEditor, "vim".to_string(), "user+verify".to_string());
    assert!(store.has_verified(&FactKey::PreferredEditor));
    assert_eq!(store.get_verified(&FactKey::PreferredEditor), Some("vim"));
}

#[test]
fn test_facts_store_unverified_not_returned_as_verified() {
    let mut store = FactsStore::new();
    store.set_unverified(FactKey::PreferredEditor, "vim".to_string(), "user_claim".to_string());
    assert!(!store.has_verified(&FactKey::PreferredEditor));
    assert_eq!(store.get_verified(&FactKey::PreferredEditor), None);
    assert!(store.get(&FactKey::PreferredEditor).is_some());
}

#[test]
fn test_facts_store_verify() {
    let mut store = FactsStore::new();
    store.set_unverified(FactKey::PreferredEditor, "vim".to_string(), "user_claim".to_string());
    assert!(!store.has_verified(&FactKey::PreferredEditor));
    store.verify(&FactKey::PreferredEditor, "probe:which vim".to_string());
    assert!(store.has_verified(&FactKey::PreferredEditor));
}

#[test]
fn test_facts_store_save_load() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("facts.json");
    let mut store = FactsStore::new();
    store.set_verified(FactKey::PreferredEditor, "vim".to_string(), "test".to_string());
    store.save_to_path(&path).unwrap();
    let loaded = FactsStore::load_from_path(&path);
    assert_eq!(loaded.get_verified(&FactKey::PreferredEditor), Some("vim"));
}

#[test]
fn test_fact_status() {
    let mut store = FactsStore::new();
    assert_eq!(store.status(&FactKey::PreferredEditor), FactStatus::Unknown);
    store.set_unverified(FactKey::PreferredEditor, "vim".to_string(), "claim".to_string());
    assert_eq!(store.status(&FactKey::PreferredEditor), FactStatus::Unverified("vim".to_string()));
    store.verify(&FactKey::PreferredEditor, "probe".to_string());
    assert_eq!(store.status(&FactKey::PreferredEditor), FactStatus::Known("vim".to_string()));
}

// === Lifecycle tests (v0.0.32) ===

#[test]
fn test_staleness_policy_defaults() {
    assert_eq!(default_policy(&FactKey::InitSystem), StalenessPolicy::Never);
    match default_policy(&FactKey::PreferredEditor) {
        // v0.0.41: editor preference TTL = 90 days
        StalenessPolicy::TTLSeconds(s) => assert_eq!(s, 90 * 24 * 3600),
        _ => panic!("expected TTL for PreferredEditor"),
    }
}

#[test]
fn test_fact_is_stale_with_ttl() {
    let mut fact = Fact::verified(FactKey::BinaryAvailable("vim".to_string()), "/usr/bin/vim".to_string(), "test".to_string());
    // Force a specific policy and timestamp for testing
    fact.policy = StalenessPolicy::TTLSeconds(3600); // 1 hour
    fact.last_verified_at = 1000;

    // Not stale if within TTL
    assert!(!fact.is_stale(1000 + 1800)); // 30 min later
    // Stale if beyond TTL
    assert!(fact.is_stale(1000 + 7200)); // 2 hours later
}

#[test]
fn test_fact_never_stale() {
    let mut fact = Fact::verified(FactKey::InitSystem, "systemd".to_string(), "test".to_string());
    fact.policy = StalenessPolicy::Never;
    fact.last_verified_at = 1000;
    // Never becomes stale
    assert!(!fact.is_stale(1000 + 365 * 24 * 3600)); // 1 year later
}

#[test]
fn test_lifecycle_transitions() {
    let mut store = FactsStore::new();
    store.set_verified(FactKey::PreferredEditor, "vim".to_string(), "test".to_string());

    // Force policy and timestamp
    if let Some(fact) = store.facts_mut().get_mut(&FactKey::PreferredEditor) {
        fact.policy = StalenessPolicy::TTLSeconds(3600);
        fact.last_verified_at = 1000;
    }

    // Initially active
    assert_eq!(store.status(&FactKey::PreferredEditor), FactStatus::Known("vim".to_string()));

    // Apply lifecycle with future time -> should become stale
    store.apply_lifecycle(1000 + 7200);
    assert_eq!(store.status(&FactKey::PreferredEditor), FactStatus::Stale("vim".to_string()));

    // Apply lifecycle with 2x TTL -> should archive
    store.apply_lifecycle(1000 + 3600 * 3);
    let fact = store.get(&FactKey::PreferredEditor).unwrap();
    assert_eq!(fact.lifecycle, FactLifecycle::Archived);
}

#[test]
fn test_invalidate_fact() {
    let mut store = FactsStore::new();
    store.set_verified(FactKey::BinaryAvailable("vim".to_string()), "/usr/bin/vim".to_string(), "test".to_string());
    assert!(store.has_verified(&FactKey::BinaryAvailable("vim".to_string())));

    // Invalidate (failed re-verification)
    store.invalidate(&FactKey::BinaryAvailable("vim".to_string()));

    // No longer usable
    assert!(!store.has_verified(&FactKey::BinaryAvailable("vim".to_string())));
    assert_eq!(store.status(&FactKey::BinaryAvailable("vim".to_string())),
               FactStatus::Stale("/usr/bin/vim".to_string()));
}

#[test]
fn test_reverify_fact() {
    let mut store = FactsStore::new();
    store.set_verified(FactKey::PreferredEditor, "vim".to_string(), "test".to_string());
    store.invalidate(&FactKey::PreferredEditor);
    assert!(!store.has_verified(&FactKey::PreferredEditor));

    // Re-verify
    store.reverify(&FactKey::PreferredEditor, "probe:which vim".to_string());
    assert!(store.has_verified(&FactKey::PreferredEditor));
    assert_eq!(store.status(&FactKey::PreferredEditor), FactStatus::Known("vim".to_string()));
}

#[test]
fn test_stale_facts_list() {
    let mut store = FactsStore::new();
    store.set_verified(FactKey::PreferredEditor, "vim".to_string(), "test".to_string());
    store.set_verified(FactKey::PreferredShell, "zsh".to_string(), "test".to_string());
    store.invalidate(&FactKey::PreferredEditor);

    let stale = store.stale_facts();
    assert_eq!(stale.len(), 1);
    assert_eq!(stale[0].value, "vim");
}

#[test]
fn test_prune_archived() {
    let mut store = FactsStore::new();
    store.set_verified(FactKey::PreferredEditor, "vim".to_string(), "test".to_string());
    store.set_verified(FactKey::PreferredShell, "zsh".to_string(), "test".to_string());

    // Archive one
    if let Some(fact) = store.facts_mut().get_mut(&FactKey::PreferredEditor) {
        fact.archive();
    }

    let removed = store.prune_archived();
    assert_eq!(removed, 1);
    assert!(store.get(&FactKey::PreferredEditor).is_none());
    assert!(store.get(&FactKey::PreferredShell).is_some());
}

#[test]
fn test_is_usable() {
    let fact = Fact::verified(FactKey::PreferredEditor, "vim".to_string(), "test".to_string());
    assert!(fact.is_usable());

    let mut stale_fact = fact.clone();
    stale_fact.mark_stale();
    assert!(!stale_fact.is_usable());

    let mut archived_fact = fact.clone();
    archived_fact.archive();
    assert!(!archived_fact.is_usable());

    let unverified = Fact::unverified(FactKey::PreferredEditor, "vim".to_string(), "claim".to_string());
    assert!(!unverified.is_usable());
}
