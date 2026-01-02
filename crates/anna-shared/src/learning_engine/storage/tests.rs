//! Tests for recipe storage.

use super::*;
use crate::learning_engine::{LearnedRecipe, RecipePattern};

fn make_recipe(id: &str, domain: &str, intent: &str) -> LearnedRecipe {
    LearnedRecipe::new(id, domain).with_pattern(RecipePattern::new(intent))
}

#[test]
fn test_library_add() {
    let mut lib = RecipeLibrary::new();
    let recipe = make_recipe("test-1", "memory", "check_ram");

    lib.add(recipe).unwrap();
    assert_eq!(lib.len(), 1);
    assert!(lib.get("test-1").is_some());
}

#[test]
fn test_library_duplicate() {
    let mut lib = RecipeLibrary::new();
    let recipe = make_recipe("test-1", "memory", "check_ram");

    lib.add(recipe.clone()).unwrap();
    let result = lib.add(recipe);
    assert!(result.is_err());
}

#[test]
fn test_library_indexes() {
    let mut lib = RecipeLibrary::new();
    lib.add(make_recipe("mem-1", "memory", "check_ram"))
        .unwrap();
    lib.add(make_recipe("mem-2", "memory", "check_swap"))
        .unwrap();
    lib.add(make_recipe("disk-1", "disk", "check_disk"))
        .unwrap();

    let memory_recipes = lib.by_domain("memory");
    assert_eq!(memory_recipes.len(), 2);

    let ram_recipes = lib.by_intent("check_ram");
    assert_eq!(ram_recipes.len(), 1);
}

#[test]
fn test_library_disable() {
    let mut lib = RecipeLibrary::new();
    lib.add(make_recipe("test-1", "memory", "check_ram"))
        .unwrap();

    assert!(lib.get("test-1").unwrap().enabled);
    lib.disable("test-1");
    assert!(!lib.get("test-1").unwrap().enabled);

    let enabled = lib.enabled();
    assert!(enabled.is_empty());
}

#[test]
fn test_library_stats() {
    let mut lib = RecipeLibrary::new();
    lib.add(make_recipe("test-1", "memory", "check_ram"))
        .unwrap();

    lib.record_success("test-1");
    lib.record_success("test-1");
    lib.record_failure("test-1");

    let recipe = lib.get("test-1").unwrap();
    assert_eq!(recipe.stats.uses, 3);
    assert_eq!(recipe.stats.successes, 2);
}
