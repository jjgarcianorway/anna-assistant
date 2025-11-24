# Legacy Recipe System (Archived)

**Status**: DEPRECATED - Not used in Anna 6.x runtime

This directory contains the old recipe-based planning system from Anna 5.x.
It has been replaced by the adaptive planner with Arch Wiki consultation.

## Why This Was Removed

The recipe system had 77+ hardcoded ActionPlan recipes that:
- Required manual maintenance for every scenario
- Could not adapt to system-specific configurations
- Lacked authoritative knowledge sources (no Arch Wiki links)
- Created multiple planning paths (confusing architecture)

## New Planning System (6.2.0+)

Anna 6.x uses a single planning path:
```
Telemetry → Arch Wiki → Planner → Plan
```

Location: `crates/anna_common/src/orchestrator/`

Components:
- `telemetry.rs` - System state summary
- `knowledge.rs` - Arch Wiki consultation
- `planner.rs` - Plan generation with safety guarantees

## This Code Is Kept Only For Reference

Do not import these modules in active code.
Do not compile them into the runtime.

If you need to understand how a specific scenario was handled in 5.x,
you can read these files, but the correct approach in 6.x is to:
1. Add a new planner slice in `orchestrator/planner.rs`
2. Add Arch Wiki knowledge in `orchestrator/knowledge.rs`
3. Extend `TelemetrySummary` if new telemetry is needed
4. Add ACTS tests proving safety guarantees
