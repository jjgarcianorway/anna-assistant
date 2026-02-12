# Anna Improvement Plan - Post Battle Test

Based on analysis of 100-question battle test results (47% success rate).

## Root Cause Identified

**Non-streaming mode lacks configuration handling.**

The battle test uses non-streaming `ralph_loop()`, which has this fatal flaw:

```rust
// ralph/commands.rs line 223
NextAction::None | NextAction::Config => Ok(Vec::new())
```

When a configuration request is detected, it returns **empty commands**, then tries to answer with no data. Result: instant failure (<300ms).

The streaming mode (`ralph_loop_streaming`) HAS proper config handling:
- Detects config keywords
- Investigates system state
- Generates ActionPlan via `handle_config_request_with_research()`
- Uses wiki research and dynamic planning

**18 of the 20 ADVANCED section failures were config requests** that hit this code path.

## Priority Fixes

### P0 - Critical (Fixes 35+ failures)

#### 1. Port Config Handling to Non-Streaming Mode

**Problem**: `ralph_loop()` doesn't handle NextAction::Config
**Impact**: 90% of ADVANCED questions fail instantly
**Fix**: Extract config handling from streaming into shared module

**Implementation**:
```rust
// ralph/mod.rs - modify ralph_loop()
match get_next_action(model, question, &state).await? {
    NextAction::Config => {
        // NEW: Handle config in non-streaming mode
        let system_state = investigate_system_state_sync(&mut state)?;
        return handle_config_sync(model, question, &system_state, &criteria)?;
    }
    NextAction::Commands(cmds) => { /* existing logic */ }
    NextAction::None => { /* existing logic */ }
}
```

**Files to modify**:
- `ralph/mod.rs` - Add config handling to ralph_loop()
- `ralph/config_handler.rs` (NEW) - Shared config logic for both modes
- `ralph/streaming.rs` - Refactor to use shared config_handler

**Estimated impact**: +18-20 successes (ADVANCED section)

#### 2. Add Configuration Templates

**Problem**: LLM generates unreliable multi-step configs
**Impact**: Even when config detection works, execution fails
**Fix**: Pre-built templates for common sysadmin tasks

**Template structure**:
```rust
pub struct ConfigTemplate {
    name: String,
    patterns: Vec<&'static str>,  // Matching patterns
    investigation: Vec<&'static str>,  // Commands to run first
    steps: Vec<TemplateStep>,  // Execution steps
    verification: Vec<&'static str>,  // Verify success
}

struct TemplateStep {
    condition: Option<String>,  // When to run this step
    command: String,
    explanation: String,
    reversible: bool,
}
```

**Priority templates** (cover 80% of failed configs):
1. SSH key setup
2. Firewall rules (UFW/iptables)
3. Systemd service creation
4. Package installation
5. Log rotation setup
6. Automatic updates/backups
7. User/group management
8. Network configuration (static IP, DNS)
9. Swap configuration
10. Boot optimization

**Files to create**:
- `templates/mod.rs` - Template matching engine
- `templates/ssh.rs` - SSH configuration
- `templates/firewall.rs` - Firewall setup
- `templates/systemd.rs` - Service management
- `templates/packages.rs` - Package operations
- `templates/backups.rs` - Backup configuration

**Estimated impact**: +12-15 successes (ADVANCED + EXPERT)

### P1 - High (Fixes 10+ failures)

#### 3. Optimize File Search Operations

**Problem**: Complex file ops timeout or fail after 50-60s
**Failed examples**:
- Q22: Find logs >100MB (65s FAIL)
- Q27: Find duplicate files (52s FAIL)
- Q34: Largest 10 files (56s FAIL)

**Fix**: Progressive search strategies + better tools

**Implementation**:
```rust
// Strategy 1: Use fd instead of find (10-100x faster)
fd --type f --size +100m  // Instead of: find / -type f -size +100M

// Strategy 2: Progressive narrowing
// Check common locations first: /var/log, /home, /tmp
// Only search entire filesystem if not found

// Strategy 3: Timeout with partial results
// Don't fail - return "Found X in /var/log, /home still searching..."

// Strategy 4: Parallel search
// Search multiple directories simultaneously
```

**Files to modify**:
- `ralph/commands.rs` - Add file search strategy hints
- `core_loop.rs` - Implement progressive timeout

**Estimated impact**: +4-5 successes (INTERMEDIATE)

#### 4. Enable Package Installation

**Problem**: Q21 "Install htop" failed (2365ms)
**Impact**: Any install request fails immediately
**Fix**: Detect install requests, execute with pacman/yay

**Implementation**:
```rust
// Detect install pattern
if question.contains("install") && !question.contains("installed") {
    let package = extract_package_name(question);

    // Check if already installed
    if is_installed(&package) {
        return Ok(format!("{} is already installed", package));
    }

    // Try pacman, fallback to yay
    install_package(&package)?;

    // Verify installation
    verify_installed(&package)?;
}
```

**Files to modify**:
- `ralph/commands.rs` - Add install detection before NextAction
- `package_manager.rs` (NEW) - Package operations

**Estimated impact**: +2-3 successes

### P2 - Medium (Improves speed)

#### 5. Cache Common Queries

**Problem**: Repeated hardware queries slow down iteration
**Fix**: Cache system info that doesn't change

**Cache targets**:
- Hardware: CPU, RAM, GPU, disk layout
- System: kernel version, hostname, timezone
- Network: interfaces (cache 60s)
- Packages: database state (cache 5m)

**Implementation**:
```rust
use std::sync::Arc;
use parking_lot::RwLock;

pub struct SystemCache {
    hardware: Arc<RwLock<Option<(HardwareInfo, Instant)>>>,
    packages: Arc<RwLock<Option<(PackageInfo, Instant)>>>,
}

impl SystemCache {
    fn get_hardware(&self) -> HardwareInfo {
        let cache = self.hardware.read();
        if let Some((info, ts)) = cache.as_ref() {
            if ts.elapsed() < Duration::from_secs(3600) {
                return info.clone();
            }
        }
        drop(cache);

        // Cache miss - fetch fresh data
        let info = fetch_hardware_info();
        *self.hardware.write() = Some((info.clone(), Instant::now()));
        info
    }
}
```

**Estimated impact**: 30-50% speed improvement on repeated queries

#### 6. Parallel Command Execution

**Problem**: Sequential commands waste time waiting
**Fix**: Run independent checks simultaneously

**Example**:
```bash
# Sequential (slow): ~5s total
df -h        # 1s
free -h      # 1s
uptime       # 1s
uname -r     # 1s
lscpu        # 1s

# Parallel (fast): ~1s total
```

**Implementation**:
```rust
use tokio::task::JoinSet;

async fn execute_parallel(commands: Vec<String>) -> Vec<Result<String>> {
    let mut set = JoinSet::new();

    for cmd in commands {
        set.spawn(async move {
            execute_command(&cmd)
        });
    }

    let mut results = Vec::new();
    while let Some(res) = set.join_next().await {
        results.push(res.unwrap());
    }
    results
}
```

**Files to modify**:
- `core_loop.rs` - Add parallel execution
- `ralph/commands.rs` - Detect independent commands

**Estimated impact**: 40% speed reduction on multi-command queries

## Implementation Order

1. **P0-1**: Port config handling (3-4 hours) → +20 successes
2. **P0-2**: Add config templates (6-8 hours) → +15 successes
3. **P1-3**: Optimize file search (2-3 hours) → +5 successes
4. **P1-4**: Enable package install (1-2 hours) → +3 successes
5. **P2-5**: Add caching (2-3 hours) → speed boost
6. **P2-6**: Parallel commands (2-3 hours) → speed boost

**Total estimated impact**: 47% → 90%+ success rate

## Testing Plan

After each fix:

```bash
# Run subset test (20 questions from failed category)
./tests/run_battle.py --subset ADVANCED

# Full test after all P0 fixes
./tests/run_battle.py

# Compare results
diff tests/battle_results.txt tests/battle_results_v2.txt
```

## Success Metrics

| Metric | Before | Target |
|--------|--------|--------|
| Overall | 47% | 90%+ |
| BASIC | 90% | 95%+ |
| INTERMEDIATE | 65% | 85%+ |
| ADVANCED | 10% | 85%+ |
| EXPERT | 50% | 75%+ |
| CHAOS | 20% | 50%+ |
| Median speed | 2.3s | <2.0s |
| p90 speed | 56s | <30s |

## Validation

After P0 fixes, re-run questions Q41-Q60 (ADVANCED) manually:

```bash
for q in Q41 Q42 Q43 ... Q60; do
    echo "Testing: $q"
    ./target/release/annactl "$q" | tee results/$q.txt
done
```

Expect 17/20 success (from current 2/20).

## Known Limitations

**Will NOT fix**:
- CHAOS category creative tasks (Q81-Q100) - require external APIs, complex automation
- Some EXPERT tasks requiring hardware (GPU passthrough, VPN server without hardware)
- Questions requiring user-specific context ("my downloads folder" without knowing path)

**Can improve but won't hit 100%**:
- Questions with ambiguous requirements
- Tasks requiring GUI interaction
- Hardware-dependent operations

Realistic target after all fixes: **85-90% overall success rate**
