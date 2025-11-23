# Beta.278: Sysadmin Report v1 - Full System Briefing Using Existing Machinery

**Status**: Implemented
**Version**: 5.7.0-beta.278
**Date**: 2025-11-23

---

## Problem Statement and Goals

**Problem**: Users need a quick, comprehensive overview of their entire system state without running multiple separate queries. While Anna provides excellent domain-specific diagnostics (health, network, disk), there was no single "give me everything" briefing that combines all subsystems into one view.

**Goals**:
1. Add full sysadmin report capability via natural language queries
2. Combine health diagnostics, proactive issues, session deltas, and domain highlights into one canonical report
3. Implement using **only NL routing** - no new CLI commands, flags, or TUI keybindings
4. Keep report concise (20-40 lines) and actionable
5. Reuse existing data sources and machinery - no new RPC methods

---

## Sysadmin Report Intent

The sysadmin report triggers when users ask for a **comprehensive system briefing** using natural language queries like:

**Triggering Queries** (~60 patterns detected):
- "give me a full system report"
- "sysadmin report"
- "show me everything"
- "overall situation"
- "summarize the system"
- "full diagnostic report"
- "complete system briefing"
- "how is my machine overall"
- "what's the full picture"

**Non-Triggering Queries** (to preserve existing routing):
- "check my health" → routes to health diagnostic
- "check my network" → routes to network diagnostic
- "what is systemd" → routes to educational answers
- "what should I fix first" → routes to proactive remediation

The routing is **conservative** to avoid false positives and preserve existing specialized query handlers.

---

## Before/After Examples

### Example 1: Full System Report

**Query**:
```bash
$ annactl "full system report"
```

**Output**:
```
[SUMMARY]
Overall status: stable with warnings, 0 critical and 2 warning level issues detected.

[HEALTH]
⚠ Root partition at 85% capacity
⚠ Memory usage at 88% (14.1GB / 16GB)

[SESSION]
Kernel unchanged: 6.17.8-arch1-1
No packages updated since last session
No reboot required

[PROACTIVE]
Health score: 82/100
⚠ High memory usage correlated with disk I/O spikes (confidence: 87%)
ℹ Nightly backup jobs timing out occasionally (confidence: 72%)

[KEY DOMAINS]
• Disk: Root partition nearing capacity threshold
• Resources: Memory usage elevated but stable

[COMMANDS]
# For detailed health analysis
$ annactl "check my system health"

# To focus on specific domains
$ annactl "check my network"
$ annactl "check my disk"
```

### Example 2: Domain-Specific Query (Unchanged Behavior)

**Query**:
```bash
$ annactl "how is my system today"
```

**Output** (routes to existing health diagnostic):
```
[DIAGNOSTIC SUMMARY]
✓ Services: All critical services running
⚠ Disk: Root partition at 85% capacity
✓ Network: Connectivity stable
⚠ Resources: Memory usage elevated

[RECOMMENDATIONS]
1. Consider cleaning up disk space on root partition
2. Monitor memory usage patterns

[COMMANDS]
$ annactl "show me disk usage"
$ annactl "what's using my memory"
```

**Key Difference**: "how is my system today" routes to health diagnostic (focused analysis), while "full system report" includes proactive issues, session deltas, and domain highlights.

---

## Implementation Notes

### Architecture

**Routing Location**: `crates/annactl/src/unified_query_handler.rs`
- **Function**: `is_sysadmin_report_query()` (lines 1237-1367)
- **Routing Tier**: TIER 0.7 (after proactive remediation, before recipes)
- **Detection**: ~60 exact multi-word patterns + combined keyword patterns
- **Conservative matching**: Avoids overriding health/network/educational queries

**Composer Location**: `crates/annactl/src/sysadmin_answers.rs`
- **Function**: `compose_sysadmin_report_answer()` (lines 1852-2060)
- **Signature**:
  ```rust
  pub fn compose_sysadmin_report_answer(
      brain: &BrainAnalysisData,
      daily_snapshot_text: Option<&str>,
      proactive_issues: &[anna_common::ipc::ProactiveIssueSummaryData],
      proactive_health_score: u8,
  ) -> String
  ```

**Handler Location**: `crates/annactl/src/unified_query_handler.rs`
- **Function**: `handle_sysadmin_report_query()` (lines 1852-1911)
- **Async RPC calls**: Fetches brain analysis via `Method::BrainAnalysis`
- **Session data**: Loads last session metadata and computes daily snapshot
- **Returns**: `UnifiedQueryResult::ConversationalAnswer` with high confidence

### Data Sources

All data is **reused from existing subsystems**:

1. **BrainAnalysisData** (via `Method::BrainAnalysis` RPC):
   - Diagnostic insights (health, services, disk, network, resources)
   - Critical/warning counts
   - Proactive issues with confidence scores
   - Proactive health score

2. **Session Metadata** (via `load_last_session()` and `compute_daily_snapshot()`):
   - Kernel version changes
   - Package updates since last session
   - Reboot requirements

3. **Severity Markers** (canonical formatting):
   - `✗` critical
   - `⚠` warning
   - `ℹ` info

### Report Structure

**Canonical Format** (6 sections, 20-40 lines):

1. **[SUMMARY]**: One-liner overall status (healthy/degraded/stable with warnings)
2. **[HEALTH]**: Top 3 diagnostic insights, sorted by severity (critical first)
3. **[SESSION]**: Daily snapshot if available (first 5 lines)
4. **[PROACTIVE]**: Health score + top 3 correlated issues with confidence
5. **[KEY DOMAINS]**: Highlights from services, disk, network, resources (max 5)
6. **[COMMANDS]**: Next actions based on health state (critical vs warning vs healthy)

**Constraints**:
- Max 3 insights shown (remaining count displayed)
- Max 3 proactive issues shown (remaining count displayed)
- Priority: Critical severity before warning
- No internal types leaked (user-friendly wording)

---

## Testing Summary

**Test Suite**: `crates/annactl/tests/regression_sysadmin_report_beta278.rs`
**Total Tests**: 26 (8 routing, 12 content, 6 formatting)

### Routing Tests (8)
1. `test_routing_sysadmin_report_exact_phrases` - Core exact phrases
2. `test_routing_full_system_report` - "full system report" variants
3. `test_routing_overall_situation` - "overall situation" variants
4. `test_routing_summarize_variants` - "summarize" imperatives
5. `test_routing_give_me_imperatives` - "give me" patterns
6. `test_routing_show_me_imperatives` - "show me" patterns
7. `test_routing_overview_phrases` - "overview" patterns
8. `test_routing_should_not_match` - Negative cases (health/network/educational)

### Content Tests (12)
1. `test_content_healthy_system_all_clear` - Healthy system reporting
2. `test_content_degraded_system_shows_issues` - Degraded system with critical/warning
3. `test_content_with_proactive_issues` - Proactive issues integration
4. `test_content_with_daily_snapshot` - Session delta display
5. `test_content_key_domains_services` - Service domain highlights
6. `test_content_key_domains_disk` - Disk domain highlights
7. `test_content_key_domains_network` - Network domain highlights
8. `test_content_commands_critical_system` - Commands for critical state
9. `test_content_commands_healthy_system` - Commands for healthy state
10. `test_content_limits_insights_to_three` - Max 3 insights constraint
11. `test_content_limits_proactive_to_three` - Max 3 proactive issues constraint
12. `test_content_priority_critical_over_warning` - Severity-based priority

### Formatting Tests (6)
1. `test_format_all_sections_present_healthy` - All 6 sections present (healthy)
2. `test_format_all_sections_present_degraded` - All 6 sections present (degraded)
3. `test_format_severity_markers` - Canonical markers (✗, ⚠, ℹ)
4. `test_format_no_internal_types_leaked` - No BrainAnalysisData, InsightData, etc.
5. `test_format_concise_report_length` - Target 20-40 lines (max 50 allowed)
6. `test_format_section_order` - Correct section ordering

**Test Execution**:
```bash
cargo test --test regression_sysadmin_report_beta278
```

All tests are **fast, small, and deterministic** (no LLM, no network, no filesystem I/O).

---

## Guarantees

1. **No New CLI Commands**: All functionality via NL routing only
2. **Deterministic Output**: Template-based composition, no LLM randomness
3. **Backwards Compatible**: Existing health/network/educational queries unchanged
4. **Concise Reports**: 20-40 lines typical, 50 lines maximum
5. **Priority-Based Display**: Critical issues always shown before warnings
6. **Data Freshness**: All data from real-time daemon analysis (no caching beyond daemon)
7. **Conservative Routing**: Won't override specific diagnostic queries

---

## Limitations

1. **NL Pattern Matching Only**: Uses ~60 hardcoded patterns, not true intent understanding
2. **Domain Extraction Heuristic**: Parses insights by category/summary, may miss edge cases
3. **No Historical Trends**: Shows current state only, no time-series analysis
4. **Max 3 Items Per Section**: Insights/proactive issues limited for brevity
5. **Session Delta Optional**: Daily snapshot only shown if previous session metadata exists
6. **No Custom Configuration**: Report structure is fixed (cannot reorder sections or customize)

---

## Future Enhancements (Not in Beta.278)

Potential improvements for future versions:
- Historical trend indicators (↑↓) for metrics
- Configurable report sections (user-defined priority)
- Export formats (JSON, YAML for automation)
- Time-range filtering for proactive issues
- Integration with external monitoring systems

---

## References

- **Specification**: PLANNING_STEP_1 → Beta.278
- **Related Docs**: `docs/PROJECT_STATUS_OVERVIEW.md`, `docs/TESTING_STRATEGY_V1.md`
- **Code Locations**:
  - Routing: `crates/annactl/src/unified_query_handler.rs:1237-1367, 164-169, 1852-1911`
  - Composer: `crates/annactl/src/sysadmin_answers.rs:1852-2060`
  - Tests: `crates/annactl/tests/regression_sysadmin_report_beta278.rs`
