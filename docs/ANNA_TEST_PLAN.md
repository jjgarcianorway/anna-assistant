# Anna Test Plan

## 🔧  Test Commands

```bash
# Full workspace test
cargo test --workspace

# Individual crate tests
cargo test -p anna_common
cargo test -p annad
cargo test -p annactl

# Specific test
cargo test -p anna_common test_name
```

---

## 📋  Test Coverage by Module

### anna_common (shared library)

| Module | Tests | Status |
|--------|-------|--------|
| `answer_engine/protocol_v15` | Serialization, verdict parsing | ✅ |
| `answer_engine/protocol_v18` | Loop state, scores | ✅ |
| `answer_engine/protocol_v19` | Subproblems, mentoring | ✅ |
| `answer_engine/protocol_v21` | Hybrid pipeline, fast path | ✅ |
| `answer_engine/protocol_v22` | Fact TTL, validation | ✅ |
| `answer_engine/protocol_v23` | Dual brain, user/system facts | ✅ |
| `answer_engine/protocol_v25` | Relevance, ambiguity | ✅ |
| `answer_engine/protocol_v26` | Update progress, watchdog state | ✅ |
| `answer_engine/auto_update` | Checksum, download state | ✅ |
| `answer_engine/daemon_watchdog` | Health checks, restart logic | ✅ |
| `answer_engine/relevance_engine` | Score calculation | ✅ |
| `answer_engine/usage_tracking` | Event tracking, patterns | ✅ |
| `answer_engine/session_awareness` | Session state | ✅ |
| `answer_engine/ambiguity_resolver` | Disambiguation | ✅ |
| `answer_engine/app_awareness` | WM detection | ✅ |
| `answer_engine/default_apps` | MIME registry | ✅ |
| `answer_engine/stats_engine` | Metrics collection | ✅ |
| `answer_engine/faster_answers` | Answer caching | ✅ |
| `answer_engine/idle_learning` | Idle conditions | ✅ |
| `answer_engine/file_scanner` | Path validation | ✅ |
| `probes/*` | Individual probe parsing | ✅ |
| `types` | Config serialization | ✅ |
| `whitelist` | Command validation | ✅ |

### annad (daemon)

| Component | Tests | Status |
|-----------|-------|--------|
| HTTP server | Request/response | ✅ |
| LLM integration | Ollama calls | ✅ |
| Evidence orchestration | Probe execution | ✅ |
| Knowledge store | SQLite operations | ✅ |

### annactl (CLI)

| Component | Tests | Status |
|-----------|-------|--------|
| Argument parsing | CLI flags | ✅ |
| Daemon connection | HTTP client | ✅ |
| Status display | Health formatting | ✅ |

---

## 🧪  Test Categories

### Unit Tests (fast, isolated)

- Protocol type serialization/deserialization
- Score calculations
- State machine transitions
- Config parsing

### Integration Tests (slower, real components)

- `annactl/tests/integration_test.rs` - CLI integration
- Daemon-to-Ollama communication (requires running Ollama)

### Manual Testing Checklist

- [ ] `annactl` - REPL mode starts
- [ ] `annactl "question"` - One-shot works
- [ ] `annactl status` - Shows health info
- [ ] `annactl --version` - Shows version
- [ ] `annactl --help` - Shows help

---

## 🔄  v0.26.0 Test Coverage

### Auto-Update Module

```rust
// auto_update.rs tests
test_update_config_default()        // Default config values
test_download_state_transitions()   // State machine
test_update_progress_eta()          // ETA calculation
test_checksum_verification()        // SHA256 verification
```

### Daemon Watchdog Module

```rust
// daemon_watchdog.rs tests
test_watchdog_default()            // Default config
test_watchdog_should_check()       // Check timing
test_watchdog_record_check()       // Health recording
test_watchdog_needs_restart()      // Restart threshold
test_watchdog_restart_stats()      // Stats collection
test_overall_health()              // Health aggregation
test_healing_trace()               // Trace lifecycle
test_recent_events()               // Event history
test_health_check_result()         // Result types
```

---

## 📈  Test Results History

| Version | Tests | Passed | Failed | Date |
|---------|-------|--------|--------|------|
| v0.25.0 | 364 | 364 | 0 | 2025-11-28 |
| v0.26.0 | 75+ | 75+ | 0 | 2025-11-28 |

---

## 🚨  Known Test Limitations

1. **Ollama Required**: Integration tests need running Ollama instance
2. **Network Tests**: Auto-update tests mock GitHub API
3. **Systemd Tests**: Cannot test real systemd in CI
4. **Health Checks**: Daemon API tests need running daemon
