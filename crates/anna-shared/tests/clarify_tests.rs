//! Tests for clarify.rs

use anna_shared::clarify::{
    build_verify_command, generate_editor_clarification, generate_editor_options_sync,
    generate_editor_options_with_cache, generate_question, is_cancel_selection, is_other_selection,
    kind_to_fact_key, needs_clarification, ClarifyAnswer, ClarifyKind, ClarifyOption,
    CLARIFY_CANCEL_KEY, CLARIFY_OTHER_KEY, KNOWN_EDITORS,
};
use anna_shared::facts::{FactKey, FactsStore};
use anna_shared::inventory::{InventoryCache, InventoryItem};

#[test]
fn test_generate_editor_question() {
    let facts = FactsStore::new();
    let q = generate_question(ClarifyKind::PreferredEditor, &facts);
    assert!(q.question.contains("editor"));
    assert!(q.verify_probe.is_some());
}

#[test]
fn test_generate_editor_with_default() {
    let mut facts = FactsStore::new();
    facts.set_verified(
        FactKey::PreferredEditor,
        "vim".to_string(),
        "user".to_string(),
    );
    let q = generate_question(ClarifyKind::PreferredEditor, &facts);
    assert_eq!(q.default, Some("vim".to_string()));
}

#[test]
fn test_build_verify_command() {
    let cmd = build_verify_command("which {}", "vim");
    assert_eq!(cmd, "which vim");

    let cmd = build_verify_command("systemctl is-active {}", "nginx");
    assert_eq!(cmd, "systemctl is-active nginx");
}

#[test]
fn test_kind_to_fact_key() {
    assert_eq!(
        kind_to_fact_key(&ClarifyKind::PreferredEditor, "vim"),
        Some(FactKey::PreferredEditor)
    );
    assert_eq!(
        kind_to_fact_key(&ClarifyKind::ServiceName, "nginx"),
        Some(FactKey::UnitExists("nginx".to_string()))
    );
}

#[test]
fn test_needs_clarification_editor() {
    let facts = FactsStore::new();

    // Needs clarification
    assert_eq!(
        needs_clarification("edit the config file", &facts),
        Some(ClarifyKind::PreferredEditor)
    );

    // Already specified editor
    assert_eq!(needs_clarification("open in vim", &facts), None);
}

#[test]
fn test_needs_clarification_with_known_fact() {
    let mut facts = FactsStore::new();
    facts.set_verified(
        FactKey::PreferredEditor,
        "vim".to_string(),
        "user".to_string(),
    );

    // No clarification needed - we know their editor
    assert_eq!(needs_clarification("edit the config file", &facts), None);
}

#[test]
fn test_known_editors_list() {
    assert!(KNOWN_EDITORS.contains(&"vim"));
    assert!(KNOWN_EDITORS.contains(&"nano"));
    assert!(KNOWN_EDITORS.contains(&"nvim"));
}

#[test]
fn test_clarify_option_with_evidence() {
    let opt = ClarifyOption::new("vim", "Vim")
        .with_evidence("installed: true")
        .with_evidence("recently used: 5 times");
    assert_eq!(opt.evidence.len(), 2);
}

#[test]
fn test_clarify_answer_structure() {
    let answer = ClarifyAnswer {
        question_id: "q1".to_string(),
        selected_key: "vim".to_string(),
    };
    assert_eq!(answer.question_id, "q1");
    assert_eq!(answer.selected_key, "vim");
}

#[test]
fn test_generate_editor_clarification() {
    let facts = FactsStore::new();
    let (question, _options) = generate_editor_clarification(&facts);
    assert_eq!(question.kind, ClarifyKind::PreferredEditor);
    assert!(question.question.contains("editor"));
}

// v0.0.36: Cancel/Other option tests
#[test]
fn test_is_cancel_selection() {
    assert!(is_cancel_selection(CLARIFY_CANCEL_KEY));
    assert!(!is_cancel_selection("vim"));
    assert!(!is_cancel_selection(CLARIFY_OTHER_KEY));
}

#[test]
fn test_is_other_selection() {
    assert!(is_other_selection(CLARIFY_OTHER_KEY));
    assert!(!is_other_selection("vim"));
    assert!(!is_other_selection(CLARIFY_CANCEL_KEY));
}

#[test]
fn test_editor_options_always_have_cancel_and_other() {
    let options = generate_editor_options_sync();
    // Should always have at least Other and Cancel
    assert!(options.len() >= 2);
    // Last should be Cancel
    assert_eq!(options.last().unwrap().key, CLARIFY_CANCEL_KEY);
    // Second to last should be Other
    let other_pos = options.len() - 2;
    assert_eq!(options[other_pos].key, CLARIFY_OTHER_KEY);
}

// v0.0.39: Test inventory cache integration
#[test]
fn test_editor_options_with_cache() {
    let mut cache = InventoryCache::new();
    // Manually add vim as installed
    cache
        .items
        .insert("vim".to_string(), InventoryItem::installed("vim", "/usr/bin/vim"));
    cache
        .items
        .insert("nano".to_string(), InventoryItem::not_installed("nano"));

    let options = generate_editor_options_with_cache(&cache);

    // Should have vim (installed), Other, and Cancel
    assert!(options.len() >= 3);

    // Check vim is present with installed evidence
    let vim_opt = options.iter().find(|o| o.key == "vim");
    assert!(vim_opt.is_some());
    assert!(vim_opt.unwrap().evidence.iter().any(|e| e.contains("installed")));

    // Check nano is NOT present (not installed)
    let nano_opt = options.iter().find(|o| o.key == "nano");
    assert!(nano_opt.is_none());
}

#[test]
fn test_editor_options_with_empty_cache_fallback() {
    let cache = InventoryCache::new();
    let options = generate_editor_options_with_cache(&cache);

    // Should still have at least Other and Cancel
    assert!(options.len() >= 2);
    assert_eq!(options.last().unwrap().key, CLARIFY_CANCEL_KEY);
}
