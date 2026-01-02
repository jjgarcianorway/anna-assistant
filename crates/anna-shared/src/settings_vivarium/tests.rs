use super::*;

#[test]
fn test_vivarium_type_display() {
    assert_eq!(format!("{}", VivariumType::Reptile), "reptile");
    assert_eq!(format!("{}", VivariumType::Amphibian), "amphibian");
}

#[test]
fn test_status_display() {
    assert_eq!(format!("{}", VivariumStatus::Setup), "setup");
    assert_eq!(format!("{}", VivariumStatus::Established), "established");
}

#[test]
fn test_config_new() {
    let c = VivariumConfig::new("test");
    assert_eq!(c.name, "test");
}

#[test]
fn test_config_builder() {
    let c = VivariumConfig::new("test")
        .vivarium_type(VivariumType::Invertebrate)
        .status(VivariumStatus::Breeding);
    assert_eq!(c.vivarium_type, VivariumType::Invertebrate);
    assert_eq!(c.status, VivariumStatus::Breeding);
}

#[test]
fn test_creature_new() {
    let c = VivariumCreature::new("c1", "Title", "Content");
    assert_eq!(c.id, "c1");
}

#[test]
fn test_creature_builder() {
    let c = VivariumCreature::new("c1", "Title", "Content")
        .enclosure(1);
    assert_eq!(c.enclosure, 1);
}

#[test]
fn test_creature_thriving() {
    let mut c = VivariumCreature::new("c1", "Title", "Content");
    c.make_struggling();
    assert!(!c.thriving);
    c.make_thriving();
    assert!(c.thriving);
}

#[test]
fn test_keeper_new() {
    let k = VivariumKeeper::new("key", "name", "c1");
    assert_eq!(k.creature_id, "c1");
}

#[test]
fn test_stats_update() {
    let mut s = VivariumStats::default();
    let creature = VivariumCreature::new("c1", "Title", "Content");
    s.update(&[creature], VivariumType::Reptile);
    assert_eq!(s.total_creatures, 1);
    assert_eq!(s.thriving, 1);
}

#[test]
fn test_vivarium_new() {
    let v = SettingsVivarium::new(VivariumConfig::default());
    assert_eq!(v.creature_count(), 0);
}

#[test]
fn test_vivarium_add_creature() {
    let mut v = SettingsVivarium::new(VivariumConfig::default());
    v.add_creature(VivariumCreature::new("c1", "Title", "Content"));
    assert_eq!(v.creature_count(), 1);
}

#[test]
fn test_registry_new() {
    let r = VivariumRegistry::new();
    assert_eq!(r.count(), 0);
}

#[test]
fn test_registry_register() {
    let mut r = VivariumRegistry::new();
    r.register("v1", SettingsVivarium::new(VivariumConfig::default()));
    assert_eq!(r.count(), 1);
}

#[test]
fn test_is_vivarium_query() {
    assert!(is_vivarium_query("settings vivarium"));
    assert!(!is_vivarium_query("hello world"));
}

#[test]
fn test_fun_fact() {
    let fact = vivarium_fun_fact();
    assert!(fact.contains("vivarium"));
}
