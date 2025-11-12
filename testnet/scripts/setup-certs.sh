#!/bin/bash
# Anna Assistant - TLS Certificate Generation for Testnet
# Phase 2.0.0-alpha.1 - Generate CA and node certificates for testing

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CERTS_DIR="${SCRIPT_DIR}/../certs"

echo "Anna Assistant - TLS Certificate Setup"
echo "======================================="
echo

# Create directory structure
mkdir -p "${CERTS_DIR}"/{ca,node1,node2,node3}

# Generate CA
echo "[1/4] Generating Certificate Authority..."
openssl req -new -x509 -days 3650 -nodes \
    -out "${CERTS_DIR}/ca/ca-cert.pem" \
    -keyout "${CERTS_DIR}/ca/ca-key.pem" \
    -subj "/C=NO/ST=Oslo/L=Oslo/O=Anna Testnet/OU=CA/CN=Anna CA"

echo "  ✓ CA certificate generated"

# Generate Node 1 certificate
echo "[2/4] Generating Node 1 certificate..."
openssl req -new -nodes \
    -out "${CERTS_DIR}/node1/cert.csr" \
    -keyout "${CERTS_DIR}/node1/key.pem" \
    -subj "/C=NO/ST=Oslo/L=Oslo/O=Anna Testnet/OU=Nodes/CN=node1.anna.local"

openssl x509 -req -days 365 \
    -in "${CERTS_DIR}/node1/cert.csr" \
    -CA "${CERTS_DIR}/ca/ca-cert.pem" \
    -CAkey "${CERTS_DIR}/ca/ca-key.pem" \
    -CAcreateserial \
    -out "${CERTS_DIR}/node1/cert.pem" \
    -extfile <(echo "subjectAltName=DNS:node1.anna.local,IP:172.20.0.10")

echo "  ✓ Node 1 certificate generated"

# Generate Node 2 certificate
echo "[3/4] Generating Node 2 certificate..."
openssl req -new -nodes \
    -out "${CERTS_DIR}/node2/cert.csr" \
    -keyout "${CERTS_DIR}/node2/key.pem" \
    -subj "/C=NO/ST=Oslo/L=Oslo/O=Anna Testnet/OU=Nodes/CN=node2.anna.local"

openssl x509 -req -days 365 \
    -in "${CERTS_DIR}/node2/cert.csr" \
    -CA "${CERTS_DIR}/ca/ca-cert.pem" \
    -CAkey "${CERTS_DIR}/ca/ca-key.pem" \
    -CAcreateserial \
    -out "${CERTS_DIR}/node2/cert.pem" \
    -extfile <(echo "subjectAltName=DNS:node2.anna.local,IP:172.20.0.11")

echo "  ✓ Node 2 certificate generated"

# Generate Node 3 certificate (can be regenerated to simulate MITM)
echo "[4/4] Generating Node 3 certificate..."
openssl req -new -nodes \
    -out "${CERTS_DIR}/node3/cert.csr" \
    -keyout "${CERTS_DIR}/node3/key.pem" \
    -subj "/C=NO/ST=Oslo/L=Oslo/O=Anna Testnet/OU=Nodes/CN=node3.anna.local"

openssl x509 -req -days 365 \
    -in "${CERTS_DIR}/node3/cert.csr" \
    -CA "${CERTS_DIR}/ca/ca-cert.pem" \
    -CAkey "${CERTS_DIR}/ca/ca-key.pem" \
    -CAcreateserial \
    -out "${CERTS_DIR}/node3/cert.pem" \
    -extfile <(echo "subjectAltName=DNS:node3.anna.local,IP:172.20.0.12")

echo "  ✓ Node 3 certificate generated"
echo

# Calculate and display certificate fingerprints (for pinning.toml)
echo "Certificate Fingerprints (SHA256):"
echo "===================================="
echo
echo "Node 1:"
openssl x509 -in "${CERTS_DIR}/node1/cert.pem" -noout -fingerprint -sha256 | sed 's/SHA256 Fingerprint=//'
echo
echo "Node 2:"
openssl x509 -in "${CERTS_DIR}/node2/cert.pem" -noout -fingerprint -sha256 | sed 's/SHA256 Fingerprint=//'
echo
echo "Node 3:"
openssl x509 -in "${CERTS_DIR}/node3/cert.pem" -noout -fingerprint -sha256 | sed 's/SHA256 Fingerprint=//'
echo

# Set permissions
chmod 600 "${CERTS_DIR}"/*/key.pem
chmod 644 "${CERTS_DIR}"/*/cert.pem
chmod 644 "${CERTS_DIR}"/ca/*.pem

echo "✓ Certificate setup complete!"
echo
echo "Next steps:"
echo "  1. Update testnet/configs/node*/pinning.toml with fingerprints above"
echo "  2. Run: docker-compose -f testnet/docker-compose.pinned.yml up -d"
echo
