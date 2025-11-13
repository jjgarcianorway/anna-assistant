# Phase 2 Overview: v2.0.0 - Secure-by-Default Ops, Observability, and Recovery

**Version Target**: v2.0.0-alpha.1
**Status**: In Progress
**Started**: 2025-11-12

---

## Objectives

Phase 2 transforms Anna from a functional distributed assistant into a production-ready, observable, and self-healing system. Core focus areas:

1. **Certificate Pinning**: Enforce cryptographic identity at the TLS layer
2. **Autonomous Recovery**: Self-healing with exponential backoff and metrics
3. **Observability**: First-class monitoring with Grafana dashboards and Prometheus alerts
4. **Packaging**: Distribution via AUR and Homebrew
5. **Production Testnet**: TLS-pinned multi-node testnet with hot reload validation

---

## Milestones

### M1: Certificate Pinning Integration ✅

**Goal**: Enforce SHA256 fingerprint pinning during TLS handshakes to prevent MITM attacks.

**Implementation**:
- Custom `ServerCertVerifier` in rustls that validates peer certificates against `/etc/anna/pinned_certs.json`
- Hard-fail on mismatch with structured logs (masked fingerprints)
- Metrics: `anna_pinning_violations_total{peer="node_id"}`
- Config format:
  ```json
  {
    "peers": {
      "node1.example.com": "sha256:AABBCC...",
      "node2.example.com": "sha256:DDEEFF..."
    }
  }
  ```

**Tests**:
- Unit tests for fingerprint computation and validation
- Integration test: accept on match, reject on mismatch
- Rotation playbook test: SIGHUP reload after updating pinned_certs.json

**Documentation**:
- `docs/CERTIFICATE_PINNING.md` with OpenSSL commands for fingerprint extraction
- Rotation playbook with zero-downtime steps

**Acceptance Criteria**:
- [ ] Custom cert verifier implemented and integrated
- [ ] Config file loaded and validated on startup
- [ ] Metrics emitted on violations
- [ ] Integration tests pass (accept/reject scenarios)
- [ ] Documentation complete with examples

---

### M2: Autonomous Recovery Supervisor ✅

**Goal**: Self-healing for transient failures in outbound peer RPCs and internal tasks.

**Implementation**:
- Lightweight supervisor in `crates/annad/src/supervisor/`
- Exponential backoff with jitter for failed tasks
- Backoff config: floor (100ms), ceiling (30s), jitter (±25%)
- Restart policies per task type (immediate, backoff, circuit-breaker)

**Metrics**:
- `anna_recovery_actions_total{action="restart|backoff|circuit_open", result="success|failure"}`
- `anna_task_restart_total{task="consensus|rpc_client|gossip"}`
- `anna_backoff_duration_seconds` histogram

**Tests**:
- Deterministic backoff tests (mocked time)
- Integration test: simulate transient peer failure, verify backoff + recovery
- Circuit breaker test: repeated failures open circuit, manual reset closes

**Acceptance Criteria**:
- [ ] Supervisor module with backoff logic
- [ ] Config schema for backoff parameters
- [ ] Metrics integration
- [ ] Unit tests for backoff math
- [ ] Integration test with transient failures

---

### M3: Observability Pack ✅

**Goal**: Production-ready monitoring with pre-built dashboards and alerts.

**Implementation**:

1. **Grafana Dashboards** (`observability/grafana/`):
   - `anna-overview.json`: System health, uptime, request rates
   - `anna-tls.json`: Handshakes, pinning violations, cert expiry
   - `anna-consensus.json`: Rounds, votes, latencies
   - `anna-rate-limiting.json`: Burst/sustained violations, backoff histograms

2. **Prometheus Alerts** (`observability/alerts/`):
   - `anna-critical.yml`:
     - `AnnaDaemonDown`: Socket unavailable for >1m
     - `AnnaPinningViolations`: Violations > 0
   - `anna-warnings.yml`:
     - `AnnaRateLimitSpike`: >10 violations/min
     - `AnnaConsensusSlow`: Round time p99 > 5s

3. **Documentation** (`docs/OBSERVABILITY.md`):
   - Import steps for dashboards
   - Alert tuning guide
   - Metric reference table

**Acceptance Criteria**:
- [ ] 4 Grafana dashboards committed
- [ ] 2 alert rule files committed
- [ ] OBSERVABILITY.md with import steps
- [ ] Screenshots in docs/screenshots/

---

### M4: Install, Self-Update, and Packaging ✅

**Goal**: Distribution via package managers and secure self-update.

**Implementation**:

1. **Installer Verification**:
   - Validate `scripts/install.sh` fails closed on SHA256 mismatch
   - Test with tampered binary to verify rejection

2. **Self-Update Check**:
   - `annactl self-update --check`: Dry-run mode, no changes applied
   - Exit codes: 0 (up-to-date), 1 (update available), 2 (error)
   - Output: JSON with current/latest versions and changelog URL

3. **AUR Package** (`packaging/aur/anna-assistant-bin/`):
   - `PKGBUILD` using release artifacts
   - SHA256 checksums from GitHub release
   - systemd service integration
   - Post-install: create anna group, enable service

4. **Homebrew Formula** (`packaging/homebrew/anna-assistant.rb`):
   - Tap: `jjgarcianorway/anna`
   - Formula downloads release binaries
   - Caveats: manual service start instructions
   - Checksum placeholders with update script

5. **Documentation** (`docs/PACKAGING.md`):
   - Maintainer steps to bump versions
   - Checksum generation commands
   - Release checklist

**Acceptance Criteria**:
- [ ] Installer SHA256 validation tested
- [ ] `annactl self-update --check` implemented with exit codes
- [ ] AUR PKGBUILD committed and validated
- [ ] Homebrew formula committed with update script
- [ ] PACKAGING.md complete

---

### M5: TLS-Pinned Testnet with Hot Reload ✅

**Goal**: Validate pinning, SIGHUP reload, and rotation in multi-node scenario.

**Implementation**:

1. **Compose File** (`testnet/docker-compose.pinned.yml`):
   - 3 nodes with mTLS
   - Mounted CA, per-node certs, and `pinned_certs.json`
   - Health checks on unix socket

2. **Test Script** (`testnet/scripts/run_tls_pinned_rounds.sh`):
   - **Round 1**: Start nodes, verify handshakes succeed
   - **Round 2**: SIGHUP reload with updated peer cert, verify accept
   - **Round 3**: Inject mismatched fingerprint in `pinned_certs.json`, verify hard-fail
   - **Round 4**: Fix fingerprint, SIGHUP reload, verify success
   - Artifacts: logs, metrics snapshots, pinning violation events → `./artifacts/testnet-pinned/`

3. **Validation**:
   - Parse logs for pinning violations
   - Query metrics endpoint for `anna_pinning_violations_total`
   - Assert connection refused on mismatch
   - Assert connection accepted after rotation

**Acceptance Criteria**:
- [ ] docker-compose.pinned.yml committed
- [ ] run_tls_pinned_rounds.sh executable and tested
- [ ] Artifacts directory structure defined
- [ ] README in testnet/ with usage instructions
- [ ] Script exits 0 on success, non-zero on failure

---

### M6: CI/CD Enhancements ✅

**Goal**: Automated validation for all Phase 2 features in CI.

**Implementation**:

1. **Matrix Job** (`.github/workflows/consensus-smoke.yml`):
   ```yaml
   strategy:
     matrix:
       test: [unit, integration, testnet-pinned]
   ```
   - `unit`: cargo test --workspace
   - `integration`: cargo test --test integration_*
   - `testnet-pinned`: run_tls_pinned_rounds.sh headless

2. **Security Guards**:
   - Pre-build: `git ls-files | grep -E '\.(key|pem|srl)$'` must return empty
   - Enforce on all pushes to main

3. **Version Validation** (on tag events):
   - Extract tag version (e.g., v2.0.0-alpha.1)
   - Build binaries, verify `annactl --version` matches tag

4. **Cargo Caching**:
   - Cache `~/.cargo` and `target/` with key based on Cargo.lock hash
   - Expected speedup: 3-5x on cache hit

**Acceptance Criteria**:
- [ ] Matrix job defined with 3 variants
- [ ] TLS material guard enforced
- [ ] Version validation on tags
- [ ] Cargo cache implemented
- [ ] CI green on main after merge

---

## Acceptance Criteria (Overall)

| Criterion | Status | Evidence |
|-----------|--------|----------|
| Certificate pinning enforced during TLS handshake | ⏳ | Integration test passes, metrics emitted |
| Supervisor restarts tasks with backoff | ⏳ | Unit tests pass, integration test validates recovery |
| Grafana dashboards and Prometheus alerts present | ⏳ | Files committed, docs/OBSERVABILITY.md |
| Installer and self-update behave as specified | ⏳ | Manual test on Rocinante, exit codes validated |
| AUR and Homebrew packaging skeletons added | ⏳ | packaging/ directory, docs/PACKAGING.md |
| TLS-pinned testnet produces artifacts | ⏳ | ./artifacts/testnet-pinned/, logs show violations |
| CI matrix green on main | ⏳ | GitHub Actions badge, test results |
| CHANGELOG bumped to 2.0.0-alpha.1 | ⏳ | Concise section per milestone |

---

## Timeline

| Milestone | Target Date | Actual |
|-----------|-------------|--------|
| M1: Certificate Pinning | 2025-11-13 | TBD |
| M2: Recovery Supervisor | 2025-11-14 | TBD |
| M3: Observability Pack | 2025-11-15 | TBD |
| M4: Packaging | 2025-11-16 | TBD |
| M5: TLS-Pinned Testnet | 2025-11-17 | TBD |
| M6: CI/CD Enhancements | 2025-11-18 | TBD |
| **v2.0.0-alpha.1 Release** | **2025-11-19** | TBD |

---

## References

- [OWASP: Transport Layer Protection](https://cheatsheetseries.owasp.org/cheatsheets/Transport_Layer_Protection_Cheat_Sheet.html)
- [rustls: Custom Certificate Verification](https://docs.rs/rustls/latest/rustls/client/trait.ServerCertVerifier.html)
- [Prometheus: Alerting Best Practices](https://prometheus.io/docs/practices/alerting/)
- [Grafana: Dashboard Best Practices](https://grafana.com/docs/grafana/latest/dashboards/build-dashboards/best-practices/)
- [Arch Wiki: PKGBUILD](https://wiki.archlinux.org/title/PKGBUILD)
- [Homebrew: Formula Cookbook](https://docs.brew.sh/Formula-Cookbook)

---

**Last Updated**: 2025-11-12
**Maintained By**: Anna Assistant Team
