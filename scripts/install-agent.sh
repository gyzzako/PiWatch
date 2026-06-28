#!/usr/bin/env bash
#
# Usage:
#   curl -sSL https://raw.githubusercontent.com/gyzzako/PiWatch/refs/heads/master/scripts/install-agent.sh | sudo bash [-s -- [options]]

# Options:
#   --uninstall             Uninstall the PiWatch agent completely (default: false)
#   --reset-config          Reset the config file (default: false)
#   --piwatch-server-url    PiWatch server URL
#   --install-token         Install token for registration


set -euo pipefail

BINARY_NAME="piwatch-agent"
INSTALL_DIR="/data/piwatch-agent"
BINARY_PATH="$INSTALL_DIR/$BINARY_NAME"
CONFIG_PATH="$INSTALL_DIR/config.json"
SERVICE_FILE="/etc/systemd/system/piwatch-agent.service"

LOG_DIR="/var/log"

RELEASE_URL="https://github.com/gyzzako/PiWatch/releases/latest/download/$BINARY_NAME"

UNINSTALL=false
RESET_CONFIG=false
PIWATCH_SERVER_URL=""
INSTALL_TOKEN=""

# -----------------------------
# PARSE ARGS
# -----------------------------
for arg in "$@"; do
    case $arg in
        --uninstall)
            UNINSTALL=true
            ;;
        --reset-config)
            RESET_CONFIG=true
            ;;
        --piwatch-server-url=*)
            PIWATCH_SERVER_URL="${arg#*=}"
            ;;
        --install-token=*)
            INSTALL_TOKEN="${arg#*=}"
            ;;
    esac
done

# -----------------------------
# UNINSTALL
# -----------------------------
if [ "$UNINSTALL" = true ]; then
    echo "[1/5] Stopping service..."
    systemctl stop piwatch-agent.service || true

    echo "[2/5] Disabling service..."
    systemctl disable piwatch-agent.service || true

    echo "[3/5] Removing files..."
    rm -rf "$INSTALL_DIR"
    rm -f "$SERVICE_FILE"

    systemctl daemon-reload

    echo "PiWatch agent uninstalled"
    exit 0
fi

# -----------------------------
# VALIDATION
# -----------------------------
if [ -z "$PIWATCH_SERVER_URL" ]; then
    echo "Missing required args:"
    echo "   --piwatch-server-url="
    exit 1
fi

# -----------------------------
# INSTALL
# -----------------------------

echo "[1/6] Creating directories..."
mkdir -p "$INSTALL_DIR"

echo "[2/6] Downloading agent..."
curl -L -o "$BINARY_PATH.tmp" "$RELEASE_URL"
chmod +x "$BINARY_PATH.tmp"
mv "$BINARY_PATH.tmp" "$BINARY_PATH"

# -----------------------------
# RESET CONFIG IF REQUESTED
# -----------------------------
if [ "$RESET_CONFIG" = true ]; then
    echo "[2.5/6] Resetting config.json..."
    rm -f "$CONFIG_PATH"
fi

echo "[3/6] Writing config.json (only if not exists)..."

if [ ! -f "$CONFIG_PATH" ]; then
    cat > "$CONFIG_PATH" <<EOF
{
  "piwatch_server_url": "$PIWATCH_SERVER_URL",
  "listening_interface": "eth0",
  "bind_port": 8887,
  "log_level": "info"
}
EOF
    echo "   → config created"
else
    echo "   → config already exists, skipping"
fi

echo "[4/6] Installing systemd service..."
cat > "$SERVICE_FILE" <<EOF
[Unit]
Description=PiWatch agent
After=network.target
StartLimitBurst=3

[Service]
Type=simple
User=root
WorkingDirectory=$INSTALL_DIR
ExecStart=$BINARY_PATH
Environment="PIWATCH_INSTALL_TOKEN=$INSTALL_TOKEN"

Restart=on-failure
RestartSec=5s

StandardOutput=append:$LOG_DIR/piwatch-agent.log
StandardError=append:$LOG_DIR/piwatch-agent.log

[Install]
WantedBy=multi-user.target
EOF

echo "[5/6] Reloading systemd..."
systemctl daemon-reload

echo "[6/6] Enabling & restarting service..."
systemctl enable piwatch-agent.service
systemctl restart piwatch-agent.service

echo "PiWatch agent installed successfully"