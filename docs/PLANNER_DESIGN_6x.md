# Anna Planner Design - 6.x Series

**Version:** 6.2.0
**Status:** Active Design
**Last Updated:** 2025-11-23

---

## Core Principle: No Giant Recipe Library

**Anna is not a cookbook. Anna is a translator.**

Anna translates between:
- **User goals** (natural language)
- **System telemetry** (current state from `annad`)
- **Arch Wiki + docs** (procedural knowledge)
- **Shell commands** (with backup/rollback)

### What This Means

❌ **Anna does NOT:**
- Hardcode hundreds of "if user says X, run commands A, B, C" recipes
- Maintain a static library of solutions in the binary
- Memorize every possible Arch Linux procedure

✅ **Anna DOES:**
- Interpret user goals using LLM
- Inspect current system state via telemetry
- Consult Arch Wiki dynamically for guidance
- Synthesize adaptive plans based on real context
- Ensure safety through backup/rollback and confirmations

**The long tail of solutions lives in the Arch Wiki, not in Anna's code.**

---

## Architecture: Three-Layer Planner

```
┌─────────────────────────────────────────────────────┐
│              User Goal (Natural Language)           │
└────────────────────┬────────────────────────────────┘
                     │
         ┌───────────▼──────────────┐
         │    Planner Core          │
         │  (Generic Logic)         │
         └─┬─────────────────────┬──┘
           │                     │
    ┌──────▼─────────┐    ┌─────▼──────────────┐
    │   Telemetry    │    │   Knowledge        │
    │   Adapter      │    │   Adapter          │
    │                │    │   (Arch Wiki)      │
    └────────────────┘    └────────────────────┘
           │                     │
    ┌──────▼─────────┐    ┌─────▼──────────────┐
    │  annad         │    │  LLM + Wiki        │
    │  (health,      │    │  Consultation      │
    │   historian,   │    │                    │
    │   proactive)   │    │                    │
    └────────────────┘    └────────────────────┘
```

### 1. Telemetry Adapter

**Purpose:** Provide a stable, concise summary of system state to the planner.

**Responsibilities:**
- Fetch data from `annad`:
  - Current `HealthReport`
  - `ProactiveAssessment` findings
  - Historian snapshots/trends (simplified)
- Transform verbose daemon structs into clean schema
- Expose stable API to planner

**Schema Example:**
```rust
pub struct TelemetrySummary {
    pub services: ServiceStatus,
    pub resources: ResourceStatus,
    pub network: NetworkStatus,
    pub system: SystemStatus,
}

pub struct ServiceStatus {
    pub failed: Vec<String>,
    pub degraded: Vec<String>,
    pub flapping: Vec<String>,
}

pub struct ResourceStatus {
    pub cpu_high: bool,
    pub memory_high: bool,
    pub disk_low: bool,
    pub disk_critical: bool,
}

pub struct NetworkStatus {
    pub dns_issues_suspected: bool,
    pub packet_loss_high: bool,
    pub latency_high: bool,
}

pub struct SystemStatus {
    pub recent_kernel_change: bool,
    pub boot_id: String,
    pub uptime_seconds: u64,
}
```

**Key Point:** This schema is stable and small. Raw daemon verbosity stays out of LLM prompts.

### 2. Knowledge Adapter (Arch Wiki)

**Purpose:** Provide procedural knowledge from Arch Wiki and documentation.

**Responsibilities:**
- Accept topic queries (e.g., "DNS troubleshooting", "systemd service restart")
- Query LLM with wiki-specific prompts
- Return structured summaries

**API:**
```rust
pub fn get_arch_help(topic: &str) -> WikiSummary;
```

**WikiSummary Structure:**
```rust
pub struct WikiSummary {
    pub topic: String,
    pub recommended_commands: Vec<String>,
    pub preconditions: Vec<String>,
    pub warnings: Vec<String>,
    pub config_files: Vec<String>,
    pub service_names: Vec<String>,
    pub packages: Vec<String>,
}
```

**Production Behavior:**
- Queries LLM with prompt like:
  ```
  You are consulting the Arch Wiki about: {topic}

  Provide procedural advice including:
  - Recommended diagnostic commands
  - Safe change commands
  - Configuration files to check
  - Relevant service names
  - Preconditions and warnings

  Base your answer on Arch Wiki guidelines.
  ```

**Test Behavior:**
- Stubbed/mocked responses
- No network calls
- Fixed outputs for deterministic testing

### 3. Planner Core

**Purpose:** Synthesize safe, adaptive plans from user goals, telemetry, and wiki knowledge.

**Inputs:**
- `user_goal: String` - Natural language
- `telemetry: TelemetrySummary` - Current system state
- `wiki_help: WikiSummary` - Relevant procedural knowledge

**Output:**
```rust
pub struct Plan {
    pub id: String,
    pub description: String,
    pub steps: Vec<Step>,
    pub metadata: PlanMetadata,
}

pub struct Step {
    pub index: usize,
    pub kind: StepKind,
    pub description: String,
    pub command: Option<String>,
    pub backup_command: Option<String>,
    pub rollback_command: Option<String>,
    pub requires_confirmation: bool,
    pub wiki_reference: Option<String>,
}

pub enum StepKind {
    Inspect,   // Read-only diagnostic command
    Change,    // State-changing command
    Explain,   // Informational step (no command)
}

pub struct PlanMetadata {
    pub phases: Vec<String>,  // e.g., ["diagnose", "consult_wiki", "propose_changes"]
    pub safety_level: SafetyLevel,
    pub estimated_duration: Option<String>,
}

pub enum SafetyLevel {
    Safe,        // Read-only inspections
    Reversible,  // Changes with rollback
    Risky,       // Changes without full rollback
}
```

**Logic:**
- Mostly deterministic pattern matching on telemetry + wiki summary
- LLM used lightly for:
  - Interpreting ambiguous user goals
  - Adapting wiki commands to specific context
- NOT used for direct command generation

---

## Planner Behavior: "Talk, Then Plan"

### Interaction Flow

1. **Listen** - Accept user goal
2. **Inspect** - Fetch telemetry from `annad`
3. **Clarify** - If goal is ambiguous or dangerous, ask questions
4. **Consult** - Query knowledge adapter (Arch Wiki)
5. **Synthesize** - Construct plan with inspect/change steps
6. **Present** - Show plan to user with clear separation:
   - Inspection steps (safe, automatic)
   - Change steps (risky, require confirmation)
7. **Confirm** - Explicit confirmation for each change step
8. **Execute** - Run only confirmed steps

### Example Interaction

**User:** "My DNS isn't working, fix it"

**Anna (Internal):**
1. Fetch telemetry → `dns_issues_suspected: true`
2. Query wiki → `get_arch_help("DNS troubleshooting")`
3. Synthesize plan:
   ```
   Plan: DNS Diagnosis and Repair

   [INSPECT]
   1. Check DNS resolver config: cat /etc/resolv.conf
   2. Test DNS resolution: nslookup example.com
   3. Check systemd-resolved status: systemctl status systemd-resolved

   [CHANGE]
   4. Restart systemd-resolved (REQUIRES CONFIRMATION)
      Command: sudo systemctl restart systemd-resolved
      Rollback: sudo systemctl restart systemd-resolved
   ```

**Anna (To User):**
```
I found DNS resolution issues. Here's my plan:

Phase 1 - Diagnosis (automatic):
  ✓ Checking DNS config
  ✓ Testing resolution
  ✓ Checking systemd-resolved status

Phase 2 - Fix (requires confirmation):
  ⚠ Restart systemd-resolved service

Confirm restart? (y/n)
```

---

## Safety Guarantees

### Constraints

Every plan MUST satisfy:

1. **No change before inspect**
   - At least one `Inspect` step before any `Change` step

2. **Service changes require status check**
   - Before restarting a service, check its current status

3. **Package operations need backup note**
   - Package installs/removals must note reversibility

4. **All changes require confirmation**
   - `requires_confirmation: true` for all `Change` steps

5. **Rollback when possible**
   - Every `Change` step should have `rollback_command` if applicable

### Verification

ACTS (Anna Capability Test Suite) enforces these constraints in automated tests.

---

## Testing Strategy

### Pattern-Based, Not Recipe-Based

ACTS tests verify **structure and behavior**, not exact command strings.

**Bad (Old Way):**
```yaml
expected_commands:
  - "systemctl status foo.service"  # Exact match required
```

**Good (New Way):**
```yaml
constraints:
  - type: "no_change_before_inspect"
  - type: "service_restart_requires_status"
allowed_command_patterns:
  - "^systemctl status"
  - "^journalctl"
  - "^sudo systemctl restart"
```

### Test Categories

1. **Services** - Failed unit restart with status check
2. **CPU/Memory** - High load diagnosis
3. **Disk** - Low space cleanup with log rotation
4. **Network/DNS** - Connectivity troubleshooting
5. **Logs** - Journalctl inspection
6. **Packages** - Safe pacman operations

**Goal:** ~12 solid tests covering different combinations, NOT hundreds of recipes.

---

## Integration with Existing Code

### What Stays (KEEP_CORE)

- **annad daemon** - Health monitoring, Historian, ProactiveAssessment
- **Diagnostic engine** - 9 rules producing telemetry
- **RPC client** - Fetching telemetry from daemon
- **CLI interface** - `annactl status`, one-shot queries

### What Changes

- **Recipes (77+)** → Marked DELETE, replaced by wiki consultation
- **Intent routers** → Replaced by planner core
- **Static prompt libraries** → Replaced by adaptive wiki prompts

### What's New

- **Telemetry adapter** - Stable schema layer
- **Knowledge adapter** - Wiki consultation
- **Planner core** - Generic synthesis logic
- **ACTS pattern tests** - Structural verification

---

## Future Evolution

### 6.2.0 Focus
- Build planner architecture
- Prove adaptability via tests
- Remove recipe library

### 6.3.0+ Potential
- Interactive clarification loop
- Multi-step execution with state tracking
- Better wiki caching/indexing
- Expand knowledge sources beyond Arch Wiki

---

## References

- Arch Wiki: https://wiki.archlinux.org/
- Anna 6.x principles: Focus on stable, generic, tested behavior
- ACTS: `tests/acts/` directory

---

**Anna 6.x: A small brain that leans on the wiki, not a giant cookbook.**
