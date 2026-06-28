# 📡 PiWatch

PiWatch is a lightweight distributed network monitoring system that tracks IP/interface changes on agents and synchronizes them with a central server (e.g. Pi-hole integration).

---

# 🚀 Quick Install

## ⚠️ Requirements

- Linux (Debian/Ubuntu recommended)
- systemd
- root access
- internet access (GitHub releases)

---

## 🧠 Architecture

- **PiWatch-server** → central API + DNS/Pi-hole reconciliation
- **PiWatch-agent** → runs on hosts / LXC containers and reports IP changes

---

## 📦 Install Server

Run this on the server machine:

```bash
curl -sSL https://raw.githubusercontent.com/gyzzako/PiWatch/refs/heads/master/scripts/install-server.sh | sudo bash -s -- \
  --pihole-url="https://YOUR_PIHOLE_URL" \
  --pihole-pass="YOUR_PIHOLE_PASSWORD" \
  --install-token="your-secure-token"
```

### 🔧 Server configuration

File location:

`/data/piwatch/config.json`

Example:

```json
{
  "pihole_url": "https://YOUR_PIHOLE_URL",
  "bind_port": 8888,
  "log_level": "info",
  "hostname_suffix": "lxc"
}
```

Configuration can also be overridden via environment variables (set in systemd service):

| Environment Variable | Config Field |
|---|---|
| `PIHOLE_URL` | `pihole_url` (required) |
| `PIHOLE_PASS` | `pihole_pass` (required) |
| `HOSTNAME_SUFFIX` | `hostname_suffix` |
| `PIWATCH_INSTALL_TOKEN` | / |

### 🧾 Server systemd service

File location:

`/etc/systemd/system/piwatch.service`

Key fields:

```ini
ExecStart=/data/piwatch/piwatch-server
Environment="PIHOLE_PASS=..."
Environment="PIHOLE_URL=..."
Environment="HOSTNAME_SUFFIX=..."
Environment="PIWATCH_INSTALL_TOKEN=..." # Optional: for agent registration
Restart=on-failure
```
If `PIWATCH_INSTALL_TOKEN` is not set, the server generates a random token at startup (check logs).

Logs:

`/var/log/piwatch.log`

---

## 🤖 Install Agent

Run this on each Proxmox LXC / Linux host:

```bash
curl -sSL https://raw.githubusercontent.com/gyzzako/PiWatch/refs/heads/master/scripts/install-agent.sh | sudo bash -s -- \
  --piwatch-server-url="https://YOUR_SERVER_URL" \
  --install-token="your-secure-token"
```

### 🔧 Agent configuration

File location:

`/data/piwatch-agent/config.json`

Example:

```json
{
  "piwatch_server_url": "https://PIWATCH_SERVER_URL",
  "listening_interface": "eth0",
  "bind_port": 8887,
  "log_level": "info"
}
```

Configuration can also be overridden via environment variables (set in systemd service):

| Environment Variable | Config Field |
| --------------------- | -------------- |
| `PIWATCH_SERVER_URL` | `piwatch_server_url` (required)|
| `LISTENING_INTERFACE` | `listening_interface` |
| `PIWATCH_INSTALL_TOKEN` | Install token (not in config file) |

### 🔐 Agent Registration

The agent requires an install token to register with the server. Set `PIWATCH_INSTALL_TOKEN` in the systemd service:

```ini
Environment="PIWATCH_INSTALL_TOKEN=your-secure-token"
```

### 🧾 Agent systemd service

File location:

`/etc/systemd/system/piwatch-agent.service`

Key fields:

```ini
ExecStart=/data/piwatch-agent/piwatch-agent
Environment="PIWATCH_INSTALL_TOKEN=..." # Install token from server
Environment="LISTENING_INTERFACE=..." # Optional: override config
Restart=on-failure
```

Logs:

`/var/log/piwatch-agent.log`

---

# 🔄 Update

To update the server or agent, re-run the install script with your options. If updating the install token, update both server and agent with the same token.

---

# 🧹 Uninstall

## Server

```bash
curl -sSL https://raw.githubusercontent.com/gyzzako/PiWatch/refs/heads/master/scripts/install-server.sh | sudo bash -s -- --uninstall
```

## Agent

```bash
curl -sSL https://raw.githubusercontent.com/gyzzako/PiWatch/refs/heads/master/scripts/install-agent.sh | sudo bash -s -- --uninstall
```

---

# 🔐 Security Notes

### 🔒 Agent registration is restricted with an install token
### ⚠️ No other authentication is implemented yet
