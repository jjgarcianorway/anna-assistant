//! Tests for staff statistics system

use super::*;

#[test]
fn test_staff_metrics_record() {
    let mut metrics = StaffMetrics::default();
    metrics.record_ticket(true, false, 85, 1000);
    metrics.record_ticket(true, false, 75, 2000);

    assert_eq!(metrics.tickets_handled, 2);
    assert_eq!(metrics.tickets_resolved, 2);
    assert_eq!(metrics.avg_reliability, 80.0);
    assert_eq!(metrics.avg_time_ms(), 1500);
}

#[test]
fn test_staff_metrics_success_rate() {
    let mut metrics = StaffMetrics::default();
    metrics.record_ticket(true, false, 80, 1000);
    metrics.record_ticket(false, true, 50, 5000);

    assert_eq!(metrics.success_rate(), 50.0);
}

#[test]
fn test_staff_stats_record() {
    let mut stats = StaffStats::default();
    stats.record_ticket("desktop_jr_sofia", true, false, 90, 500);
    stats.record_ticket("desktop_jr_sofia", true, false, 80, 600);
    stats.record_ticket("network_jr_michael", true, false, 85, 700);

    assert_eq!(stats.by_staff.len(), 2);
    assert_eq!(
        stats
            .by_staff
            .get("desktop_jr_sofia")
            .unwrap()
            .tickets_handled,
        2
    );
}

#[test]
fn test_top_performers() {
    let mut stats = StaffStats::default();
    stats.record_ticket("a", true, false, 80, 100);
    stats.record_ticket("b", true, false, 80, 100);
    stats.record_ticket("b", true, false, 80, 100);
    stats.record_ticket("c", true, false, 80, 100);
    stats.record_ticket("c", true, false, 80, 100);
    stats.record_ticket("c", true, false, 80, 100);

    let top = stats.top_performers(2);
    assert_eq!(top.len(), 2);
    assert_eq!(top[0].0, "c");
    assert_eq!(top[1].0, "b");
}

#[test]
fn test_xp_calculation() {
    let mut metrics = StaffMetrics::default();
    // v0.0.315: Base: 10xp + reliability bonus: (85-60)*2 = 50xp = 60 total
    metrics.record_ticket(true, false, 85, 1000);
    assert_eq!(metrics.xp, 60); // 10 + 50
    assert_eq!(metrics.level, 1); // < 100 = Novice
}

#[test]
fn test_xp_level_progression() {
    let mut metrics = StaffMetrics::default();
    // Simulate 5 high-reliability resolved tickets
    for _ in 0..5 {
        metrics.record_ticket(true, false, 90, 1000);
    }
    // v0.0.315: Each ticket: 10 + (90-60)*2 + 15 = 85 xp, total = 425 xp
    assert_eq!(metrics.xp, 425);
    assert_eq!(metrics.level, 3); // 300-699 = Competent
}

#[test]
fn test_xp_penalty() {
    let mut metrics = StaffMetrics::default();
    // First earn some XP
    metrics.record_ticket(true, false, 80, 1000); // +50 xp (10 + 40)
    assert_eq!(metrics.xp, 50);

    // v0.0.315: Unresolved with low reliability = penalty
    metrics.record_ticket(false, false, 30, 1000); // -15 xp
    assert_eq!(metrics.xp, 35);

    // Unresolved with medium reliability = smaller penalty
    metrics.record_ticket(false, false, 50, 1000); // -5 xp
    assert_eq!(metrics.xp, 30);

    // Unresolved but decent reliability = no penalty
    metrics.record_ticket(false, false, 70, 1000); // 0 xp change
    assert_eq!(metrics.xp, 30);

    // XP can't go below 0
    let mut fresh = StaffMetrics::default();
    fresh.record_ticket(false, false, 20, 1000); // -15 but floor at 0
    assert_eq!(fresh.xp, 0);
}

#[test]
fn test_staff_feedback() {
    let mut stats = StaffStats::default();
    // First create a staff entry via record_ticket
    stats.record_ticket("desktop_jr", true, false, 80, 1000);
    let initial_xp = stats.by_staff.get("desktop_jr").unwrap().xp;

    // Test positive feedback (+5 XP)
    let result = stats.apply_feedback("desktop_jr", true);
    assert!(result.is_some());
    let r = result.unwrap();
    assert_eq!(r.new_xp, initial_xp + 5);
    assert!(r.helpful);

    // Test negative feedback (-10 XP)
    let before = stats.by_staff.get("desktop_jr").unwrap().xp;
    let result = stats.apply_feedback("desktop_jr", false);
    assert!(result.is_some());
    let r = result.unwrap();
    assert_eq!(r.new_xp, before.saturating_sub(10));
    assert!(!r.helpful);

    // Test feedback for non-existent staff returns None
    assert!(stats.apply_feedback("unknown_person", true).is_none());
}

#[test]
fn test_xp_to_level() {
    assert_eq!(xp_to_level(0), 1); // Novice
    assert_eq!(xp_to_level(99), 1); // Novice
    assert_eq!(xp_to_level(100), 2); // Apprentice
    assert_eq!(xp_to_level(299), 2); // Apprentice
    assert_eq!(xp_to_level(300), 3); // Competent
    assert_eq!(xp_to_level(699), 3); // Competent
    assert_eq!(xp_to_level(700), 4); // Expert
    assert_eq!(xp_to_level(1500), 5); // Master
    assert_eq!(xp_to_level(3000), 6); // Principal
}
