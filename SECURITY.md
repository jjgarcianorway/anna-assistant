# Security Policy

## Reporting Security Vulnerabilities

If you discover a security vulnerability in Anna Assistant, please report it privately:

**Email**: jjgarcianorway@gmail.com
**Subject**: [SECURITY] Anna Assistant Vulnerability Report

Please include:
- Description of the vulnerability
- Steps to reproduce
- Potential impact
- Suggested fix (if any)

We aim to respond within 48 hours and provide a fix within 7 days for critical issues.

---

## Secrets Management Policy

Anna Assistant follows strict secrets management practices:

### What We Prevent

✅ **Never commit**:
- Private keys (*.key)
- Certificates (*.pem, *.crt)
- Certificate signing requests (*.csr)
- Certificate serial numbers (*.srl)
- API tokens or credentials
- Configuration files with secrets

### How We Prevent It

1. **Gitignore Rules** (`.gitignore`):
   ```
   testnet/config/tls/
   **/*.key
   **/*.pem
   **/*.srl
   **/*.crt
   **/*.csr
   ```

2. **Pre-commit Hooks** (`.pre-commit-config.yaml`):
   - `detect-secrets` from Yelp
   - Custom TLS material blocking hooks
   - Repository-wide validation on push

3. **CI Security Guards** (`.github/workflows/consensus-smoke.yml`):
   - Pre-build check fails if TLS materials are tracked
   - Ephemeral certificate generation for tests
   - No certificates ever stored in CI artifacts

4. **Developer Workflow**:
   ```bash
   # Install pre-commit hooks
   pip install pre-commit
   pre-commit install

   # Generate test certificates locally (never commit)
   ./scripts/gen-selfsigned-ca.sh
   ```

### TLS Certificate Management

See `testnet/config/README.md` for certificate generation guidance.

**Key Principles**:
- Certificates are generated locally or ephemerally in CI
- Private keys never leave the developer's machine or CI workspace
- Production certificates use proper PKI and secret management (Vault, etc.)

---

## Security Incident History

### v1.16.1-alpha.1 (2025-11-12): TLS Materials Purge

**Incident**: GitGuardian detected committed private keys in `testnet/config/tls/`

**Remediation**:
1. Purged all TLS materials from git history using `git-filter-repo`
2. Removed 9 files: `ca.key`, `ca.pem`, `ca.srl`, `node_*.key`, `node_*.pem`
3. Implemented pre-commit and CI guards
4. Force-pushed rewritten history (all commit SHAs changed)

**Impact**: Test certificates only, no production keys exposed

**Timeline**:
- Detection: 2025-11-12 (GitGuardian alert)
- Remediation: Same day (3 hours)
- Verification: Pending GitGuardian rescan

**References**:
- CHANGELOG.md v1.16.1-alpha.1 entry
- OWASP Key Management Cheat Sheet
- GitGuardian Secrets Detection Guide

---

## Security Features

### Systemd Hardening (Phase 0.4 + Phase 3.9)

Anna runs with a strict systemd security sandbox. **Phase 3.9** adds enhanced hardening:

```ini
[Service]
# User/Group isolation
User=annad
Group=anna
SupplementaryGroups=

# Capability restrictions (Phase 3.9)
CapabilityBoundingSet=CAP_DAC_OVERRIDE CAP_CHOWN CAP_FOWNER CAP_SYS_ADMIN
AmbientCapabilities=
NoNewPrivileges=true

# File system restrictions
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/var/log/anna /var/lib/anna /run/anna

# Kernel restrictions
ProtectKernelTunables=true
ProtectKernelModules=true
ProtectKernelLogs=true
ProtectControlGroups=true

# Network restrictions
RestrictAddressFamilies=AF_UNIX AF_INET AF_INET6
IPAddressDeny=any
RestrictNamespaces=true
LockPersonality=true
RestrictRealtime=true

# System call filtering
SystemCallFilter=@system-service
SystemCallFilter=~@privileged @resources @obsolete

# Device access
DevicePolicy=closed
DeviceAllow=/dev/null rw
DeviceAllow=/dev/zero rw
DeviceAllow=/dev/urandom r
```

**To apply Phase 3.9 hardening**:

```bash
# Backup and edit service file
sudo cp /usr/lib/systemd/system/annad.service /etc/systemd/system/annad.service.backup
sudo systemctl edit --full annad.service
# (Add directives above)

# Reload and restart
sudo systemctl daemon-reload
sudo systemctl restart annad
sudo systemctl status annad
```

### Rate Limiting (Phase 1.16)

Dual-tier protection:
- Burst: 20 requests / 10 seconds
- Sustained: 100 requests / 60 seconds

### Certificate Pinning (Phase 1.16)

SHA256 fingerprint validation infrastructure (Phase 2 integration planned).

### Cryptographic Identity (Phase 1.7)

Ed25519 signatures for distributed consensus.

---

## System Hardening Guide (Phase 3.9)

### Permission Model

Anna uses a layered permission model:

#### 🟢 User-Safe Commands (No Special Permissions)
- `help`, `status`, `health`, `metrics`, `profile`, `ping`
- `learn`, `predict` (read-only analysis)
- **Risk**: None - read-only operations
- **Requirement**: User in `anna` group

#### 🟡 Advanced Commands (Root/Sudo Required)
- `init`, `update`, `install`, `doctor`, `repair`, `monitor`
- **Risk**: Medium - modifies system state
- **Requirement**: User must be in `anna` group + use sudo

#### 🔴 Internal Commands (Developer/Diagnostic)
- `sentinel`, `conscience`, `consensus`
- **Risk**: Low - diagnostic/monitoring only
- **Requirement**: Developer mode or explicit flag

### File System Security

Verify and enforce correct permissions:

```bash
# Configuration directory (root-only write)
sudo chown -R root:anna /etc/anna
sudo chmod 755 /etc/anna
sudo chmod 644 /etc/anna/*.toml

# Log directory (anna group readable - v3.9.1)
sudo chown -R root:anna /var/log/anna
sudo chmod 750 /var/log/anna
sudo chmod 640 /var/log/anna/*.jsonl

# Socket directory (anna group writable - v3.9.1)
sudo chown -R root:anna /run/anna
sudo chmod 770 /run/anna
sudo chmod 660 /run/anna/anna.sock

# State directory (anna group readable - v3.9.1)
sudo chown -R root:anna /var/lib/anna
sudo chmod 750 /var/lib/anna
sudo chmod 640 /var/lib/anna/*.db

# Reports directory (anna group writable - v3.9.1 FIX)
sudo mkdir -p /var/lib/anna/reports
sudo chown root:anna /var/lib/anna/reports
sudo chmod 770 /var/lib/anna/reports
# Set default ACL for future files
sudo setfacl -d -m g:anna:rwx /var/lib/anna/reports
```

### Socket Security

Unix domain socket with group-based access:

```bash
# Verify socket permissions
ls -la /run/anna/annad.sock
# Expected: srw-rw---- 1 annad anna

# Only users in 'anna' group can connect
id $USER | grep anna

# Add user to group
sudo usermod -aG anna $USER
newgrp anna  # Activate new group
```

### Self-Healing Safety (Phase 3.9)

**Self-healing is DISABLED by default**:

```toml
# /etc/anna/sentinel.toml
[self_healing]
enabled = false  # Must be explicitly enabled
max_actions_per_hour = 3
allow_service_restart = false
allow_package_update = false
```

**Risk Level Enforcement**:

| Risk Level | Examples | Auto-Healing |
|------------|----------|--------------|
| None | Monitoring, logging | ✅ Always safe |
| Low | Clear cache, restart non-critical services | ✅ When enabled |
| Medium | Update packages, restart critical services | ❌ Manual only |
| High | Modify config files | ❌ Manual only |
| Critical | System-wide changes | ❌ Manual only |

**Safety Guarantees**:
- No automatic actions without `enable_self_healing = true`
- Medium+ risk actions NEVER execute automatically
- All actions logged with reasoning in `/var/log/anna/actions.log`
- User can always rollback via `annactl rollback`

### Audit and Logging

All commands are logged for audit purposes:

```bash
# Command audit log (JSON format)
/var/log/anna/ctl.jsonl

# Example entry
{
  "ts": "2025-11-13T10:30:45Z",
  "req_id": "abc123",
  "state": "configured",
  "command": "update",
  "allowed": true,
  "args": ["--dry-run"],
  "exit_code": 0,
  "duration_ms": 1234,
  "ok": true,
  "citation": "[archwiki:system_maintenance]"
}

# Query recent commands
jq -r '.command' /var/log/anna/ctl.jsonl | tail -20

# Find failed commands
jq 'select(.ok == false)' /var/log/anna/ctl.jsonl

# Average command duration
jq -s 'map(.duration_ms) | add / length' /var/log/anna/ctl.jsonl
```

## Auto-Update Security (Phase 3.10)

### Installation Source Detection

Anna automatically detects how it was installed:

- **AUR/Pacman**: Auto-update **disabled** (respects package manager)
- **Manual (GitHub/curl)**: Auto-update **enabled** (safe upgrade path)

```bash
# Check installation source
sudo annactl doctor
# Shows: "Installation Source: AUR Package (anna-assistant-bin)"
#    or: "Installation Source: Manual Installation (/usr/local/bin)"
```

### Upgrade Security

When upgrading manually-installed Anna:

1. **SHA256 Verification**: All binaries verified against GitHub checksums
2. **Backup Before Replace**: Previous version saved to `/var/lib/anna/backup/`
3. **Atomic Updates**: Binary replacement is atomic (no partial installs)
4. **Rollback Support**: `sudo annactl rollback` restores from backup
5. **Network Security**: GitHub API over HTTPS with 10s timeout
6. **Permission Check**: Only root can perform upgrades

```bash
# Safe upgrade workflow
sudo annactl upgrade           # Interactive with confirmation
sudo annactl upgrade --check   # Check only, no install
sudo annactl rollback          # Restore previous version
```

### Daemon Auto-Update Behavior

For manual installations, the daemon checks for updates every 24 hours:

- **What it does**: Queries GitHub API, logs availability
- **What it doesn't do**: Never auto-installs without explicit user action
- **Logs**: `/var/log/anna/` and `journalctl -u annad`
- **Disable**: AUR installations disable this automatically

### Monitoring Security

Anna's monitoring components are opt-in and localhost-only:

```bash
# Prometheus (localhost only)
# http://localhost:9090

# Grafana (localhost only, default: admin/admin)
# http://localhost:3000

# For remote access, use SSH tunnels (never expose ports):
ssh -L 3000:localhost:3000 user@host
ssh -L 9090:localhost:9090 user@host

# Firewall rules (if needed)
sudo ufw allow from 127.0.0.1 to any port 9090
sudo ufw allow from 127.0.0.1 to any port 3000
```

### Security Checklist

After installation, verify:

- [ ] Daemon running as dedicated `annad` user (not root)
- [ ] Your user added to `anna` group
- [ ] Socket permissions correct (`srw-rw---- annad:anna`)
- [ ] Configuration directory owned by root (`/etc/anna`)
- [ ] Self-healing disabled by default
- [ ] Logs writable by anna group
- [ ] Systemd hardening directives applied
- [ ] Monitoring (if installed) bound to localhost only
- [ ] SSH tunnels configured for remote access
- [ ] Regular updates enabled

---

## GitGuardian Integration

Anna Assistant uses GitGuardian for continuous secrets scanning:

- **Public Dashboard**: [GitGuardian](https://www.gitguardian.com/)
- **Scan Frequency**: Every push to main
- **Incident Response**: Automated alerts to maintainers

### For Contributors

If you receive a GitGuardian alert:
1. **Stop**: Do not push any more commits
2. **Revert**: Use `git reset` to remove the commit locally
3. **Notify**: Email security contact immediately
4. **Rotate**: If the secret was real (not test), rotate it immediately

### Verifying Repository Status

```bash
# Check for accidentally committed secrets
git ls-files | grep -E '\.(key|pem|srl|crt|csr)$'

# Should return no results
```

---

## Security Audit History

- **v1.0.0-rc.1**: Initial security audit (see `SECURITY_AUDIT.md`)
- **v1.16.1-alpha.1**: TLS materials purge and prevention system

---

## Responsible Disclosure

We follow coordinated disclosure:
1. Report received → Acknowledgment within 48 hours
2. Validation → Confirmed within 7 days
3. Fix developed → Tested and reviewed
4. Release → Security advisory published
5. Public disclosure → 90 days after fix release

---

## References

- [OWASP Key Management Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Key_Management_Cheat_Sheet.html)
- [GitGuardian Secrets Detection](https://www.gitguardian.com/)
- [Pre-commit Framework](https://pre-commit.com/)
- [detect-secrets by Yelp](https://github.com/Yelp/detect-secrets)

---

**Last Updated**: 2025-11-13 (Phase 3.9 hardening documentation added)
**Maintained By**: Anna Assistant Security Team
