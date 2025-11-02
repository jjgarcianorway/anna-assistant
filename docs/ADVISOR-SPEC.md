# Advisor Specification

**Anna v0.13.2 "Orion II - Autonomy Prep"**

## Overview

The Advisor module is Anna's centralized recommendation engine - the "brain" that transforms raw radar data into actionable insights. It implements a rule-based system that evaluates system health and generates prioritized recommendations.

## Goals

1. **Centralization**: Single source of truth for all recommendation logic
2. **Extensibility**: Easy to add new rules without code changes
3. **Prioritization**: Critical issues surface first
4. **Actionability**: Every recommendation includes concrete steps
5. **Context-Aware**: Recommendations based on actual metric thresholds

## Architecture

### Components

```
┌─────────────────────────────────────────┐
│           Advisor Engine                │
├─────────────────────────────────────────┤
│                                         │
│  ┌──────────────┐  ┌──────────────┐   │
│  │ Built-in     │  │ Custom Rules │   │
│  │ Rules        │  │ (User Config)│   │
│  └──────┬───────┘  └──────┬───────┘   │
│         └────────┬─────────┘           │
│                  │                     │
│         ┌────────▼────────┐            │
│         │  Rule Engine    │            │
│         │  - Evaluate     │            │
│         │  - Sort         │            │
│         │  - Filter       │            │
│         └────────┬────────┘            │
│                  │                     │
│         ┌────────▼────────┐            │
│         │ Recommendations │            │
│         └─────────────────┘            │
└─────────────────────────────────────────┘
```

### Data Structures

#### Rule

```rust
pub struct Rule {
    pub id: String,              // Unique identifier (e.g., "critical_security_updates")
    pub category: String,         // "hardware", "software", "user"
    pub priority: String,         // "critical", "high", "medium", "low"
    pub title: String,           // User-facing title
    pub condition: Condition,     // When this rule triggers
    pub message: String,         // Explanation of the issue
    pub action: String,          // Concrete fix steps
    pub impact: String,          // Expected improvement (e.g., "+3 Software")
    pub emoji: String,           // Visual indicator
}
```

#### Condition

```rust
pub enum Condition {
    Threshold {
        metric: String,     // e.g., "software.os_freshness"
        operator: String,   // "<=", ">=", "==", "!=", "<", ">"
        value: u8,          // Threshold (0-10 scale)
    },
    And {
        conditions: Vec<Condition>,
    },
    Or {
        conditions: Vec<Condition>,
    },
}
```

**Examples**:

```rust
// Simple threshold
Condition::Threshold {
    metric: "hardware.disk_free".to_string(),
    operator: "<=".to_string(),
    value: 3,
}

// Compound (AND)
Condition::And {
    conditions: vec![
        Condition::Threshold {
            metric: "software.os_freshness".to_string(),
            operator: "<=".to_string(),
            value: 5,
        },
        Condition::Threshold {
            metric: "software.security".to_string(),
            operator: "<=".to_string(),
            value: 4,
        },
    ],
}
```

#### Recommendation

```rust
pub struct Recommendation {
    pub priority: String,
    pub category: String,
    pub title: String,
    pub reason: String,     // Filled from rule.message
    pub action: String,     // Concrete steps
    pub emoji: String,
    pub impact: String,
}
```

### Advisor API

```rust
impl Advisor {
    /// Create new advisor with default + custom rules
    pub fn new() -> Result<Self>;

    /// Evaluate all rules against radar snapshot
    pub fn evaluate(&self, snapshot: &RadarSnapshot) -> Vec<Recommendation>;

    /// Get top N recommendations (sorted by priority)
    pub fn top_recommendations(&self, snapshot: &RadarSnapshot, n: usize) -> Vec<Recommendation>;
}
```

## Built-in Rules

### Critical Priority (Immediate Action Required)

| ID | Title | Condition | Action |
|----|-------|-----------|--------|
| `critical_security_updates` | Security updates pending | `software.os_freshness <= 5` | `sudo pacman -Syu` or `sudo apt update && upgrade` |
| `critical_disk_space` | Critical disk space low | `hardware.disk_free <= 3` | `sudo pacman -Sc` or `sudo apt clean && autoremove` |
| `critical_security_hardening` | Security hardening needed | `software.security <= 4` | Enable firewall, configure SELinux/AppArmor |

### High Priority (Address Soon)

| ID | Title | Condition | Action |
|----|-------|-----------|--------|
| `high_thermal_issues` | High CPU temperatures | `hardware.cpu_thermal <= 5` | Check cooling, clean filters, verify fans |
| `high_memory_pressure` | High memory pressure | `hardware.memory <= 5` | Add swap, enable zram, close apps |
| `high_package_updates` | Package updates available | `software.packages <= 6` | Review and apply updates |
| `high_failing_services` | Critical services failing | `software.services <= 6` | Check `systemctl --failed`, review logs |
| `high_no_backups` | No backup system detected | `software.backups <= 5` | Set up timeshift, restic, or borg |

### Medium Priority (Plan to Address)

| ID | Title | Condition | Action |
|----|-------|-----------|--------|
| `medium_disk_space_warning` | Disk space running low | `hardware.disk_free <= 6` | Clean cache, remove old logs |
| `medium_basic_fs` | Basic filesystem detected | `hardware.fs_features <= 6` | Research btrfs/zfs migration |
| `medium_log_noise` | Excessive log errors | `software.log_noise <= 6` | Review `journalctl -p err -b` |
| `medium_user_irregularity` | Irregular usage patterns | `user.regularity <= 6` | Establish maintenance schedule |
| `medium_workspace_clutter` | Workspace needs organization | `user.workspace <= 6` | Organize Downloads, Desktop, tmp |

### Low Priority (Optimization Opportunities)

| ID | Title | Condition | Action |
|----|-------|-----------|--------|
| `low_gpu_optimization` | GPU optimization opportunity | `hardware.gpu <= 7` | Update drivers, enable hw acceleration |
| `low_boot_optimization` | Boot time optimization | `hardware.boot <= 7` | `systemd-analyze blame`, disable services |
| `low_containers_optimization` | Container optimization | `software.containers <= 7` | Clean images: `docker system prune` |
| `low_power_management` | Power management tuning | `user.power <= 7` | Install powertop/tlp, review profiles |

## Priority Ranking

Rules are sorted by priority before presentation:

```rust
critical (0) > high (1) > medium (2) > low (3)
```

Within same priority, rules appear in evaluation order (typically by severity of score).

## Custom Rules

### Configuration Directory

Custom rules can be added to: `~/.config/anna/advisor.d/`

Supported formats:
- `*.json` - JSON array of rules
- `*.yaml` - YAML array of rules (future)

### Example Custom Rule

**File**: `~/.config/anna/advisor.d/my-rules.json`

```json
[
  {
    "id": "custom_docker_running",
    "category": "software",
    "priority": "low",
    "title": "Docker service not optimal",
    "condition": {
      "type": "threshold",
      "metric": "software.containers",
      "operator": "<=",
      "value": 6
    },
    "message": "Docker containers could be better organized or some may be stale",
    "action": "Run 'docker ps -a' to list containers, 'docker system prune' to clean up",
    "impact": "+1 Software",
    "emoji": "🐳"
  }
]
```

### Rule Validation

Rules are validated on load:
- ✅ All required fields present
- ✅ Valid condition syntax
- ✅ Valid metric names
- ✅ Valid operators
- ❌ Invalid rules skipped with warning

## Metric Names

### Hardware Metrics

- `hardware.overall`
- `hardware.cpu_throughput`
- `hardware.cpu_thermal`
- `hardware.memory`
- `hardware.disk_health`
- `hardware.disk_free`
- `hardware.fs_features`
- `hardware.gpu`
- `hardware.network`
- `hardware.boot`

### Software Metrics

- `software.overall`
- `software.os_freshness`
- `software.kernel`
- `software.packages`
- `software.services`
- `software.security`
- `software.containers`
- `software.fs_integrity`
- `software.backups`
- `software.log_noise`

### User Metrics

- `user.overall`
- `user.regularity`
- `user.workspace`
- `user.updates`
- `user.backups`
- `user.risk`
- `user.connectivity`
- `user.power`
- `user.warnings`

## Integration

### Report Command

The report command uses the advisor to generate recommendations:

```rust
fn generate_recommendations(snapshot: &RadarSnapshot) -> Vec<Recommendation> {
    match Advisor::new() {
        Ok(advisor) => advisor.top_recommendations(snapshot, 5),
        Err(_) => Vec::new(),  // Graceful degradation
    }
}
```

**Top 5 Limit**: Reports show the 5 most critical recommendations by default.

### Standalone Usage

The advisor can be used independently:

```rust
use anna::advisor::Advisor;

let advisor = Advisor::new()?;
let snapshot = fetch_radar_snapshot().await?;
let recommendations = advisor.evaluate(&snapshot);

for rec in recommendations {
    println!("{}: {}", rec.priority, rec.title);
}
```

## Performance

### Evaluation Complexity

- **Per Rule**: O(1) - Simple metric lookup and comparison
- **Total**: O(R) where R = number of rules
- **Current**: ~18 built-in rules, <1ms evaluation time

### Rule Loading

- **Built-in**: Hardcoded, no I/O
- **Custom**: File read on `Advisor::new()`
- **Typical**: <10ms including disk I/O

## Testing

### Unit Tests (8 implemented)

1. `test_threshold_condition` - Simple metric comparison
2. `test_and_condition` - Compound AND logic
3. `test_or_condition` - Compound OR logic
4. `test_evaluate_returns_sorted` - Priority ordering
5. `test_top_recommendations_truncates` - Limit enforcement
6. `test_metric_parsing` - Metric name resolution
7. `test_comparison_operators` - All 6 operators
8. `test_priority_ordering` - Sort stability

### Integration Tests (Pending)

1. Custom rule loading
2. Invalid rule handling
3. YAML config parsing
4. Concurrent advisor instances
5. Large rule sets (100+ rules)

## Error Handling

### Invalid Metrics

```rust
fn get_metric_value(&self, metric: &str, snapshot: &RadarSnapshot) -> u8 {
    // ...
    _ => 10,  // Unknown metrics default to "perfect" to avoid false alarms
}
```

**Philosophy**: Conservative - don't warn about metrics we can't measure.

### Rule Load Failures

```rust
if let Ok(custom_rules) = Self::load_custom_rules() {
    rules.extend(custom_rules);
}
// Continue with built-in rules if custom rules fail
```

**Graceful Degradation**: Advisor works even if custom rules fail to load.

### Invalid Conditions

```rust
match serde_json::from_str::<Rule>(&content) {
    Ok(rules) => return Ok(rules),
    Err(e) => {
        eprintln!("Warning: Failed to load rules from {:?}: {}", path, e);
    }
}
```

Individual rule failures don't prevent loading other rules.

## Example Outputs

### Healthy System (Score 9/10)

```
No critical or high-priority recommendations
```

Advisor returns empty list when all metrics >= 7.

### Moderate Issues (Score 6/10)

```
━━━ Top Recommendations ━━━

1. 🔒 [CRITICAL] Security updates pending
   → Apply updates: sudo pacman -Syu
   Impact: +3 Software

2. ⚠️ [HIGH] High memory pressure
   → Add swap, enable zram, close apps
   Impact: +2 Hardware

3. 📦 [MEDIUM] Disk space running low
   → Clean cache: sudo pacman -Sc
   Impact: +2 Hardware
```

### Critical System (Score 3/10)

```
━━━ Top Recommendations ━━━

1. 💾 [CRITICAL] Critical disk space low
   → Immediate cleanup required
   Impact: +5 Hardware

2. 🔒 [CRITICAL] Security updates pending
   → Apply updates immediately
   Impact: +3 Software

3. 🛡️ [CRITICAL] Security hardening needed
   → Enable firewall, configure SELinux
   Impact: +4 Software

4. ⚠️ [HIGH] High CPU temperatures
   → Check cooling system urgently
   Impact: +3 Hardware

5. 🔧 [HIGH] Critical services failing
   → Review systemctl --failed
   Impact: +3 Software
```

## Future Enhancements

### v0.14.x

1. **Learning System**
   - Track which recommendations user acts on
   - Adjust priorities based on user patterns
   - Suppress repeatedly ignored recommendations

2. **Context-Aware Rules**
   - Time-of-day considerations (backups at night)
   - Distro-specific recommendations
   - Hardware-specific advice (laptop vs server)

3. **Impact Tracking**
   - Measure score change after recommendation
   - Build efficacy database
   - Prioritize high-impact actions

4. **Automated Actions**
   - Safe operations (cache cleaning)
   - Dry-run mode with preview
   - Rollback capability

### Under Consideration

- Natural language rule definition
- ML-based anomaly detection
- Community rule repository
- Recommendation dependencies (do X before Y)

## Security Considerations

### Custom Rule Risks

**Threat**: Malicious custom rules could suggest harmful commands.

**Mitigation**:
- Custom rules are user-installed (local threat model)
- No automatic rule downloads
- All actions require explicit user execution
- No elevated privileges in advisor itself

### Command Injection

**Threat**: Rule `action` fields contain shell commands.

**Mitigation**:
- Actions are display-only (not auto-executed)
- User manually copies/runs commands
- Future auto-execution will use safe command builders

## API Stability

**Stability Level**: Beta (v0.13.x)

- Rule format may change in minor versions
- Backward compatibility for built-in rules
- Custom rules may need updates

**Expected Stable**: v0.14.0

## References

- Implementation: `src/annactl/src/advisor.rs`
- Integration: `src/annactl/src/report_cmd.rs`
- Tests: `src/annactl/src/advisor.rs` (tests module)
- Rule Examples: `examples/advisor-rules/` (future)

---

**Document Version**: 1.0
**Last Updated**: 2025-11-02
**Author**: Anna Development Team
