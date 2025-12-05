//! Tests for deterministic answer generation and routing.

use anna_shared::parsers::{
    BlockDevice, BlockDeviceType, CpuInfo, DiskUsage, MemoryInfo, ParsedProbeData, ServiceState,
    ServiceStatus,
};
use annad::answers::{format_bytes_human, generate_answer, generate_health_summary};
use annad::router::{classify_query, get_route, QueryClass};

#[test]
fn test_memory_answer_format() {
    let data = ParsedProbeData::Memory(MemoryInfo {
        total_bytes: 16_000_000_000,
        used_bytes: 8_000_000_000,
        free_bytes: 4_000_000_000,
        shared_bytes: 100_000_000,
        buff_cache_bytes: 2_000_000_000,
        available_bytes: 6_000_000_000,
        swap_total_bytes: None,
        swap_used_bytes: None,
        swap_free_bytes: None,
    });

    let answer = generate_answer(QueryClass::MemoryUsage, &data).unwrap();

    // Verify extractable claims
    assert!(answer.contains("8000000000B used"));
    assert!(answer.contains("16000000000B total"));
    assert!(answer.contains("6000000000B available"));
}

#[test]
fn test_disk_answer_format() {
    let data = ParsedProbeData::Disk(vec![
        DiskUsage {
            filesystem: "/dev/sda1".to_string(),
            mount: "/".to_string(),
            size_bytes: 100_000_000_000,
            used_bytes: 85_000_000_000,
            available_bytes: 15_000_000_000,
            percent_used: 85,
        },
        DiskUsage {
            filesystem: "/dev/sdb1".to_string(),
            mount: "/home".to_string(),
            size_bytes: 500_000_000_000,
            used_bytes: 250_000_000_000,
            available_bytes: 250_000_000_000,
            percent_used: 50,
        },
    ]);

    let answer = generate_answer(QueryClass::DiskUsage, &data).unwrap();

    // Verify extractable claims
    assert!(answer.contains("/ is 85% full"));
    assert!(answer.contains("/home is 50% full"));
}

#[test]
fn test_service_answer_format() {
    let data = ParsedProbeData::Services(vec![
        ServiceStatus {
            name: "nginx.service".to_string(),
            state: ServiceState::Running,
            description: None,
        },
        ServiceStatus {
            name: "sshd.service".to_string(),
            state: ServiceState::Active,
            description: None,
        },
    ]);

    let answer = generate_answer(QueryClass::ServiceStatus, &data).unwrap();

    // Verify extractable claims
    assert!(answer.contains("nginx.service is running"));
    assert!(answer.contains("sshd.service is active"));
}

#[test]
fn test_wrong_data_type_returns_none() {
    let memory_data = ParsedProbeData::Memory(MemoryInfo {
        total_bytes: 16_000_000_000,
        used_bytes: 8_000_000_000,
        free_bytes: 4_000_000_000,
        shared_bytes: 0,
        buff_cache_bytes: 0,
        available_bytes: 6_000_000_000,
        swap_total_bytes: None,
        swap_used_bytes: None,
        swap_free_bytes: None,
    });

    // DiskUsage class with memory data should return None
    assert!(generate_answer(QueryClass::DiskUsage, &memory_data).is_none());
}

#[test]
fn test_format_bytes_human() {
    assert_eq!(format_bytes_human(500), "500B");
    assert_eq!(format_bytes_human(1024), "1.0 KiB (1024B)");
    assert_eq!(format_bytes_human(1_073_741_824), "1.0 GiB (1073741824B)");
    assert_eq!(format_bytes_human(8_804_682_957), "8.2 GiB (8804682957B)");
}

#[test]
fn test_health_summary_format() {
    let probes = vec![
        ParsedProbeData::Memory(MemoryInfo {
            total_bytes: 16_000_000_000,
            used_bytes: 8_000_000_000,
            free_bytes: 4_000_000_000,
            shared_bytes: 0,
            buff_cache_bytes: 2_000_000_000,
            available_bytes: 6_000_000_000,
            swap_total_bytes: None,
            swap_used_bytes: None,
            swap_free_bytes: None,
        }),
        ParsedProbeData::Cpu(CpuInfo {
            architecture: "x86_64".to_string(),
            model_name: "Intel Core i9".to_string(),
            cpu_count: 16,
            cores_per_socket: Some(8),
            threads_per_core: Some(2),
            sockets: Some(1),
            ..Default::default()
        }),
        ParsedProbeData::Disk(vec![DiskUsage {
            filesystem: "/dev/sda1".to_string(),
            mount: "/".to_string(),
            size_bytes: 100_000_000_000,
            used_bytes: 75_000_000_000,
            available_bytes: 25_000_000_000,
            percent_used: 75,
        }]),
    ];

    let summary = generate_health_summary(&probes).unwrap();
    assert!(summary.contains("System Health Summary"));
    assert!(summary.contains("Intel Core i9"));
    assert!(summary.contains("Logical CPUs: 16"));
    assert!(summary.contains("/ is 75% full"));
}

#[test]
fn test_health_summary_with_block_devices() {
    let probes = vec![
        ParsedProbeData::BlockDevices(vec![
            BlockDevice {
                name: "nvme0n1".to_string(),
                size_bytes: 1_000_000_000_000,
                device_type: BlockDeviceType::Disk,
                mountpoints: vec![],
                parent: None,
                read_only: false,
            },
            BlockDevice {
                name: "sda".to_string(),
                size_bytes: 500_000_000_000,
                device_type: BlockDeviceType::Disk,
                mountpoints: vec![],
                parent: None,
                read_only: false,
            },
        ]),
    ];

    let summary = generate_health_summary(&probes).unwrap();
    assert!(summary.contains("Storage:"));
    assert!(summary.contains("nvme0n1"));
    assert!(summary.contains("sda"));
}

#[test]
fn test_health_summary_empty_returns_none() {
    let probes: Vec<ParsedProbeData> = vec![];
    assert!(generate_health_summary(&probes).is_none());
}

#[test]
fn test_health_summary_disk_warnings() {
    let probes = vec![
        ParsedProbeData::Disk(vec![
            DiskUsage {
                filesystem: "/dev/sda1".to_string(),
                mount: "/".to_string(),
                size_bytes: 100_000_000_000,
                used_bytes: 95_000_000_000,
                available_bytes: 5_000_000_000,
                percent_used: 95, // Critical
            },
            DiskUsage {
                filesystem: "/dev/sdb1".to_string(),
                mount: "/home".to_string(),
                size_bytes: 500_000_000_000,
                used_bytes: 425_000_000_000,
                available_bytes: 75_000_000_000,
                percent_used: 85, // Warning
            },
        ]),
    ];

    let summary = generate_health_summary(&probes).unwrap();
    assert!(summary.contains("[CRITICAL]"));
    assert!(summary.contains("[WARNING]"));
}

// === SystemHealthSummary Routing Tests ===
// v0.0.35: "health" routes to SystemTriage (fast path), "summary" to SystemHealthSummary

#[test]
fn test_system_health_summary_routing_health() {
    // v0.0.35: "health" now routes to SystemTriage for fast error checking
    assert_eq!(classify_query("system health"), QueryClass::SystemTriage);
    assert_eq!(classify_query("health check"), QueryClass::SystemTriage);
    assert_eq!(classify_query("show health"), QueryClass::SystemTriage);
}

#[test]
fn test_system_health_summary_routing_summary() {
    assert_eq!(classify_query("system summary"), QueryClass::SystemHealthSummary);
    assert_eq!(classify_query("show summary"), QueryClass::SystemHealthSummary);
}

#[test]
fn test_system_health_summary_routing_status_report() {
    assert_eq!(classify_query("status report"), QueryClass::SystemHealthSummary);
    assert_eq!(classify_query("system status"), QueryClass::SystemHealthSummary);
}

#[test]
fn test_system_health_summary_routing_overview() {
    assert_eq!(classify_query("system overview"), QueryClass::SystemHealthSummary);
    assert_eq!(classify_query("show overview"), QueryClass::SystemHealthSummary);
}

#[test]
fn test_system_health_summary_probes() {
    // v0.0.35: "system health" now routes to SystemTriage, test the full summary route
    let route = get_route("system summary");
    assert_eq!(route.class, QueryClass::SystemHealthSummary);
    // v0.45.x: Probes use command-style names
    assert!(route.probes.contains(&"disk_usage".to_string()));
    assert!(route.probes.contains(&"free".to_string()));
    assert!(route.probes.contains(&"failed_units".to_string()));
    assert!(route.probes.contains(&"top_cpu".to_string()));
    // v0.45.x: SystemHealthSummary needs LLM interpretation, not deterministic
    assert!(!route.can_answer_deterministically());
}

#[test]
fn test_system_triage_probes() {
    // v0.0.35: SystemTriage fast path probes
    let route = get_route("system health");
    assert_eq!(route.class, QueryClass::SystemTriage);
    assert!(route.probes.contains(&"journal_errors".to_string()));
    assert!(route.probes.contains(&"journal_warnings".to_string()));
    assert!(route.probes.contains(&"failed_units".to_string()));
    assert!(route.probes.contains(&"boot_time".to_string()));
    assert!(route.can_answer_deterministically());
}
