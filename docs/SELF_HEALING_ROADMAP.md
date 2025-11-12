# Self-Healing Capabilities Roadmap

**Phase 3.6+: From Reactive to Proactive** - Anna learns to fix herself

## Vision

Transform Anna from a reactive system maintenance tool into a proactive, self-healing assistant that anticipates problems, takes preventive action, and repairs issues automatically while keeping the user informed and in control.

## Design Principles

1. **Safety First**: Never perform destructive actions without user approval
2. **Transparency**: Always explain what's wrong and what will be fixed
3. **Learning**: Get smarter over time by tracking what works
4. **Reversible**: Every automatic action can be undone
5. **Gradual Autonomy**: Start with suggestions, progress to automatic fixes
6. **User Control**: Users can configure healing policies

## Healing Maturity Levels

### Level 0: Detection Only (Current - v3.0.0-alpha.3)
**Status**: ✅ Implemented

- Detect system issues via probes
- Report status to user
- User manually runs commands

**Commands**:
- `annactl health` - Show what's wrong
- `annactl doctor` - Diagnose and suggest fixes
- `annactl repair` - Execute fixes manually

### Level 1: Guided Repair (v3.0.0-beta.1)
**Status**: 📋 Planned

- Automatically suggest appropriate fixes
- Explain what each fix does
- Ask for user confirmation before action
- Track success/failure of repairs

**New Features**:
```bash
# Smart suggestions based on system state
$ annactl health
System Status: DEGRADED

Detected Issues:
  • 3 failed systemd services
  • 12 outdated packages
  • Low disk space warning

💡 Recommended Actions:
  1. annactl repair --service systemd-failed-units
     → Restart failed services (safe, reversible)
  2. annactl cleanup --cache
     → Free 2.3 GB of package cache (safe)
  3. annactl update
     → Update 12 packages (may require reboot)

Auto-fix available for items 1-2. Run:
  annactl auto-heal --items 1,2

For item 3, manual review recommended.
```

**Implementation**:
- Add `auto-heal` command with confirmation dialogs
- Track repair success rate in persistent context
- Learn which repairs users approve
- Generate safer suggestions over time

### Level 2: Supervised Healing (v3.0.0)
**Status**: 🎯 Target for stable release

- Automatically fix "safe" issues without asking
- Notify user after healing
- Require approval for "risky" operations
- Provide undo mechanism for last 10 actions

**Healing Policy**:
```rust
pub enum HealingPolicy {
    Manual,           // Always ask (Level 0)
    Guided,           // Suggest and confirm (Level 1)
    Supervised,       // Auto-fix safe issues, ask for risky (Level 2)
    Autonomous,       // Fix everything automatically (Level 3 - future)
}

pub enum ActionRisk {
    Safe,             // Restart service, clear cache
    Low,              // Update single package
    Medium,           // Update system, modify config
    High,             // Filesystem operations, major changes
    Critical,         // Partitioning, bootloader, etc.
}
```

**Example Workflow**:
```
15:32 - Anna detected 2 failed services: sshd, nginx
15:32 - [AUTO-HEAL] Restarted sshd ✓
15:32 - [AUTO-HEAL] Restarted nginx ✓
15:32 - System state: DEGRADED → HEALTHY

💡 Anna automatically healed 2 issues.
   View details: annactl history --last 2
   Undo if needed: annactl rollback --last 2
```

**Safety Mechanisms**:
- Dry-run mode for all auto-actions
- Snapshot before filesystem changes
- Service restart only (no service stops unless necessary)
- Package updates in safe mode (skip kernel unless approved)

### Level 3: Autonomous Healing (v4.0.0+)
**Status**: 🚀 Future vision

- Predict issues before they become critical
- Schedule maintenance during idle times
- Coordinate with user's work patterns
- Self-optimize monitoring and healing policies

**Capabilities**:
- **Predictive Maintenance**: "RAM usage trending up, preemptively cleared caches"
- **Time-Aware**: "Scheduled system update for 2 AM when system is idle"
- **Pattern Learning**: "User typically updates on Mondays at 6 PM, suggesting now"
- **Self-Tuning**: "Increased probe frequency for disk checks (detected pattern of disk issues)"

## Implementation Phases

### Phase 3.6: Healing Infrastructure (v3.0.0-alpha.4)

**Goals**: Build foundation for self-healing

**Tasks**:
1. Create healing policy configuration
2. Implement action risk assessment
3. Add rollback/undo mechanism
4. Track healing success metrics

**Files to Create**:
```
crates/anna_common/src/healing/
├── mod.rs              # Public API
├── policy.rs           # Healing policy management
├── risk.rs             # Risk assessment
├── actions.rs          # Healable actions catalog
├── rollback.rs         # Undo mechanism
└── metrics.rs          # Success tracking
```

**Config File** (`/etc/anna/healing.toml`):
```toml
[healing]
# Healing policy: manual, guided, supervised, autonomous
policy = "guided"

# Maximum actions per healing session
max_actions_per_session = 5

# Require explicit approval for these risk levels
require_approval = ["medium", "high", "critical"]

# Auto-heal these action categories
auto_heal = [
    "service_restart",      # Restart failed services
    "cache_cleanup",        # Clear package cache
    "log_rotation",         # Rotate old logs
    "orphan_removal",       # Remove orphaned packages
]

# Never auto-heal these categories (always ask)
never_auto_heal = [
    "filesystem_modify",    # Disk operations
    "bootloader_change",    # Bootloader updates
    "kernel_update",        # Kernel changes
    "partition_operations", # Disk partitioning
]

[healing.schedule]
# Preferred maintenance window
preferred_hours = [22, 23, 0, 1, 2, 3]  # 10 PM - 4 AM
avoid_hours = [9, 10, 11, 14, 15, 16]   # Work hours

# Maintenance days (0 = Sunday, 6 = Saturday)
preferred_days = [0, 6]  # Weekends
avoid_days = []

[healing.rollback]
# Number of actions to keep for rollback
history_size = 10

# Auto-rollback on failure
auto_rollback_on_failure = true

# Rollback timeout (minutes)
rollback_timeout = 60
```

### Phase 3.7: Safe Auto-Healing (v3.0.0-beta.1)

**Goals**: Implement Level 1 (Guided Repair)

**New Commands**:
```bash
# Configure healing policy
annactl healing config --policy guided

# Enable/disable auto-healing
annactl healing enable
annactl healing disable

# List healable issues
annactl healing list

# Auto-heal with confirmation
annactl auto-heal

# View healing history
annactl healing history

# Undo last healing action
annactl healing undo
annactl healing undo --action-id abc123

# Show healing statistics
annactl healing stats
```

**Example Implementation**:
```rust
pub async fn execute_auto_heal(
    dry_run: bool,
    items: Option<Vec<usize>>,
) -> Result<()> {
    let policy = load_healing_policy()?;

    if policy.mode == HealingPolicy::Manual {
        println!("Auto-healing is disabled. Enable with: annactl healing config --policy guided");
        return Ok(());
    }

    // Get current issues
    let issues = detect_healable_issues().await?;

    if issues.is_empty() {
        println!("✓ No issues detected. System is healthy!");
        return Ok(());
    }

    // Filter by user selection if provided
    let selected_issues = if let Some(item_indices) = items {
        filter_issues_by_indices(&issues, item_indices)
    } else {
        issues.clone()
    };

    // Risk assessment
    let (safe_issues, risky_issues) = assess_risk(&selected_issues);

    // Display plan
    println!("🔍 Healing Plan:\n");
    for (i, issue) in safe_issues.iter().enumerate() {
        println!("  {}. [SAFE] {} - {}", i + 1, issue.title, issue.description);
        println!("     Action: {}", issue.heal_action);
    }

    if !risky_issues.is_empty() {
        println!("\n⚠️  Risky Actions (require approval):\n");
        for (i, issue) in risky_issues.iter().enumerate() {
            println!("  {}. [{}] {} - {}",
                i + safe_issues.len() + 1,
                issue.risk_level,
                issue.title,
                issue.description
            );
            println!("     Action: {}", issue.heal_action);
        }
    }

    // Confirmation
    if !dry_run {
        println!("\n Proceed with {} safe actions?", safe_issues.len());
        if !risky_issues.is_empty() {
            println!("   ({} risky actions will be skipped)", risky_issues.len());
        }

        eprint!("Continue? [y/N]: ");
        std::io::stderr().flush()?;

        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;

        if !input.trim().eq_ignore_ascii_case("y") {
            println!("Healing cancelled.");
            return Ok(());
        }
    }

    // Execute healing
    let results = execute_healing_actions(safe_issues, dry_run).await?;

    // Report results
    report_healing_results(&results);

    // Store in persistent context
    record_healing_session(&results).await?;

    Ok(())
}
```

### Phase 3.8: Supervised Autonomy (v3.0.0)

**Goals**: Implement Level 2 (Supervised Healing)

**Features**:
1. **Background Healing Service**
   - Runs as part of annad daemon
   - Checks for issues every N minutes
   - Auto-heals safe issues
   - Notifies user via system notifications

2. **Desktop Notifications**
   ```
   [Anna Assistant]
   🔧 Auto-healed 2 issues:
      • Restarted failed service: nginx
      • Cleared 1.2 GB of package cache

   System state: DEGRADED → HEALTHY

   [View Details] [Undo]
   ```

3. **Healing Dashboard** (in Grafana)
   - Healing success rate over time
   - Most common issues
   - Time saved by auto-healing
   - Rollback frequency

4. **Smart Scheduling**
   - Learn user's activity patterns
   - Schedule maintenance during idle hours
   - Avoid healing during peak usage

**Implementation**:
```rust
// Add to annad daemon main loop
tokio::spawn(async move {
    let healing_interval = tokio::time::Duration::from_secs(300); // 5 minutes

    loop {
        tokio::time::sleep(healing_interval).await;

        // Check healing policy
        let policy = match healing::load_policy().await {
            Ok(p) => p,
            Err(e) => {
                warn!("Failed to load healing policy: {}", e);
                continue;
            }
        };

        if policy.mode != HealingPolicy::Supervised {
            continue; // Skip if not in supervised mode
        }

        // Detect issues
        let issues = match healing::detect_issues().await {
            Ok(i) => i,
            Err(e) => {
                warn!("Failed to detect issues: {}", e);
                continue;
            }
        };

        if issues.is_empty() {
            continue;
        }

        // Filter for auto-healable issues
        let auto_healable = healing::filter_auto_healable(&issues, &policy);

        if auto_healable.is_empty() {
            info!("Found {} issues, but none are auto-healable", issues.len());
            continue;
        }

        // Execute healing
        info!("Auto-healing {} issues", auto_healable.len());
        let results = healing::execute_healing_actions(auto_healable, false).await?;

        // Notify user
        if results.success_count > 0 {
            notify_user_desktop(format!(
                "🔧 Auto-healed {} issues. System state: {}",
                results.success_count,
                results.new_state
            )).await?;
        }

        // Record in persistent context
        context::record_healing_session(&results).await?;
    }
});
```

### Phase 3.9: Learning & Optimization (v3.1.0)

**Goals**: Make healing smarter over time

**Features**:
1. **Pattern Recognition**
   - Detect recurring issues
   - Learn which fixes work best
   - Recommend permanent solutions

2. **Success Tracking**
   ```sql
   -- Query from persistent context
   SELECT
       issue_type,
       COUNT(*) as occurrences,
       AVG(CASE WHEN outcome = 'success' THEN 1 ELSE 0 END) as success_rate,
       AVG(duration_ms) as avg_duration
   FROM action_history
   WHERE action_type = 'auto_heal'
   GROUP BY issue_type
   ORDER BY occurrences DESC;
   ```

3. **Intelligent Recommendations**
   ```bash
   $ annactl healing insights

   📊 Healing Insights (Last 30 Days)

   Most Common Issues:
     1. Service crashes: nginx (12 times)
        → Consider: Increase memory limits for nginx
        → Tutorial: annactl learn nginx-tuning

     2. Disk space warnings (8 times)
        → Consider: Enable automatic cache cleanup
        → Run: annactl healing config --auto-cleanup

     3. Failed package updates (5 times)
        → Root cause: Insufficient disk space
        → Fix: Increase /var partition or enable cleanup

   Success Rate: 94% (47/50 auto-heal attempts)
   Time Saved: ~2.5 hours of manual intervention
   ```

4. **Proactive Suggestions**
   - Detect trends before they become problems
   - Suggest preventive actions
   - Schedule maintenance proactively

### Phase 4.0: Predictive Healing (v4.0.0+)

**Goals**: Prevent issues before they happen

**Advanced Features**:
1. **Trend Analysis**
   - Memory usage trending up → preemptively free cache
   - Disk filling up → schedule cleanup before critical
   - Service restarts increasing → investigate root cause

2. **Anomaly Detection**
   - Detect unusual patterns
   - Alert before issues manifest
   - Suggest root cause analysis

3. **Capacity Planning**
   - Predict when resources will be exhausted
   - Recommend hardware upgrades
   - Optimize resource allocation

4. **Self-Optimization**
   - Tune monitoring frequency based on system health
   - Adjust healing aggressiveness based on success rate
   - Learn user preferences automatically

## Safety Guarantees

### Pre-Flight Checks

Before any healing action:
1. ✅ Verify system state is as expected
2. ✅ Check sufficient resources available
3. ✅ Confirm no conflicting operations running
4. ✅ Create rollback snapshot if needed
5. ✅ Validate healing action is appropriate

### Rollback Mechanism

```rust
pub struct RollbackSnapshot {
    action_id: Uuid,
    timestamp: DateTime<Utc>,
    action_type: String,
    pre_state: SystemSnapshot,
    post_state: SystemSnapshot,
    rollback_steps: Vec<RollbackStep>,
}

pub enum RollbackStep {
    RestartService { name: String, was_active: bool },
    RestoreFile { path: PathBuf, backup_path: PathBuf },
    ReinstallPackage { name: String, version: String },
    RevertConfig { path: PathBuf, backup_content: String },
}

pub async fn rollback_action(action_id: Uuid) -> Result<()> {
    let snapshot = load_snapshot(action_id)?;

    for step in snapshot.rollback_steps.iter().rev() {
        match step {
            RollbackStep::RestartService { name, was_active } => {
                if *was_active {
                    restart_service(name).await?;
                } else {
                    stop_service(name).await?;
                }
            }
            RollbackStep::RestoreFile { path, backup_path } => {
                restore_file_from_backup(path, backup_path).await?;
            }
            // ... other steps
        }
    }

    Ok(())
}
```

### Circuit Breaker

Prevent runaway healing:
```rust
pub struct HealingCircuitBreaker {
    max_failures: usize,
    failure_window: Duration,
    recent_failures: Vec<DateTime<Utc>>,
}

impl HealingCircuitBreaker {
    pub fn should_allow_healing(&mut self) -> bool {
        // Remove old failures outside window
        let cutoff = Utc::now() - self.failure_window;
        self.recent_failures.retain(|t| *t > cutoff);

        // Check if too many recent failures
        if self.recent_failures.len() >= self.max_failures {
            warn!(
                "Healing circuit breaker triggered: {} failures in last {:?}",
                self.recent_failures.len(),
                self.failure_window
            );
            return false;
        }

        true
    }

    pub fn record_failure(&mut self) {
        self.recent_failures.push(Utc::now());
    }
}
```

## User Control

### Healing Configuration UI

```bash
# View current policy
$ annactl healing config --show
Current Healing Policy: supervised
Auto-heal enabled: yes
Max actions per session: 5
Circuit breaker: 3 failures per 1 hour

Auto-heal categories:
  ✓ service_restart
  ✓ cache_cleanup
  ✓ log_rotation
  ✓ orphan_removal

Never auto-heal:
  ✗ filesystem_modify
  ✗ bootloader_change
  ✗ kernel_update

# Disable auto-healing
$ annactl healing disable
Auto-healing disabled. Anna will only suggest fixes.

# Enable specific category
$ annactl healing config --enable service_restart

# Disable specific category
$ annactl healing config --disable cache_cleanup

# Set healing window
$ annactl healing config --hours 22-4 --days weekend
```

### Healing Consent

For sensitive operations, always require explicit consent:
```rust
pub enum ConsentLevel {
    None,           // No consent needed (read-only)
    Implied,        // User enabled auto-heal (safe actions)
    Explicit,       // Must confirm each time (risky actions)
    PerSession,     // Confirm at session start, not per action
}
```

## Metrics & Observability

### New Prometheus Metrics

```rust
// Healing metrics
pub healing_attempts_total: IntCounterVec,        // by issue_type, outcome
pub healing_duration_seconds: HistogramVec,       // by issue_type
pub healing_success_rate: Gauge,                  // rolling 24h
pub healable_issues_current: IntGauge,            // current count
pub auto_heal_enabled: IntGauge,                  // 0=disabled, 1=enabled
pub rollback_operations_total: IntCounterVec,     // by reason
pub circuit_breaker_trips_total: IntCounter,     // safety trigger count
```

### Grafana Dashboard

New "Anna Healing" dashboard with:
- Healing success rate over time
- Most common issues
- Time to heal (duration histogram)
- Rollback frequency
- Circuit breaker trips
- User approval rate

## Testing Strategy

### Unit Tests
```rust
#[tokio::test]
async fn test_healing_policy_enforcement() {
    let policy = HealingPolicy::Supervised;
    let action = HealingAction {
        risk_level: ActionRisk::Medium,
        // ...
    };

    assert!(!should_auto_heal(&action, &policy));
}

#[tokio::test]
async fn test_rollback_service_restart() {
    let snapshot = create_service_snapshot("nginx").await?;
    stop_service("nginx").await?;
    rollback_action(snapshot.action_id).await?;

    assert!(is_service_running("nginx").await?);
}
```

### Integration Tests
```bash
# Test healing workflow
cargo test --test healing_integration -- --test-threads=1

# Test rollback mechanism
cargo test --test rollback_integration -- --test-threads=1
```

### Manual Testing
```bash
# Test guided healing
sudo annactl healing config --policy guided
sudo systemctl stop nginx
annactl auto-heal

# Test supervised healing
sudo annactl healing config --policy supervised
sudo systemctl stop nginx
sleep 300  # Wait for auto-heal
annactl status

# Test rollback
annactl healing history
annactl healing undo --last
```

## Documentation

### User Guide
- `docs/user-guide/HEALING.md` - Complete healing guide
- `docs/user-guide/HEALING_POLICIES.md` - Policy configuration
- `docs/user-guide/ROLLBACK.md` - Undo guide

### Admin Guide
- `docs/admin-guide/HEALING_ARCHITECTURE.md` - Technical overview
- `docs/admin-guide/HEALING_TUNING.md` - Performance tuning
- `docs/admin-guide/HEALING_SECURITY.md` - Security considerations

## Migration Path

### v3.0.0-alpha.3 → v3.0.0-alpha.4
- Add healing configuration file
- No behavior change (healing disabled by default)
- Users can opt-in to guided healing

### v3.0.0-alpha.4 → v3.0.0-beta.1
- Guided healing becomes default
- Users can configure auto-heal categories
- Desktop notifications available

### v3.0.0-beta.1 → v3.0.0
- Supervised healing available
- Requires explicit opt-in
- Background healing service

### v3.0.0 → v4.0.0
- Predictive healing available
- ML-based anomaly detection
- Advanced pattern recognition

## Success Criteria

### Phase 3.6 (Infrastructure)
- ✅ Healing policy configuration implemented
- ✅ Risk assessment framework working
- ✅ Rollback mechanism tested
- ✅ Metrics collection functional

### Phase 3.7 (Guided Repair)
- ✅ `annactl auto-heal` command working
- ✅ User approval flow intuitive
- ✅ Success rate tracking accurate
- ✅ Rollback successful in all test cases

### Phase 3.8 (Supervised)
- ✅ Background healing service stable
- ✅ Desktop notifications working
- ✅ Zero unintended system changes
- ✅ Circuit breaker prevents runaway actions
- ✅ User satisfaction >90%

### Phase 4.0 (Predictive)
- ✅ Trend detection accuracy >85%
- ✅ False positive rate <5%
- ✅ Issue prevention >50% of cases
- ✅ User trust maintained

---

**Status**: Phase 3.6 - Roadmap complete
**Next**: Phase 3.6.1 - Implement healing policy configuration
**Author**: Anna Self-Healing Team
**License**: Custom (see LICENSE file)

Citation: [chaos-engineering:netflix], [self-healing:kubernetes-operators], [undo:git-revert-patterns]
