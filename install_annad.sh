#!/usr/bin/env bash
set -e

SERVICE_NAME="annad"
BIN_DAEMON="annad"
BIN_CTL="annactl"
SERVICE_PATH="/etc/systemd/system/${SERVICE_NAME}.service"
DATA_DIR="/var/lib/anna"

echo "Installing Anna..."

# Detect if already installed
if systemctl list-units --type=service | grep -q "${SERVICE_NAME}.service"; then
    echo "Anna is already installed."
    # Check for updates
    cargo build --quiet --release -p annad -p annactl
    LOCAL_VER=$(grep '^version' cmd/annad/Cargo.toml | head -1 | cut -d '"' -f2)
    INSTALLED_VER=$(sudo grep -m1 'Description=' "$SERVICE_PATH" | grep -oP '\d+\.\d+\.\d+' || true)
    if [ "$LOCAL_VER" == "$INSTALLED_VER" ]; then
        echo "No update needed. Version already matches."
        exit 0
    else
        echo "Updating existing Anna installation to v${LOCAL_VER}..."
        sudo systemctl stop "$SERVICE_NAME"
    fi
fi

# Build binaries (workspace-safe)
cargo build --release -p annad -p annactl

# Install binaries globally
sudo install -Dm755 target/release/${BIN_DAEMON} /usr/local/bin/${BIN_DAEMON}
sudo install -Dm755 target/release/${BIN_CTL}   /usr/local/bin/${BIN_CTL}

# Create systemd service
SERVICE_FILE=$(mktemp)
cat > "$SERVICE_FILE" <<EOF
[Unit]
Description=Anna - System Assistant Daemon v$(grep '^version' cmd/annad/Cargo.toml | head -1 | cut -d '"' -f2)
After=network.target

[Service]
ExecStart=/usr/local/bin/${BIN_DAEMON}
Restart=always
RestartSec=3
User=root
WorkingDirectory=/var/lib/anna
StandardOutput=journal
StandardError=journal

[Install]
WantedBy=multi-user.target
EOF

# Move service file and enable
sudo mkdir -p /var/lib/anna
sudo mv "$SERVICE_FILE" "$SERVICE_PATH"
sudo systemctl daemon-reload
sudo systemctl enable "$SERVICE_NAME"
sudo systemctl restart "$SERVICE_NAME"

echo "Installation complete."

# Verify service status
sleep 1
if systemctl is-active --quiet "$SERVICE_NAME"; then
    echo
    echo "Note: annactl installed globally at /usr/local/bin/${BIN_CTL}"
    echo "status: active"
    echo
    echo "data_dir=${DATA_DIR}"
    echo
    journalctl -u "$SERVICE_NAME" --since "1 minute ago" -n 5 --no-pager
else
    echo "Anna failed to start. Check logs with: sudo journalctl -u $SERVICE_NAME -xe"
    exit 1
fi
