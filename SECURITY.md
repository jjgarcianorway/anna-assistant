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

### Systemd Hardening (Phase 0.4)

Anna runs with a strict systemd security sandbox:

```ini
NoNewPrivileges=yes
PrivateTmp=yes
ProtectSystem=strict
ProtectHome=yes
ProtectKernelTunables=yes
ProtectKernelModules=yes
ProtectControlGroups=yes
RestrictAddressFamilies=AF_UNIX AF_INET AF_INET6
RestrictNamespaces=yes
LockPersonality=yes
RestrictRealtime=yes
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

**Last Updated**: 2025-11-12
**Maintained By**: Anna Assistant Security Team
