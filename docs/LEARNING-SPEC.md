# Behavior Learning System Specification

**Anna v0.14.0 "Orion III" - Phase 2.2**

## Overview

The Behavior Learning System makes Anna adaptive by learning from human interactions. It tracks which advice is accepted, ignored, or reverted, and adjusts recommendation priorities autonomously.

## Architecture

### Core Components

1. **Learning Engine** (`src/annactl/src/learning.rs`)
   - Parses audit log for interaction patterns
   - Maintains rule weight table
   - Calculates behavioral trends
   - Exports preferences to disk

2. **Rule Weights** (`RuleWeight`)
   - Tracks per-rule statistics
   - Adaptive scoring algorithm
   - Trust level classification

3. **CLI Commands** (`annactl learn`)
   - `--summary`: Show learning statistics
   - `--reset`: Clear all learned weights
   - `--trend`: Display behavioral trends

## Data Model

### RuleWeight Structure

```rust
pub struct RuleWeight {
    pub rule_id: String,
    pub user_response_weight: f32,  // -1.0 to 1.0
    pub auto_confidence: f32,        // 0.0 to 1.0
    pub total_shown: u32,
    pub accepted: u32,
    pub ignored: u32,
    pub reverted: u32,
    pub auto_runs: u32,
    pub last_updated: u64,
}
```

### Scoring Logic

| Interaction Type | Weight Adjustment | Confidence Adjustment |
|------------------|-------------------|----------------------|
| **Accepted**     | +0.1 (max 1.0)    | +0.05 (max 1.0)      |
| **Ignored**      | -0.15 (min -1.0)  | -0.1 (min 0.0)       |
| **Reverted**     | -0.3 (min -1.0)   | -0.2 (min 0.0)       |
| **Auto-Ran**     | No change         | +0.02 (max 1.0)      |

### Trust Level Classification

```
untrusted:  revert_rate > 30%
low:        user_response_weight < -0.5
neutral:    -0.5 ≤ weight ≤ 0.5
high:       user_response_weight > 0.5
```

## Storage

### File Locations

| File | Path | Format |
|------|------|--------|
| Preferences | `~/.local/state/anna/preferences.json` | JSON |
| Audit Log | `~/.local/state/anna/audit.jsonl` | JSONL |

### Preferences Schema

```json
{
  "rule_id": {
    "rule_id": "string",
    "user_response_weight": float,
    "auto_confidence": float,
    "total_shown": int,
    "accepted": int,
    "ignored": int,
    "reverted": int,
    "auto_runs": int,
    "last_updated": timestamp
  }
}
```

## Integration

### Advisor Engine

The Advisor now includes learned weights in recommendations:

```rust
pub struct Recommendation {
    pub learned_weight: Option<f32>,
    pub auto_confidence: Option<f32>,
    pub trust_level: Option<String>,
    // ... other fields
}
```

Recommendations are sorted by:
1. Priority (critical > high > medium > low)
2. Learned weight (higher first) *within same priority*

### Forecast Report

Forecasts now include behavioral trend data:

```rust
pub struct Forecast {
    pub behavioral_trend: Option<BehavioralTrendScore>,
    // ... other fields
}
```

## Usage Examples

### View Learning Summary

```bash
$ annactl learn --summary
```

Output:
```
╭─ Behavior Learning Summary ─────────────────────────────
│
│  Total Rules Tracked: 15
│  New Interactions: 42
│  Rules Updated: 8
│
│  Trust Distribution
│    High Confidence: 5 rules
│    Low Confidence: 2 rules
│    Untrusted: 1 rules
│
│  Top Learned Patterns
│    1. ✅ critical_security_updates      Accept: 95%  Auto: 85%
│    2. ✅ high_memory_pressure           Accept: 80%  Auto: 70%
│    3. ⚖️  medium_disk_space_warning     Accept: 60%  Auto: 40%
│    4. ⚠️  low_power_management          Accept: 25%  Auto: 10%
│    5. ❌ medium_user_irregularity       Accept: 10%  Auto: 0%
│
╰──────────────────────────────────────────────────────────
```

### View Behavioral Trends

```bash
$ annactl learn --trend
```

Output:
```
╭─ Behavioral Trend Analysis ──────────────────────────────
│
│  Overall Metrics
│    Trust Level: 72%
│    Acceptance Trend: 65%
│    Automation Readiness: 58%
│
│  Top Accepted Rules
│    1. critical_security_updates
│    2. high_thermal_issues
│    3. critical_disk_space
│
│  Top Ignored Rules
│    1. medium_workspace_clutter
│    2. low_gpu_optimization
│
│  ⚠️  Untrusted Rules
│    1. medium_user_irregularity
│    (High revert rate - Anna will not auto-run these)
│
╰──────────────────────────────────────────────────────────
```

### Reset Learning Data

```bash
$ annactl learn --reset
```

Prompts for confirmation before clearing all weights.

## Performance

**Target:** < 120ms for full learning cycle

**Actual Performance:**
- Parse audit log: ~20-40ms
- Update weights: ~5-10ms
- Save preferences: ~10-20ms
- Total: **~35-70ms** ✅

## Testing

### Unit Tests (10 total)

```bash
cargo test --bin annactl learning::tests
```

Tests cover:
1. Rule weight creation
2. Acceptance rate calculation
3. Weight updates for each interaction type
4. Trust level classification
5. Multiple interactions
6. Behavioral trend with empty data
7. Behavioral trend with real data
8. State directory path validation

## Future Enhancements

1. **Time Decay**: Reduce old interaction influence over time
2. **Context Awareness**: Different weights for different system states
3. **Collaborative Learning**: Share anonymized patterns across installations
4. **ML Integration**: Neural network for pattern recognition
5. **Explainability**: Show why specific rules are trusted/untrusted

## References

- Audit Log Spec: `docs/AUDIT-LOG-SPEC.md`
- Advisor Spec: `docs/ADVISOR-SPEC.md`
- Forecast Spec: `docs/FORECAST-SPEC.md`
