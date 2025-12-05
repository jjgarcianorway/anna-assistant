//! Tests for snapshot.rs

use anna_shared::rpc::ProbeResult;
use anna_shared::snapshot::{
    capture_snapshot, diff_snapshots, format_deltas_text, has_actionable_deltas, DeltaItem,
    SystemSnapshot,
};

#[test]
fn test_snapshot_creation() {
    let mut snap = SystemSnapshot::new();
    snap.add_disk("/", 50);
    snap.add_disk("/home", 30);
    snap.add_failed_service("nginx.service");

    assert_eq!(snap.disk.get("/"), Some(&50));
    assert!(snap.failed_services.contains(&"nginx.service".to_string()));
}

#[test]
fn test_memory_percent() {
    let mut snap = SystemSnapshot::new();
    snap.set_memory(16_000_000_000, 8_000_000_000);
    assert_eq!(snap.memory_percent(), 50);
}

#[test]
fn test_diff_no_changes() {
    let snap = SystemSnapshot::new();
    let deltas = diff_snapshots(&snap, &snap);
    assert!(deltas.is_empty());
}

#[test]
fn test_diff_disk_warning_threshold() {
    let mut prev = SystemSnapshot::new();
    prev.add_disk("/", 80);

    let mut curr = SystemSnapshot::new();
    curr.add_disk("/", 87);

    let deltas = diff_snapshots(&prev, &curr);
    assert_eq!(deltas.len(), 1);
    assert!(matches!(deltas[0], DeltaItem::DiskWarning { .. }));
}

#[test]
fn test_diff_disk_critical_threshold() {
    let mut prev = SystemSnapshot::new();
    prev.add_disk("/", 90);

    let mut curr = SystemSnapshot::new();
    curr.add_disk("/", 96);

    let deltas = diff_snapshots(&prev, &curr);
    assert_eq!(deltas.len(), 1);
    assert!(matches!(deltas[0], DeltaItem::DiskCritical { .. }));
}

#[test]
fn test_diff_disk_increase() {
    let mut prev = SystemSnapshot::new();
    prev.add_disk("/", 50);

    let mut curr = SystemSnapshot::new();
    curr.add_disk("/", 56); // +6 >= threshold of 5

    let deltas = diff_snapshots(&prev, &curr);
    assert_eq!(deltas.len(), 1);
    assert!(matches!(deltas[0], DeltaItem::DiskIncreased { .. }));
}

#[test]
fn test_diff_new_failed_service() {
    let prev = SystemSnapshot::new();

    let mut curr = SystemSnapshot::new();
    curr.add_failed_service("nginx.service");

    let deltas = diff_snapshots(&prev, &curr);
    assert_eq!(deltas.len(), 1);
    assert!(matches!(deltas[0], DeltaItem::NewFailedService { .. }));
}

#[test]
fn test_diff_service_recovered() {
    let mut prev = SystemSnapshot::new();
    prev.add_failed_service("nginx.service");

    let curr = SystemSnapshot::new();

    let deltas = diff_snapshots(&prev, &curr);
    assert_eq!(deltas.len(), 1);
    assert!(matches!(deltas[0], DeltaItem::ServiceRecovered { .. }));
}

#[test]
fn test_diff_memory_high() {
    let mut prev = SystemSnapshot::new();
    prev.set_memory(16_000, 12_000); // 75%

    let mut curr = SystemSnapshot::new();
    curr.set_memory(16_000, 14_000); // 87.5%

    let deltas = diff_snapshots(&prev, &curr);
    assert_eq!(deltas.len(), 1);
    assert!(matches!(deltas[0], DeltaItem::MemoryHigh { .. }));
}

#[test]
fn test_format_no_deltas() {
    let text = format_deltas_text(&[]);
    assert_eq!(text, "No new warnings since last check.");
}

#[test]
fn test_format_deltas_capped() {
    let deltas: Vec<DeltaItem> = (0..10)
        .map(|i| DeltaItem::NewFailedService {
            unit: format!("service{}.service", i),
        })
        .collect();

    let text = format_deltas_text(&deltas);
    assert!(text.contains("... and"));
    assert!(text.lines().count() <= 5);
}

#[test]
fn test_has_actionable_deltas() {
    let deltas = vec![DeltaItem::DiskIncreased {
        mount: "/".to_string(),
        prev: 50,
        curr: 56,
    }];
    assert!(!has_actionable_deltas(&deltas)); // Increase alone isn't actionable

    let deltas = vec![DeltaItem::DiskWarning {
        mount: "/".to_string(),
        prev: 80,
        curr: 87,
    }];
    assert!(has_actionable_deltas(&deltas)); // Warning is actionable
}

#[test]
fn test_snapshot_freshness() {
    let mut snap = SystemSnapshot::now();
    assert!(snap.is_fresh(300)); // Fresh within 5 minutes

    snap.captured_at = 0;
    assert!(!snap.is_fresh(300)); // No timestamp = stale
}

#[test]
fn test_deterministic_ordering() {
    let mut snap1 = SystemSnapshot::new();
    snap1.add_disk("/home", 50);
    snap1.add_disk("/", 60);
    snap1.add_disk("/var", 40);

    let mut snap2 = SystemSnapshot::new();
    snap2.add_disk("/", 60);
    snap2.add_disk("/var", 40);
    snap2.add_disk("/home", 50);

    // BTreeMap ensures same order regardless of insertion order
    assert_eq!(snap1.disk, snap2.disk);
}

// === Capture tests ===

fn mock_probe(cmd: &str, stdout: &str) -> ProbeResult {
    ProbeResult {
        command: cmd.to_string(),
        exit_code: 0,
        stdout: stdout.to_string(),
        stderr: String::new(),
        timing_ms: 10,
    }
}

#[test]
fn test_capture_df_output() {
    let df_output = r#"Filesystem     1K-blocks     Used Available Use% Mounted on
/dev/sda1      100000000 50000000  50000000  50% /
/dev/sda2      200000000 170000000 30000000  85% /home
tmpfs            8000000  1000000   7000000  12% /tmp"#;

    let probes = vec![mock_probe("df -h", df_output)];
    let snap = capture_snapshot(&probes);

    assert_eq!(snap.disk.get("/"), Some(&50));
    assert_eq!(snap.disk.get("/home"), Some(&85));
    assert_eq!(snap.disk.get("/tmp"), Some(&12));
}

#[test]
fn test_capture_free_output() {
    let free_output = r#"              total        used        free      shared  buff/cache   available
Mem:    16000000000  8000000000  4000000000   500000000  3500000000  7000000000
Swap:    8000000000  1000000000  7000000000"#;

    let probes = vec![mock_probe("free -b", free_output)];
    let snap = capture_snapshot(&probes);

    assert_eq!(snap.memory_total_bytes, 16000000000);
    assert_eq!(snap.memory_used_bytes, 8000000000);
    assert_eq!(snap.memory_percent(), 50);
}

#[test]
fn test_capture_failed_services() {
    let failed_output = r#"  UNIT                    LOAD   ACTIVE SUB    DESCRIPTION
● nginx.service          loaded failed failed  A high performance web server
● docker.service         loaded failed failed  Docker Application Container

2 loaded units listed."#;

    let probes = vec![mock_probe("systemctl --failed", failed_output)];
    let snap = capture_snapshot(&probes);

    assert_eq!(snap.failed_services.len(), 2);
    assert!(snap.failed_services.contains(&"nginx.service".to_string()));
    assert!(snap.failed_services.contains(&"docker.service".to_string()));
}

#[test]
fn test_capture_no_failed_services() {
    let failed_output = r#"  UNIT LOAD ACTIVE SUB DESCRIPTION
0 loaded units listed."#;

    let probes = vec![mock_probe("systemctl --failed", failed_output)];
    let snap = capture_snapshot(&probes);

    assert!(snap.failed_services.is_empty());
}

#[test]
fn test_capture_combined_probes() {
    let df_output = "Filesystem 1K-blocks Used Available Use% Mounted on\n/dev/sda1 100000 70000 30000 70% /";
    let free_output = "              total        used        free\nMem:    16000000000  12000000000  4000000000";
    let failed_output = "● nginx.service loaded failed failed nginx";

    let probes = vec![
        mock_probe("df -h", df_output),
        mock_probe("free -b", free_output),
        mock_probe("systemctl --failed", failed_output),
    ];
    let snap = capture_snapshot(&probes);

    assert_eq!(snap.disk.get("/"), Some(&70));
    assert_eq!(snap.memory_percent(), 75);
    assert_eq!(snap.failed_services.len(), 1);
}

#[test]
fn test_capture_ignores_failed_probes() {
    let probes = vec![ProbeResult {
        command: "df -h".to_string(),
        exit_code: 1, // Failed
        stdout: "Filesystem 1K-blocks Used Available Use% Mounted on\n/dev/sda1 100 90 10 90% /".to_string(),
        stderr: "Error".to_string(),
        timing_ms: 10,
    }];
    let snap = capture_snapshot(&probes);

    assert!(snap.disk.is_empty()); // Should not capture from failed probe
}

// === Delta detection golden tests ===

#[test]
fn test_no_delta_below_threshold() {
    let mut prev = SystemSnapshot::new();
    prev.add_disk("/", 50);

    let mut curr = SystemSnapshot::new();
    curr.add_disk("/", 53); // +3, below threshold of 5

    let deltas = diff_snapshots(&prev, &curr);
    assert!(deltas.is_empty());
}

#[test]
fn test_multiple_deltas_ordered() {
    let mut prev = SystemSnapshot::new();
    prev.add_disk("/", 80);
    prev.add_disk("/home", 70);

    let mut curr = SystemSnapshot::new();
    curr.add_disk("/", 90);  // Warning
    curr.add_disk("/home", 96); // Critical
    curr.add_failed_service("nginx.service");

    let deltas = diff_snapshots(&prev, &curr);

    // Should have 3 deltas: disk / warning, disk /home critical, new failed service
    assert_eq!(deltas.len(), 3);
}

#[test]
fn test_delta_format_single_line() {
    let delta = DeltaItem::DiskWarning {
        mount: "/".to_string(),
        prev: 80,
        curr: 87,
    };
    let text = delta.format();
    assert!(text.contains("/"));
    assert!(text.contains("87%"));
    assert!(text.contains("80%"));
}
