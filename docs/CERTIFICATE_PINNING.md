# Certificate Pinning Guide

**Status**: Phase 2 (v2.0.0-alpha.1) - Fully Integrated
**Feature**: SHA256 fingerprint-based certificate pinning for TLS connections

---

## Overview

Certificate pinning enforces cryptographic identity by validating peer certificates against known SHA256 fingerprints during TLS handshakes. This prevents MITM attacks even if a Certificate Authority is compromised.

**Implementation**: Custom rustls `ServerCertVerifier` with fail-closed enforcement.

**Phase 2 Status**:
- ✅ SHA256 fingerprint computation
- ✅ Configuration loading and validation
- ✅ rustls ServerCertVerifier integration
- ✅ Prometheus metric: `anna_pinning_violations_total{peer}`
- ✅ Masked fingerprint logging
- ✅ Hot-reload support via SIGHUP

## Extracting Certificate Fingerprints

### From PEM Certificate (OpenSSL)

```bash
openssl x509 -in cert.pem -noout -fingerprint -sha256 | \
  sed 's/SHA256 Fingerprint=/sha256:/' | \
  tr -d ':' | \
  tr '[:upper:]' '[:lower:]'
```

**Example Output**:
```
sha256:e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855
```

### From Live TLS Server

```bash
echo | openssl s_client -connect node1.example.com:8443 -showcerts 2>/dev/null | \
  openssl x509 -noout -fingerprint -sha256 | \
  sed 's/SHA256 Fingerprint=/sha256:/' | \
  tr -d ':' | \
  tr '[:upper:]' '[:lower:]'
```

### Batch Extract (All Testnet Certificates)

```bash
for cert in testnet/config/tls/*.pem; do
    [[ $cert == *"ca.pem" ]] && continue
    hostname=$(basename "$cert" .pem)
    fp=$(openssl x509 -in "$cert" -noout -fingerprint -sha256 | \
        sed 's/SHA256 Fingerprint=/sha256:/' | tr -d ':' | tr '[:upper:]' '[:lower:]')
    echo "  \"$hostname\": \"$fp\""
done
```

---

## Configuration

### File Location

`/etc/anna/pinned_certs.json`

### Schema (Phase 2)

```json
{
  "enforce": true,
  "peers": {
    "node1.example.com": "sha256:HEXDIGEST64CHARS",
    "node2.example.com": "sha256:HEXDIGEST64CHARS",
    "192.168.1.10": "sha256:HEXDIGEST64CHARS"
  }
}
```

**Fields**:
- `enforce` (boolean): Fail-closed on mismatch. Default: `true`
- `peers` (object): Map of hostname/IP → SHA256 fingerprint

**Note**: Phase 2 uses simplified schema. Old Phase 1.16 format is deprecated.

## Security Considerations

### When to Use Pinning

✅ **Recommended for:**
- Production deployments with static infrastructure
- High-security environments
- Long-lived peer relationships

⚠️ **Not recommended for:**
- Development/testing environments with frequent cert rotation
- Dynamic peer discovery
- Short-lived connections

### Certificate Rotation Playbook

**Zero-Downtime Rotation** (4 steps):

```bash
# Step 1: Generate new certificate
openssl req -new -nodes -newkey rsa:4096 \
  -keyout node1.new.key -out node1.new.csr \
  -subj "/CN=node1.example.com"

openssl x509 -req -in node1.new.csr -CA ca.pem -CAkey ca.key \
  -CAcreateserial -out node1.new.pem -days 365

# Step 2: Extract new fingerprint
NEW_FP=$(openssl x509 -in node1.new.pem -noout -fingerprint -sha256 | \
  sed 's/SHA256 Fingerprint=/sha256:/' | tr -d ':' | tr '[:upper:]' '[:lower:]')

# Step 3: Update pinning config on ALL peers
# Edit /etc/anna/pinned_certs.json on all nodes
sudo systemctl reload annad  # or kill -HUP

# Step 4: Deploy new cert to node1
sudo cp node1.new.pem /etc/anna/tls/node1.pem
sudo cp node1.new.key /etc/anna/tls/node1.key
sudo systemctl reload annad
```

**Verify**: Check `anna_pinning_violations_total` metric (should be 0)

### Threat Model

Certificate pinning defends against:
- Compromised Certificate Authority (CA)
- Fraudulent certificate issuance
- Advanced persistent threats (APT)

Does NOT defend against:
- Private key theft (attacker has valid cert + matching key)
- Misconfigured pinning (wrong fingerprints)
- Pinning config file tampering (protect with file integrity monitoring)

## Monitoring (Phase 2)

### Prometheus Metric

```
anna_pinning_violations_total{peer="node1.example.com"} 0
anna_pinning_violations_total{peer="node2.example.com"} 0
```

**Labels**:
- `peer`: Hostname or IP of peer with mismatched certificate

**Alert Example**:
```yaml
- alert: AnnaCertificatePinningViolation
  expr: increase(anna_pinning_violations_total[5m]) > 0
  for: 1m
  labels:
    severity: critical
  annotations:
    summary: "Certificate pinning violation"
    description: "Peer {{ $labels.peer }} presented unexpected certificate. Possible MITM."
```

### Logs

Violations logged with **masked fingerprints** (security):

```
ERROR peer=node1.example.com expected="sha256:1234567...90abcdef" actual="sha256:abcdef0...12345678" msg="Certificate pinning violation detected"
```

Only first 15 and last 8 characters shown to prevent log-based attacks.

## Troubleshooting

### Fingerprint Mismatch

```
Certificate fingerprint mismatch for node_001: expected abc123..., got def456...
```

**Causes:**
1. Certificate was rotated but pinning config not updated
2. Man-in-the-middle attack
3. Misconfigured node ID mapping

**Resolution:**
1. Verify the certificate file: `openssl x509 -in cert.pem -text -noout`
2. Re-compute fingerprint: `./scripts/print-cert-fingerprint.sh cert.pem`
3. Update pinning config if legitimate rotation
4. Investigate security incident if unexpected

### No Pinned Certificate

```
No pinned certificate for node node_004
```

**Resolution:**
Add node_004's fingerprint to `/etc/anna/pinned_certs.json`:

```bash
./scripts/print-cert-fingerprint.sh /etc/anna/certs/node_004.pem
# Copy output to pinned_certs.json
```

## Implementation Notes

Phase 1.16 provides the infrastructure and tooling for certificate pinning. Full integration into the TLS handshake process (custom `ServerCertVerifier`) is planned for Phase 2.

Current implementation:
- Fingerprint computation: ✅ Complete
- Configuration management: ✅ Complete
- Validation logic: ✅ Complete
- TLS integration: ⏳ Phase 2

## References

- [OWASP Certificate Pinning Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Pinning_Cheat_Sheet.html)
- [RFC 7469: Public Key Pinning Extension for HTTP](https://tools.ietf.org/html/rfc7469)
- [Rustls Documentation](https://docs.rs/rustls/)

---

Citation: [OWASP:Certificate_Pinning]
