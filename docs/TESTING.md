# Anna Assistant - Testing Guide

**Version: 0.0.45**

This document describes how to run Anna's test suites and interpret results.

---

## Quick Start

```bash
# Run all unit tests
cargo test

# Run deep test harness (comprehensive CLI testing)
./scripts/anna_deep_test.sh

# Run with release build
./scripts/anna_deep_test.sh --release
```

---

## Test Suites

### 1. Unit Tests (Rust)

Standard Rust unit tests for individual modules.

```bash
# Run all tests
cargo test

# Run tests for a specific crate
cargo test -p anna_common
cargo test -p annactl
cargo test -p annad

# Run tests with output
cargo test -- --nocapture

# Run a specific test
cargo test test_doctor_selection
```

**Key test modules:**
- `anna_common::doctor_registry` - Doctor selection logic
- `anna_common::tools` - Tool catalog
- `anna_common::policy` - Policy engine
- `anna_common::redaction` - Secrets redaction
- `annactl::pipeline` - Request pipeline

### 2. Deep Test Harness

The deep test harness (`scripts/anna_deep_test.sh`) performs comprehensive CLI testing and produces a detailed artifact directory.

```bash
./scripts/anna_deep_test.sh [--release]
```

**Output:** `anna-deep-test-YYYYMMDD-HHMMSS/`

**Contents:**
| File | Description |
|------|-------------|
| `REPORT.md` | Human-readable test report |
| `report.json` | Machine-readable test data |
| `environment.txt` | System environment capture |
| `transcripts/` | Full CLI output for each test |
| `cases/` | Copies of case files (sanitized) |

**Test Categories:**

#### A) Translator Stability (50 queries)
- Runs 50 small read-only queries in a loop
- Counts: parse success, retry success, deterministic fallback
- Target: <20% fallback rate

#### B) Read-Only Correctness
Tests that answers contain concrete, evidence-cited values:
- "what cpu do i have" - should show CPU model
- "what kernel version am i using" - should show kernel string
- "how much memory do i have" - should show RAM size
- "how much disk space is free on /" - should show free space
- "is NetworkManager running" - should show service status
- "is pipewire running" - should show audio service status
- "what is my default route" - should show gateway
- "show me recent errors" - should show journal entries

#### C) Doctor Auto-Trigger
Tests that problem descriptions route to the correct doctor:
- "wifi disconnecting" -> Network Doctor
- "no sound" -> Audio Doctor
- "boot is slow" -> Boot Doctor

#### D) Policy Gating
Tests that mutations require confirmation:
- Attempts a file deletion
- Verifies file is not deleted without confirmation

#### E) Case Files
Tests case file creation and accessibility:
- Checks `/var/lib/anna/cases/` exists
- Copies sample case files to artifact

### 3. Smoke Tests (CI)

Quick sanity checks run in CI:

```bash
./scripts/anna_test.sh all
```

Tests:
- Version output format
- Status command performance
- Help output

---

## Interpreting Results

### Deep Test Report

The `REPORT.md` contains:

1. **Summary Table**: Total tests, passed, failed, skipped
2. **Translator Stability**: LLM success rate and fallback count
3. **Per-Category Results**: Details for each test category

### Key Metrics

| Metric | Good | Warning | Bad |
|--------|------|---------|-----|
| Pass Rate | >90% | 70-90% | <70% |
| LLM Success Rate | >80% | 60-80% | <60% |
| Fallback Rate | <10% | 10-20% | >20% |

### Common Issues

**High fallback rate:**
- Ollama not running or model not loaded
- Model too slow, hitting timeouts
- Network issues with Ollama API

**Correctness failures:**
- Missing tool implementation
- Daemon not running (no snapshots)
- Wrong evidence being returned

**Doctor not triggered:**
- Keywords not matching
- Translator classifying as system_query instead of doctor_query

---

## Running Tests Locally

### Prerequisites

1. Build annactl:
   ```bash
   cargo build --release
   ```

2. Start the daemon (for snapshot-dependent tests):
   ```bash
   sudo systemctl start annad
   ```

3. Ensure Ollama is running (for LLM tests):
   ```bash
   ollama serve
   ```

### Full Test Run

```bash
# Build
cargo build --release

# Unit tests
cargo test

# Deep test
./scripts/anna_deep_test.sh --release

# View report
cat anna-deep-test-*/REPORT.md
```

---

## CI/CD Integration

Tests run automatically on:
- Every PR to main
- Every push to main

CI jobs:
1. `build` - Compile debug and release
2. `test` - Run unit tests
3. `clippy` - Lint checks
4. `fmt` - Code formatting
5. `audit` - Dependency security
6. `smoke` - CLI sanity checks
7. `hygiene` - No dead code
8. `security` - Permission checks

**Rule:** All checks must pass before merge.

---

## Adding New Tests

### Unit Tests

Add tests in the same file as the module:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_my_function() {
        let result = my_function();
        assert!(result.is_ok());
    }
}
```

### Deep Test Queries

Edit `scripts/anna_deep_test.sh` to add new queries to the appropriate test suite.

---

## Troubleshooting

### Tests hang

- Check if Ollama is running and responsive
- Check LLM timeout settings in config
- Try with `ANNA_DEBUG=1` for verbose output

### Permission denied

- Some tests require root (daemon, cases directory)
- Run with sudo or as appropriate user

### No case files

- Daemon may not have written cases yet
- Run some queries first to generate cases

---

## Files

| Path | Description |
|------|-------------|
| `scripts/anna_deep_test.sh` | Deep test harness |
| `scripts/anna_test.sh` | Smoke tests |
| `docs/TESTING.md` | This document |
| `crates/*/tests/` | Unit test files |
| `.github/workflows/ci.yml` | CI configuration |
