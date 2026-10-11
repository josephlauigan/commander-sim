//! `lan_addresses()`: this computer's addresses on the local network (a best guess: the one with the route out first).

use std::net::{IpAddr, ToSocketAddrs, UdpSocket};

pub fn lan_addresses() -> Vec<String> {
    let mut out: Vec<String> = vec![];
    // no packet is sent: connecting a UDP socket only picks the interface
    if let Ok(u) = UdpSocket::bind("0.0.0.0:0")
        && u.connect("10.255.255.255:1").is_ok()
        && let Ok(a) = u.local_addr()
    {
        out.push(a.ip().to_string());
    }
    let host = std::fs::read_to_string("/proc/sys/kernel/hostname")
        .or_else(|_| std::fs::read_to_string("/etc/hostname"))
        .map(|s| s.trim().to_string())
        .unwrap_or_default();
    if !host.is_empty()
        && let Ok(addrs) = (host.as_str(), 0).to_socket_addrs()
    {
        for a in addrs {
            if let IpAddr::V4(ip) = a.ip() {
                let s = ip.to_string();
                if !s.starts_with("127.") && !out.contains(&s) {
                    out.push(s);
                }
            }
        }
    }
    out.retain(|ip| !ip.starts_with("127."));
    out
}

#[cfg(test)]
mod tests {
    #[test]
    fn never_loopback() {
        assert!(super::lan_addresses().iter().all(|a| !a.starts_with("127.")));
    }
}
