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
cd RouterAgent
chmod +x install.sh
./install.sh
```
Or run this one liner 
```bash
git clone https://github.com/VihasMethnula/RouterAgent.git && cd RouterAgent && chmod +x install.sh && ./install.sh
```
This installs the `Router` binary to your Cargo bin directory, making it available globally.
Now you can acess the web dashboard at (http://localhost:5090) or if that doesnt work try (http://127.0.0.1:5090). 

## Usage

Connect to your ESP router's Wi-Fi network, then run:

```bash
Router
```
This brings you to the TUI

If youre a GUI guy you can use the web dashboard (http://localhost:5090)

Once running:

- The dashboard updates automatically every 2s
- Press `s` to toggle the network scan panel and list devices on the network
- Press `q` to quit cleanly

## Credits

- [Martin Ger](https://github.com/martin-ger) — ESP router firmware this project connects to

## Supporters 

- [Rashmina Adithya](https://github.com/rashmina-a) - Add web interface , Animations , Banners and systemd service 

## Version 

v2.5