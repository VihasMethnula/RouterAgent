import requests
from bs4 import BeautifulSoup
import time
import re
import os

ROUTER_URL = "http://192.168.4.1"

def clear_terminal():
    """Clears the terminal screen to keep the real-time layout static."""
    os.system('cls' if os.name == 'nt' else 'clear')

def get_hermes_stats():
    try:
        response = requests.get(ROUTER_URL, timeout=3)
        
        if response.status_code == 200:
            soup = BeautifulSoup(response.text, 'html.parser')
            page_text = soup.get_text()
            
            # 1. Parse Connected Clients (Matches 'AP Clients:X')
            clients = "0"
            match_clients = re.search(r'AP\s+Clients:\s*(\d+)', page_text, re.IGNORECASE)
            if match_clients:
                clients = match_clients.group(1)

            # 2. Parse Signal Strength (Extracts the negative number inside brackets near Uplink)
            rssi = "N/A"
            match_rssi = re.search(r'(-\d+)\s*dBm', page_text)
            if match_rssi:
                rssi = f"{match_rssi.group(1)} dBm"

            # 3. Parse System Uptime
            uptime = "N/A"
            if "Uptime:" in page_text:
                match_time = re.search(r'Uptime:\s*([\d:]+(?:\s+\([^)]+\))?)', page_text)
                if match_time:
                    uptime = match_time.group(1).strip()

            # 4. Parse Bandwidth Usage Data (Matches 'Bytes:X sent / Y received')
            sent_data = "0 MB"
            recv_data = "0 MB"
            match_bytes = re.search(r'(\d+(?:\.\d+)?\s*\w+)\s*sent\s*/\s*(\d+(?:\.\d+)?\s*\w+)\s*received', page_text, re.IGNORECASE)
            if match_bytes:
                sent_data = match_bytes.group(1).strip()
                recv_data = match_bytes.group(2).strip()

            return {
                "status": "ONLINE",
                "clients": clients,
                "rssi": rssi,
                "uptime": uptime,
                "sent": sent_data,
                "received": recv_data
            }
            
    except requests.exceptions.RequestException:
        return {
            "status": "OFFLINE (Check Wi-Fi Connection)",
            "clients": "0",
            "rssi": "N/A",
            "uptime": "N/A",
            "sent": "N/A",
            "received": "N/A"
        }

def render_dashboard(data):
    """Prints a clean, structured terminal layout block."""
    clear_terminal()
    print(" " * 55)
    print("  Router Agent ")
    print(" " * 55)
    print(f"  System Status   : {data['status']}")
    print(f"  Signal (RSSI)   : {data['rssi']}")
    print(f"  Active Clients  : {data['clients']} device(s) connected")
    print(f"  Router Uptime   : {data['uptime']}")
    print(" " * 55)
    print(f"  Data Uploaded   : {data['sent']}")
    print(f"  Data Downloaded : {data['received']}")
    print(" " * 55)
    print(" ")

# --- Main Runtime Execution ---
if __name__ == "__main__":
    try:
        while True:
            metrics = get_hermes_stats()
            render_dashboard(metrics)
            time.sleep(2)  # Updates layout every 2 seconds
    except KeyboardInterrupt:
        clear_terminal()
        print("\n[!] Hermes monitoring pipeline stopped cleanly.\n")