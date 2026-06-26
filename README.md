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
  --pihole-pass="YOUR_PIHOLE_PASSWORD"
```

### 🔧 Server configuration

File location:

`/data/piwatch/config.json`

Example:

```json
{
  "pihole_url": "https://YOUR_PIHOLE_URL",
  "bind_port": 8888,
  "log_level": "info"
}
```

### 🧾 Server systemd service

File location:

`/etc/systemd/system/piwatch.service`

Key fields:

```ini
ExecStart=/data/piwatch/piwatch-server
Environment="PIHOLE_PASS=..."
Restart=on-failure
```

Logs:

`/var/log/piwatch.log`

---

## 🤖 Install Agent

Run this on each Proxmox LXC / Linux host:

```bash
curl -sSL https://raw.githubusercontent.com/gyzzako/PiWatch/refs/heads/master/scripts/install-agent.sh | sudo bash -s -- \
  --piwatch-server-url="https://YOUR_SERVER_URL"
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

### 🧾 Agent systemd service

File location:

`/etc/systemd/system/piwatch-agent.service`

Key fields:

```ini
ExecStart=/data/piwatch-agent/piwatch-agent
Restart=on-failure
```

Logs:

`/var/log/piwatch-agent.log`

---

# 🔄 Update

Re-run install scripts:

## Server

```bash
curl -sSL https://raw.githubusercontent.com/gyzzako/PiWatch/refs/heads/master/scripts/install-server.sh | sudo bash
```

## Agent

```bash
curl -sSL https://raw.githubusercontent.com/gyzzako/PiWatch/refs/heads/master/scripts/install-agent.sh | sudo bash
```

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

## ⚠️ No authentication is implemented yet
