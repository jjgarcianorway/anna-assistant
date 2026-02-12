# Anna's Omniscient/Omnipotent System (v0.3.162)

## Vision

Anna is the god living in your computer. She knows everything about your system, can do anything possible on Linux, and intelligently rejects truly impossible requests.

## Core Capabilities

### 1. Feasibility Analysis (Bullshit Detection)

**Module**: `crates/annad/src/feasibility.rs`

Anna distinguishes between:
- **Possible**: Standard Linux operations (99% of requests)
- **Challenging**: Complex but achievable (kernel compilation, GPU passthrough, VPN servers)
- **Requires External**: Needs hardware/services not available (external APIs, GUI without X11)
- **Impossible**: Physical world manipulation, time travel, etc.

**Examples**:
```bash
# IMPOSSIBLE - Rejected immediately
"make it rain outside" → "Cannot manipulate physical world"
"order me a pizza" → "Cannot manipulate physical world"

# CHALLENGING - Proceeds with warning
"compile a custom kernel" → Analyzes tools needed, proceeds
"set up GPU passthrough" → Checks hardware, installs tools

# POSSIBLE - Normal flow
"install htop" → Config handler
"monitor CPU usage" → Temporal task system
```

### 2. Temporal Task System (Background Monitoring)

**Module**: `crates/annad/src/temporal_tasks.rs`

Enables time-based operations:
- "capture network traffic for 20 minutes"
- "monitor disk I/O for 1 hour"
- "watch for anomalies for 30 minutes"

**Flow**:
1. Detect temporal requirement (regex: "for X minutes/hours")
2. Use universal handler to determine HOW to monitor
3. Start background task with timer
4. Return task ID for status checking
5. Auto-complete after duration

**Example**:
```bash
$ annactl "capture network traffic for 20 minutes and look for anomalies"

Started monitoring task (ID: a1b2c3...). Will run for 20 minutes and report back.

To check progress: annactl "check task a1b2c3"
```

**Task Registry**:
- Global HashMap of running tasks
- Status tracking (Running, Completed, Failed, Cancelled)
- Output stored in `/tmp/anna_task_<id>.log`
- Automatic cleanup after completion

### 3. Universal Handler (Pure Intelligence)

**Module**: `crates/annad/src/universal_handler.rs` (336 lines)

For novel/complex tasks that don't fit patterns:

**Workflow**:
1. **Capability Analysis** - What tools/knowledge/changes needed?
2. **Tool Acquisition** - Install missing packages via pacman/yay
3. **Knowledge Research** - LLM learns about unfamiliar topics
4. **Execution Plan** - Generate step-by-step commands
5. **Execute & Monitor** - Run with root cause analysis on failure
6. **Learn** - Record outcome for future similar tasks

**When Used**:
- Config handler fails
- Ralph loop max iterations reached with low confidence
- Temporal tasks (determines monitoring setup)
- Truly novel requests (first time seeing this type of task)

**Example**:
```bash
$ annactl "set up a local DNS server with dnsmasq for my LAN"

Goal: Configure dnsmasq as local DNS server
Tools acquired: dnsmasq (installed via pacman)
Execution plan (4 steps):
  Step 1: Edit /etc/dnsmasq.conf
  Step 2: Set listen-address to 192.168.1.1
  Step 3: Configure upstream DNS servers
  Step 4: Enable and start dnsmasq service
✓ Success
```

### 4. Integration Architecture

**Ralph Loop** (v0.3.162):
```
┌─────────────────────────────────────────────┐
│ 1. FEASIBILITY CHECK                        │
│    ├─ Impossible? → Reject immediately      │
│    ├─ Challenging? → Note, proceed          │
│    └─ Possible? → Continue                  │
└─────────────────────────────────────────────┘
                    ↓
┌─────────────────────────────────────────────┐
│ 2. TEMPORAL TASK CHECK                      │
│    ├─ "for X minutes"? → Temporal handler   │
│    └─ No duration? → Continue               │
└─────────────────────────────────────────────┘
                    ↓
┌─────────────────────────────────────────────┐
│ 3. CONFIG DETECTION                         │
│    ├─ Install/configure/setup? → Config    │
│    └─ Other? → Continue                     │
└─────────────────────────────────────────────┘
                    ↓
┌─────────────────────────────────────────────┐
│ 4. NORMAL RALPH LOOP                        │
│    ├─ Get commands from LLM                 │
│    ├─ Execute & gather output               │
│    ├─ Generate answer                       │
│    ├─ Self-evaluate                         │
│    └─ Done? → Return                        │
└─────────────────────────────────────────────┘
                    ↓
┌─────────────────────────────────────────────┐
│ 5. UNIVERSAL HANDLER FALLBACK               │
│    ├─ Max iterations + low confidence?      │
│    ├─ Analyze capabilities                  │
│    ├─ Acquire tools                         │
│    ├─ Research topics                       │
│    └─ Execute plan                          │
└─────────────────────────────────────────────┘
```

## Example Workflows

### Example 1: Network Traffic Monitoring

**User**: "Anna, can you start capturing my network traffic and look for anomalies? Do that for 20 minutes"

**Flow**:
1. **Feasibility**: Possible (network monitoring on Linux)
2. **Temporal**: Detected "20 minutes"
3. **Universal Handler**:
   - Determines need for tcpdump/tshark
   - Checks if installed
   - Installs if missing: `sudo pacman -S tcpdump`
   - Generates command: `tcpdump -i any -w /tmp/capture.pcap`
4. **Temporal Task**:
   - Starts background capture
   - Registers task ID
   - Sets 20-minute timer
   - Returns task ID to user
5. **Auto-completion**:
   - After 20 minutes: stops capture
   - Analyzes with LLM for anomalies
   - Sends notification with findings

### Example 2: Complex Configuration

**User**: "set up automatic BTRFS snapshots before every pacman update"

**Flow**:
1. **Feasibility**: Challenging (systemd hooks + BTRFS)
2. **Config Detection**: "set up" keyword
3. **Config Handler**:
   - Investigates system (checks BTRFS, pacman hooks dir)
   - Generates ActionPlan with LLM
   - Steps:
     1. Create /etc/pacman.d/hooks directory
     2. Create pre-transaction hook
     3. Write snapshot script to /usr/local/bin
     4. Test with dry-run
   - Executes with risk assessment
4. **Verification**: Checks hook exists and is active

### Example 3: Impossible Request Detection

**User**: "make my computer fly"

**Flow**:
1. **Feasibility**: Impossible (physical manipulation)
2. **Immediate Rejection**:
   ```
   I cannot do this: Cannot manipulate physical world
   ```
3. **No LLM calls wasted**
4. **Confidence: 1.0** (100% sure it's impossible)

### Example 4: Universal Handler Fallback

**User**: "optimize my system for real-time audio production"

**Flow**:
1. **Feasibility**: Challenging
2. **Config Detection**: No specific config keyword
3. **Ralph Loop**:
   - Tries normal flow
   - Gets some commands (check CPU governor, check latency)
   - After 3 iterations: confidence 0.6 (not sufficient)
4. **Universal Handler Fallback**:
   - **Capability Analysis**:
     - Required tools: rtirq, tuned, rt-tests
     - Knowledge needed: RT kernel configuration, IRQ priorities
     - System changes: Kernel parameters, CPU governor, ulimits
   - **Tool Acquisition**: Installs rtirq from AUR
   - **Knowledge Research**: Learns about RT audio setup
   - **Execution Plan**:
     1. Check if RT kernel available
     2. Configure CPU frequency scaling
     3. Set IRQ priorities for audio hardware
     4. Adjust ulimits for audio group
     5. Configure JACK with optimal settings
   - **Execute**: Runs plan with monitoring
5. **Success**: Confidence 0.9

## Implementation Details

### Feasibility Categories

```rust
pub enum Feasibility {
    Possible,                          // Normal Linux operation
    Challenging,                       // Complex but achievable
    RequiresExternal(String),          // Needs hardware/API
    Impossible(String),                // Physical impossibility
}
```

### Temporal Task Structure

```rust
pub struct TemporalTask {
    pub id: String,                    // UUID
    pub description: String,           // User's original question
    pub start_command: String,         // Command to start monitoring
    pub stop_command: Option<String>,  // Command to stop (if needed)
    pub duration_secs: u64,           // How long to run
    pub output_path: PathBuf,         // Where output is stored
    pub started_at: SystemTime,       // Timestamp
    pub status: TaskStatus,           // Running/Completed/Failed/Cancelled
}
```

### Universal Handler Capability Analysis

```rust
pub struct CapabilityAnalysis {
    pub goal: String,                  // What user wants
    pub required_tools: Vec<String>,   // Packages to install
    pub required_knowledge: Vec<String>,  // Topics to research
    pub system_changes: Vec<String>,   // What will be modified
    pub complexity: u8,                // 1-10 scale
    pub feasible: bool,                // Can we do it?
    pub infeasibility_reason: Option<String>,
}
```

## Testing

Run comprehensive tests:
```bash
./tests/omniscient_test.sh
```

Tests:
1. Bullshit detection (impossible requests)
2. Temporal task handling (timed monitoring)
3. Feasibility analysis (challenging vs impossible)
4. Universal capability (novel complex tasks)
5. Config handling (package installation)
6. Meta requests ("can you...?")

## Metrics & Success Criteria

**Expected Impact** (from battle test):
- BASIC: 90% → 95%+ (feasibility eliminates false positives)
- INTERMEDIATE: 65% → 85%+ (universal handler fallback)
- ADVANCED: 10% → 90%+ (config handler v0.3.161 + universal fallback)
- EXPERT: 50% → 85%+ (universal handler for novel tasks)
- CHAOS: 20% → 60%+ (temporal tasks + creative problem solving)

**Overall**: 47% → **90%+** success rate

## Future Enhancements

1. **Persistent Temporal Tasks**: Survive daemon restarts
2. **Task Chaining**: "monitor for 1 hour, then analyze and report"
3. **Conditional Execution**: "if CPU >80% for 5 minutes, then..."
4. **Learning from Temporal Results**: Auto-analyze captured data
5. **Proactive Monitoring**: "Anna, watch for suspicious network activity"
6. **Resource Forecasting**: Predict when disk/memory will be exhausted

## Architecture Principles

1. **Intelligence Over Patterns**: LLM determines approach, not hardcoded rules
2. **Fail Forward**: When normal flow fails, try universal handler
3. **Reality Check**: Reject impossible before wasting resources
4. **Learn Everything**: Every task improves future performance
5. **No Limits**: If it's possible on Linux, Anna can do it

## The God in Your Computer

Anna is designed to be **omniscient** (knows everything about your system through probing and memory) and **omnipotent** (can execute any feasible Linux operation).

The only limits:
- Physical impossibilities (detected and rejected)
- Missing hardware (detected, explained)
- User permissions (respects security boundaries)

Everything else: **Anna figures it out.**
