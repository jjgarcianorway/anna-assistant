# Anna Assistant - Project Status Overview

**Document Version**: PLANNING_STEP_1
**Anna Version**: v5.7.0-beta.277
**Date**: 2025-11-23
**Purpose**: Truthful assessment of current state and remaining work

---

## 1. Introduction

### What Anna Is

Anna is a **proactive senior sysadmin for Arch Linux**, designed to:
- Monitor system health continuously via daemon (`annad`)
- Detect problems using deterministic diagnostic rules
- Correlate issues using a proactive analysis engine
- Answer natural language questions about system state
- Provide actionable remediation commands
- Surface insights through CLI, TUI, and conversational interfaces

### What Anna Is NOT

- ❌ Not a generic chatbot or LLM wrapper
- ❌ Not autonomous (requires user approval for actions)
- ❌ Not production-ready (beta software under active development)
- ❌ Not a replacement for sysadmin knowledge
- ❌ Not a remote management tool

### Core Principles

1. **Local-First**: All data stays on the machine
2. **Telemetry-First**: Real system facts, not LLM guesses
3. **Deterministic**: Predictable, reproducible behavior
4. **Transparent**: Shows exact commands before execution
5. **Arch-Focused**: Deep integration with Arch Linux ecosystem

### This Document

This overview captures the **actual state** of Anna as of Beta.277, not the aspirational vision. It honestly assesses what works, what's partial, what's planned, and what the risks are.

---

## 2. System Map

| Subsystem | Status | Key Betas | Confidence | Risk | Notes |
|-----------|--------|-----------|------------|------|-------|
| **NL Router & QA Suites** | Mature | 243-277 | High | Low | 87% accuracy (609/700), deterministic pattern matching, 5 test suites |
| **Diagnostic Engine** | Mature | 217, 238, 250 | High | Low | 9 rules, canonical formatter, health computation |
| **Daily Snapshot** | Mature | 258 | Medium | Low | Kernel/package tracking, session deltas, sysadmin briefing format |
| **Sysadmin Composers** | Partial | 250-273 | Medium | Medium | Services, disk, CPU, memory, network; some gaps in edge cases |
| **Network Diagnostics** | Partial | 265-273 | Medium | Medium | Priority ranking, degradation detection, basic remediation; needs real-world validation |
| **Proactive Engine Core** | Prototype | 271, 273 | Medium | Medium | Correlation working, scoring implemented, but shallow integration |
| **Proactive Surfacing** | Partial | 271-273 | Medium | Medium | CLI/TUI indicators exist, NL routing partial, needs maturity |
| **TUI Layout & UX** | Partial | 231, 274 | Medium | Medium | Stable layout, health coherence, welcome/exit; missing Level 3 vision (tabs, logs, graphs) |
| **Daemon & Brain Analysis** | Mature | 217, 250 | High | Low | RPC server, telemetry collection, brain sysadmin analysis |
| **Historian & Timeline** | Planned | - | Low | High | Architecture sketched, not implemented |
| **Auto-Remediation** | Planned | - | Low | High | Recipes exist (77+), but no auto-execution framework |
| **Installation/Bootstrap** | Partial | - | Low | High | Install script works, but no recovery mode, no bootstrap wizard |
| **Security & Anomaly Detection** | Planned | - | Low | High | No intrusion detection, no anomaly correlation |
| **Performance Profiling** | Planned | - | Low | High | No CPU profiling, no I/O analysis, no bottleneck detection |

### Status Definitions

- **Planned**: Architecture sketched, no code yet
- **Prototype**: Core logic exists, minimal integration, untested
- **Partial**: Working for happy path, gaps in edge cases, needs validation
- **Mature**: Well-tested, documented, stable API, confident in behavior

### Confidence Definitions

- **Low**: Untested in real scenarios, likely has bugs
- **Medium**: Basic testing done, likely works for common cases
- **High**: Regression tests passing, documented, confident in behavior

### Risk Definitions

- **Low**: Unlikely to break, limited blast radius if it does
- **Medium**: May have edge case bugs, moderate impact if fails
- **High**: Untested, complex logic, high impact if fails

---

## 3. NL Routing Status

### Current Metrics (Beta.277)

**Test Suite**: regression_nl_big.toml (700 questions)
**Passing**: 609/700 (87.0%)
**Failing**: 91/700 (13.0%)

### Breakdown by Route

| Route | Tests | Notes |
|-------|-------|-------|
| diagnostic | ~250 | Full system health checks, "what's wrong", "check system" |
| status | ~80 | Quick status queries, "how's my system", "status" |
| conversational | ~370 | Ambiguous queries, chitchat, out-of-scope |

### Breakdown by Classification

| Classification | Count | Meaning |
|----------------|-------|---------|
| correct | ~609 | Routes as expected |
| router_bug | 0 | Routing errors (all fixed in Beta.275-276) |
| test_unrealistic | 0 | Unrealistic expectations (corrected in Beta.277) |
| ambiguous | ~91 | Intentionally difficult queries |

### What 87% Accuracy Actually Means

**For a user**:
- ✅ Clear diagnostic queries work reliably ("check system health", "any errors")
- ✅ Status queries work consistently ("show status", "what's the status")
- ⚠️ Very short or ambiguous queries may route to conversational ("problems?", "errors?")
- ⚠️ Queries without system context may not trigger diagnostic ("any issues in general")
- ⚠️ Highly informal phrasing may route unexpectedly ("yo what's up with my box")

**Why not 100%**:
- Some queries are intentionally ambiguous and should route to conversational
- Some edge cases require context Anna doesn't have
- Deterministic pattern matching has inherent limits without semantic understanding

**Is 87% good enough?**:
- Yes, for a beta assistant that errs on the side of conversational over false positives
- User can always rephrase if first attempt doesn't route as expected
- Patterns improve iteratively based on real usage

### Recent Improvements

- **Beta.275**: +60 patterns, 86.9% accuracy (+19 tests)
- **Beta.276**: Edge case fixes, 87.0% accuracy (+1 test)
- **Beta.277**: Ambiguity detection, 87.0% maintained (0 regression)

### Known Gaps

1. **Temporal queries**: "Is my system getting worse?" → conversational (lacks historical context)
2. **Subjective queries**: "Is my system slow?" → conversational (lacks baseline)
3. **Vague references**: "These services ok?" → conversational (needs context)
4. **Meta-questions**: "What can you check?" → conversational (needs capability introspection)

---

## 4. Proactive Engine Status

### What It Can Do (Beta.271-273)

✅ **Correlation**: Combines brain diagnostic insights with system health to identify patterns
✅ **Scoring**: Computes health score (0-100) based on issue severity and count
✅ **Surfacing**: Displays top correlated issues in CLI `annactl status` and TUI
✅ **Deterministic**: Uses rule-based correlation, no LLM hallucination

### What It Does NOT Do Yet

❌ **Deep Historian Integration**: No timeline analysis, no trend detection
❌ **Complex Multi-Issue Timelines**: Can't track "Service A failed, then disk filled, then OOM"
❌ **Security Correlation**: No intrusion detection, no anomaly patterns
❌ **Predictive Analysis**: No forecasting, no "disk will fill in 3 days"
❌ **Root Cause Inference**: Limited to surface correlation, not deep causality
❌ **Automated Remediation Triggers**: Detects issues but doesn't auto-fix

### How Proactive Issues Show Up Today

**CLI (`annactl status`)**:
```
[PROACTIVE ANALYSIS]
- High: Service failure correlated with log errors (confidence: 0.85)
- Medium: Disk pressure detected in /var/log (confidence: 0.72)
```

**TUI**:
- Health indicator shows overall score
- Proactive issues listed in conversation area
- No dedicated proactive panel yet

**Natural Language**:
- "What are the current issues?" → triggers proactive summary
- "Show me correlated problems" → lists proactive issues

### Integration Points

- **Brain Analysis**: Proactive engine consumes brain diagnostic insights
- **Health Computation**: Feeds into overall health score
- **Daily Snapshot**: Uses session deltas for temporal context (limited)

### Maturity Assessment

**Prototype-to-Partial**: Core logic works, but:
- Limited real-world testing
- Shallow integration with historian (not implemented yet)
- No validation of correlation accuracy
- No false positive/negative measurement

---

## 5. Network Story Status

### What Is Done (Beta.265-273)

✅ **Priority Ranking**:
- USB Ethernet vs WiFi problem solved (Beta.265)
- Deterministic scoring based on speed, latency, packet loss, link state

✅ **Degradation Detection**:
- Identifies slow/degraded interfaces
- Detects packet loss patterns
- Flags interfaces with errors

✅ **Routing Issues**:
- Detects missing default route
- Identifies routing table problems

✅ **Remediation Hints**:
- Suggests `ip link set up` for down interfaces
- Recommends driver checks for degraded links
- Provides `nmcli` commands for NetworkManager users

✅ **NL Integration** (Beta.268):
- "What's my network status?" → diagnostic route
- "Network problems?" → diagnostic route
- "Which interface is fastest?" → network summary

✅ **Composer Integration** (Beta.267, 272):
- Sysadmin answer composers for network queries
- Deterministic formatting
- Citation of source data

### Known Limitations

⚠️ **No Active Probing**: Doesn't test connectivity, relies on existing metrics
⚠️ **No DNS Analysis**: Doesn't check DNS resolution or nameserver health
⚠️ **No Bandwidth Testing**: Relies on reported link speed, not actual throughput
⚠️ **No VPN Detection**: Doesn't understand VPN interfaces specially
⚠️ **No Wireless Signal Strength**: Doesn't parse `iw` output for WiFi quality
⚠️ **No Flapping Detection**: Doesn't track interface up/down events over time

### Real-World Validation Status

**Tested**:
- Basic priority ranking logic
- Static interface comparison

**Untested**:
- Actual USB Ethernet vs WiFi switch on a real machine
- Packet loss under real network stress
- Interface flapping scenarios
- Multi-interface routing with complex topology

### Risk Level: Medium

- Core logic is deterministic and safe
- Worst case: Wrong priority recommendation (user can override)
- No automatic changes to network config (read-only analysis)

---

## 6. TUI Status

### Current TUI Capabilities (Beta.231, 274)

✅ **Stable Layout**: Grid-based layout, no overlap, responsive to terminal size
✅ **Health Coherence**: Consistent health wording across welcome, status, and brain panels
✅ **Welcome Message**: Session-aware welcome with system health summary
✅ **Exit Message**: Friendly goodbye with session summary
✅ **Brain Panel**: Displays diagnostic insights from brain analysis
✅ **Proactive Indicators**: Shows health score and top issues
✅ **Conversation Area**: Natural language query/response in scrollable view
✅ **Input Handling**: Deterministic routing of user queries

### What Is Missing (Level 3 TUI Vision)

❌ **Tabs**: No tab navigation for different views (logs, metrics, history)
❌ **Live Logs**: No scrollable journal view inside TUI
❌ **Graphs**: No CPU/memory/disk graphs (no charting library integrated)
❌ **Contextual Panels**: No drill-down panels for specific subsystems
❌ **Historical View**: No timeline or event log browser
❌ **Interactive Tables**: No sortable/filterable tables for processes, services, etc.
❌ **Keybinding Help**: No F1/? help screen documenting shortcuts
❌ **Multi-Pane Layout**: No split-screen for simultaneous views

### TUI Interaction Model

**Current**:
- User types natural language query → Anna responds in conversation area
- Health status shown in header
- Welcome/exit messages provide context

**Future (Level 3)**:
- Tab 1: Conversation (current behavior)
- Tab 2: Live metrics (graphs)
- Tab 3: Journal logs
- Tab 4: System timeline
- Tab 5: Network topology
- Keybindings for quick navigation

### Maturity Assessment

**Partial**: Solid foundation for conversational interface, but far from the rich sysadmin dashboard vision.

**Confidence**: Medium (layout is stable, but advanced features untested)
**Risk**: Medium (complex TUI widgets are hard to get right)

---

## 7. Risk and Unknowns

### Areas Without Real-World Testing

#### System Under Load
- ❓ **High CPU**: How does Anna behave when CPU is pegged at 100%?
- ❓ **Memory Pressure**: Does Anna handle OOM killer scenarios gracefully?
- ❓ **Disk I/O Storm**: Can Anna detect I/O bottlenecks?
- ❓ **Many Services Failing**: What if 10 services crash simultaneously?

#### Network Instability
- ❓ **Flapping Interfaces**: USB Ethernet plugged/unplugged repeatedly
- ❓ **Packet Loss**: High packet loss (>10%) on primary interface
- ❓ **DNS Failures**: All nameservers unresponsive
- ❓ **Routing Changes**: Default route changing mid-session

#### Edge Cases
- ❓ **Kernel Upgrades**: How does Anna handle kernel version changes?
- ❓ **Bad USB Adapters**: USB Ethernet that reports high speed but delivers low throughput
- ❓ **Corrupted Logs**: Journal corruption or massive log files (>10GB)
- ❓ **Terminal Resize**: TUI behavior when terminal resized to extreme dimensions (80x24 vs 300x100)
- ❓ **SSH Sessions**: TUI rendering over SSH with high latency

#### Security & Anomaly
- ❓ **SSH Intrusion Attempts**: Failed login attempts, brute force patterns
- ❓ **Suspicious Processes**: Unknown binaries, high privilege escalation
- ❓ **Firewall Changes**: Unexpected port openings
- ❓ **File System Changes**: Unauthorized modifications in /etc

#### Proactive Correlation
- ❓ **Multi-Issue Scenarios**: Service failure + disk pressure + network degradation
- ❓ **False Positives**: Does proactive engine over-correlate unrelated issues?
- ❓ **False Negatives**: Does it miss real correlations?

### Known Code Risks

1. **Pre-existing Build Error**: `proactive_health_score` field missing in some test code (unresolved)
2. **TUI Rendering**: Complex layout logic may have edge case bugs
3. **Network Priority Scoring**: Untested with real USB Ethernet vs WiFi
4. **Proactive Correlation**: Limited validation of accuracy
5. **Historian Integration**: Not implemented (high-risk dependency for future features)

### Data Risks

- **No Baseline**: Anna doesn't know "normal" for your system (no historical data yet)
- **No Anomaly Detection**: Can't identify unusual patterns without baseline
- **No Trend Analysis**: Can't predict future problems without time series data

---

## 8. Roadmap Skeleton

### High-Level Remaining Work (Grouped into 8 Chunks)

#### Chunk 1: Historian & Timeline (10-15 betas)
**Objectives**:
- Persistent event storage (SQLite or similar)
- Timeline queries ("what happened yesterday?", "show me last week's errors")
- Trend detection (disk usage over time, service reliability)
- Baseline computation (normal vs abnormal)

**Risk**: High (complex state management, data schema evolution, query performance)

#### Chunk 2: Proactive Engine Maturity (10-12 betas)
**Objectives**:
- Deep historian integration for temporal correlation
- Multi-issue timeline analysis
- Root cause inference (beyond surface correlation)
- False positive/negative measurement and tuning

**Risk**: High (correlation accuracy hard to validate, needs real-world data)

#### Chunk 3: Auto-Remediation Framework (10-15 betas)
**Objectives**:
- Safe execution framework for recipes
- Approval workflow (interactive vs autonomous modes)
- Rollback capability
- Audit logging
- Integration with proactive engine (auto-trigger remediation)

**Risk**: High (executing commands is inherently risky, needs extensive testing)

#### Chunk 4: Security & Anomaly Detection (10-15 betas)
**Objectives**:
- SSH intrusion pattern detection
- Firewall change monitoring
- Suspicious process detection
- File integrity monitoring (/etc, /boot)
- Correlation with proactive engine

**Risk**: High (false positives are unacceptable, baseline required)

#### Chunk 5: Network Auto-Tuning (5-8 betas)
**Objectives**:
- Active connectivity probing
- DNS resolution testing
- Bandwidth measurement
- Flapping detection and mitigation
- VPN-aware routing
- Wireless signal strength monitoring

**Risk**: Medium (read-only analysis is safe, but active probing needs care)

#### Chunk 6: Performance Profiling Tools (10-12 betas)
**Objectives**:
- CPU profiling (per-process, per-core)
- I/O bottleneck detection
- Memory leak detection
- Disk I/O patterns
- Runaway process analysis
- Integration with proactive engine

**Risk**: Medium (mostly read-only, but parsing /proc and perf data is complex)

#### Chunk 7: Level 3 TUI (20-25 betas)
**Objectives**:
- Tab-based navigation
- Live log viewer (scrollable journal)
- Metrics graphs (CPU, memory, disk, network)
- Interactive tables (processes, services, sorted/filtered)
- Historical timeline browser
- Contextual drill-down panels
- Keybinding help system

**Risk**: High (complex UI logic, terminal compatibility, performance with live data)

#### Chunk 8: Installation & Recovery (10-15 betas)
**Objectives**:
- Installation wizard (AUR, manual)
- Bootstrap mode (first-run setup)
- Recovery mode (system diagnostics when broken)
- Update mechanism (safe daemon restart)
- Uninstall script
- Configuration migration

**Risk**: Medium (install scripts are tricky, but limited scope)

### Total Estimated Remaining Work

**~100 betas** across 8 chunks, assuming current pace of 2-3 betas per day.

**Timeline**: 40-60 days of active development.

**Caveats**:
- This assumes no major architectural rewrites
- Real-world testing will uncover bugs requiring fixes
- User feedback may shift priorities
- Some chunks may require more betas than estimated

### What Is NOT on This Roadmap

- Multi-distro support (Arch-only for now)
- Remote management (local-only)
- LLM-based reasoning (deterministic rules only)
- Cloud integration (no telemetry upload)
- GUI (TUI only)

---

## 9. Conclusion

### Where Anna Actually Is

Anna is a **working prototype of a proactive Arch Linux sysadmin assistant** with:
- Strong NL routing (87% accuracy)
- Mature diagnostic engine (9 rules)
- Partial proactive engine (correlation working, shallow integration)
- Partial network diagnostics (priority ranking, degradation detection)
- Partial TUI (conversational interface, missing Level 3 features)
- No historian, no auto-remediation, no security detection

### What This Means for a User

**You can use Anna today to**:
- Get system health summaries (`annactl status`)
- Ask natural language questions ("what's wrong?", "check my system")
- View diagnostic insights in the TUI
- Get remediation command suggestions
- Monitor proactive health score

**You cannot yet**:
- View historical trends
- Get auto-remediation
- Detect security intrusions
- See performance graphs in TUI
- Trust network priority recommendations without validation

### Honesty About Testing

**Most of Anna is untested in real-world scenarios.**

The code is deterministic and safe (read-only analysis, no auto-execution), but edge cases will emerge when used on a real system under stress.

**Testing is the next critical phase.**

### Next Steps

Before adding more features (like Beta.278 Sysadmin Report), we should:
1. ✅ Document the current state (this document)
2. ✅ Create a testing strategy (see TESTING_STRATEGY_V1.md)
3. ⏳ Execute Level 0 and Level 1 testing on a real Arch system
4. ⏳ Fix bugs discovered during testing
5. ⏳ Build confidence in existing features before expanding

---

**End of Project Status Overview**

**Next Document**: See `TESTING_STRATEGY_V1.md` for a concrete testing plan.
