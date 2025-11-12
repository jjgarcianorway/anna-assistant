# Certificate Pinning Guide (Phase 1.16)

## Overview

Certificate pinning provides an additional layer of security for mTLS connections by validating that peer certificates match pre-approved SHA256 fingerprints. This prevents man-in-the-middle attacks even if the CA is compromised.

## Status

**Phase 1.16**: Infrastructure and tooling implemented. Full TLS integration planned for Phase 2.

- ✅ SHA256 fingerprint computation
- ✅ Pinning configuration structure
- ✅ Fingerprint validation logic
- ⏳ TLS handshake integration (Phase 2)

## Computing Certificate Fingerprints

Use the provided script to compute SHA256 fingerprints:

```bash
./scripts/print-cert-fingerprint.sh /etc/anna/certs/peer_node_001.pem
```

Output:
```
Certificate: /etc/anna/certs/peer_node_001.pem
SHA256 Fingerprint: a1b2c3d4e5f6...

Add to /etc/anna/pinned_certs.json:
{
  "enable_pinning": true,
  "pin_client_certs": false,
  "pins": {
    "node_001": "a1b2c3d4e5f6..."
  }
}
```

## Configuration

Create `/etc/anna/pinned_certs.json`:

```json
{
  "enable_pinning": true,
  "pin_client_certs": false,
  "pins": {
    "node_001": "a1b2c3d4e5f6789012345678901234567890abcdef1234567890abcdef123456",
    "node_002": "b2c3d4e5f6789012345678901234567890abcdef1234567890abcdef12345678",
    "node_003": "c3d4e5f6789012345678901234567890abcdef1234567890abcdef1234567890"
  }
}
```

### Configuration Fields

- **enable_pinning** (bool): Master switch for certificate pinning
- **pin_client_certs** (bool): Also validate client certificates (in addition to server certs)
- **pins** (map): Node ID → SHA256 fingerprint mapping

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

### Certificate Rotation

When rotating certificates:

1. **Pre-compute new fingerprints:**
   ```bash
   ./scripts/print-cert-fingerprint.sh /etc/anna/certs/new_cert.pem
   ```

2. **Update pinning config with BOTH old and new fingerprints** (dual-pinning period)

3. **Deploy new certs to all nodes**

4. **Remove old fingerprints after rollout completes**

### Threat Model

Certificate pinning defends against:
- Compromised Certificate Authority (CA)
- Fraudulent certificate issuance
- Advanced persistent threats (APT)

Does NOT defend against:
- Private key theft (attacker has valid cert + matching key)
- Misconfigured pinning (wrong fingerprints)
- Pinning config file tampering (protect with file integrity monitoring)

## API Usage (Phase 1.16)

```rust
use anna_common::network::PinningConfig;

// Load configuration
let config = PinningConfig::load_from_file("/etc/anna/pinned_certs.json").await?;

// Validate certificate fingerprint
let cert_der: &[u8] = /* DER-encoded certificate */;
let node_id = "node_001";

if config.validate_fingerprint(node_id, cert_der) {
    println!("✓ Certificate fingerprint matches");
} else {
    println!("✗ Certificate fingerprint mismatch - possible MITM attack!");
}

// Compute fingerprint
let fingerprint = PinningConfig::compute_fingerprint(cert_der);
println!("SHA256: {}", fingerprint);
```

## Metrics

Certificate pinning violations are tracked via:

```
annad_tls_handshake_total{status="cert_pinning_failed"}
```

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
