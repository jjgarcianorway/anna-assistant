# Anna Assistant v0.12.7 Development Roadmap

## Overview

**Version**: v0.12.7-pre
**Focus**: Daemon Self-Diagnostics, Dynamic Reload, Enhanced Observability
**Status**: Development (pending v0.12.6-pre validation)

v0.12.7 builds on the daemon restart reliability improvements from v0.12.6-pre by adding comprehensive self-diagnostics, configuration hot-reload, and enhanced RPC error handling.

## Prerequisites

Before starting v0.12.7 development:
- [ ] v0.12.6-pre validated (run `sudo ./scripts/upgrade_to_v0126.sh`)
- [ ] Smoke tests passing (10/10 checks)
- [ ] No RPC timeouts or version mismatches
- [ ] Storage commands working (`annactl storage btrfs show`)

## Feature Areas

### 1. Enhanced Health Checks (Priority: HIGH)

#### Objective
Provide comprehensive daemon self-diagnostics beyond basic systemd status checks.

#### Components

**1.1 Health Metrics Collection**
- **RPC Latency Monitoring**
  - Track request/response times for all RPC methods
  - Maintain sliding window (last 100 requests)
  - Alert on p99 latency > 500ms
  - File: `src/annad/src/health_metrics.rs`

- **Memory/CPU Tracking**
  - Read from `/proc/self/status` and `/proc/self/stat`
  - Track: RSS, VmSize, CPU time, thread count
  - Alert on memory > 70MB (systemd limit is 80MB)
  - File: `src/annad/src/resource_monitor.rs`

- **Event Queue Depth**
  - Track pending events in EventEngine queue
  - Alert on queue depth > 100 (potential backlog)
  - Measure processing rate (events/sec)
  - File: `src/annad/src/events.rs` (extend existing)

**1.2 Health Check Command**
- Extend `annactl doctor check` with daemon-specific checks
- New checks:
  - RPC latency (average, p95, p99)
  - Memory usage (current, peak, limit)
  - Queue health (pending, rate, oldest event)
  - Socket health (file exists, permissions, response time)
  - Capabilities health (active count, degraded count, failure rate)

**1.3 Health Endpoints**
- Add JSON RPC method: `get_health_metrics`
- Return structured health data:
  ```json
  {
    "rpc_latency_ms": {"avg": 12, "p95": 45, "p99": 78},
    "memory_mb": {"current": 22, "peak": 27, "limit": 80},
    "queue": {"depth": 3, "rate": 15.2, "oldest_sec": 2},
    "uptime_sec": 86400,
    "capabilities": {"active": 4, "degraded": 1}
  }
  ```

**1.4 Health Dashboard**
- Add `annactl health` command with TUI display
- Show real-time metrics (updating every 1s if `--watch`)
- Color-coded status: green (healthy), yellow (warning), red (critical)

#### Implementation Files
- `src/annad/src/health_metrics.rs` (new)
- `src/annad/src/resource_monitor.rs` (new)
- `src/annad/src/rpc_server.rs` (extend with health endpoint)
- `src/annactl/src/health_cmd.rs` (new)
- `src/annactl/src/doctor_cmd.rs` (extend with daemon checks)

#### Success Metrics
- [ ] `annactl health` shows real-time metrics
- [ ] `annactl doctor check` includes 5 new daemon checks
- [ ] RPC latency tracked with p99 < 100ms under normal load
- [ ] Memory alerts trigger before systemd OOM kill

---

### 2. Dynamic Reload (Priority: MEDIUM)

#### Objective
Reload configuration without full daemon restart, reducing downtime.

#### Components

**2.1 SIGHUP Handler**
- Implement signal handler in daemon
- On SIGHUP:
  - Reload `/etc/anna/config.toml`
  - Reload policy files from `/etc/anna/policies.d/`
  - Reinitialize modules (if enabled/disabled changed)
  - Preserve in-memory state (telemetry, event history)
- File: `src/annad/src/signal_handlers.rs` (new)

**2.2 Config Validation**
- Validate new config before applying
- If validation fails: log error, keep old config, return non-zero
- Support dry-run: `annactl config validate /etc/anna/config.toml`

**2.3 Reload Command**
- Add `annactl reload` command
- Sends SIGHUP to daemon via `kill -HUP $(pgrep annad)`
- Wait for confirmation via RPC health check
- Show before/after comparison of config

**2.4 Auto-reload on File Change (Future)**
- Use `inotify` to watch config files
- Auto-reload on modification (opt-in via config)
- Rate-limit: max 1 reload per 5 seconds

#### Implementation Files
- `src/annad/src/signal_handlers.rs` (new)
- `src/annad/src/config_reload.rs` (new)
- `src/annactl/src/reload_cmd.rs` (new)
- `src/annad/src/main.rs` (register signal handlers)

#### Success Metrics
- [ ] `annactl reload` reloads config without restart
- [ ] Config errors logged without crashing daemon
- [ ] Uptime preserved across reloads
- [ ] No event loss during reload

---

### 3. Storage Enhancements (Priority: MEDIUM)

#### Objective
Richer Btrfs insights and automation support.

#### Components

**3.1 Subvolume Tree Visualization**
- Add `annactl storage btrfs tree` command
- ASCII tree showing subvolume hierarchy:
  ```
  /
  ├── @ (rw, root)
  ├── @home (rw)
  └── @snapshots (ro)
      ├── @-2025-11-01-pre-upgrade (ro)
      └── @-2025-11-02-backup (ro)
  ```
- Show mount status, read-only flag, size

**3.2 Snapshot Diff Preview**
- Add `annactl storage btrfs diff <snap1> <snap2>`
- Show file changes between snapshots
- Summarize: files added, removed, modified, size delta

**3.3 Balance Status Tracking**
- Add balance status to `annactl storage btrfs show`
- Show: last balance time, data/metadata usage
- Recommend balance if metadata > 80% full

**3.4 Scrub Scheduling**
- Track last scrub time in telemetry
- Recommend scrub if > 30 days since last
- Show scrub results: errors found, time taken

#### Implementation Files
- `src/annad/src/storage_v10.rs` (extend BtrfsProfile)
- `src/annactl/src/storage_cmd.rs` (add tree, diff subcommands)
- `scripts/btrfs/scrub.sh` (new automation script)

#### Success Metrics
- [ ] `annactl storage btrfs tree` shows hierarchy
- [ ] `annactl storage btrfs diff` compares snapshots
- [ ] Balance recommendations in advisor
- [ ] Scrub status in health checks

---

### 4. Better RPC Errors (Priority: LOW)

#### Objective
Clear, actionable error messages with retry logic.

#### Components

**4.1 Structured Error Codes**
- Define error enum:
  ```rust
  pub enum RpcError {
      SocketNotFound,      // /run/anna/annad.sock missing
      SocketPermission,    // Permission denied
      Timeout(Duration),   // Read/write timeout
      DaemonCrashed,       // Process not running
      InvalidResponse,     // Malformed JSON
      MethodNotFound(String), // Unrecognized subcommand
  }
  ```
- Include error code in JSON response

**4.2 Retry Logic**
- Exponential backoff: 100ms, 200ms, 400ms (max 3 retries)
- Only retry on transient errors (timeout, connection refused)
- Don't retry on permanent errors (method not found, invalid args)

**4.3 User-Friendly Messages**
- Map error codes to helpful messages:
  - `SocketNotFound`: "Daemon not running. Start with: sudo systemctl start annad"
  - `Timeout`: "Daemon not responding. Check logs: sudo journalctl -u annad -n 20"
  - `MethodNotFound`: "Command not supported. Upgrade daemon to match annactl version."

**4.4 Timeout Categorization**
- Distinguish:
  - Socket missing (instant failure)
  - Connection timeout (daemon starting)
  - Read timeout (daemon busy/hung)
- Provide specific guidance for each

#### Implementation Files
- `src/anna_common/src/rpc_client.rs` (extend with error types)
- `src/anna_common/src/rpc_server.rs` (return structured errors)
- `src/annactl/src/main.rs` (pretty-print errors)

#### Success Metrics
- [ ] Clear error messages with actionable next steps
- [ ] Retry logic reduces transient failures
- [ ] Error codes documented in help text
- [ ] No generic "RPC failed" messages

---

## Development Phases

### Phase 1: Health Metrics Foundation (Week 1)
**Goal**: Core health metrics collection and storage

- [ ] Create `health_metrics.rs` with metrics structs
- [ ] Implement RPC latency tracking (wrap all RPC calls)
- [ ] Implement resource monitoring (memory, CPU, threads)
- [ ] Add `get_health_metrics` RPC method
- [ ] Unit tests for metric collection

**Deliverable**: Daemon collects health metrics, accessible via RPC

### Phase 2: Health Commands (Week 1)
**Goal**: User-facing health diagnostics

- [ ] Create `annactl health` command with TUI display
- [ ] Extend `annactl doctor check` with daemon checks
- [ ] Add `--watch` mode for real-time monitoring
- [ ] Integration tests for health commands

**Deliverable**: `annactl health` shows live daemon metrics

### Phase 3: Dynamic Reload (Week 2)
**Goal**: SIGHUP-based config reload

- [ ] Implement SIGHUP signal handler
- [ ] Add config validation function
- [ ] Create `annactl reload` command
- [ ] Test reload with config changes
- [ ] Document reload limitations (what doesn't reload)

**Deliverable**: `annactl reload` reloads config without downtime

### Phase 4: Storage Enhancements (Week 2)
**Goal**: Richer Btrfs tooling

- [ ] Implement subvolume tree parsing
- [ ] Create `annactl storage btrfs tree` command
- [ ] Add snapshot diff functionality
- [ ] Extend advisor with balance/scrub checks

**Deliverable**: `annactl storage btrfs tree` visualizes hierarchy

### Phase 5: RPC Error Improvements (Week 3)
**Goal**: Better error handling

- [ ] Define structured error types
- [ ] Implement retry logic with backoff
- [ ] Map errors to user-friendly messages
- [ ] Update all RPC call sites

**Deliverable**: Clear, actionable error messages throughout CLI

### Phase 6: Testing & Documentation (Week 3)
**Goal**: Comprehensive validation

- [ ] Write smoke tests for all new features
- [ ] Update `verify_v0122.sh` with health checks
- [ ] Document all new commands in `--help`
- [ ] Create `docs/HEALTH-CHECKS.md` guide
- [ ] Update `CHANGELOG.md` with v0.12.7 entry

**Deliverable**: All tests passing, docs complete

---

## Testing Strategy

### Unit Tests
- `health_metrics.rs`: Metric collection, averaging, percentiles
- `resource_monitor.rs`: /proc parsing, memory calculations
- `config_reload.rs`: Config validation, error handling
- `rpc_client.rs`: Retry logic, timeout handling

### Integration Tests
- Health metrics end-to-end (daemon → RPC → annactl)
- Config reload (modify config → reload → verify changes)
- Storage tree (parse btrfs output → display tree)

### Smoke Tests
- Add to `verify_v0122.sh`:
  - `annactl health` returns metrics
  - `annactl reload` succeeds
  - `annactl storage btrfs tree` shows hierarchy
  - Error messages are user-friendly

### Performance Tests
- RPC latency under load (100 req/s)
- Memory usage during reload
- Queue processing rate (events/sec)

---

## Documentation

### New Docs
- `docs/HEALTH-CHECKS.md` - Health monitoring guide
- `docs/DYNAMIC-RELOAD.md` - Config reload usage
- `docs/STORAGE-TREE.md` - Btrfs hierarchy visualization

### Updated Docs
- `docs/ADVISOR-ARCH.md` - Add health-based advisor rules
- `docs/STORAGE-BTRFS.md` - Add tree and diff sections
- `CHANGELOG.md` - v0.12.7 entry

### Help Text
- `annactl health --help`
- `annactl reload --help`
- `annactl storage btrfs tree --help`
- `annactl storage btrfs diff --help`

---

## Success Criteria

v0.12.7 is ready for release when:

- [ ] All 4 feature areas implemented and tested
- [ ] Smoke tests pass (10/10 checks + new v0.12.7 checks)
- [ ] No regressions from v0.12.6-pre
- [ ] Documentation complete and accurate
- [ ] Performance metrics acceptable:
  - RPC p99 latency < 100ms
  - Memory usage < 60MB steady state
  - Config reload < 100ms
- [ ] Error messages are clear and actionable

---

## Known Limitations

### Out of Scope for v0.12.7
- Remote health monitoring (Prometheus/Grafana)
- Automated health-based remediation
- Config hot-reload for all settings (some require restart)
- Multi-daemon orchestration

### Future Work (v0.12.8+)
- Telemetry export (Prometheus format)
- Health-based auto-scaling (dynamic CPU limits)
- Distributed tracing (OpenTelemetry)
- Config versioning and rollback

---

## Dependencies

### Build Dependencies
- No new external crates needed for Phase 1-3
- Consider for Phase 4+:
  - `signal-hook` for advanced signal handling
  - `notify` for inotify-based file watching

### System Dependencies
- Linux signals (SIGHUP)
- /proc filesystem (resource monitoring)
- btrfs-progs (for tree parsing)

---

## Timeline

**Start**: After v0.12.6-pre validation
**Target Release**: ~3 weeks from start
**Milestones**:
- Week 1: Health checks complete
- Week 2: Dynamic reload + storage enhancements
- Week 3: RPC errors + testing + docs

---

## Contact & Collaboration

**Lead**: Claude Code (via jjgarcianorway)
**Repository**: https://github.com/jjgarcianorway/anna-assistant
**Issues**: Report bugs via GitHub issues

---

**Last Updated**: 2025-11-02
**Version**: v0.12.7-pre (planning)
