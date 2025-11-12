#!/bin/bash
# Print SHA256 fingerprint of TLS certificate (Phase 1.16)

set -e

if [ $# -eq 0 ]; then
    echo "Usage: $0 <certificate.pem>"
    echo ""
    echo "Prints SHA256 fingerprint for certificate pinning."
    exit 1
fi

CERT_FILE="$1"

if [ ! -f "$CERT_FILE" ]; then
    echo "Error: Certificate file not found: $CERT_FILE"
    exit 1
fi

# Extract DER format and compute SHA256
FINGERPRINT=$(openssl x509 -in "$CERT_FILE" -outform DER | sha256sum | awk '{print $1}')

echo "Certificate: $CERT_FILE"
echo "SHA256 Fingerprint: $FINGERPRINT"
echo ""
echo "Add to /etc/anna/pinned_certs.json:"
echo "{"
echo "  \"enable_pinning\": true,"
echo "  \"pin_client_certs\": false,"
echo "  \"pins\": {"
echo "    \"node_001\": \"$FINGERPRINT\""
echo "  }"
echo "}"
