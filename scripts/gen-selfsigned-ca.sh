#!/bin/bash
# Generate self-signed CA and certificates for Anna testnet (Phase 1.11)
# Creates CA, server, and client certificates with proper SANs for Docker hostnames

set -e
set -u

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CERT_DIR="${CERT_DIR:-$SCRIPT_DIR/../testnet/config/tls}"

echo "=== Anna Self-Signed CA Generator ==="
echo "Output directory: $CERT_DIR"
echo

# Create output directory
mkdir -p "$CERT_DIR"
cd "$CERT_DIR"

# Generate CA private key
echo "[1/7] Generating CA private key..."
openssl genrsa -out ca.key 4096 2>/dev/null
chmod 600 ca.key

# Generate CA certificate
echo "[2/7] Generating CA certificate..."
openssl req -new -x509 -days 3650 -key ca.key -out ca.pem \
    -subj "/C=NO/ST=Oslo/L=Oslo/O=Anna Assistant/OU=Testnet CA/CN=Anna Testnet CA" \
    2>/dev/null

echo "✓ CA certificate created: ca.pem"
echo

# Function to generate node certificates
generate_node_cert() {
    local NODE_ID=$1
    local NODE_NUM=$2

    echo "[$((2 + NODE_NUM * 2 + 1))/7] Generating ${NODE_ID} certificate..."

    # Generate private key
    openssl genrsa -out "${NODE_ID}.key" 2048 2>/dev/null
    chmod 600 "${NODE_ID}.key"

    # Create certificate signing request
    openssl req -new -key "${NODE_ID}.key" -out "${NODE_ID}.csr" \
        -subj "/C=NO/ST=Oslo/L=Oslo/O=Anna Assistant/OU=Testnet/CN=${NODE_ID}" \
        2>/dev/null

    # Create SAN config
    cat > "${NODE_ID}.ext" <<EOF
authorityKeyIdentifier=keyid,issuer
basicConstraints=CA:FALSE
keyUsage = digitalSignature, nonRepudiation, keyEncipherment, dataEncipherment
subjectAltName = @alt_names

[alt_names]
DNS.1 = ${NODE_ID}
DNS.2 = anna-${NODE_ID}
DNS.3 = localhost
IP.1 = 127.0.0.1
EOF

    # Sign certificate with CA
    openssl x509 -req -in "${NODE_ID}.csr" -CA ca.pem -CAkey ca.key \
        -CAcreateserial -out "${NODE_ID}.pem" -days 365 \
        -extfile "${NODE_ID}.ext" 2>/dev/null

    # Clean up temporary files
    rm "${NODE_ID}.csr" "${NODE_ID}.ext"

    echo "✓ ${NODE_ID} certificate created"
}

# Generate certificates for 3 nodes
for i in 1 2 3; do
    generate_node_cert "node_${i}" $((i - 1))
done

echo
echo "=== Certificate Summary ==="
echo "CA certificate:        ca.pem"
echo "CA private key:        ca.key"
echo

for i in 1 2 3; do
    echo "Node $i certificate:    node_${i}.pem"
    echo "Node $i private key:    node_${i}.key"
done

echo
echo "=== File Permissions ==="
ls -lh *.key *.pem | awk '{print $1, $9}'

echo
echo "=== Certificate Validation ==="
for i in 1 2 3; do
    if openssl verify -CAfile ca.pem "node_${i}.pem" > /dev/null 2>&1; then
        echo "✓ node_${i}.pem: Valid"
    else
        echo "✗ node_${i}.pem: INVALID"
    fi
done

echo
echo "=== SAN Verification ==="
for i in 1 2 3; do
    echo "node_${i} SANs:"
    openssl x509 -in "node_${i}.pem" -noout -text | grep -A 1 "Subject Alternative Name" | tail -1 | sed 's/^[[:space:]]*/  /'
done

echo
echo "✓ Self-signed CA and certificates generated successfully"
echo "  Certificates valid for 365 days"
echo "  CA valid for 10 years"
echo
echo "Usage:"
echo "  - Copy ca.pem to all nodes for peer verification"
echo "  - Each node uses its own node_N.pem and node_N.key"
echo "  - Set file permissions: chmod 600 *.key, chmod 644 *.pem"
