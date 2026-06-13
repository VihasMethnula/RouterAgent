use std::{thread, time::Duration};
use regex::Regex;

const ROUTER_URL: &str = "http://192.168.4.1";

struct Stats {
    status:   String,
    clients:  String,
    rssi:     String,
    uptime:   String,
    sent:     String,
    received: String,
}

fn clear_terminal() {
    if cfg!(target_os = "windows") {
        std::process::Command::new("cmd").args(["/C", "cls"]).status().ok();
    } else {
        std::process::Command::new("clear").status().ok();
    }
}

fn get_hermes_stats() -> Stats {
    let offline = Stats {
        status:   "OFFLINE (Check Wi-Fi Connection)".into(),
        clients:  "0".into(),
        rssi:     "N/A".into(),
        uptime:   "N/A".into(),
        sent:     "N/A".into(),
        received: "N/A".into(),
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

    // Strip HTML tags to plain text (same as soup.get_text())
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

    // 4. Bandwidth
    let bw = Regex::new(r"(?i)(\d+(?:\.\d+)?\s*\w+)\s+sent\s*/\s*(\d+(?:\.\d+)?\s*\w+)\s+received")
        .unwrap();

    let (sent, received) = bw.captures(&plain)
        .map(|c| (c[1].trim().to_string(), c[2].trim().to_string()))
        .unwrap_or_else(|| ("0 MB".into(), "0 MB".into()));

    Stats {
        status: "ONLINE".into(),
        clients,
        rssi,
        uptime,
        sent,
        received,
    }
}

fn render_dashboard(d: &Stats) {
    clear_terminal();
    
    println!(" ");
    println!("  Router Agent ");
    println!(" ");
    println!("  System Status   : {}", d.status);
    println!("  Signal (RSSI)   : {}", d.rssi);
    println!("  Active Clients  : {} device(s) connected", d.clients);
    println!("  Router Uptime   : {}", d.uptime);
    println!(" ");
    println!("  Data Uploaded   : {}", d.sent);
    println!("  Data Downloaded : {}", d.received);
    println!(" ");
    println!();
}

fn main() {
    loop {
        let stats = get_hermes_stats();
        render_dashboard(&stats);
        thread::sleep(Duration::from_secs(2));
    }
}