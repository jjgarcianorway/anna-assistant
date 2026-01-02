//! Knowledge Learning Tests

use super::store::KnowledgeLearningStore;
use super::types::SolvedTicketRecord;
use crate::intent_policy::IntentCategory;

fn current_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

#[test]
fn test_record_ticket() {
    let mut store = KnowledgeLearningStore::default();

    let record = SolvedTicketRecord {
        ticket_id: "TEST-001".to_string(),
        intent: IntentCategory::DiagnoseServiceFailure,
        domain: "services".to_string(),
        query_pattern: "diagnose_service_failure".to_string(),
        probes_used: vec!["systemctl_failed".to_string()],
        probe_effectiveness: [("systemctl_failed".to_string(), 85)].into_iter().collect(),
        docs_consulted: vec![],
        answer_confidence: 90,
        was_grounded: true,
        citations_used: vec!["man systemctl".to_string()],
        timestamp: current_secs(),
        feedback: None,
    };

    store.record_ticket(record);
    assert_eq!(store.stats.tickets_recorded, 1);
    assert!(store.stats.grounding_rate > 0.9);
}

#[test]
fn test_effective_probes() {
    let mut store = KnowledgeLearningStore::default();

    // Add multiple tickets with same probe being effective
    for i in 0..5 {
        let record = SolvedTicketRecord {
            ticket_id: format!("TEST-{:03}", i),
            intent: IntentCategory::DiagnoseServiceFailure,
            domain: "services".to_string(),
            query_pattern: "diagnose_service_failure".to_string(),
            probes_used: vec!["systemctl_failed".to_string()],
            probe_effectiveness: [("systemctl_failed".to_string(), 85)].into_iter().collect(),
            docs_consulted: vec![],
            answer_confidence: 90,
            was_grounded: true,
            citations_used: vec![],
            timestamp: current_secs(),
            feedback: None,
        };
        store.record_ticket(record);
    }

    let effective = store.effective_probes_for_intent(IntentCategory::DiagnoseServiceFailure);
    assert!(effective.contains(&"systemctl_failed".to_string()));
}

#[test]
fn test_analyze_and_propose() {
    let mut store = KnowledgeLearningStore::default();

    // Add enough tickets to trigger proposal
    for i in 0..5 {
        let record = SolvedTicketRecord {
            ticket_id: format!("TEST-{:03}", i),
            intent: IntentCategory::InspectDiskUsage,
            domain: "storage".to_string(),
            query_pattern: "inspect_disk_usage".to_string(),
            probes_used: vec!["df_root".to_string(), "lsblk".to_string()],
            probe_effectiveness: [("df_root".to_string(), 90), ("lsblk".to_string(), 75)]
                .into_iter()
                .collect(),
            docs_consulted: vec![],
            answer_confidence: 85,
            was_grounded: true,
            citations_used: vec![],
            timestamp: current_secs(),
            feedback: None,
        };
        store.record_ticket(record);
    }

    let proposals = store.analyze_and_propose();
    assert!(!proposals.is_empty());
    assert!(proposals[0].probes.contains(&"df_root".to_string()));
}
