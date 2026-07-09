use warp::Filter;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::config::Config;
use crate::Device;
use crate::Stats;

pub type SharedStats = Arc<RwLock<Stats>>;
pub type SharedDevices = Arc<RwLock<Vec<Device>>>;

pub fn create_web_interface(
    _config: Config,
    stats: SharedStats,
    devices: SharedDevices,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let stats_filter = warp::any().map(move || stats.clone());
    let devices_filter = warp::any().map(move || devices.clone());

    let index = warp::path::end()
        .map(|| warp::reply::html(INDEX_HTML));

    let api_stats = warp::path!("api" / "stats")
        .and(stats_filter.clone())
        .and_then(get_stats);

    let api_devices = warp::path!("api" / "devices")
        .and(devices_filter.clone())
        .and_then(get_devices);

    let routes = index
        .or(api_stats)
        .or(api_devices)
        .with(warp::cors().allow_any_origin());

    routes
}

async fn get_stats(stats: SharedStats) -> Result<impl warp::Reply, warp::Rejection> {
    let stats = stats.read().await;
    Ok(warp::reply::json(&StatsResponse {
        status: stats.status.clone(),
        clients: stats.clients.clone(),
        rssi: stats.rssi.clone(),
        uptime: stats.uptime.clone(),
        sent: stats.sent.clone(),
        received: stats.received.clone(),
    }))
}

async fn get_devices(devices: SharedDevices) -> Result<impl warp::Reply, warp::Rejection> {
    let devices = devices.read().await;
    let response: Vec<DeviceResponse> = devices.iter().map(|d| DeviceResponse {
        ip: d.ip.clone(),
        hostname: d.hostname.clone(),
        mac: d.mac.clone(),
    }).collect();
    Ok(warp::reply::json(&response))
}

#[derive(serde::Serialize)]
struct StatsResponse {
    status: String,
    clients: String,
    rssi: String,
    uptime: String,
    sent: String,
    received: String,
}

#[derive(serde::Serialize)]
struct DeviceResponse {
    ip: String,
    hostname: String,
    mac: String,
}

const INDEX_HTML: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Router Agent - Terminal Dashboard</title>
    <style>
        * {
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }
        
        body {
            background-color: #0a0a0a;
            color: #00ff00;
            font-family: 'Courier New', Courier, monospace;
            min-height: 100vh;
            padding: 20px;
        }
        
        .container {
            max-width: 1200px;
            margin: 0 auto;
        }
        
        .header {
            border: 2px solid #00ff00;
            padding: 20px;
            margin-bottom: 20px;
            position: relative;
        }
        
        .header::before {
            content: '';
            position: absolute;
            top: 0;
            left: 0;
            right: 0;
            height: 2px;
            background: linear-gradient(90deg, transparent, #00ff00, transparent);
        }
        
        .title {
            font-size: 24px;
            text-align: center;
            color: #00ff00;
            text-shadow: 0 0 10px #00ff00;
        }
        
        .subtitle {
            text-align: center;
            color: #00aa00;
            margin-top: 5px;
        }
        
        .grid {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
            gap: 20px;
            margin-bottom: 20px;
        }
        
        .panel {
            border: 1px solid #00aa00;
            padding: 15px;
            background-color: #0d0d0d;
        }
        
        .panel-title {
            color: #00ffff;
            font-size: 14px;
            margin-bottom: 10px;
            border-bottom: 1px solid #00aa00;
            padding-bottom: 5px;
        }
        
        .stat-row {
            display: flex;
            justify-content: space-between;
            margin: 8px 0;
            padding: 5px;
            border-left: 2px solid #00aa00;
        }
        
        .stat-label {
            color: #888888;
        }
        
        .stat-value {
            color: #00ff00;
            font-weight: bold;
        }
        
        .stat-value.online {
            color: #00ff00;
            text-shadow: 0 0 5px #00ff00;
        }
        
        .stat-value.offline {
            color: #ff0000;
            text-shadow: 0 0 5px #ff0000;
        }
        
        .stat-value.warning {
            color: #ffff00;
            text-shadow: 0 0 5px #ffff00;
        }
        
        .devices-panel {
            grid-column: 1 / -1;
        }
        
        .devices-table {
            width: 100%;
            border-collapse: collapse;
            margin-top: 10px;
        }
        
        .devices-table th,
        .devices-table td {
            border: 1px solid #00aa00;
            padding: 10px;
            text-align: left;
        }
        
        .devices-table th {
            background-color: #1a1a1a;
            color: #00ffff;
        }
        
        .devices-table tr:hover {
            background-color: #1a1a1a;
        }
        
        .device-ip {
            color: #00ff00;
        }
        
        .device-hostname {
            color: #ff00ff;
        }
        
        .device-mac {
            color: #888888;
        }
        
        .status-indicator {
            display: inline-block;
            width: 10px;
            height: 10px;
            border-radius: 50%;
            margin-right: 5px;
            animation: pulse 2s infinite;
        }
        
        .status-indicator.online {
            background-color: #00ff00;
            box-shadow: 0 0 10px #00ff00;
        }
        
        .status-indicator.offline {
            background-color: #ff0000;
            box-shadow: 0 0 10px #ff0000;
        }
        
        @keyframes pulse {
            0%, 100% { opacity: 1; }
            50% { opacity: 0.5; }
        }
        
        .terminal-cursor {
            display: inline-block;
            width: 8px;
            height: 16px;
            background-color: #00ff00;
            animation: blink 1s step-end infinite;
            vertical-align: middle;
            margin-left: 5px;
        }
        
        @keyframes blink {
            0%, 100% { opacity: 1; }
            50% { opacity: 0; }
        }
        
        .footer {
            text-align: center;
            color: #00aa00;
            margin-top: 20px;
            padding: 10px;
            border-top: 1px solid #00aa00;
        }
        
        .ascii-art {
            color: #00ff00;
            font-size: 12px;
            line-height: 1.2;
            margin-bottom: 20px;
            text-align: center;
            text-shadow: 0 0 5px #00ff00;
        }
        
        .loading {
            text-align: center;
            padding: 40px;
        }
        
        .loading-dots {
            display: inline-block;
        }
        
        @keyframes loading {
            0%, 20% { content: '.'; }
            40% { content: '..'; }
            60%, 100% { content: '...'; }
        }
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <div class="ascii-art">
█▀▀█ █▀▀█ █▀▀█ █▀▀█ █▀▀█ █▀▀▀
█  █ █  █ █  █ █  █ █  █ █▀▀
▀▀▀▀ ▀▀▀▀ ▀▀▀▀ ▀▀▀▀ ▀▀▀▀ ▀▀▀▀
            </div>
            <h1 class="title">ROUTER AGENT TERMINAL</h1>
            <p class="subtitle">Network Monitoring Dashboard</p>
        </div>
        
        <div class="grid">
            <div class="panel">
                <div class="panel-title">[ SYSTEM STATUS ]</div>
                <div class="stat-row">
                    <span class="stat-label">Status:</span>
                    <span class="stat-value" id="status">Loading...</span>
                </div>
                <div class="stat-row">
                    <span class="stat-label">Uptime:</span>
                    <span class="stat-value" id="uptime">--</span>
                </div>
                <div class="stat-row">
                    <span class="stat-label">Signal:</span>
                    <span class="stat-value" id="rssi">--</span>
                </div>
            </div>
            
            <div class="panel">
                <div class="panel-title">[ NETWORK STATS ]</div>
                <div class="stat-row">
                    <span class="stat-label">Clients:</span>
                    <span class="stat-value" id="clients">--</span>
                </div>
                <div class="stat-row">
                    <span class="stat-label">Upload:</span>
                    <span class="stat-value" id="sent">--</span>
                </div>
                <div class="stat-row">
                    <span class="stat-label">Download:</span>
                    <span class="stat-value" id="received">--</span>
                </div>
            </div>
            
            <div class="panel devices-panel">
                <div class="panel-title">[ CONNECTED DEVICES ]</div>
                <table class="devices-table">
                    <thead>
                        <tr>
                            <th>IP ADDRESS</th>
                            <th>HOSTNAME</th>
                            <th>MAC ADDRESS</th>
                        </tr>
                    </thead>
                    <tbody id="devices-list">
                        <tr>
                            <td colspan="3" style="text-align: center;">Loading devices...</td>
                        </tr>
                    </tbody>
                </table>
            </div>
        </div>
        
        <div class="footer">
            <p>Router Agent v0.1.0 | <span id="last-update">Last Update: --</span><span class="terminal-cursor"></span></p>
        </div>
    </div>
    
    <script>
        async function fetchStats() {
            try {
                const response = await fetch('/api/stats');
                const data = await response.json();
                
                const statusEl = document.getElementById('status');
                const isOnline = data.status.includes('ONLINE');
                statusEl.innerHTML = `<span class="status-indicator ${isOnline ? 'online' : 'offline'}"></span>${data.status}`;
                statusEl.className = `stat-value ${isOnline ? 'online' : 'offline'}`;
                
                document.getElementById('uptime').textContent = data.uptime;
                document.getElementById('rssi').textContent = data.rssi;
                document.getElementById('clients').textContent = data.clients + ' device(s)';
                document.getElementById('sent').textContent = data.sent;
                document.getElementById('received').textContent = data.received;
                
                updateLastUpdate();
            } catch (error) {
                console.error('Failed to fetch stats:', error);
            }
        }
        
        async function fetchDevices() {
            try {
                const response = await fetch('/api/devices');
                const data = await response.json();
                
                const tbody = document.getElementById('devices-list');
                tbody.innerHTML = '';
                
                if (data.length === 0) {
                    tbody.innerHTML = '<tr><td colspan="3" style="text-align: center;">No devices found</td></tr>';
                    return;
                }
                
                data.forEach(device => {
                    const row = document.createElement('tr');
                    row.innerHTML = `
                        <td class="device-ip">${device.ip}</td>
                        <td class="device-hostname">${device.hostname}</td>
                        <td class="device-mac">${device.mac || '(this device)'}</td>
                    `;
                    tbody.appendChild(row);
                });
                
                updateLastUpdate();
            } catch (error) {
                console.error('Failed to fetch devices:', error);
            }
        }
        
        function updateLastUpdate() {
            const now = new Date();
            const timeStr = now.toLocaleTimeString();
            document.getElementById('last-update').textContent = `Last Update: ${timeStr}`;
        }
        
        function updateData() {
            fetchStats();
            fetchDevices();
        }
        
        updateData();
        setInterval(updateData, 1000);
    </script>
</body>
</html>"#;
