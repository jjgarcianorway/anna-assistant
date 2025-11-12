# Anna Assistant - Observability Guide

**Phase 2.0.0-alpha.1** - Grafana Dashboards & Prometheus Alerts

Citation: [prometheus:best-practices][grafana:dashboard-design][sre:golden-signals]

## Overview

This guide covers setting up monitoring and alerting for Anna Assistant using Prometheus and Grafana.

## Architecture

```
Anna Assistant (annad)
  ↓ expose
Prometheus Metrics (:9090/metrics)
  ↓ scrape
Prometheus Server
  ↓ query & alert
Grafana Dashboards + Alertmanager
```

## Available Metrics

### Consensus Metrics (Phase 1.9)
- `anna_consensus_rounds_total` - Total consensus rounds completed
- `anna_byzantine_nodes_total` - Detected Byzantine nodes (gauge)
- `anna_quorum_size` - Required quorum size (gauge)

### Temporal Integrity (Phase 1.10)
- `anna_average_tis` - Average temporal integrity score (0.0-1.0)
- `anna_peer_request_total{peer, status}` - Peer requests by status
- `anna_peer_reload_total{result}` - Peer configuration reloads
- `anna_migration_events_total{result}` - State migration events

### Network (Phase 1.11)
- `anna_peer_backoff_seconds{peer}` - Peer backoff duration histogram

### TLS (Phase 1.13)
- `anna_tls_handshakes_total{status}` - TLS handshakes by status

### Rate Limiting (Phase 1.15)
- `anna_rate_limit_violations_total{scope}` - Rate limit violations

### Certificate Pinning (Phase 2)
- `anna_pinning_violations_total{peer}` - Certificate pinning violations

## Installation

### 1. Prometheus Setup

#### Install Prometheus

**Arch Linux:**
```bash
sudo pacman -S prometheus
```

**macOS:**
```bash
brew install prometheus
```

**Docker:**
```bash
docker run -d -p 9090:9090 \
  -v $(pwd)/observability/prometheus:/etc/prometheus \
  prom/prometheus
```

#### Configure Prometheus

Create `/etc/prometheus/prometheus.yml`:

```yaml
global:
  scrape_interval: 15s
  evaluation_interval: 15s

# Load alert rules
rule_files:
  - "/path/to/anna-assistant/observability/prometheus/anna-critical.yml"
  - "/path/to/anna-assistant/observability/prometheus/anna-warnings.yml"

# Scrape Anna Assistant metrics
scrape_configs:
  - job_name: 'anna-assistant'
    static_configs:
      - targets: ['localhost:9090']
    metrics_path: '/metrics'
```

#### Start Prometheus

```bash
# Systemd
sudo systemctl enable --now prometheus

# Docker
docker run -d -p 9090:9090 \
  -v /etc/prometheus/prometheus.yml:/etc/prometheus/prometheus.yml \
  prom/prometheus

# Direct
prometheus --config.file=/etc/prometheus/prometheus.yml
```

Verify: http://localhost:9090

### 2. Grafana Setup

#### Install Grafana

**Arch Linux:**
```bash
sudo pacman -S grafana
sudo systemctl enable --now grafana
```

**macOS:**
```bash
brew install grafana
brew services start grafana
```

**Docker:**
```bash
docker run -d -p 3000:3000 \
  -v grafana-storage:/var/lib/grafana \
  grafana/grafana-oss
```

#### Configure Data Source

1. Open Grafana: http://localhost:3000 (default: admin/admin)
2. Go to **Configuration** → **Data Sources** → **Add data source**
3. Select **Prometheus**
4. URL: `http://localhost:9090`
5. Click **Save & Test**

#### Import Dashboards

**Option 1: Via UI**
1. Go to **Dashboards** → **Import**
2. Upload JSON file or paste JSON
3. Select Prometheus data source
4. Click **Import**

Import these dashboards in order:
- `observability/grafana/anna-overview.json` (Overview)
- `observability/grafana/anna-tls.json` (TLS & Pinning)
- `observability/grafana/anna-consensus.json` (Consensus Details)
- `observability/grafana/anna-rate-limiting.json` (Rate Limiting)

**Option 2: Via CLI**
```bash
# Copy dashboards to Grafana provisioning directory
sudo cp observability/grafana/*.json /etc/grafana/provisioning/dashboards/

# Restart Grafana
sudo systemctl restart grafana
```

### 3. Alertmanager Setup (Optional)

For alert notifications via email, Slack, PagerDuty, etc.

```bash
# Install
sudo pacman -S prometheus-alertmanager  # Arch
brew install alertmanager              # macOS

# Configure /etc/alertmanager/alertmanager.yml
route:
  receiver: 'team-ops'
receivers:
  - name: 'team-ops'
    slack_configs:
      - api_url: 'https://hooks.slack.com/services/YOUR/WEBHOOK/URL'
        channel: '#ops-alerts'
```

## Dashboard Guide

### Anna Overview Dashboard
- **Purpose**: High-level system health
- **Key Metrics**:
  - Total consensus rounds completed
  - Byzantine nodes detected
  - Average Temporal Integrity Score (TIS)
  - Quorum size
  - Round rate over time
  - Peer request rates
- **Use Case**: Daily operations monitoring, SRE on-call

### Anna TLS Dashboard
- **Purpose**: TLS security monitoring
- **Key Metrics**:
  - Total TLS handshakes (success/failure)
  - Certificate pinning violations by peer
  - Handshake rate by status
  - Violation distribution
- **Use Case**: Security incident response, compliance auditing

### Anna Consensus Dashboard
- **Purpose**: Detailed consensus behavior
- **Key Metrics**:
  - Byzantine node trends
  - Peer backoff duration (p95)
  - Peer reload events
- **Use Case**: Consensus debugging, network health analysis

### Anna Rate Limiting Dashboard
- **Purpose**: Rate limiting effectiveness
- **Key Metrics**:
  - Total violations
  - Violations by scope (peer vs token)
  - Violation rate over time
- **Use Case**: DDoS detection, abuse prevention

## Alert Guide

### Critical Alerts (anna-critical.yml)

**Require immediate action** - Page on-call engineer

1. **AnnaByzantineNodesDetected**
   - **Trigger**: `anna_byzantine_nodes_total > 0` for 1m
   - **Action**: Investigate node logs, check for compromised peers
   - **Runbook**: `docs/runbooks/byzantine-nodes.md`

2. **AnnaCertificatePinningViolation**
   - **Trigger**: Pinning violation detected
   - **Action**: Possible MITM attack - isolate peer, verify certificates
   - **Runbook**: `docs/runbooks/pinning-violations.md`

3. **AnnaConsensusStalled**
   - **Trigger**: No rounds completed in 5m
   - **Action**: Check network connectivity, quorum status
   - **Runbook**: `docs/runbooks/consensus-stalled.md`

4. **AnnaTemporalIntegrityLow**
   - **Trigger**: TIS < 0.5 for 5m
   - **Action**: Network synchronization compromised, check NTP
   - **Runbook**: `docs/runbooks/low-tis.md`

5. **AnnaTLSHandshakeFailureSpike**
   - **Trigger**: Failure rate > 1/sec for 2m
   - **Action**: Check TLS certificates, network issues
   - **Runbook**: `docs/runbooks/tls-failures.md`

6. **AnnaQuorumLost**
   - **Trigger**: Quorum size < 3 for 1m
   - **Action**: Add nodes or investigate node failures
   - **Runbook**: `docs/runbooks/quorum-lost.md`

### Warning Alerts (anna-warnings.yml)

**Require investigation** - Monitor and escalate if degraded

1. **AnnaTemporalIntegrityDegraded** - TIS < 0.7 for 10m
2. **AnnaHighRateLimitViolations** - Rate > 0.5/sec for 5m
3. **AnnaPeerRequestFailures** - Failure rate > 0.1/sec for 5m
4. **AnnaHighPeerBackoff** - P95 > 2s for 5m
5. **AnnaConsensusSlowdown** - Round rate < 0.1/sec for 5m
6. **AnnaMigrationFailures** - Migration failures detected
7. **AnnaTLSHandshakeLatency** - Rate decreased >50% vs 10m ago

## SLOs & SLIs

### Availability SLO: 99.9% uptime
- **SLI**: `(1 - rate(anna_consensus_rounds_total[30d] == 0)) * 100`
- **Target**: 99.9% (43.2 minutes downtime/month)

### Consensus Performance SLO: <1s round time
- **SLI**: `histogram_quantile(0.95, rate(anna_consensus_rounds_duration_bucket[5m]))`
- **Target**: p95 < 1000ms

### Security SLO: Zero pinning violations
- **SLI**: `increase(anna_pinning_violations_total[30d])`
- **Target**: 0 violations

## Troubleshooting

### Metrics Not Showing

**Problem**: Grafana shows "No data"

**Solutions**:
1. Verify annad is running: `sudo systemctl status annad`
2. Check metrics endpoint: `curl http://localhost:9090/metrics`
3. Verify Prometheus is scraping: http://localhost:9090/targets
4. Check Prometheus logs: `sudo journalctl -u prometheus -f`

### Alerts Not Firing

**Problem**: Expected alert not triggering

**Solutions**:
1. Check alert rules loaded: http://localhost:9090/rules
2. Verify alert expression in Prometheus: http://localhost:9090/graph
3. Check Alertmanager status: http://localhost:9093
4. Review alert evaluation interval in `prometheus.yml`

### Dashboard Import Fails

**Problem**: JSON import error in Grafana

**Solutions**:
1. Verify JSON syntax: `jq . observability/grafana/anna-overview.json`
2. Ensure Prometheus datasource exists and is named "Prometheus"
3. Check Grafana version compatibility (>= 8.0.0)
4. Review Grafana logs: `sudo journalctl -u grafana -f`

## Integration with CI/CD

Add to `.github/workflows/release.yml`:

```yaml
- name: Validate observability configs
  run: |
    promtool check rules observability/prometheus/anna-critical.yml
    promtool check rules observability/prometheus/anna-warnings.yml
```

## Next Steps

1. Set up Alertmanager with Slack/PagerDuty
2. Configure retention policies (Prometheus: 15d, Grafana: 90d)
3. Add custom dashboards for specific use cases
4. Implement distributed tracing (Jaeger/Tempo)
5. Set up log aggregation (Loki/ELK)

## References

- [Prometheus Documentation](https://prometheus.io/docs/)
- [Grafana Dashboard Best Practices](https://grafana.com/docs/grafana/latest/best-practices/)
- [SRE Book - Monitoring Distributed Systems](https://sre.google/sre-book/monitoring-distributed-systems/)
- [Four Golden Signals](https://sre.google/sre-book/monitoring-distributed-systems/#xref_monitoring_golden-signals)

---

**v1.0 - Phase 2.0.0-alpha.1**

For questions or issues, see: https://github.com/jjgarcianorway/anna-assistant/issues
