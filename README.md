# RouterAgent

A live terminal dashboard for monitoring an ESP-based Wi-Fi router, written in Rust.

Built to work with an ESP router flashed using [Martin Ger's](https://github.com/martin-ger) firmware — huge thanks for the foundation this project builds on.


> **You must be connected to your ESP router's Wi-Fi network to use this tool.** RouterAgent talks directly to the router at `192.168.4.1` and scans the `192.168.4.0/24` subnet, so it only works while you're on that network.

## Features

- Live-updating terminal dashboard (refreshes every 500ms)
- Router status, signal strength (RSSI), uptime, and connected client count
- Real-time upload/download speed tracking with animated progress bars
- Internet ping latency check (color-coded by speed)
- On-demand local network scan (press `s`) to list connected devices with IP, hostname, and MAC address
- Color-coded status indicators (green/yellow/red based on values)
- Pulsing status dot and spinner animations
- Color-cycling header art

> ⚠️ **You must be connected to your ESP router's Wi-Fi network to use this tool.** RouterAgent talks directly to the router at `192.168.4.1` and scans the `192.168.4.0/24` subnet, so it only works while you're on that network.

## Features

- Live-updating terminal dashboard (refreshes every 2 seconds)
- Router status, signal strength (RSSI), uptime, and connected client count
- Real-time upload/download speed tracking
- Internet ping latency check
- On-demand local network scan (press `s`) to list connected devices with IP, hostname, and MAC address


## Requirements

- A router running Martin Ger's ESP firmware
- `nmap` installed and available via `sudo` (used for the network scan)
- `ping` available on your system

## Installation

```bash
git clone https://github.com/VihasMethnula/RouterAgent.git
cd RouterAgent/router-agent
cargo install --path .
```

Or copy the install.sh raw and run it

This installs the `Router` binary to your Cargo bin directory, making it available globally.

## Running as a service (Linux/systemd)

The repository ships a systemd unit file (`router-agent.service`) that runs the dashboard in headless mode and exposes it on the configured port.

```bash
# 1. Install the binary
cargo install --path router-agent

# 2. Symlink it to /usr/local/bin so the unit file's ExecStart resolves
sudo ln -sf "$HOME/.cargo/bin/router" /usr/local/bin/router

# 3. Install and start the service
sudo cp router-agent.service /etc/systemd/system/router-agent.service
sudo systemctl daemon-reload
sudo systemctl enable --now router-agent

# 4. Follow the logs
journalctl -u router-agent -f
```

The unit runs as `root` so `sudo nmap` works without extra configuration. Config is read from `~/.config/router/config.yaml` (created with defaults on first run).

## Usage

Connect to your ESP router's Wi-Fi network, then run:

```bash
Router
```

Once running:

- The dashboard updates automatically every 2s
- Press `s` to toggle the network scan panel and list devices on the network
- Press `q` to quit cleanly

## Credits

- [Martin Ger](https://github.com/martin-ger) — ESP router firmware this project connects to
