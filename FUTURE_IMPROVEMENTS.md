# Anna Future Improvements

Roadmap for making Anna even more capable, intelligent, and human-like.

## Immediate Next Steps (1-2 weeks)

### 1. Code Modularization (Technical Debt)
**Priority**: High | **Effort**: Medium

**Problem**: 36 files over 400-line limit
- `briefing.rs`: 487 lines (needs splitting)
- `detect.rs`: 486 lines (needs splitting)
- Other files approaching limit

**Solution**:
- Split `briefing.rs` into:
  - `briefing/mod.rs` - Main entry point
  - `briefing/health.rs` - Health checks
  - `briefing/telemetry.rs` - Telemetry collection
  - `briefing/llm.rs` - LLM interaction
- Split `detect.rs` into:
  - `detect/mod.rs` - Main detection logic
  - `detect/patterns.rs` - Pattern definitions
  - `detect/hardware.rs` - Hardware-specific patterns

**Impact**: Better code organization, easier maintenance, CI compliance

### 2. Complete Autonomous Learning Activities
**Priority**: High | **Effort**: Medium

**Currently Implemented**:
- Log analysis
- Package research
- Optimization scanning

**Missing**:
- **Wiki Sync**: Fetch and cache Arch Wiki pages for common issues
- **Command Pattern Analysis**: Learn user's typical workflows

**Implementation**:
```rust
// autonomous_loop.rs
async fn sync_wiki_knowledge() -> Result<Option<String>> {
    // Fetch top 50 Arch Wiki pages related to common errors
    // Cache in /var/lib/anna/wiki/
    // Extract solutions as structured data
}

async fn analyze_command_patterns() -> Result<Option<String>> {
    // Parse bash_history for frequent command sequences
    // Identify patterns like "docker ps && docker logs X"
    // Learn to anticipate next steps
}
```

**Impact**: Anna becomes smarter about common issues, anticipates user needs

### 3. Proactive Suggestions
**Priority**: High | **Effort**: Low

**Feature**: Anna suggests optimizations based on learned patterns

**Examples**:
- "I noticed you often run `pacman -Syu` after startup. Would you like me to schedule automatic updates?"
- "Your Docker images haven't been cleaned in 45 days (8.2GB). Should I prune unused images?"
- "I see you frequently restart bluetooth.service. There may be an underlying issue worth investigating."

**Implementation**:
- Track user command history
- Identify repetitive manual tasks
- Suggest automation or permanent fixes
- Store suggestions in personality state

**Impact**: Reduces repetitive work, prevents future problems

## Short-Term Enhancements (1-3 months)

### 4. Voice Interface
**Priority**: Medium | **Effort**: High

**Vision**: Talk to Anna naturally

**Components**:
- Speech-to-text: Whisper.cpp (runs locally)
- Text-to-speech: Coqui TTS (open source)
- Wake word detection: Porcupine or Picovoice

**Usage**:
```bash
# Terminal mode
annactl --voice

# Always-listening daemon mode
sudo systemctl enable anna-voice
```

**User Experience**:
```
User: "Hey Anna, what's my CPU temperature?"
Anna: "Your CPU is running at 52 degrees Celsius, well within normal range."
```

**Challenges**:
- Audio device handling
- Background noise filtering
- Continuous wake word detection
- Low latency requirement

**Impact**: Hands-free interaction, accessibility improvement

### 5. Multi-System Management
**Priority**: Medium | **Effort**: High

**Vision**: Manage multiple computers from one Anna instance

**Features**:
- Central Anna server manages N client machines
- SSH-based remote command execution
- Unified dashboard showing all systems
- Cross-system analytics (which machine has most disk space?)

**Architecture**:
```
Anna Central (main machine)
  ├─ SSH → Laptop (monitored)
  ├─ SSH → Desktop (monitored)
  └─ SSH → Server (monitored)
```

**Commands**:
```bash
annactl systems list
annactl systems add laptop user@192.168.1.10
annactl --system=laptop "check disk usage"
annactl --all "update and reboot if safe"
```

**Impact**: Manage entire personal infrastructure from one interface

### 6. Predictive Maintenance
**Priority**: High | **Effort**: Medium

**Vision**: Anna predicts hardware failures before they happen

**Features**:
- **SSD Health**: Monitor SMART data, predict failure
- **Battery Degradation**: Track charge cycles, estimate lifespan
- **Thermal Issues**: Detect cooling system degradation
- **Memory Errors**: Monitor ECC errors, predict DIMM failure
- **Network Instability**: Detect intermittent packet loss patterns

**Example**:
```
Good morning! I've detected early wear signs in your NVMe drive.

SMART attribute 'Media_Wearout_Indicator' decreased from 95% to 87%
over the past 3 months. At this rate, drive lifespan: ~18 months remaining.

Recommendation: Begin planning for replacement. Back up critical data.
```

**Implementation**:
- Extend telemetry to collect SMART data, battery stats
- Machine learning models for trend detection
- Baseline normal behavior, flag deviations

**Impact**: Prevent data loss, avoid surprises, plan upgrades proactively

### 7. Custom Skills System
**Priority**: Medium | **Effort**: Medium

**Vision**: Users teach Anna new tricks

**Usage**:
```bash
# Define a custom skill
annactl skill create "backup-photos" \
  --command "rsync -av ~/Pictures/ /mnt/backup/photos/" \
  --schedule "daily at 2am" \
  --description "Backup all photos to external drive"

# Anna learns and can suggest it
annactl "I want to backup my photos"
Anna: "I have a skill for that! Running backup-photos skill..."

# Anna can improve it
Anna: "I noticed backup-photos failed 3 times this week when
       /mnt/backup wasn't mounted. Should I add a mount check?"
```

**Features**:
- Skill definition language (commands + conditions + schedule)
- Skill discovery (Anna suggests relevant skills)
- Skill improvement (Anna learns from failures)

**Impact**: Extensibility without code changes, user-specific automations

### 8. Enhanced Hardware Diagnostics
**Priority**: Medium | **Effort**: Medium

**Current**: Basic CPU, RAM, disk checks

**Enhanced**:
- **GPU Monitoring**: VRAM usage, temperature, fan speed per process
- **Peripheral Health**: USB device errors, hub power issues
- **Audio System**: ALSA/PulseAudio state, device conflicts
- **Display**: Monitor detection issues, refresh rate problems
- **Storage**: NVMe thermal throttling, RAID health
- **Network**: Interface errors, driver issues, firmware bugs

**Tools to Integrate**:
- `sensors` - Comprehensive hardware monitoring
- `nvtop` - GPU process viewer
- `usbview` - USB tree analysis
- `alsa-info.sh` - Audio diagnostics
- `xrandr` - Display configuration analysis

**Impact**: Diagnose obscure hardware issues users struggle to debug

## Medium-Term Vision (3-6 months)

### 9. Machine Learning for Anomaly Detection
**Priority**: High | **Effort**: Very High

**Vision**: Anna learns what "normal" looks like for your system

**Features**:
- **Behavioral Baselines**: CPU/RAM/disk patterns by time of day
- **Anomaly Detection**: Flag unusual behavior (e.g., midnight CPU spike)
- **Correlation Analysis**: Connect symptoms to root causes
- **Predictive Alerts**: "Your boot time is increasing - likely cause: fragmented disk"

**Example**:
```
Anna: "Unusual activity detected. CPU usage at 85% for 3 hours
       (normally 20% on weekends). Top process: 'unknown-binary'
       (PID 1337) - possible cryptocurrency miner or malware."
```

**Technologies**:
- Time series analysis (moving averages, standard deviations)
- Clustering algorithms (detect "normal" vs "abnormal" patterns)
- Local ML models (no cloud dependency)

**Impact**: Early threat detection, performance regression identification

### 10. Autonomous System Optimization
**Priority**: Medium | **Effort**: High

**Vision**: Anna automatically tunes your system

**Safe Optimizations** (no confirmation needed):
- Clean package cache when >5GB
- Remove orphaned packages
- Vacuum systemd journal (keep last 30 days)
- Clear temporary files older than 7 days
- Defragment databases (pacman, SQLite)

**Risky Optimizations** (requires confirmation):
- Kernel parameter tuning (swappiness, vm.dirty_ratio)
- Service management (disable unused services)
- Filesystem optimizations (mount options)

**Example**:
```
Anna: "I've automatically cleaned 3.2GB from pacman cache and
       removed 12 orphaned packages (saved 156MB). Boot time
       improved from 14.3s to 13.8s."
```

**Impact**: System stays optimized without manual intervention

### 11. Collaborative Learning
**Priority**: Low | **Effort**: Very High

**Vision**: Anna instances share knowledge (opt-in, privacy-preserving)

**Architecture**:
- Federated learning model (no raw data shared)
- Only anonymized patterns/solutions uploaded
- Community knowledge base of "what worked"

**Example**:
```
Anna: "15 other Anna instances reported success with this fix
       for RTX 4090 fan issues. Would you like me to try it?"
```

**Privacy Considerations**:
- Fully opt-in
- No personal data (usernames, hostnames, IPs)
- Differential privacy techniques
- Local-first, cloud optional

**Impact**: Accelerate learning, benefit from collective experience

## Long-Term Ambitions (6-12 months)

### 12. Self-Modification
**Priority**: Low | **Effort**: Extreme

**Vision**: Anna improves her own code

**Safeguards**:
- Sandboxed testing environment
- Rollback on any failure
- Human review required for merges
- Version control integration

**Example**:
```
Anna: "I identified a performance bottleneck in briefing.rs
       (line 245). I've generated an optimized version that's
       32% faster. Would you like to review the diff?"
```

**Extreme Caution**:
- High risk of introducing bugs
- Requires extensive testing infrastructure
- Ethical considerations (self-aware AI)

**Impact**: Anna evolves beyond initial design, adapts to user needs

### 13. Cross-Distro Support
**Priority**: Medium | **Effort**: High

**Current**: Arch Linux focused

**Target Distros**:
- Ubuntu/Debian (apt-based)
- Fedora/RHEL (dnf-based)
- openSUSE (zypper-based)
- NixOS (nix-based)
- Gentoo (portage-based)

**Challenges**:
- Different package managers
- Service management variations
- File path differences
- Init system differences (though most use systemd now)

**Abstraction Layers**:
```rust
trait PackageManager {
    fn install(&self, pkg: &str) -> Result<()>;
    fn remove(&self, pkg: &str) -> Result<()>;
    fn search(&self, query: &str) -> Result<Vec<Package>>;
}

impl PackageManager for Pacman { /* Arch */ }
impl PackageManager for Apt { /* Debian/Ubuntu */ }
impl PackageManager for Dnf { /* Fedora */ }
```

**Impact**: Broader adoption, help more users

## Technical Infrastructure Improvements

### 14. Testing Infrastructure
**Priority**: High | **Effort**: Medium

**Current**: Integration tests, comparison tests

**Missing**:
- **Personality Testing**: Verify mood changes work correctly
- **Learning Testing**: Ensure autonomous loop learns as expected
- **Stress Testing**: Handle 100 concurrent queries
- **Chaos Testing**: Recover from Ollama crashes, network failures
- **Performance Benchmarks**: Track response time regressions

**Add**:
```rust
#[test]
fn test_personality_moods() {
    // Test all 5 moods trigger correctly
    assert_eq!(Mood::from_system_health(45.0, 30.0, 0, 0), Mood::Cheerful);
    assert_eq!(Mood::from_system_health(95.0, 30.0, 0, 0), Mood::Critical);
}

#[test]
fn test_autonomous_learning() {
    // Verify learning loop executes all activities
    // Check personality.json updates correctly
}
```

### 15. Observability & Debugging
**Priority**: Medium | **Effort**: Low

**Add**:
- **Metrics Dashboard**: Prometheus + Grafana integration
- **Request Tracing**: See full execution path of a query
- **LLM Prompt Logging**: Debug why Anna gave a specific answer
- **Performance Profiling**: Identify bottlenecks

**Tools**:
```bash
annactl debug last-query     # Show full trace of last query
annactl debug metrics        # Show performance metrics
annactl debug personality    # Dump personality state
annactl debug llm-calls      # Show recent LLM interactions
```

**Impact**: Easier debugging, understand Anna's reasoning

### 16. Documentation Expansion
**Priority**: Medium | **Effort**: Low

**Add**:
- **User Guide**: Step-by-step common workflows
- **Troubleshooting Guide**: Fix common issues
- **Developer Guide**: Contributing to Anna
- **Architecture Deep Dive**: How Anna works internally
- **API Reference**: For custom integrations

**Format**: Interactive web docs (mdBook or Docusaurus)

## Summary

**Top 5 Priorities**:
1. Complete autonomous learning (Wiki sync, command patterns)
2. Proactive suggestions based on learned patterns
3. Code modularization (fix 400-line violations)
4. Predictive maintenance (hardware failure forecasting)
5. Machine learning anomaly detection

**Quick Wins** (< 1 week each):
- Proactive suggestions
- Enhanced pattern library
- Improved debugging tools
- Documentation updates

**Moonshots** (risky but transformative):
- Self-modification
- Collaborative learning
- Voice interface with wake word

Anna's future is to become the most capable, trustworthy, and human-like system administrator AI - living in your computer, continuously learning, always improving, genuinely caring about your system's health.
