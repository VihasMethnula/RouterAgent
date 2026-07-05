use std::{
    io::{self, Read},
    sync::{Arc, Mutex},
    thread,
    time::{Duration, Instant},
};
use regex::Regex;

const ROUTER_URL: &str = "http://192.168.4.1";
const NETWORK:    &str = "192.168.4.0/24";

// ── ASCII Art (edit this to customize) ────────────────────────────────────────
const ASCII_ART: &str = r#"
▗▄▄▖  ▗▄▖ ▗▖ ▗▖▗▄▄▄▖▗▄▄▄▖▗▄▄▖      ▗▄▖  ▗▄▄▖▗▄▄▄▖▗▖  ▗▖▗▄▄▄▖
▐▌ ▐▌▐▌ ▐▌▐▌ ▐▌  █  ▐▌   ▐▌ ▐▌    ▐▌ ▐▌▐▌   ▐▌   ▐▛▚▖▐▌  █  
▐▛▀▚▖▐▌ ▐▌▐▌ ▐▌  █  ▐▛▀▀▘▐▛▀▚▖    ▐▛▀▜▌▐▌▝▜▌▐▛▀▀▘▐▌ ▝▜▌  █  
▐▌ ▐▌▝▚▄▞▘▝▚▄▞▘  █  ▐▙▄▄▖▐▌ ▐▌    ▐▌ ▐▌▝▚▄▞▘▐▙▄▄▖▐▌  ▐▌  █  
              By Vihas Methnula :)                                                            
"#;

// ── ANSI Colors ───────────────────────────────────────────────────────────────
const RESET:   &str = "\x1b[0m";
const BOLD:    &str = "\x1b[1m";
const DIM:     &str = "\x1b[2m";
const RED:     &str = "\x1b[31m";
const GREEN:   &str = "\x1b[32m";
const YELLOW:  &str = "\x1b[33m";
const BLUE:    &str = "\x1b[34m";
const MAGENTA: &str = "\x1b[35m";
const CYAN:    &str = "\x1b[36m";
const WHITE:   &str = "\x1b[37m";

// ── Color helpers ─────────────────────────────────────────────────────────────
fn color_status(status: &str) -> String {
    if status.contains("ONLINE") {
        format!("{}{}● ONLINE{}", GREEN, BOLD, RESET)
    } else {
        format!("{}{}● OFFLINE{}", RED, BOLD, RESET)
    }
}

fn color_ping(ping: &str) -> String {
    if ping == "Timeout" {
        return format!("{}{} Timeout{}", RED, BOLD, RESET);
    }
    let ms: f64 = ping.replace(" ms", "").trim().parse().unwrap_or(9999.0);
    if ms < 50.0 {
        format!("{}{}{} ms{}", GREEN, BOLD, ping, RESET)
    } else if ms < 150.0 {
        format!("{}{}{} ms{}", YELLOW, BOLD, ping, RESET)
    } else {
        format!("{}{}{} ms{}", RED, BOLD, ping, RESET)
    }
}

fn color_rssi(rssi: &str) -> String {
    if rssi == "N/A" {
        return format!("{}{}N/A{}", DIM, BOLD, RESET);
    }
    let dbm: f64 = rssi.replace(" dBm", "").trim().parse().unwrap_or(-999.0);
    if dbm >= -50.0 {
        format!("{}{}{} (Excellent){}", GREEN, BOLD, rssi, RESET)
    } else if dbm >= -65.0 {
        format!("{}{}{} (Good){}", GREEN, BOLD, rssi, RESET)
    } else if dbm >= -75.0 {
        format!("{}{}{} (Fair){}", YELLOW, BOLD, rssi, RESET)
    } else {
        format!("{}{}{} (Weak){}", RED, BOLD, rssi, RESET)
    }
}

fn color_clients(clients: &str) -> String {
    let n: i32 = clients.parse().unwrap_or(0);
    if n == 0 {
        format!("{}{}0 device(s){}", RED, BOLD, RESET)
    } else if n <= 3 {
        format!("{}{}{} device(s){}", GREEN, BOLD, clients, RESET)
    } else if n <= 8 {
        format!("{}{}{} device(s){}", YELLOW, BOLD, clients, RESET)
    } else {
        format!("{}{}{} device(s){}", RED, BOLD, clients, RESET)
    }
}

// ── Spinner animation ─────────────────────────────────────────────────────────
const SPINNER_CHARS: &[char] = &['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];

fn spinner_frame(tick: usize) -> char {
    SPINNER_CHARS[tick % SPINNER_CHARS.len()]
}

// ── Pulsing dot animation ─────────────────────────────────────────────────────
const PULSE_CHARS: &[&str] = &["○", "◎", "●", "◎"];

fn pulse_frame(tick: usize) -> &'static str {
    PULSE_CHARS[tick % PULSE_CHARS.len()]
}

// ── Progress bar ──────────────────────────────────────────────────────────────
fn progress_bar(value: f64, max: f64, width: usize) -> String {
    let ratio = if max > 0.0 { (value / max).min(1.0) } else { 0.0 };
    let filled = (ratio * width as f64) as usize;
    let empty = width.saturating_sub(filled);

    let bar_color = if ratio < 0.3 {
        GREEN
    } else if ratio < 0.7 {
        YELLOW
    } else {
        RED
    };

    format!("{}{}{}{}{}{}{}{}",
        bar_color, BOLD,
        "█".repeat(filled),
        DIM,
        "░".repeat(empty),
        RESET, RESET, RESET
    )
}

// ── Header color cycling ──────────────────────────────────────────────────────
const HEADER_COLORS: &[&str] = &[CYAN, BLUE, MAGENTA, RED, YELLOW, GREEN];

fn header_color(tick: usize) -> &'static str {
    HEADER_COLORS[tick % HEADER_COLORS.len()]
}

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
    print!("\x1b[2J\x1b[H");
    io::Write::flush(&mut io::stdout()).ok();
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

    let re_host = Regex::new(r"Nmap scan report for (.+?) \((\d+\.\d+\.\d+\.\d+)\)|Nmap scan report for (\d+\.\d+\.\d+\.\d+)").unwrap();
    let re_mac  = Regex::new(r"MAC Address: ([0-9A-F:]{17})").unwrap();

    for line in text.lines() {
        if let Some(cap) = re_host.captures(line) {
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
                current_host = format!("{}|{}", cap[1].trim(), cap[2].trim());
            } else {
                current_host = format!("{}|{}", cap[3].trim(), cap[3].trim());
            }
        } else if let Some(cap) = re_mac.captures(line) {
            current_mac = cap[1].to_string();
        }
    }
    if !current_host.is_empty() {
        let parts: Vec<&str> = current_host.splitn(2, '|').collect();
        devices.push(Device {
            hostname: parts.get(0).unwrap_or(&"").to_string(),
            ip:       parts.get(1).unwrap_or(&"").to_string(),
            mac:      current_mac,
        });
    }
    for dev in &mut devices {
        match dev.hostname.as_str() {
            "_gateway"    => dev.hostname = "Router".into(),
            "192.168.4.1" => dev.hostname = "Router".into(),
            "192.168.4.2" => dev.hostname = "Methnula".into(),
            _             => {}
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
    tick: usize,
    max_upload: f64,
    max_download: f64,
    sent_bytes: f64,
    received_bytes: f64,
) {
    clear_terminal();

    let hc = header_color(tick);
    println!("{}{}{}{}{}", hc, BOLD, ASCII_ART, RESET, RESET);

    println!("{}{}─────────────────────────────────────────────────────────────{}", DIM, BOLD, RESET);
    println!();

    let pulse = pulse_frame(tick);
    let status_color = if d.status.contains("ONLINE") { GREEN } else { RED };
    println!("  {}{}{} Status    : {}{}{}", status_color, BOLD, pulse, RESET, color_status(&d.status), RESET);

    println!("  {}{}◈{} Internet  : {}", CYAN, BOLD, RESET, color_ping(latency));

    println!("  {}{}◈{} Signal    : {}", CYAN, BOLD, RESET, color_rssi(&d.rssi));

    println!("  {}{}◈{} Clients   : {}", CYAN, BOLD, RESET, color_clients(&d.clients));

    println!("  {}{}◈{} Uptime    : {}{}{}{}{}", CYAN, BOLD, RESET, WHITE, BOLD, d.uptime, RESET, RESET);

    println!();

    let up_bar = progress_bar(sent_bytes, max_upload, 30);
    println!("  {}↑{} Upload    : {}{} {}", GREEN, RESET, WHITE, d.sent, upload);
    println!("  {}  {}           {} {}", DIM, RESET, up_bar, RESET);

    let down_bar = progress_bar(received_bytes, max_download, 30);
    println!("  {}↓{} Download  : {}{} {}", BLUE, RESET, WHITE, d.received, download);
    println!("  {}  {}           {} {}", DIM, RESET, down_bar, RESET);

    println!();
    println!("{}{}─────────────────────────────────────────────────────────────{}", DIM, BOLD, RESET);
    println!();

    if scan_open {
        println!("  {}{}[s] Network Scan{}", YELLOW, BOLD, RESET);
    } else {
        println!("  {}[s] Network Scan{}", DIM, RESET);
    }
    println!();

    if scan_open {
        println!("  {}{}── Network Devices ─────────────────────────────────────{}", CYAN, BOLD, RESET);
        if scanning {
            let sp = spinner_frame(tick);
            println!("  {}{}{} Scanning network{}... (this takes ~3 seconds){}", MAGENTA, BOLD, sp, RESET, RESET);
        } else if devices.is_empty() {
            println!("  {}No devices found.{}", RED, RESET);
        } else {
            println!("  {}{}{:<16}  {:<20}  {}{}", DIM, BOLD, "IP ADDRESS", "HOSTNAME", "MAC ADDRESS", RESET);
            println!("  {}──────────────────────────────────────────────────────────{}", DIM, RESET);
            for dev in devices {
                let mac_str = if dev.mac.is_empty() { "(this device)".into() }
                              else { dev.mac.as_str() };
                let host_color = match dev.hostname.as_str() {
                    "Router" => CYAN,
                    "Methnula" => MAGENTA,
                    _ => WHITE,
                };
                println!("  {}{:<16}  {}{:<20}{}  {}{}{}",
                    WHITE, dev.ip,
                    host_color, dev.hostname, RESET,
                    DIM, mac_str, RESET
                );
            }
        }
        println!("  {}──────────────────────────────────────────────────────────{}", DIM, RESET);
        println!();
    }

    println!("{}  Press [s] to toggle scan  •  Press [q] to quit{}", DIM, RESET);
    println!();
}

// ── Main ──────────────────────────────────────────────────────────────────────
fn main() {
    set_raw_mode(true);

    let scan_open: Arc<Mutex<bool>>        = Arc::new(Mutex::new(false));
    let scanning:  Arc<Mutex<bool>>        = Arc::new(Mutex::new(false));
    let devices:   Arc<Mutex<Vec<Device>>> = Arc::new(Mutex::new(vec![]));
    let quit:      Arc<Mutex<bool>>        = Arc::new(Mutex::new(false));

    let scan_open_kb = Arc::clone(&scan_open);
    let scanning_kb  = Arc::clone(&scanning);
    let devices_kb   = Arc::clone(&devices);
    let quit_kb      = Arc::clone(&quit);

    thread::spawn(move || {
        let stdin = io::stdin();
        let mut buf = [0u8; 1];
        loop {
            if stdin.lock().read(&mut buf).is_ok() {
                match buf[0] {
                    b's' | b'S' => {
                        let mut open = scan_open_kb.lock().unwrap();
                        *open = !*open;
                        if *open {
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
                    b'q' | b'Q' => {
                        *quit_kb.lock().unwrap() = true;
                        break;
                    }
                    _ => {}
                }
            }
        }
    });

    let mut prev_sent:     f64 = 0.0;
    let mut prev_received: f64 = 0.0;
    let mut prev_time = Instant::now();
    let mut first = true;
    let mut tick: usize = 0;
    let mut max_upload: f64 = 1.0;
    let mut max_download: f64 = 1.0;

    loop {
        if *quit.lock().unwrap() {
            break;
        }

        let stats   = get_hermes_stats();
        let latency = get_ping_latency();
        let now     = Instant::now();
        let elapsed = now.duration_since(prev_time).as_secs_f64();

        let (upload, download) = if first || elapsed == 0.0 {
            ("-- KB/s".into(), "-- KB/s".into())
        } else {
            let up   = (stats.sent_bytes     - prev_sent)     / elapsed;
            let down = (stats.received_bytes - prev_received) / elapsed;
            if up > max_upload   { max_upload = up * 1.2; }
            if down > max_download { max_download = down * 1.2; }
            (format_speed(up), format_speed(down))
        };

        {
            let open     = *scan_open.lock().unwrap();
            let scanning = *scanning.lock().unwrap();
            let devs     = devices.lock().unwrap().clone();
            render(&stats, &upload, &download, &latency, open, &devs, scanning,
                   tick, max_upload, max_download, stats.sent_bytes, stats.received_bytes);
        }

        prev_sent     = stats.sent_bytes;
        prev_received = stats.received_bytes;
        prev_time     = now;
        first         = false;
        tick          = tick.wrapping_add(1);

        thread::sleep(Duration::from_millis(500));
    }

    set_raw_mode(false);
    clear_terminal();
    println!("{}{}Router Agent closed.{}", GREEN, BOLD, RESET);
}
