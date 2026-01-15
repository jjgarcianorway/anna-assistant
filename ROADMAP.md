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

## Frozen (Not Planned)

- No new specialists
- No stats expansion
- No cloud integrations
- No telemetry

## Future Phases

### Phase 20: Model Selection Benchmark + Hardware-Aware Routing
- Model performance benchmarking
- Hardware capability detection
- Automatic model selection based on query complexity
- Memory-aware routing
