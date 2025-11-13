# Automated Monitoring Setup

**Phase 3.5: Zero-Config Observability** - One command to beautiful dashboards

## Overview

The `annactl setup-monitoring` command provides a fully automated, zero-configuration path from bare system to production-ready observability stack with beautiful dashboards optimized for Anna's metrics.

## Design Goals

1. **One Command Setup**: `annactl setup-monitoring` → working dashboards in <60 seconds
2. **Resource-Aware**: Automatically adapts to minimal/light/full modes
3. **Beautiful by Default**: Pre-configured dashboards that look professional
4. **Secure by Default**: TLS, authentication, isolation
5. **Idempotent**: Safe to run multiple times
6. **Upgradeable**: Preserves user customizations during updates
7. **Removable**: Clean uninstall with `annactl setup-monitoring --remove`

## User Experience

### Success Path

```bash
$ sudo annactl setup-monitoring

🔍 Detecting system resources...
   • RAM: 7.5 GB available of 16 GB total
   • CPU: 8 cores
   • Disk: 450 GB available
   • Mode: LIGHT (Prometheus + lightweight Grafana)

📦 Installing monitoring stack...
   ✓ Prometheus v2.45.0 configured
   ✓ Grafana v10.0.0 configured
   ✓ TLS certificates generated
   ✓ Systemd services created
   ✓ Dashboards provisioned (4 dashboards)

🚀 Starting services...
   ✓ prometheus.service started
   ✓ grafana-server.service started

🎉 Setup complete!

   Grafana: https://localhost:3000
   Username: admin
   Password: <randomly generated, shown once>

   Pre-installed dashboards:
   • Anna Overview       - System health at a glance
   • Resource Metrics    - Memory, CPU, disk trends
   • Action History      - Command success rates
   • Consensus Health    - Distributed system metrics

💡 Next steps:
   • Visit Grafana and change the admin password
   • Explore the pre-configured dashboards
   • Customize alert rules if desired

   Run 'annactl monitor status' to check stack health.
```

### Minimal Mode Warning

```bash
$ sudo annactl setup-monitoring

⚠️  Adaptive Intelligence Warning
═══════════════════════════════════════════════════════════════

  Your system is running in MINIMAL mode (< 2 GB RAM).

  External monitoring is NOT recommended. Anna includes built-in
  monitoring that is sufficient for resource-constrained systems:

    • annactl health      - System health checks
    • annactl status      - Current system state
    • annactl metrics     - Real-time resource metrics

  If you still want external monitoring, use:
    annactl setup-monitoring --force-minimal

Continue? [y/N]: n
Setup cancelled.
```

## Architecture

### Stack Components

#### Mode: Minimal (< 2 GB RAM)
- **Not Recommended**: Built-in monitoring only
- If forced: Prometheus only, no Grafana
- Minimal scrape interval: 120s
- Short retention: 7 days

#### Mode: Light (2-4 GB RAM)
- **Recommended**: Balanced observability
- Prometheus (configured lean)
- Grafana (lightweight, limited plugins)
- Scrape interval: 60s
- Retention: 30 days
- 4 core dashboards

#### Mode: Full (> 4 GB RAM + GUI)
- **Full Featured**: Production-grade stack
- Prometheus (full configuration)
- Grafana (all features, plugins)
- Alertmanager
- Node Exporter (optional)
- Scrape interval: 15s
- Retention: 90 days
- 6+ dashboards including advanced analytics

### Directory Structure

```
/var/lib/anna/monitoring/
├── prometheus/
│   ├── prometheus.yml           # Generated config
│   ├── data/                    # Time-series database
│   └── rules/                   # Alert rules
├── grafana/
│   ├── grafana.ini              # Generated config
│   ├── data/                    # Grafana database
│   ├── dashboards/              # Provisioned dashboards
│   │   ├── anna-overview.json
│   │   ├── anna-resources.json
│   │   ├── anna-actions.json
│   │   └── anna-consensus.json
│   └── provisioning/
│       ├── datasources/
│       │   └── prometheus.yml   # Auto-configured
│       └── dashboards/
│           └── anna.yml         # Dashboard provider
├── tls/
│   ├── ca.crt
│   ├── server.crt
│   └── server.key
└── secrets/
    └── grafana-admin-password   # Random password
```

### Systemd Services

#### `/etc/systemd/system/anna-prometheus.service`

```ini
[Unit]
Description=Prometheus for Anna Assistant
After=network.target annad.service
Wants=annad.service

[Service]
Type=simple
User=anna
Group=anna
ExecStart=/usr/local/bin/prometheus \
    --config.file=/var/lib/anna/monitoring/prometheus/prometheus.yml \
    --storage.tsdb.path=/var/lib/anna/monitoring/prometheus/data \
    --storage.tsdb.retention.time=30d \
    --web.listen-address=127.0.0.1:9090

Restart=on-failure
RestartSec=5s

[Install]
WantedBy=multi-user.target
```

#### `/etc/systemd/system/anna-grafana.service`

```ini
[Unit]
Description=Grafana for Anna Assistant
After=network.target anna-prometheus.service
Wants=anna-prometheus.service

[Service]
Type=simple
User=anna
Group=anna
Environment="GF_PATHS_CONFIG=/var/lib/anna/monitoring/grafana/grafana.ini"
Environment="GF_PATHS_DATA=/var/lib/anna/monitoring/grafana/data"
Environment="GF_PATHS_PROVISIONING=/var/lib/anna/monitoring/grafana/provisioning"
ExecStart=/usr/bin/grafana-server \
    --homepath /usr/share/grafana

Restart=on-failure
RestartSec=5s

[Install]
WantedBy=multi-user.target
```

## Dashboard Designs

### 1. Anna Overview Dashboard

**Purpose**: Executive summary of system health

**Panels**:
- **System Status** (Stat): Current state (Healthy/Degraded/Critical)
- **Monitoring Mode** (Stat): Current mode (Minimal/Light/Full)
- **Resource Constraints** (Stat): Yes/No with color coding
- **Uptime** (Stat): System uptime in human-readable format
- **Memory Usage** (Gauge): Percentage with thresholds
- **Disk Usage** (Gauge): Percentage with thresholds
- **CPU Cores** (Stat): Available cores
- **Recent Actions** (Bar gauge): Count by action type (last 24h)
- **Success Rate** (Time series): Rolling 24-hour success rate
- **Failed Probes** (Table): Current failed probes with severity

**Layout**: 3x4 grid, auto-refresh 60s

**Colors**:
- Healthy: Green (#73BF69)
- Degraded: Yellow (#F2CC0C)
- Critical: Red (#E02F44)
- Neutral: Blue (#5794F2)

### 2. Resource Metrics Dashboard

**Purpose**: Deep dive into system resources over time

**Panels**:
- **Memory Timeline** (Time series):
  - Total memory (constant line)
  - Available memory (area fill)
  - Used memory (calculated)
- **Memory Percentage** (Time series): % used over time
- **Disk Timeline** (Time series):
  - Total disk (constant line)
  - Available disk (area fill)
- **Disk Percentage** (Time series): % used over time
- **CPU Cores** (Stat): Static display
- **Uptime** (Time series): Cumulative uptime
- **Mode Changes** (State timeline): When monitoring mode changed
- **Constraint Events** (State timeline): When constraints activated

**Time Range**: Last 24 hours (adjustable)
**Auto-refresh**: 30s

### 3. Action History Dashboard

**Purpose**: Track Anna's actions and their outcomes

**Panels**:
- **Actions per Hour** (Time series): Action volume over time
- **Success Rate** (Time series): Percentage successful by hour
- **Actions by Type** (Pie chart): Distribution of action types
- **Action Duration** (Time series): How long actions take
- **Failed Actions** (Table): Recent failures with error messages
- **Top Affected Packages** (Bar chart): Most frequently updated packages
- **Action Trends** (Heatmap): Actions by hour of day and day of week

**Time Range**: Last 7 days (adjustable)
**Auto-refresh**: 60s

### 4. Consensus Health Dashboard

**Purpose**: Monitor distributed consensus (Phase 1.7+)

**Panels**:
- **Cluster Size** (Stat): Number of nodes
- **Cluster State** (Stat): Leader/Follower/Candidate
- **Term Number** (Time series): Raft term progression
- **Leader Elections** (Bar chart): Election events over time
- **Log Replication Lag** (Time series): Follower lag in entries
- **Heartbeat Latency** (Time series): Network health
- **Consensus Success Rate** (Gauge): Agreement percentage
- **Failed Proposals** (Table): Recent failed proposals

**Time Range**: Last 1 hour (adjustable)
**Auto-refresh**: 10s

## Prometheus Configuration Templates

### Light Mode (`prometheus-light.yml`)

```yaml
global:
  scrape_interval: 60s
  evaluation_interval: 60s
  scrape_timeout: 10s

scrape_configs:
  - job_name: 'anna-daemon'
    static_configs:
      - targets: ['localhost:9091']
    relabel_configs:
      - source_labels: [__address__]
        target_label: instance
        replacement: 'anna-daemon'

rule_files:
  - /var/lib/anna/monitoring/prometheus/rules/*.yml

storage:
  tsdb:
    retention.time: 30d
    retention.size: 2GB
```

### Full Mode (`prometheus-full.yml`)

```yaml
global:
  scrape_interval: 15s
  evaluation_interval: 15s
  scrape_timeout: 10s

scrape_configs:
  - job_name: 'anna-daemon'
    static_configs:
      - targets: ['localhost:9091']
    relabel_configs:
      - source_labels: [__address__]
        target_label: instance
        replacement: 'anna-daemon'

  - job_name: 'node-exporter'
    static_configs:
      - targets: ['localhost:9100']

  - job_name: 'prometheus'
    static_configs:
      - targets: ['localhost:9090']

rule_files:
  - /var/lib/anna/monitoring/prometheus/rules/*.yml

alerting:
  alertmanagers:
    - static_configs:
        - targets: ['localhost:9093']

storage:
  tsdb:
    retention.time: 90d
    retention.size: 10GB
```

## Grafana Configuration Template

### `grafana.ini` (Light Mode)

```ini
[server]
protocol = https
http_addr = 127.0.0.1
http_port = 3000
cert_file = /var/lib/anna/monitoring/tls/server.crt
cert_key = /var/lib/anna/monitoring/tls/server.key

[database]
type = sqlite3
path = /var/lib/anna/monitoring/grafana/data/grafana.db

[security]
admin_user = admin
admin_password = __GENERATED_PASSWORD__
secret_key = __GENERATED_SECRET__
disable_gravatar = true
cookie_secure = true
cookie_samesite = strict

[auth.anonymous]
enabled = false

[users]
allow_sign_up = false
allow_org_create = false

[analytics]
reporting_enabled = false
check_for_updates = false

[log]
mode = file
level = warn

[paths]
data = /var/lib/anna/monitoring/grafana/data
logs = /var/lib/anna/monitoring/grafana/logs
plugins = /var/lib/anna/monitoring/grafana/plugins
provisioning = /var/lib/anna/monitoring/grafana/provisioning
```

## Implementation Plan

### Phase 3.5.1: Command Structure

Add new command to `crates/annactl/src/main.rs`:

```rust
#[derive(Subcommand)]
pub enum Commands {
    // ... existing commands

    /// Automated monitoring stack setup
    #[command(name = "setup-monitoring")]
    SetupMonitoring {
        /// Force setup in minimal mode (not recommended)
        #[arg(long)]
        force_minimal: bool,

        /// Override monitoring mode (minimal/light/full)
        #[arg(long)]
        force_mode: Option<String>,

        /// Remove monitoring stack
        #[arg(long)]
        remove: bool,

        /// Dry run (show what would be done)
        #[arg(long)]
        dry_run: bool,
    },
}
```

### Phase 3.5.2: Setup Workflow

```rust
async fn execute_setup_monitoring(
    force_minimal: bool,
    force_mode: Option<String>,
    remove: bool,
    dry_run: bool,
) -> Result<()> {
    if remove {
        return remove_monitoring_stack(dry_run).await;
    }

    // Step 1: Detect system profile
    let profile = detect_system_profile().await?;

    // Step 2: Determine monitoring mode
    let mode = determine_monitoring_mode(&profile, force_mode)?;

    // Step 3: Warn if minimal and not forced
    if mode == MonitoringMode::Minimal && !force_minimal {
        warn_minimal_mode()?;
        return Ok(());
    }

    // Step 4: Check if already installed
    let existing = check_existing_installation().await?;
    if existing {
        println!("Monitoring stack already installed. Upgrading...");
    }

    // Step 5: Download binaries if needed
    download_binaries(&mode, dry_run).await?;

    // Step 6: Generate configurations
    generate_prometheus_config(&mode, dry_run)?;
    generate_grafana_config(&mode, dry_run)?;

    // Step 7: Generate TLS certificates
    generate_tls_certificates(dry_run)?;

    // Step 8: Provision dashboards
    provision_dashboards(&mode, dry_run)?;

    // Step 9: Create systemd services
    create_systemd_services(&mode, dry_run)?;

    // Step 10: Start services
    start_services(dry_run).await?;

    // Step 11: Validate installation
    validate_installation().await?;

    // Step 12: Display success message
    display_success_message(&mode)?;

    Ok(())
}
```

### Phase 3.5.3: Dashboard JSON Generation

Create `crates/annactl/src/monitoring/dashboards.rs`:

```rust
pub fn generate_overview_dashboard() -> serde_json::Value {
    json!({
        "dashboard": {
            "title": "Anna Overview",
            "tags": ["anna", "overview"],
            "timezone": "browser",
            "panels": [
                {
                    "id": 1,
                    "title": "System Status",
                    "type": "stat",
                    "targets": [{
                        "expr": "anna_system_state",
                        "legendFormat": "Status"
                    }],
                    "fieldConfig": {
                        "defaults": {
                            "mappings": [
                                {"value": 0, "text": "Healthy", "color": "green"},
                                {"value": 1, "text": "Degraded", "color": "yellow"},
                                {"value": 2, "text": "Critical", "color": "red"}
                            ]
                        }
                    },
                    "gridPos": {"h": 4, "w": 6, "x": 0, "y": 0}
                },
                // ... more panels
            ]
        }
    })
}
```

### Phase 3.5.4: Binary Management

```rust
async fn download_binaries(mode: &MonitoringMode, dry_run: bool) -> Result<()> {
    let prometheus_version = "2.45.0";
    let grafana_version = "10.0.0";

    // Check if already downloaded
    if Path::new("/usr/local/bin/prometheus").exists() &&
       Path::new("/usr/bin/grafana-server").exists() {
        println!("✓ Binaries already installed");
        return Ok(());
    }

    if dry_run {
        println!("[DRY RUN] Would download Prometheus v{}", prometheus_version);
        println!("[DRY RUN] Would download Grafana v{}", grafana_version);
        return Ok(());
    }

    // Download and install via pacman or direct download
    install_prometheus(prometheus_version).await?;

    if mode != &MonitoringMode::Minimal {
        install_grafana(grafana_version).await?;
    }

    Ok(())
}
```

## Security Considerations

### TLS Certificate Generation

```bash
# Self-signed CA for local development
openssl req -x509 -newkey rsa:4096 -nodes \
    -keyout /var/lib/anna/monitoring/tls/ca.key \
    -out /var/lib/anna/monitoring/tls/ca.crt \
    -days 365 \
    -subj "/CN=Anna Monitoring CA"

# Server certificate
openssl req -newkey rsa:2048 -nodes \
    -keyout /var/lib/anna/monitoring/tls/server.key \
    -out /var/lib/anna/monitoring/tls/server.csr \
    -subj "/CN=localhost"

openssl x509 -req \
    -in /var/lib/anna/monitoring/tls/server.csr \
    -CA /var/lib/anna/monitoring/tls/ca.crt \
    -CAkey /var/lib/anna/monitoring/tls/ca.key \
    -CAcreateserial \
    -out /var/lib/anna/monitoring/tls/server.crt \
    -days 365
```

### Password Generation

```rust
fn generate_secure_password() -> String {
    use rand::Rng;
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789!@#$%^&*";
    let mut rng = rand::thread_rng();

    (0..32)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}
```

### File Permissions

```bash
# All monitoring files owned by anna:anna
chown -R anna:anna /var/lib/anna/monitoring

# Secrets directory restricted
chmod 700 /var/lib/anna/monitoring/secrets
chmod 600 /var/lib/anna/monitoring/secrets/*

# TLS certificates readable by services
chmod 755 /var/lib/anna/monitoring/tls
chmod 644 /var/lib/anna/monitoring/tls/*.crt
chmod 600 /var/lib/anna/monitoring/tls/*.key
```

## Testing

### Manual Testing

```bash
# Test minimal mode warning
annactl setup-monitoring

# Test forced minimal mode
sudo annactl setup-monitoring --force-minimal

# Test light mode (typical)
sudo annactl setup-monitoring

# Test full mode override
sudo annactl setup-monitoring --force-mode full

# Test dry run
sudo annactl setup-monitoring --dry-run

# Test removal
sudo annactl setup-monitoring --remove

# Test idempotency
sudo annactl setup-monitoring
sudo annactl setup-monitoring  # Should upgrade/verify
```

### Integration Tests

```rust
#[tokio::test]
async fn test_setup_monitoring_light_mode() {
    let result = execute_setup_monitoring(false, Some("light".to_string()), false, true).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_dashboard_generation() {
    let dashboard = generate_overview_dashboard();
    assert_eq!(dashboard["dashboard"]["title"], "Anna Overview");
}

#[tokio::test]
async fn test_prometheus_config_generation() {
    let config = generate_prometheus_config(&MonitoringMode::Light, true);
    assert!(config.is_ok());
    let config_str = std::fs::read_to_string("/tmp/prometheus-test.yml").unwrap();
    assert!(config_str.contains("scrape_interval: 60s"));
}
```

## Upgrade Strategy

### Preserving Customizations

```rust
async fn upgrade_monitoring_stack() -> Result<()> {
    // 1. Backup existing configs
    backup_directory("/var/lib/anna/monitoring", "/var/lib/anna/monitoring.backup")?;

    // 2. Stop services
    stop_service("anna-prometheus")?;
    stop_service("anna-grafana")?;

    // 3. Upgrade binaries
    upgrade_prometheus().await?;
    upgrade_grafana().await?;

    // 4. Merge configs (preserve user customizations)
    merge_configs("/var/lib/anna/monitoring/prometheus/prometheus.yml")?;
    merge_configs("/var/lib/anna/monitoring/grafana/grafana.ini")?;

    // 5. Update dashboards (preserve user-created ones)
    update_provisioned_dashboards()?;

    // 6. Restart services
    start_service("anna-prometheus")?;
    start_service("anna-grafana")?;

    // 7. Validate
    validate_installation().await?;

    Ok(())
}
```

## User Documentation

### Quick Start Guide

```markdown
# Monitoring Setup Guide

## Installation

One command to set up beautiful dashboards:

```bash
sudo annactl setup-monitoring
```

Anna will automatically:
- Detect your system resources
- Choose the optimal monitoring mode
- Install Prometheus and Grafana
- Configure secure access
- Provision beautiful dashboards

## Accessing Grafana

After setup, visit: https://localhost:3000

Login with:
- Username: `admin`
- Password: (shown during setup)

**Important**: Change the password immediately after first login!

## Pre-Installed Dashboards

### Anna Overview
Your system health at a glance. Check this dashboard daily.

### Resource Metrics
Deep dive into memory, CPU, and disk usage over time.

### Action History
Track what Anna has done and how successful she's been.

### Consensus Health
(Advanced) Monitor distributed consensus if you're running multiple nodes.

## Customizing Dashboards

You can customize any dashboard:
1. Click "Dashboard settings" (gear icon)
2. Make your changes
3. Click "Save dashboard"
4. Give it a new name to avoid overwriting

Your custom dashboards will be preserved during upgrades.

## Troubleshooting

### Grafana won't start
```bash
sudo journalctl -u anna-grafana -f
```

### Prometheus won't start
```bash
sudo journalctl -u anna-prometheus -f
```

### Check service status
```bash
annactl monitor status
```

### Remove and reinstall
```bash
sudo annactl setup-monitoring --remove
sudo annactl setup-monitoring
```
```

## Future Enhancements

### Phase 3.6: Alert Rules

Pre-configured alert rules for common issues:

```yaml
groups:
  - name: anna_alerts
    interval: 60s
    rules:
      - alert: HighMemoryUsage
        expr: (anna_system_memory_total_mb - anna_system_memory_available_mb) / anna_system_memory_total_mb > 0.9
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "High memory usage detected"
          description: "Memory usage is above 90% for 5 minutes"

      - alert: LowDiskSpace
        expr: anna_system_disk_available_gb / anna_system_disk_total_gb < 0.1
        for: 1m
        labels:
          severity: critical
        annotations:
          summary: "Low disk space"
          description: "Less than 10% disk space remaining"

      - alert: SystemDegraded
        expr: anna_system_state == 1
        for: 10m
        labels:
          severity: warning
        annotations:
          summary: "System in degraded state"
          description: "Anna detected system degradation for 10 minutes"
```

### Phase 3.7: Mobile-Responsive Dashboards

Optimize dashboard layouts for mobile viewing:
- Collapsible panels
- Touch-friendly controls
- Auto-scaling graphs
- Mobile-optimized legends

### Phase 3.8: Dashboard Export/Import

```bash
# Export custom dashboard
annactl monitoring export-dashboard my-custom-dashboard.json

# Import dashboard from file
annactl monitoring import-dashboard my-custom-dashboard.json

# Share dashboards with community
annactl monitoring publish-dashboard my-custom-dashboard.json
```

---

**Status**: Phase 3.5 - Design complete
**Next**: Phase 3.5.1 - Implement command structure
**Author**: Anna Observability Team
**License**: Custom (see LICENSE file)

Citation: [prometheus:configuration], [grafana:provisioning], [systemd:service-units]
