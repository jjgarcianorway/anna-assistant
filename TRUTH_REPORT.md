# Truth Report: v0.0.6 Grounding Fixes

This document describes what was broken in Anna's responses before v0.0.6 and how it was fixed.

## Observed Problems

### Problem 1: Invented Version Numbers

**Observed behavior**: When asked about Anna's version, the LLM would sometimes claim to be "v0.0.1" or invent version numbers because the system prompt hardcoded "v0.0.1".

**Example bad output**:
```
User: what version are you?
Anna: I am Anna version 0.0.1...
```

**Root cause**: The system prompt in `rpc_handler.rs` was hardcoded:
```rust
let system_prompt = r#"You are Anna, a helpful local AI assistant for Linux systems.
For v0.0.1, you can only perform READ-ONLY operations..."#;
```

### Problem 2: Generic Advice Instead of Actual Data

**Observed behavior**: When asked "what CPU do I have?" or "what processes are using the most memory?", Anna would suggest running commands like `lscpu` or `ps aux` instead of answering directly from available data.

**Example bad output**:
```
User: what cpu do i have?
Anna: To find out your CPU, you can run `lscpu` or check `/proc/cpuinfo`...
```

**Root cause**: The system prompt did not inject any runtime context. The LLM had no access to:
- Hardware information already collected by annad
- Any probe results from the system

## Fixes Applied

### Fix 1: Runtime Context Injection

Created a `RuntimeContext` struct that is built fresh for every LLM request:

```rust
pub struct RuntimeContext {
    pub version: String,           // From VERSION constant
    pub daemon_running: bool,
    pub capabilities: Capabilities,
    pub hardware: HardwareSummary, // From annad's hardware probe
    pub probes: HashMap<String, String>,
}
```

This context is injected into every system prompt, making it impossible for the LLM to guess wrong.

### Fix 2: Grounded System Prompt

The new system prompt includes:

1. **Authoritative runtime context block** with exact version, hardware, and capabilities
2. **Strict grounding rules**:
   - Never invent facts not in context
   - Answer hardware questions directly from snapshot
   - Never suggest manual commands when data is available
   - Never claim capabilities not in the flags

### Fix 3: Auto-Probes

For queries about processes, memory, disk, or network, annad now automatically runs the relevant probe and injects the results into the context:

- Query contains "memory" or "process" → runs `ps aux --sort=-%mem`
- Query contains "disk" or "storage" → runs `df -h`
- Query contains "network" or "ip" → runs `ip addr show`

This means Anna can answer "what processes are using the most memory?" with actual data, not suggestions.

### Fix 4: Probe RPC Method

Added a new `probe` RPC method that can run safe read-only system queries on demand. Probe types:
- `top_memory` - Top processes by memory
- `top_cpu` - Top processes by CPU
- `disk_usage` - Filesystem usage
- `network_interfaces` - Network info

## Verification

After v0.0.6:

**CPU query**:
```
User: what cpu do i have?
Anna: You have an AMD Ryzen 9 7945HX with 32 cores and 31.0 GB RAM.
```

**Memory query**:
```
User: what processes are using the most memory?
Anna: Based on the current system probe, the top memory-consuming processes are:
[actual ps output with process names, PIDs, and memory usage]
```

**Version query**:
```
User: what version are you?
Anna: I am Anna version 0.0.6.
```

## Files Changed

- `crates/anna-shared/src/rpc.rs` - Added RuntimeContext, Capabilities, HardwareSummary, ProbeType
- `crates/annad/src/rpc_handler.rs` - Complete rewrite with grounded system prompt
- `crates/annad/src/probes.rs` - New module for running safe system queries
- `SPEC.md` - Updated to v0.0.6 with grounding policy
- `README.md` - Updated features and version
