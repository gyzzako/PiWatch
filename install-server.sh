#!/usr/bin/env bash
set -euo pipefail

BINARY_NAME="PiWatch"
INSTALL_DIR="/data/piwatch"
BINARY_PATH="$INSTALL_DIR/$BINARY_NAME"
CONFIG_PATH="$INSTALL_DIR/config.json"
SERVICE_FILE="/etc/systemd/system/piwatch.service"

LOG_DIR="/var/log"

RELEASE_URL="https://github.com/gyzzako/PiWatch/releases/latest/download/PiWatch"

UNINSTALL=false
PIHOLE_URL=""
PIHOLE_PASS=""

# -----------------------------
# PARSE ARGS
# -----------------------------
for arg in "$@"; do
    case $arg in
        --uninstall)
            UNINSTALL=true
            ;;
        --pihole-url=*)
            PIHOLE_URL="${arg#*=}"
            ;;
        --pihole-pass=*)
            PIHOLE_PASS="${arg#*=}"
            ;;
    esac
done

# -----------------------------
# UNINSTALL
# -----------------------------
if [ "$UNINSTALL" = true ]; then
    echo "[1/5] Stopping service..."
    systemctl stop piwatch.service || true

    echo "[2/5] Disabling service..."
    systemctl disable piwatch.service || true

    echo "[3/5] Removing files..."
    rm -rf "$INSTALL_DIR"
    rm -f "$SERVICE_FILE"

    systemctl daemon-reload

    echo "PiWatch uninstalled"
    exit 0
fi

# -----------------------------
# VALIDATION
# -----------------------------
if [ -z "$PIHOLE_URL" ] || [ -z "$PIHOLE_PASS" ]; then
    echo "Missing required args:"
    echo "   --pihole-url="
    echo "   --pihole-pass="
    exit 1
fi

# -----------------------------
# INSTALL
# -----------------------------

echo "[1/6] Creating directories..."
mkdir -p "$INSTALL_DIR"

echo "[2/6] Downloading..."
curl -L -o "$BINARY_PATH.tmp" "$RELEASE_URL"
chmod +x "$BINARY_PATH.tmp"
mv "$BINARY_PATH.tmp" "$BINARY_PATH"


echo "[3/6] Writing config.json (only if not exists)..."

if [ ! -f "$CONFIG_PATH" ]; then
    cat > "$CONFIG_PATH" <<EOF
{
  "pihole_url": "$PIHOLE_URL",
  "bind_port": 8888,
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
Description=PiWatch
After=network.target
StartLimitIntervalSec=60
StartLimitBurst=3

[Service]
Type=simple
User=root
WorkingDirectory=$INSTALL_DIR
ExecStart=$BINARY_PATH
Environment="PIHOLE_PASS=$PIHOLE_PASS"

Restart=on-failure
RestartSec=5s

StandardOutput=append:$LOG_DIR/piwatch.log
StandardError=append:$LOG_DIR/piwatch.log

[Install]
WantedBy=multi-user.target
EOF

echo "[5/6] Reloading systemd..."
systemctl daemon-reload

echo "[6/6] Enabling & restarting service..."
systemctl enable piwatch.service
systemctl restart piwatch.service

echo "PiWatch installed successfully"