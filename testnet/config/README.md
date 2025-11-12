# Testnet Configuration

This directory contains configuration templates for the Anna Assistant testnet.

## TLS Certificates

**IMPORTANT**: TLS certificates and private keys are **never** stored in git.

### Generating Certificates Locally

To run the testnet locally, generate ephemeral TLS certificates:

```bash
# From repository root
./scripts/gen-selfsigned-ca.sh

# This creates:
# - testnet/config/tls/ca.key (CA private key)
# - testnet/config/tls/ca.pem (CA certificate)
# - testnet/config/tls/node_*.key (Node private keys)
# - testnet/config/tls/node_*.pem (Node certificates)
```

These files are git-ignored and must be generated locally or in CI.

### CI/CD

The CI workflow (`.github/workflows/consensus-smoke.yml`) automatically generates certificates before running tests. No certificates are ever committed to the repository.

### Docker Compose

The `docker-compose.yaml` file expects certificates at:
- `./testnet/config/tls/ca.pem`
- `./testnet/config/tls/node_N.pem`
- `./testnet/config/tls/node_N.key`

Run `./scripts/gen-selfsigned-ca.sh` before `docker-compose up`.

## Security Policy

- ✅ **DO**: Generate certificates locally for testing
- ✅ **DO**: Use the provided generator script
- ✅ **DO**: Rotate certificates regularly in production
- ❌ **DON'T**: Commit `.key`, `.pem`, `.srl`, `.crt`, or `.csr` files
- ❌ **DON'T**: Share private keys
- ❌ **DON'T**: Use testnet certificates in production

## Production Deployment

For production deployments, use proper certificate management:

1. Generate production certificates with your PKI
2. Store certificates securely (e.g., Vault, cloud secrets manager)
3. Use certificate rotation policies
4. Enable certificate pinning (see `docs/CERTIFICATE_PINNING.md`)

See `docs/PRODUCTION_DEPLOYMENT.md` for complete deployment guide.
