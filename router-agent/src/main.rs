use std::{thread, time::{Duration, Instant}};
use regex::Regex;

const ROUTER_URL: &str = "http://192.168.4.1";

struct Stats {
    status:   String,
    clients:  String,
    rssi:     String,
    uptime:   String,
    sent:     String,
    received: String,
    sent_bytes:     f64,
    received_bytes: f64,
}

fn clear_terminal() {
    std::process::Command::new("clear").status().ok();
}

// Parse "6.8 MB", "150.0 MB", "1.2 GB", "512 KB" etc. into bytes as f64
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

fn format_speed(bytes_per_sec: f64) -> String {
    if bytes_per_sec < 0.0 {
        return "0 B/s".into(); // reset or counter wrap
    }
    if bytes_per_sec >= 1_073_741_824.0 {
        format!("{:.1} GB/s", bytes_per_sec / 1_073_741_824.0)
    } else if bytes_per_sec >= 1_048_576.0 {
        format!("{:.1} MB/s", bytes_per_sec / 1_048_576.0)
    } else if bytes_per_sec >= 1_024.0 {
        format!("{:.1} KB/s", bytes_per_sec / 1_024.0)
    } else {
        format!("{:.0} B/s", bytes_per_sec)
    }
}

fn get_ping_latency() -> String {
    let start = Instant::now();
    let result = std::process::Command::new("ping")
        .args(["-c", "1", "-W", "2", "8.8.8.8"])
        .output();

    match result {
        Ok(output) if output.status.success() => {
            let elapsed = start.elapsed().as_millis();
            format!("{} ms", elapsed)
        }
        _ => "Timeout".into(),
    }
}

fn get_hermes_stats() -> Stats {
    let offline = Stats {
        status:         "OFFLINE (Check Wi-Fi Connection)".into(),
        clients:        "0".into(),
        rssi:           "N/A".into(),
        uptime:         "N/A".into(),
        sent:           "N/A".into(),
        received:       "N/A".into(),
        sent_bytes:     0.0,
        received_bytes: 0.0,
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

    // 1. AP Clients
    let clients = Regex::new(r"(?i)AP\s+Clients:\s*(\d+)")
        .unwrap()
        .captures(&plain)
        .map(|c| c[1].to_string())
        .unwrap_or_else(|| "0".into());

    // 2. RSSI / Signal
    let rssi = Regex::new(r"(-\d+)\s*dBm")
        .unwrap()
        .captures(&plain)
        .map(|c| format!("{} dBm", &c[1]))
        .unwrap_or_else(|| "N/A".into());

    // 3. Uptime
    let uptime = Regex::new(r"Uptime:\s*([\d:]+(?:\s+\([^)]+\))?)")
        .unwrap()
        .captures(&plain)
        .map(|c| c[1].trim().to_string())
        .unwrap_or_else(|| "N/A".into());

    // 4. Bandwidth totals
    let bw = Regex::new(r"(?i)(\d+(?:\.\d+)?\s*\w+)\s+sent\s*/\s*(\d+(?:\.\d+)?\s*\w+)\s+received")
        .unwrap();

    let (sent, received, sent_bytes, received_bytes) = bw.captures(&plain)
        .map(|c| {
            let s = c[1].trim().to_string();
            let r = c[2].trim().to_string();
            let sb = parse_bytes(&s);
            let rb = parse_bytes(&r);
            (s, r, sb, rb)
        })
        .unwrap_or_else(|| ("0 MB".into(), "0 MB".into(), 0.0, 0.0));

    Stats {
        status: "ONLINE".into(),
        clients,
        rssi,
        uptime,
        sent,
        received,
        sent_bytes,
        received_bytes,
    }
}

fn render_dashboard(d: &Stats, upload_speed: &str, download_speed: &str, latency: &str) {
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
    println!("  Data Uploaded   : {}  ↑ {}", d.sent, upload_speed);
    println!("  Data Downloaded : {}  ↓ {}", d.received, download_speed);
    println!();
}

fn main() {
    let mut prev_sent:     f64 = 0.0;
    let mut prev_received: f64 = 0.0;
    let mut prev_time = Instant::now();
    let mut first = true;

    loop {
        let stats = get_hermes_stats();
        let now = Instant::now();
        let elapsed = now.duration_since(prev_time).as_secs_f64();

        let (upload_speed, download_speed) = if first || elapsed == 0.0 {
            ("-- KB/s".into(), "-- KB/s".into())
        } else {
            let up   = (stats.sent_bytes     - prev_sent)     / elapsed;
            let down = (stats.received_bytes - prev_received) / elapsed;
            (format_speed(up), format_speed(down))
        };

        let latency = get_ping_latency();
        render_dashboard(&stats, &upload_speed, &download_speed, &latency);

        prev_sent     = stats.sent_bytes;
        prev_received = stats.received_bytes;
        prev_time     = now;
        first         = false;

        thread::sleep(Duration::from_secs(2));
    }
}