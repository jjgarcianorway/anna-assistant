// v0.0.767: Settings Vineyard Tests
// Tests for vineyard module

use super::*;

#[test]
fn test_vineyard_type_display() {
    assert_eq!(format!("{}", VineyardType::RedWine), "red-wine");
    assert_eq!(format!("{}", VineyardType::WhiteWine), "white-wine");
}

#[test]
fn test_status_display() {
    assert_eq!(format!("{}", VineyardStatus::Pruned), "pruned");
    assert_eq!(format!("{}", VineyardStatus::Vintage), "vintage");
}

#[test]
fn test_config_new() {
    let c = VineyardConfig::new("test");
    assert_eq!(c.name, "test");
}

#[test]
fn test_config_builder() {
    let c = VineyardConfig::new("test")
        .vineyard_type(VineyardType::TableGrape)
        .status(VineyardStatus::Ripening);
    assert_eq!(c.vineyard_type, VineyardType::TableGrape);
    assert_eq!(c.status, VineyardStatus::Ripening);
}

#[test]
fn test_vine_new() {
    let v = VineyardVine::new("v1", "Title", "Content");
    assert_eq!(v.id, "v1");
}

#[test]
fn test_vine_builder() {
    let v = VineyardVine::new("v1", "Title", "Content")
        .terrace(1);
    assert_eq!(v.terrace, 1);
}

#[test]
fn test_vine_bearing() {
    let mut v = VineyardVine::new("v1", "Title", "Content");
    v.make_dormant();
    assert!(!v.bearing);
    v.make_bearing();
    assert!(v.bearing);
}

#[test]
fn test_vintner_new() {
    let v = VineyardVintner::new("key", "name", "v1");
    assert_eq!(v.vine_id, "v1");
}

#[test]
fn test_stats_update() {
    let mut s = VineyardStats::default();
    let vine = VineyardVine::new("v1", "Title", "Content");
    s.update(&[vine], VineyardType::RedWine);
    assert_eq!(s.total_vines, 1);
    assert_eq!(s.bearing, 1);
}

#[test]
fn test_vineyard_new() {
    let v = SettingsVineyard::new(VineyardConfig::default());
    assert_eq!(v.vine_count(), 0);
}

#[test]
fn test_vineyard_add_vine() {
    let mut v = SettingsVineyard::new(VineyardConfig::default());
    v.add_vine(VineyardVine::new("v1", "Title", "Content"));
    assert_eq!(v.vine_count(), 1);
}

#[test]
fn test_registry_new() {
    let r = VineyardRegistry::new();
    assert_eq!(r.count(), 0);
}

#[test]
fn test_registry_register() {
    let mut r = VineyardRegistry::new();
    r.register("v1", SettingsVineyard::new(VineyardConfig::default()));
    assert_eq!(r.count(), 1);
}

#[test]
fn test_is_vineyard_query() {
    assert!(is_vineyard_query("settings vineyard"));
    assert!(!is_vineyard_query("hello world"));
}

#[test]
fn test_fun_fact() {
    let fact = vineyard_fun_fact();
    assert!(fact.contains("vineyard"));
}
