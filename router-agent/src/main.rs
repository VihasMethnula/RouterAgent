use std::{
    io::{self, Read},
    sync::{Arc, Mutex},
    thread,
    time::{Duration, Instant},
};
use regex::Regex;

const ROUTER_URL: &str = "http://192.168.4.1";
const NETWORK:    &str = "192.168.4.0/24";

// ── Device found by nmap ─────────────────────────────────────────────────────
#[derive(Clone)]
struct Device {
    ip:       String,
    hostname: String,
    mac:      String,
}

// ── Router stats ─────────────────────────────────────────────────────────────
struct Stats {
    status:         String,
    clients:        String,
    rssi:           String,
    uptime:         String,
    sent:           String,
    received:       String,
    sent_bytes:     f64,
    received_bytes: f64,
}

// ── Terminal helpers ──────────────────────────────────────────────────────────
fn clear_terminal() {
    std::process::Command::new("clear").status().ok();
}

// Put terminal in raw mode so we can read single keypresses without Enter
fn set_raw_mode(raw: bool) {
    if raw {
        std::process::Command::new("stty").args(["-echo", "cbreak"]).status().ok();
    } else {
        std::process::Command::new("stty").args(["echo", "-cbreak"]).status().ok();
    }
}

// ── Byte parsing / formatting ─────────────────────────────────────────────────
fn parse_bytes(s: &str) -> f64 {
    let s = s.trim().to_lowercase();
    let parts: Vec<&str> = s.splitn(2, ' ').collect();
    if parts.len() < 2 { return 0.0; }
    let value: f64 = parts[0].parse().unwrap_or(0.0);
    match parts[1].trim() {
        "b"  | "bytes" => value,
        "kb" | "kib"   => value * 1_024.0,
        "mb" | "mib"   => value * 1_048_576.0,
        "gb" | "gib"   => value * 1_073_741_824.0,
        _              => value,
    }
}

fn format_speed(bps: f64) -> String {
    if bps < 0.0                { return "0 B/s".into(); }
    if bps >= 1_073_741_824.0  { return format!("{:.1} GB/s", bps / 1_073_741_824.0); }
    if bps >= 1_048_576.0      { return format!("{:.1} MB/s", bps / 1_048_576.0); }
    if bps >= 1_024.0          { return format!("{:.1} KB/s", bps / 1_024.0); }
    format!("{:.0} B/s", bps)
}

// ── Ping ──────────────────────────────────────────────────────────────────────
fn get_ping_latency() -> String {
    let start = Instant::now();
    let ok = std::process::Command::new("ping")
        .args(["-c", "1", "-W", "2", "8.8.8.8"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);
    if ok { format!("{} ms", start.elapsed().as_millis()) } else { "Timeout".into() }
}

// ── nmap scan ─────────────────────────────────────────────────────────────────
fn run_nmap_scan() -> Vec<Device> {
    let output = std::process::Command::new("sudo")
        .args(["nmap", "-sn", NETWORK])
        .output();

    let text = match output {
        Ok(o) => String::from_utf8_lossy(&o.stdout).to_string(),
        Err(_) => return vec![],
    };

    let mut devices = Vec::new();
    let mut current_host = String::new();
    let mut current_mac  = String::new();

    // Regex patterns matching your nmap output format
    let re_host = Regex::new(r"Nmap scan report for (.+?) \((\d+\.\d+\.\d+\.\d+)\)|Nmap scan report for (\d+\.\d+\.\d+\.\d+)").unwrap();
    let re_mac  = Regex::new(r"MAC Address: ([0-9A-F:]{17})").unwrap();

    for line in text.lines() {
        if let Some(cap) = re_host.captures(line) {
            // Push previous device if any
            if !current_host.is_empty() {
                let parts: Vec<&str> = current_host.splitn(2, '|').collect();
                devices.push(Device {
                    hostname: parts.get(0).unwrap_or(&"").to_string(),
                    ip:       parts.get(1).unwrap_or(&"").to_string(),
                    mac:      current_mac.clone(),
                });
                current_mac.clear();
            }
            if cap.get(1).is_some() {
                // "hostname (ip)" format
                current_host = format!("{}|{}", cap[1].trim(), cap[2].trim());
            } else {
                // bare IP format
                current_host = format!("{}|{}", cap[3].trim(), cap[3].trim());
            }
        } else if let Some(cap) = re_mac.captures(line) {
            current_mac = cap[1].to_string();
        }
    }
    // Push last device
    if !current_host.is_empty() {
        let parts: Vec<&str> = current_host.splitn(2, '|').collect();
        devices.push(Device {
            hostname: parts.get(0).unwrap_or(&"").to_string(),
            ip:       parts.get(1).unwrap_or(&"").to_string(),
            mac:      current_mac,
        });
    }
    // Rename _gateway to Router
    for dev in &mut devices {
        if dev.hostname == "_gateway" {
            dev.hostname = "Router".into();
        }
    }

    devices
}

// ── Router stats scrape ───────────────────────────────────────────────────────
fn get_hermes_stats() -> Stats {
    let offline = Stats {
        status: "OFFLINE (Check Wi-Fi Connection)".into(),
        clients: "0".into(), rssi: "N/A".into(), uptime: "N/A".into(),
        sent: "N/A".into(), received: "N/A".into(),
        sent_bytes: 0.0, received_bytes: 0.0,
    };

    let body = match reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(3))
        .build()
        .and_then(|c| c.get(ROUTER_URL).send())
        .and_then(|r| r.text())
    {
        Ok(b) => b,
        Err(_) => return offline,
    };

    let plain = Regex::new(r"<[^>]+>").unwrap().replace_all(&body, " ").to_string();

    let clients = Regex::new(r"(?i)AP\s+Clients:\s*(\d+)").unwrap()
        .captures(&plain).map(|c| c[1].to_string()).unwrap_or_else(|| "0".into());

    let rssi = Regex::new(r"(-\d+)\s*dBm").unwrap()
        .captures(&plain).map(|c| format!("{} dBm", &c[1])).unwrap_or_else(|| "N/A".into());

    let uptime = Regex::new(r"Uptime:\s*([\d:]+(?:\s+\([^)]+\))?)").unwrap()
        .captures(&plain).map(|c| c[1].trim().to_string()).unwrap_or_else(|| "N/A".into());

    let bw = Regex::new(r"(?i)(\d+(?:\.\d+)?\s*\w+)\s+sent\s*/\s*(\d+(?:\.\d+)?\s*\w+)\s+received").unwrap();
    let (sent, received, sent_bytes, received_bytes) = bw.captures(&plain)
        .map(|c| {
            let s = c[1].trim().to_string();
            let r = c[2].trim().to_string();
            let sb = parse_bytes(&s);
            let rb = parse_bytes(&r);
            (s, r, sb, rb)
        })
        .unwrap_or_else(|| ("0 MB".into(), "0 MB".into(), 0.0, 0.0));

    Stats { status: "ONLINE".into(), clients, rssi, uptime, sent, received, sent_bytes, received_bytes }
}

// ── Render ────────────────────────────────────────────────────────────────────
fn render(
    d: &Stats,
    upload: &str,
    download: &str,
    latency: &str,
    scan_open: bool,
    devices: &[Device],
    scanning: bool,
) {
    clear_terminal();
    println!();
    println!("  Router Agent");
    println!();
    println!("  System Status   : {}", d.status);
    println!("  Internet Ping   : {}", latency);
    println!("  Signal (RSSI)   : {}", d.rssi);
    println!("  Active Clients  : {} device(s) connected", d.clients);
    println!("  Router Uptime   : {}", d.uptime);
    println!();
    println!("  Data Uploaded   : {}  ↑ {}", d.sent, upload);
    println!("  Data Downloaded : {}  ↓ {}", d.received, download);
    println!();
    println!("  [s] Network Scan");
    println!();

    if scan_open {
        println!("  ── Network Devices ────────────────────────────────");
        if scanning {
            println!("  Scanning... (this takes ~3 seconds)");
        } else if devices.is_empty() {
            println!("  No devices found.");
        } else {
            for dev in devices {
                let mac_str = if dev.mac.is_empty() { "           (this device)".into() }
                              else { format!("  {}", dev.mac) };
                println!("  {:<16}  {:<20}{}", dev.ip, dev.hostname, mac_str);
            }
        }
        println!("  ───────────────────────────────────────────────────");
        println!();
    }
}

// ── Main ──────────────────────────────────────────────────────────────────────
fn main() {
    set_raw_mode(true);

    // Shared state between main loop and keyboard thread
    let scan_open: Arc<Mutex<bool>>      = Arc::new(Mutex::new(false));
    let scanning:  Arc<Mutex<bool>>      = Arc::new(Mutex::new(false));
    let devices:   Arc<Mutex<Vec<Device>>> = Arc::new(Mutex::new(vec![]));

    // Keyboard listener thread
    let scan_open_kb = Arc::clone(&scan_open);
    let scanning_kb  = Arc::clone(&scanning);
    let devices_kb   = Arc::clone(&devices);

    thread::spawn(move || {
        let stdin = io::stdin();
        let mut buf = [0u8; 1];
        loop {
            if stdin.lock().read(&mut buf).is_ok() {
                if buf[0] == b's' || buf[0] == b'S' {
                    let mut open = scan_open_kb.lock().unwrap();
                    *open = !*open;
                    if *open {
                        // Start scan in background
                        let scanning2 = Arc::clone(&scanning_kb);
                        let devices2  = Arc::clone(&devices_kb);
                        *scanning_kb.lock().unwrap() = true;
                        thread::spawn(move || {
                            let found = run_nmap_scan();
                            *devices2.lock().unwrap()  = found;
                            *scanning2.lock().unwrap() = false;
                        });
                    }
                }
            }
        }
    });

    let mut prev_sent:     f64 = 0.0;
    let mut prev_received: f64 = 0.0;
    let mut prev_time = Instant::now();
    let mut first = true;

    loop {
        let stats   = get_hermes_stats();
        let latency = get_ping_latency();
        let now     = Instant::now();
        let elapsed = now.duration_since(prev_time).as_secs_f64();

        let (upload, download) = if first || elapsed == 0.0 {
            ("-- KB/s".into(), "-- KB/s".into())
        } else {
            let up   = (stats.sent_bytes     - prev_sent)     / elapsed;
            let down = (stats.received_bytes - prev_received) / elapsed;
            (format_speed(up), format_speed(down))
        };

        {
            let open     = *scan_open.lock().unwrap();
            let scanning = *scanning.lock().unwrap();
            let devs     = devices.lock().unwrap().clone();
            render(&stats, &upload, &download, &latency, open, &devs, scanning);
        }

        prev_sent     = stats.sent_bytes;
        prev_received = stats.received_bytes;
        prev_time     = now;
        first         = false;

        thread::sleep(Duration::from_secs(2));
    }
}