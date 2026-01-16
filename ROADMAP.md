# Anna Roadmap

## Completed Phases

### Phase 1-9: Foundation
- Basic query pipeline
- Probe system
- LLM integration (Ollama)
- Recipe learning
- Session memory

### Phase 10: Specialist System
- 8 domains x 2 levels = 16 specialists
- Deterministic routing
- Escalation rules

### Phase 11-12: Trust Boundaries
- Exposure levels (Silent/Summary/Dialogue/Debug)
- ExposureGate centralized filtering
- Forbidden pattern sanitization

### Phase 13-14: Production Hardening
- E2E smoke tests
- Error recovery tests
- Performance budgets

### Phase 15: Answer Contract (v0.3.47-48)
- FinalAnswer sanitization
- No manual commands in output
- Power management routing fix

### Phase 16: Action Execution (v0.3.49)
- ActionPlan structure
- Template plans (GDM, sleep, lid)
- pkexec privilege escalation
- User confirmation flow

### Phase 17: Verification & Rollback (v0.3.50)
- Pre-execution state capture
- Per-step verification
- Automatic rollback on failure
- Idempotency (preflight checks)
- 400-line gate enforcement

### Phase 18: UX Polish (v0.3.50)
- Calmer prefix ("Anna:" instead of "ANSWER:")
- Concise error messages
- Consistent status indicators
- Silent spinners allowed

### Phase 19: UX Regression Lock (v0.3.52)
- Canonical UX spec (docs/UX_SPEC.md)
- Snapshot tests for rendering
- Golden transcript fixtures
- Pattern-based contract validation
- CI-integrated regression gate

### Phase 20: User Experience Reality Pass (v0.3.53)
- Golden fixtures for Silent/Summary/Debug exposure levels
- Deterministic streaming termination
- Plan confirmation lifecycle edge cases (TTL, invalid input, expired)
- CI hygiene verification

### Phase 21: Status as a Product (v0.3.54)
- STATUS_SPEC.md canonical contract
- 7 deterministic sections: VERSION, UPDATES, SERVICE, PERMISSIONS, CONFIG, HELPERS, MODELS
- Exposure-gated visibility
- Status golden fixtures (healthy, daemon_down, no_group, no_updates)

### Phase 22: Reality Pass: Read-Only Intelligence (v0.3.55)
- READ_ONLY vs MUTATING intent classification
- Answer contract enforcement (no "would you like" for READ_ONLY)
- Probe deduplication via ProbeLedger
- Iteration limits (READ_ONLY: 3, MUTATING: 5)
- Evidence formatting rules (max 3 items)
- Streaming heartbeat for long operations
- Transcript contamination fixes
- Golden fixtures for swap/audio/thermal/bluetooth scenarios

### Phase 23: Truthful Telemetry (v0.3.56)
- Outcome ledger at /var/lib/anna/outcomes.jsonl
- OutcomeRecord: ts_utc, request_id, mode, intent, outcome, escalated, duration_ms
- Stats aggregated from ledger (no fake numbers)
- Removed XP bars, titles, fake reliability percentages
- Unknown values show [!] indicator

### Phase 24: Confidence Calibration (v0.3.57)
- telemetry_consumer.rs: rolling window aggregation
- policy.rs: pure dial-setting functions from aggregates
- Iteration limits modulated by success rate
- Confidence phrasing modulated by track record
- PolicyBasis step type for Debug-mode tracing
- Cold start conservative defaults

### Phase 25: Execution Safety and Reversibility Hardening (v0.3.58)
- PreflightResult, VerificationStatus, Reversibility enums
- Verification strictness: Unknown = Failed
- Elevated confirmation for non-reversible actions
- Outcome recording for all action paths
- Telemetry enrichment: preflight, verification, elevated_confirmation

### Phase 26: Decision Quality and Abstention Discipline (v0.3.59)
- Abstained outcome: low confidence + no error = neutral outcome
- AbstentionReason enum: LowConfidence, InsufficientEvidence, ConflictingData
- Success rate excludes abstained (denominator = resolved + failed only)
- Abstention stats in `annactl stats` output
- AskResult extended with abstained flag and final_confidence

### Phase 27: Evidence-Driven Control Policy (v0.3.60)
- ProbeEffectivenessRecord: resolution_rate, abstention_rate, avg_duration_ms
- FactFingerprint: xxhash64 of normalized probe outputs
- EvidenceSnapshot: probes_run, evidence_completeness, fingerprint
- Computed confidence: 0.4*completeness + 0.3*similarity + 0.3*history
- adaptive_iteration_budget(): slow probe detection, hard cap at 6
- OutcomeRecord: probes_used, fingerprint_hash fields
- Cold start protection: <50 outcomes uses LLM confidence

## Frozen (Not Planned)

- No new specialists
- No stats expansion
- No cloud integrations
- No telemetry

## Future Phases

### Phase 28: Model Selection Benchmark + Hardware-Aware Routing
- Model performance benchmarking
- Hardware capability detection
- Automatic model selection based on query complexity
- Memory-aware routing
