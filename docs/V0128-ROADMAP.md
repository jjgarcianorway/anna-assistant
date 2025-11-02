# Anna v0.12.8-pre Development Roadmap

## Overview

**Version**: v0.12.8-pre
**Start Date**: 2025-11-02
**Target Duration**: 2-3 days
**Focus Areas**: RPC Error Improvements, Advanced Storage Features, Live Telemetry

---

## 🎯 Release Objectives

v0.12.8-pre builds upon the solid foundation of v0.12.7-pre (health monitoring and dynamic reload) by adding:

1. **Structured Error Handling** - Professional error codes, retry logic, user-friendly messages
2. **Advanced Storage Features** - Tree visualization, snapshot diff, status parsing
3. **Live Telemetry** - Watch mode, real-time metrics, complete queue tracking

---

## 📦 Three-Phase Development Plan

### Phase 1: Structured RPC Error Codes (Priority: HIGH)

**Duration**: ~2 hours
**Complexity**: Medium
**Dependencies**: None

#### Objective

Replace generic RPC errors with structured, actionable error codes that enable intelligent retry logic and provide clear user guidance.

#### Components

**1.1 Error Code Taxonomy**

Define comprehensive error code hierarchy:

```rust
pub enum RpcErrorCode {
    // Connection errors (1000-1099)
    ConnectionRefused = 1000,
    ConnectionTimeout = 1001,
    SocketNotFound = 1002,
    PermissionDenied = 1003,

    // Request errors (2000-2099)
    InvalidRequest = 2000,
    MalformedJson = 2001,
    MissingParameter = 2002,
    InvalidParameter = 2003,

    // Server errors (3000-3099)
    InternalError = 3000,
    DatabaseError = 3001,
    CollectionFailed = 3002,
    Timeout = 3003,

    // Resource errors (4000-4099)
    ResourceNotFound = 4000,
    ResourceBusy = 4001,
    QuotaExceeded = 4002,
}
```

**1.2 Error Metadata**

Each error includes:
- **Code**: Numeric identifier
- **Message**: Human-readable description
- **Retryable**: Boolean (can this be retried?)
- **Retry-After**: Duration (if applicable)
- **Help**: Actionable suggestion for user
- **Context**: Additional debug information

**1.3 Retry Policy**

```rust
pub struct RetryPolicy {
    max_attempts: u32,          // Default: 3
    initial_delay_ms: u64,      // Default: 100
    max_delay_ms: u64,          // Default: 5000
    backoff_multiplier: f32,    // Default: 2.0
    jitter_factor: f32,         // Default: 0.1
}
```

Backoff curve:
- Attempt 1: 100ms
- Attempt 2: 200ms + jitter
- Attempt 3: 400ms + jitter
- Attempt 4+: Cap at 5000ms

**1.4 CLI Error Display**

Beautiful error formatting:
```
╭─ RPC Error ──────────────────────────────────────────
│
│  Error Code:    1001 (ConnectionTimeout)
│  Severity:      Warning
│  Retryable:     Yes
│  Attempts:      3/3
│  Total Time:    850ms
│
│  Message:
│  Connection to daemon timed out after 2 seconds.
│
│  Possible Causes:
│  • Daemon is not running
│  • Daemon is overloaded
│  • Network latency issues
│
│  Suggested Actions:
│  1. Check daemon status: sudo systemctl status annad
│  2. Check daemon logs: sudo journalctl -u annad -n 20
│  3. Restart daemon: sudo systemctl restart annad
│
╰──────────────────────────────────────────────────────
```

#### Implementation Files

- `src/annad/src/rpc_errors.rs` (new) - Error codes and metadata
- `src/annad/src/rpc_v10.rs` (modify) - Return structured errors
- `src/annactl/src/error_display.rs` (new) - CLI error formatting
- `src/annactl/src/main.rs` (modify) - Integrate retry logic

#### Success Metrics

- [ ] All RPC errors use structured codes
- [ ] Retry logic works with exponential backoff
- [ ] CLI displays helpful error messages
- [ ] Error rate tracked in health metrics
- [ ] Unit tests cover error mapping and retry

---

### Phase 2: Snapshot Diff & Visualization (Priority: MEDIUM)

**Duration**: ~3 hours
**Complexity**: High
**Dependencies**: Phase 1 (for error handling)

#### Objective

Provide advanced Btrfs insights with visual tree representation and snapshot comparison tools.

#### Components

**2.1 Subvolume Tree Visualization**

Command: `annactl storage btrfs tree`

Output:
```
/  (476.9 GB, Btrfs)
├── @  (rw, default, 145.2 GB)
│   ├── .snapshots/
│   │   ├── 1  (ro, 12.3 GB, 2025-10-30 14:23)
│   │   ├── 2  (ro, 12.5 GB, 2025-10-31 09:15)
│   │   └── 3  (ro, 12.8 GB, 2025-11-01 16:42)
│   ├── usr/  (data)
│   ├── var/  (data)
│   └── home/ → @home  (subvolume)
├── @home  (rw, 89.4 GB)
│   └── user/  (data)
└── @var  (rw, 23.1 GB)
    ├── log/  (data)
    └── lib/  (data)
```

Features:
- ASCII tree with box-drawing characters
- Size information per subvolume
- Mount point indicators
- Snapshot ages
- Read-only status

**2.2 Snapshot Diff**

Command: `annactl storage btrfs diff <snap1> <snap2>`

Output:
```
╭─ Snapshot Diff ──────────────────────────────────────────────────
│
│  Base:     /.snapshots/1 (2025-10-30 14:23:15)
│  Current:  /.snapshots/2 (2025-10-31 09:15:42)
│  Duration: 18h 52m
│
│  Summary:
│  • Added:     23 files    (+145.2 MB)
│  • Modified:  156 files   (+89.3 MB, -45.1 MB)
│  • Deleted:   5 files     (-2.1 MB)
│  • Net:       +187.3 MB
│
│  Top Changes:
│    +125.3 MB  /usr/lib/libfoo.so (new)
│    +45.8 MB   /var/cache/pacman/pkg/ (added packages)
│    -23.1 MB   /tmp/old-cache (deleted)
│
│  Package Changes:
│    ✓ Installed: linux-6.17.7 (78.2 MB)
│    ✓ Upgraded: systemd 256.7 → 256.8 (12.3 MB delta)
│    ✗ Removed: old-package (2.1 MB)
│
╰──────────────────────────────────────────────────────────────────
```

**2.3 Balance/Scrub Status Parsing**

Improve Btrfs health reporting:
- Parse `btrfs balance status`
- Parse `btrfs scrub status`
- Show progress bars for in-progress operations
- Estimate time remaining
- Alert on errors found

#### Implementation Files

- `src/annad/src/storage_btrfs.rs` (extend) - Tree building, diff logic
- `src/annactl/src/storage_cmd.rs` (extend) - Tree/diff commands
- `src/annad/src/rpc_v10.rs` (add methods) - `storage_tree`, `storage_diff`

#### Success Metrics

- [ ] Tree visualization works for complex layouts
- [ ] Snapshot diff shows accurate file changes
- [ ] Balance/scrub status parsed correctly
- [ ] Performance acceptable (<2s for tree, <5s for diff)
- [ ] JSON output mode available

---

### Phase 3: Live Telemetry & Watch Mode (Priority: MEDIUM)

**Duration**: ~2 hours
**Complexity**: Medium
**Dependencies**: Phase 1 (for error handling)

#### Objective

Implement real-time monitoring with live-updating displays and complete queue metrics.

#### Components

**3.1 Watch Mode**

Command: `annactl health --watch`

Behavior:
- Updates every 1 second
- Clears terminal and redraws
- Shows delta indicators (↑ ↓ →)
- Colorizes changes (red for worse, green for better)
- Press Ctrl+C to exit

Example:
```
╭─ Anna Daemon Health (Live) ──────────────────────────
│  Last Update: 10:35:42  (refreshes every 1s)
│
│  RPC Latency:   p99=78ms ✓  (→ no change)
│  Memory:        25.8MB ↑    (+0.4MB in last 5s)
│  Queue Depth:   7 ↑         (+2 events)
│  Uptime:        1h 35m
│
│  Press Ctrl+C to exit
╰──────────────────────────────────────────────────────
```

**3.2 Queue Metrics (Complete)**

Implement TODOs from v0.12.7-pre:
- **Queue rate**: Calculate events/second over last 60s
- **Oldest event age**: Track timestamp of oldest pending event
- **Event histogram**: Show distribution by domain

**3.3 Event Count from Database**

Query actual event counts instead of hardcoded `1`:
```sql
SELECT COUNT(*) FROM events WHERE timestamp > ?
```

#### Implementation Files

- `src/annactl/src/health_cmd.rs` (extend) - Watch mode loop
- `src/annad/src/health_metrics.rs` (extend) - Queue rate calculation
- `src/annad/src/events.rs` (extend) - Oldest event tracking
- `src/annad/src/rpc_v10.rs` (fix) - Query event count from DB

#### Success Metrics

- [ ] Watch mode updates smoothly
- [ ] No flickering or tearing
- [ ] Queue rate calculated accurately
- [ ] Oldest event age tracked
- [ ] Event counts from database correct
- [ ] Terminal restored properly on exit

---

## 📊 Technical Specifications

### Error Code Ranges

| Range | Category | Retryable |
|-------|----------|-----------|
| 1000-1099 | Connection errors | Yes (with backoff) |
| 2000-2099 | Client errors | No |
| 3000-3099 | Server errors | Yes (with backoff) |
| 4000-4099 | Resource errors | Conditional |

### Retry Algorithm

```
delay = min(
    initial_delay * (multiplier ^ attempt),
    max_delay
) * (1.0 + random(-jitter, +jitter))
```

### Performance Targets

| Operation | Target | Max Acceptable |
|-----------|--------|----------------|
| Error display | <10ms | <50ms |
| Retry decision | <1ms | <5ms |
| Tree build | <500ms | <2s |
| Snapshot diff | <2s | <10s |
| Watch refresh | <100ms | <500ms |

---

## 🧪 Testing Strategy

### Phase 1: Error Handling

- Unit tests for error code mapping
- Unit tests for retry backoff calculation
- Integration tests for retry flow
- CLI error display visual verification

### Phase 2: Storage Features

- Unit tests for tree building algorithm
- Unit tests for snapshot diff logic
- Integration tests with mock Btrfs filesystem
- Performance tests with large subvolume counts

### Phase 3: Live Telemetry

- Unit tests for queue rate calculation
- Integration tests for watch mode
- Terminal rendering tests
- Ctrl+C signal handling tests

---

## 📚 Documentation Plan

### Phase 1
- `docs/V0128-PHASE1-IMPLEMENTATION.md` - Error handling guide
- Error code reference table
- Retry policy configuration guide
- CLI error examples

### Phase 2
- `docs/V0128-PHASE2-IMPLEMENTATION.md` - Storage features guide
- Tree visualization examples
- Snapshot diff use cases
- Performance benchmarks

### Phase 3
- `docs/V0128-PHASE3-IMPLEMENTATION.md` - Live telemetry guide
- Watch mode usage instructions
- Queue metrics interpretation
- Terminal compatibility notes

### Final
- `docs/V0128-RELEASE-SUMMARY.md` - Complete release overview
- `CHANGELOG.md` - Updated with all features
- Migration guide (v0.12.7-pre → v0.12.8-pre)

---

## 🎯 Success Metrics

### Phase 1
- [ ] Zero generic "RPC error" messages
- [ ] Retry success rate >70% for transient failures
- [ ] Error display time <50ms
- [ ] User satisfaction with error clarity

### Phase 2
- [ ] Tree depth ≥5 levels supported
- [ ] Diff accuracy 100% (vs manual comparison)
- [ ] Performance within targets
- [ ] JSON output parseable

### Phase 3
- [ ] Watch mode refresh rate 1Hz stable
- [ ] Queue rate accuracy ±5%
- [ ] Terminal restored 100% of the time
- [ ] No memory leaks during watch

### Overall
- [ ] Build: 0 errors, ≤40 warnings
- [ ] Tests: 100% passing (30+ tests)
- [ ] Performance: <1% overhead vs v0.12.7-pre
- [ ] Memory: <10 KB additional footprint
- [ ] Zero regressions

---

## 🚧 Known Risks

### Phase 1
- **Risk**: Retry logic could cause excessive load
- **Mitigation**: Cap max attempts, exponential backoff, rate limiting

### Phase 2
- **Risk**: Snapshot diff could be slow on large filesystems
- **Mitigation**: Add progress indicator, implement streaming, add timeout

### Phase 3
- **Risk**: Watch mode could flicker on slow terminals
- **Mitigation**: Use alternate screen buffer, double buffering

---

## 🔄 Migration Path

### From v0.12.7-pre

**Automatic**:
- Error handling enhanced (backward compatible)
- New commands available immediately
- No configuration changes required

**Optional**:
- Try new `annactl storage btrfs tree` command
- Use `annactl health --watch` for monitoring
- Configure retry policy (if defaults not suitable)

**Breaking Changes**: None

---

## 📅 Timeline

| Phase | Duration | Cumulative |
|-------|----------|------------|
| Phase 1: RPC Errors | 2 hours | 2 hours |
| Phase 2: Storage Features | 3 hours | 5 hours |
| Phase 3: Live Telemetry | 2 hours | 7 hours |
| Testing & Docs | 2 hours | 9 hours |
| **Total** | **9 hours** | **~2 days** |

---

## 🎉 Expected Outcomes

By end of v0.12.8-pre development:

1. **Professional Error Handling**
   - Users never see cryptic errors
   - Transient failures auto-retry
   - Clear guidance on resolution

2. **Advanced Storage Insights**
   - Visual understanding of Btrfs layout
   - Ability to compare snapshots
   - Better health monitoring

3. **Live Monitoring**
   - Real-time daemon health
   - Accurate queue metrics
   - Smooth watch mode experience

4. **Production Readiness**
   - Comprehensive error coverage
   - Robust retry logic
   - Polished user experience

---

**Roadmap Prepared by**: Claude Code
**Date**: 2025-11-02
**Version**: v0.12.8-pre
**Status**: Ready for Development
