# Proactive Engine Architecture Design
**Anna Assistant - Proactive Sysadmin Autonomy Level 1**

**Status**: Architecture Design Phase
**Target Betas**: 270, 271, 272
**Scope**: Deterministic diagnostic correlation and proactive issue surfacing

---

## Executive Summary

The Proactive Engine transforms Anna from a reactive diagnostic tool into a **proactive senior-engineer-level system monitor** that:
- Detects issues before users ask
- Correlates multiple signals into coherent root-cause reports
- Prioritizes issues by severity and impact
- Surfaces actionable recommendations automatically
- Maintains 100% deterministic operation (zero LLM involvement)

---

## Design Principles

### 1. Deterministic Core
- **No LLM in critical paths**: All correlation, prioritization, and root-cause analysis uses rule-based logic
- **Reproducible**: Same inputs always produce same outputs
- **Testable**: Full unit test coverage with predictable behavior
- **Explainable**: Every conclusion traceable to specific telemetry evidence

### 2. Correlation Intelligence
- **Multi-signal fusion**: Combine evidence from multiple subsystems (network, disk, services, logs, CPU, memory)
- **Root-cause deduction**: Identify underlying causes, not just symptoms
- **Temporal awareness**: Track trends, flapping, escalation, and recovery
- **Context preservation**: Link related issues across time windows

### 3. Zero Surface Changes
- **No new CLI commands**: Integrate into existing `status`, TUI, and NL query flows
- **Canonical formatting**: All output follows [SUMMARY] + [DETAILS] + [COMMANDS] pattern
- **Backward compatibility**: Existing tests remain valid

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│                     PROACTIVE ENGINE                         │
│                  (proactive_engine.rs)                       │
└─────────────────────────────────────────────────────────────┘
                            │
        ┌───────────────────┼───────────────────┐
        ▼                   ▼                   ▼
┌──────────────┐   ┌──────────────┐   ┌──────────────┐
│  Data Input  │   │  Correlation │   │    Output    │
│    Layer     │   │    Engine    │   │   Layer      │
└──────────────┘   └──────────────┘   └──────────────┘
        │                   │                   │
        ▼                   ▼                   ▼
┌──────────────┐   ┌──────────────┐   ┌──────────────┐
│ • Brain      │   │ • Rule-based │   │ • Prioritized│
│ • Health     │   │ • Pattern    │   │   Insights   │
│ • Network    │   │   matching   │   │ • Remediation│
│ • Historian  │   │ • Temporal   │   │   routing    │
│ • SystemdLog │   │   analysis   │   │ • Trend data │
└──────────────┘   └──────────────┘   └──────────────┘
```

---

## Core Data Structures

### ProactiveAssessment
The primary output of the proactive engine.

```rust
pub struct ProactiveAssessment {
    /// When this assessment was generated
    pub timestamp: DateTime<Utc>,

    /// Correlated issues (root causes + symptoms)
    pub correlated_issues: Vec<CorrelatedIssue>,

    /// Detected trends (degradation, improvement, flapping)
    pub trends: Vec<TrendObservation>,

    /// Recovery notices (issues resolved since last check)
    pub recoveries: Vec<RecoveryNotice>,

    /// Overall system health score (0-100)
    pub health_score: u8,

    /// Highest severity present
    pub max_severity: DiagnosticSeverity,

    /// Total issue count by severity
    pub critical_count: usize,
    pub warning_count: usize,
    pub info_count: usize,
}
```

### CorrelatedIssue
A root-cause analysis linking multiple signals.

```rust
pub struct CorrelatedIssue {
    /// Unique correlation ID
    pub correlation_id: String,

    /// Root cause identified
    pub root_cause: RootCause,

    /// All contributing signals
    pub contributing_signals: Vec<Signal>,

    /// Severity (highest from all signals)
    pub severity: DiagnosticSeverity,

    /// Human-readable summary
    pub summary: String,

    /// Technical details
    pub details: String,

    /// Remediation steps (commands to fix)
    pub remediation_commands: Vec<String>,

    /// Confidence level (0-100)
    pub confidence: u8,

    /// First detected timestamp
    pub first_seen: DateTime<Utc>,

    /// Last updated timestamp
    pub last_seen: DateTime<Utc>,
}
```

### RootCause (enum)
Known root-cause categories.

```rust
pub enum RootCause {
    // Network root causes
    NetworkRoutingConflict { duplicate_routes: Vec<String> },
    NetworkPriorityMismatch { slow_interface: String, fast_interface: String },
    NetworkQualityDegradation { packet_loss: f64, latency_ms: f64 },

    // Disk root causes
    DiskPressure { mountpoint: String, usage_percent: u8, inode_exhaustion: bool },
    DiskLogGrowth { log_path: String, growth_rate_mb_per_hour: f64 },

    // Service root causes
    ServiceFlapping { service_name: String, restart_count: u32 },
    ServiceUnderLoad { service_name: String, cpu_percent: f64, memory_mb: u64 },
    ServiceConfigError { service_name: String, error_message: String },

    // Resource root causes
    MemoryPressure { ram_percent: f64, swap_percent: Option<f64> },
    CpuOverload { load_per_core: f64, runaway_process: Option<String> },

    // System root causes
    KernelRegression { boot_errors: u32, driver_failures: Vec<String> },
    DeviceHotplug { added: Vec<String>, removed: Vec<String> },
}
```

### Signal
Individual telemetry evidence point.

```rust
pub struct Signal {
    /// Where signal came from
    pub source: SignalSource,

    /// What was detected
    pub observation: String,

    /// Raw telemetry value
    pub value: SignalValue,

    /// When it was observed
    pub timestamp: DateTime<Utc>,
}

pub enum SignalSource {
    BrainInsight { rule_id: String },
    HealthReport { subsystem: String },
    NetworkMonitoring { metric: String },
    Historian { event_type: String },
    SystemdJournal { unit: String },
}

pub enum SignalValue {
    Boolean(bool),
    Percentage(f64),
    Count(u32),
    Latency(f64), // milliseconds
    Text(String),
}
```

### TrendObservation
Temporal patterns detected.

```rust
pub struct TrendObservation {
    /// What is trending
    pub subject: String,

    /// Type of trend
    pub trend_type: TrendType,

    /// How long trend has been observed
    pub duration_hours: u32,

    /// Severity if trend continues
    pub projected_severity: DiagnosticSeverity,

    /// Recommendation to prevent escalation
    pub recommendation: String,
}

pub enum TrendType {
    Escalating,      // Getting worse
    Flapping,        // Oscillating
    Degrading,       // Slowly declining
    Improving,       // Getting better
    Recurring,       // Pattern repeats
}
```

### RecoveryNotice
Issues that have been resolved.

```rust
pub struct RecoveryNotice {
    /// What recovered
    pub subject: String,

    /// When it recovered
    pub recovery_time: DateTime<Utc>,

    /// How long it was an issue
    pub duration_hours: u32,

    /// What fixed it (if known)
    pub resolution: Option<String>,
}
```

---

## Correlation Engine Rules

### Rule Priority Hierarchy
Issues are correlated in this order:

1. **Critical System Failures** (services, disk, kernel)
2. **Resource Exhaustion** (memory, CPU, disk space)
3. **Network Degradation** (routing, quality, configuration)
4. **Trends and Predictions** (escalation warnings)
5. **Informational** (recoveries, improvements)

### Correlation Matrix

#### Network Correlation Rules

**Rule: Routing Conflict Detection**
```
IF:
  - duplicate_default_routes detected (from brain)
  - high packet loss (from network monitoring)
  - connection timeouts (from logs)
THEN:
  RootCause = NetworkRoutingConflict
  Severity = Critical
  Remediation = Remove duplicate route + restart NetworkManager
```

**Rule: Priority Mismatch Correlation**
```
IF:
  - network_priority_mismatch detected (from brain)
  - slow interface has default route
  - fast interface available but not used
THEN:
  RootCause = NetworkPriorityMismatch
  Severity = Warning
  Remediation = Disconnect slow interface OR adjust route metrics
```

**Rule: Quality Degradation Correlation**
```
IF:
  - high_packet_loss > 5% (from network monitoring)
  - high_latency > 200ms (from network monitoring)
  - no routing conflicts
THEN:
  RootCause = NetworkQualityDegradation
  Severity = Warning or Critical (based on thresholds)
  Remediation = Check WiFi signal, test cables, restart router
```

#### Disk Correlation Rules

**Rule: Disk Pressure Detection**
```
IF:
  - disk_space_critical OR disk_space_warning (from brain)
  - inode usage > 90% (from df -i)
  - large log growth (from log monitoring)
THEN:
  RootCause = DiskPressure
  Severity = Critical or Warning
  Remediation = Clean package cache, rotate logs, find large files
```

**Rule: Log Growth Correlation**
```
IF:
  - disk usage increasing
  - /var/log growth > 100 MB/hour
  - specific service logging excessively
THEN:
  RootCause = DiskLogGrowth
  Severity = Warning
  Remediation = Rotate logs, reduce log level, investigate service
```

#### Service Correlation Rules

**Rule: Service Flapping Detection**
```
IF:
  - service restarted > 3 times in 1 hour (from systemd journal)
  - service currently running
THEN:
  RootCause = ServiceFlapping
  Severity = Warning
  Remediation = Check service dependencies, review configuration
```

**Rule: Service Under Load**
```
IF:
  - service consuming > 80% CPU
  - service memory usage increasing
  - service responding slowly
THEN:
  RootCause = ServiceUnderLoad
  Severity = Warning or Critical
  Remediation = Check resource limits, review workload, scale if needed
```

**Rule: Service Configuration Error**
```
IF:
  - service failed (from brain)
  - exit code indicates config error
  - no dependency failures
THEN:
  RootCause = ServiceConfigError
  Severity = Critical
  Remediation = Check config syntax, review recent changes
```

#### Resource Correlation Rules

**Rule: Memory Pressure Correlation**
```
IF:
  - memory_pressure_critical OR memory_pressure_warning (from brain)
  - swap usage > 50% (if swap exists)
  - OOM killer events (from logs)
THEN:
  RootCause = MemoryPressure
  Severity = Critical or Warning
  Remediation = Identify memory hogs, add swap, kill runaway processes
```

**Rule: CPU Overload Correlation**
```
IF:
  - cpu_overload_critical OR cpu_high_load (from brain)
  - load per core > 2.0
  - specific process consuming > 80% CPU
THEN:
  RootCause = CpuOverload
  Severity = Critical or Warning
  Remediation = Identify runaway process, adjust priorities, scale workload
```

#### System Correlation Rules

**Rule: Kernel Regression Detection**
```
IF:
  - recent kernel update (from package history)
  - boot errors increased
  - driver failures in logs
THEN:
  RootCause = KernelRegression
  Severity = Critical
  Remediation = Boot previous kernel, report bug, rollback update
```

**Rule: Device Hotplug Correlation**
```
IF:
  - USB/PCI device added or removed (from logs)
  - network interface changed
  - routing changed after device event
THEN:
  RootCause = DeviceHotplug
  Severity = Info
  Context = Explain why network/system changed
```

---

## Temporal Analysis

### Trend Detection Windows
- **Short-term**: Last 1 hour (detect spikes, flapping)
- **Medium-term**: Last 24 hours (detect escalation, degradation)
- **Long-term**: Last 7 days (detect recurring patterns)

### Flapping Detection
```
IF:
  - Issue appears 3+ times in 1 hour
  - Issue resolves between appearances
THEN:
  TrendType = Flapping
  Recommendation = "Investigate intermittent cause"
```

### Escalation Detection
```
IF:
  - Issue severity increased over time
  - Warning → Critical within 24 hours
THEN:
  TrendType = Escalating
  Recommendation = "Take immediate action before failure"
```

### Recovery Tracking
```
IF:
  - Issue present in previous assessment
  - Issue NOT present in current assessment
  - Resolution time < 24 hours ago
THEN:
  Add RecoveryNotice
  Message = "Issue resolved: [description]"
```

---

## Integration Points

### 1. Daemon Health Pipeline (`annad`)
**File**: `crates/annad/src/steward/health.rs`

**Change**: Add proactive assessment to `HealthReport`
```rust
pub struct HealthReport {
    // ... existing fields
    pub proactive_assessment: Option<ProactiveAssessment>, // NEW
}
```

**Call point**: After brain analysis in `check_health()`
```rust
let brain_insights = analyze_system_health(&report);
let proactive_assessment = proactive_engine::assess(
    &brain_insights,
    &report,
    &network_monitoring,
    &historian_summary,
);
```

### 2. Status Command (`annactl status`)
**File**: `crates/annactl/src/status_command.rs`

**Change**: Surface proactive assessment in status output
```rust
// After showing brain insights
if let Some(assessment) = health.proactive_assessment {
    display_proactive_assessment(&assessment);
}
```

### 3. TUI Welcome Screen
**File**: `crates/annactl/src/tui/flow.rs`

**Change**: Show top 3 correlated issues on welcome screen
```rust
if let Some(assessment) = state.health.proactive_assessment {
    render_top_issues(&assessment.correlated_issues[..3]);
}
```

### 4. TUI Exit Summary
**File**: `crates/annactl/src/tui/flow.rs`

**Change**: Show recoveries and new issues on exit
```rust
if let Some(assessment) = state.health.proactive_assessment {
    show_session_changes(&assessment);
}
```

### 5. Natural Language Queries
**File**: `crates/annactl/src/unified_query_handler.rs`

**Change**: Route proactive questions to assessment
```rust
"what's wrong?" => show_correlated_issues()
"any problems?" => show_all_issues()
"what recovered?" => show_recoveries()
"what's trending?" => show_trends()
```

---

## Health Score Calculation

**Formula**:
```
health_score = 100
  - (critical_issues × 20)
  - (warning_issues × 10)
  - (escalating_trends × 5)
  - (flapping_issues × 3)

Clamped to [0, 100]
```

**Interpretation**:
- **90-100**: Healthy (green)
- **70-89**: Degraded (yellow)
- **50-69**: Warning (orange)
- **0-49**: Critical (red)

---

## Output Formatting

### CLI Status Output
```
[SYSTEM HEALTH]
Score: 72/100 (Degraded)

[CRITICAL ISSUES] (1)
• Disk pressure on /var (95% full, inodes 92%)
  └─ Root cause: Log growth + package cache
  └─ Fix: Clean package cache, rotate logs
  └─ Commands: pacman -Sc, journalctl --vacuum-size=100M

[WARNINGS] (2)
• Network priority mismatch (eth0 100Mbps > wlan0 866Mbps)
  └─ Root cause: USB Ethernet adapter taking priority
  └─ Fix: Disconnect eth0 or adjust route metrics
  └─ Commands: nmcli connection down eth0

• Service flapping: nginx.service (5 restarts in 1h)
  └─ Root cause: Configuration reload loop
  └─ Fix: Check nginx.conf syntax, review recent changes
  └─ Commands: nginx -t, systemctl status nginx

[TRENDS] (1)
• CPU load escalating (0.8 → 1.2 → 1.8 over 6 hours)
  └─ Projected: Critical within 2 hours
  └─ Recommendation: Identify and address runaway process now

[RECOVERIES] (1)
✓ Memory pressure resolved (2 hours ago)
  └─ Was: 94% RAM usage
  └─ Now: 62% RAM usage
```

### TUI Welcome Display
```
╔═══════════════════════════════════════════════════════════╗
║           ANNA SYSTEM ASSISTANT - HEALTH: 72/100          ║
╚═══════════════════════════════════════════════════════════╝

🔴 CRITICAL (1)
   Disk pressure on /var (95% full)
   → Run: pacman -Sc && journalctl --vacuum-size=100M

⚠️  WARNINGS (2)
   Network priority mismatch
   Service flapping: nginx.service

📈 TRENDS (1)
   CPU load escalating - take action within 2 hours

✅ RECOVERIES (1)
   Memory pressure resolved

Press 'h' for help, 'q' to quit, or type a question...
```

---

## Testing Strategy

### Regression Test Suites

#### 1. Core Correlation Tests (20 tests)
**File**: `crates/annad/tests/regression_proactive_correlation.rs`

Tests:
- Network routing conflict correlation
- Network priority mismatch correlation
- Disk pressure correlation (space + inodes)
- Disk log growth correlation
- Service flapping detection
- Service under load correlation
- Memory pressure correlation
- CPU overload correlation
- Kernel regression detection
- Device hotplug correlation
- Multiple signals → single root cause
- No false positives (healthy system)
- Confidence scoring
- Severity escalation
- Deduplication of correlated issues

#### 2. Temporal Analysis Tests (15 tests)
**File**: `crates/annad/tests/regression_proactive_trends.rs`

Tests:
- Flapping detection (3+ occurrences in 1 hour)
- Escalation detection (warning → critical)
- Degradation detection (slow decline)
- Improvement detection (getting better)
- Recurring pattern detection
- Recovery tracking (issue resolved)
- Trend projection accuracy
- Time window boundaries
- Trend deduplication
- Multiple concurrent trends

#### 3. Integration Tests (15 tests)
**File**: `crates/annactl/tests/regression_proactive_integration.rs`

Tests:
- HealthReport includes ProactiveAssessment
- Status command displays assessment
- TUI welcome shows top issues
- TUI exit shows recoveries
- NL query "what's wrong?" routes correctly
- Health score calculation
- Severity prioritization
- Remediation command generation
- Canonical format compliance
- No Rust types in output
- Backward compatibility (existing tests still pass)

#### 4. Output Formatting Tests (10 tests)
**File**: `crates/annactl/tests/regression_proactive_output.rs`

Tests:
- CLI status output format
- TUI welcome format
- TUI exit summary format
- Correlated issue display
- Trend display
- Recovery display
- Health score display
- Empty assessment (healthy system)
- Multiple issues sorted by priority
- Unicode safety in output

---

## Implementation Phases

### Beta.270: Core Proactive Engine
**Scope**:
- Implement `proactive_engine.rs` module in `annad`
- Implement all data structures (ProactiveAssessment, CorrelatedIssue, etc.)
- Implement correlation rules for network, disk, services
- Implement temporal analysis (trends, recovery tracking)
- Unit tests for correlation logic (20 tests)

**Files**:
- `crates/annad/src/intel/proactive_engine.rs` (NEW)
- `crates/annad/tests/regression_proactive_correlation.rs` (NEW)
- `crates/annad/tests/regression_proactive_trends.rs` (NEW)
- `docs/BETA_270_NOTES.md` (NEW)
- `docs/ROOT_CAUSE_CORRELATION_MATRIX.md` (NEW)

### Beta.271: Health Pipeline Integration
**Scope**:
- Add `proactive_assessment` to `HealthReport`
- Call proactive engine in `check_health()`
- Surface assessment in status command
- Surface assessment in TUI welcome/exit
- Integration tests (15 tests)

**Files**:
- `crates/annad/src/steward/health.rs` (MODIFIED)
- `crates/annad/src/steward/types.rs` (MODIFIED)
- `crates/annactl/src/status_command.rs` (MODIFIED)
- `crates/annactl/src/tui/flow.rs` (MODIFIED)
- `crates/annactl/tests/regression_proactive_integration.rs` (NEW)
- `docs/BETA_271_NOTES.md` (NEW)

### Beta.272: Remediation Integration & Polish
**Scope**:
- Route correlated issues to remediation composers
- Integrate with Beta.269 remediation framework
- NL query routing for proactive questions
- Health score calculation
- Output formatting tests (10 tests)
- Final documentation

**Files**:
- `crates/annactl/src/sysadmin_answers.rs` (MODIFIED)
- `crates/annactl/src/unified_query_handler.rs` (MODIFIED)
- `crates/annactl/tests/regression_proactive_output.rs` (NEW)
- `docs/BETA_272_NOTES.md` (NEW)
- `CHANGELOG.md` (UPDATED)

---

## Success Criteria

### Functional Requirements
- ✅ Detects issues proactively (without user asking)
- ✅ Correlates multiple signals into root causes
- ✅ Prioritizes by severity and impact
- ✅ Tracks trends (flapping, escalation, recovery)
- ✅ Surfaces actionable remediation
- ✅ 100% deterministic (zero LLM)

### Quality Requirements
- ✅ 60+ regression tests passing
- ✅ No false positives on healthy systems
- ✅ Canonical output formatting
- ✅ No Rust types leaked to output
- ✅ Backward compatibility maintained
- ✅ All existing tests still pass

### Performance Requirements
- ✅ Assessment generation < 100ms
- ✅ No impact on daemon health check cycle
- ✅ Minimal memory overhead (<10MB)

---

## Open Questions for Approval

1. **Correlation confidence thresholds**: Should we surface low-confidence correlations (50-70%) or only high-confidence (>70%)?

2. **Trend window tuning**: Are 1h/24h/7d the right windows, or should we add 15min for ultra-fast detection?

3. **Health score weights**: Current formula uses 20/10/5/3 for critical/warning/trend/flapping. Should these be tunable?

4. **Recovery notification TTL**: How long should we show recovery notices? (Current: 24 hours)

5. **Maximum displayed issues**: Should we cap display at top 10 issues, or show all?

6. **Historian integration depth**: Should we correlate with historian boot events, package updates, and configuration changes from the start, or add in later phase?

---

## Next Steps

**Awaiting approval on**:
1. Overall architecture design
2. Data structure definitions
3. Correlation rule matrix
4. Integration points
5. Testing strategy
6. Implementation phases

**Once approved, will proceed with**:
1. Beta.270 implementation (core engine + tests)
2. Beta.271 implementation (health integration + tests)
3. Beta.272 implementation (remediation + polish + tests)
4. Full documentation suite
5. Version bump and release

---

**End of Design Document**
