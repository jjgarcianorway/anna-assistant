# Telemetry Specification

Phase 23: Truthful Telemetry

## Outcome Ledger

Location: `/var/lib/anna/outcomes.jsonl`

### Record Format

```jsonl
{"ts_utc":"2026-01-15T10:00:00Z","request_id":"uuid","mode":"DIALOGUE","intent":"READ_ONLY","outcome":"resolved","escalated":false,"duration_ms":150}
```

### Fields

| Field | Type | Values |
|-------|------|--------|
| ts_utc | RFC3339 | UTC timestamp |
| request_id | UUID | Unique per request |
| mode | enum | DIALOGUE, ACTION |
| intent | enum | READ_ONLY, MUTATING |
| outcome | enum | resolved, failed, cancelled, expired |
| escalated | bool | Whether escalation occurred |
| duration_ms | u64 | Total request duration |

### Outcome Definitions

- **resolved**: Answer delivered and accepted
- **failed**: Execution attempted but errored
- **cancelled**: User rejected action plan
- **expired**: Request timed out or TTL expired

### Invariants

1. Exactly one record per request
2. Append-only (no edits, no deletes)
3. All stats derived from ledger (no invented numbers)

## Stats Display

`annactl stats` shows:

- **REQUESTS**: total, read_only, mutating
- **TIMING**: avg, p50, p90 (detailed mode)
- **ESCALATION**: total, rate%
- **OUTCOMES**: resolved, failed, cancelled, expired, success rate

Unknown values display `[!]` instead of fake numbers.
