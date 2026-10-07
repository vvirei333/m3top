use std::fs;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NetInterface {
    pub name: String,
    pub rx_bytes: u64,
    pub tx_bytes: u64,
}

/// Per-interface counters (loopback excluded) + rate in bytes/s (filled in by Sampler).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct NetStats {
    pub interfaces: Vec<NetInterface>,
    pub rx_bytes: u64,
    pub tx_bytes: u64,
    pub rx_per_sec: f64,
    pub tx_per_sec: f64,
}

pub fn parse_net_dev(text: &str) -> NetStats {
    let mut s = NetStats::default();
    for line in text.lines().skip(2) {
        let Some((name, rest)) = line.split_once(':') else { continue };
        let name = name.trim();
        if name == "lo" {
            continue;
        }
        let f: Vec<u64> = rest.split_whitespace().map(|v| v.parse().unwrap_or(0)).collect();
        if f.len() < 9 {
            continue;
        }
        s.rx_bytes += f[0];
        s.tx_bytes += f[8];
        s.interfaces.push(NetInterface { name: name.to_string(), rx_bytes: f[0], tx_bytes: f[8] });
    }
    s
}

pub fn read_net_dev() -> Option<NetStats> {
    fs::read_to_string("/proc/net/dev").ok().map(|t| parse_net_dev(&t))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_and_skips_lo() {
        let t = "Inter-|   Receive |  Transmit\n face |bytes packets errs drop fifo frame compressed multicast|bytes packets errs drop fifo colls carrier compressed\n    lo: 100 1 0 0 0 0 0 0 100 1 0 0 0 0 0 0\n  eth0: 1000 5 0 0 0 0 0 0 2000 5 0 0 0 0 0 0\n";
        let s = parse_net_dev(t);
        assert_eq!(s.interfaces.len(), 1);
        assert_eq!((s.rx_bytes, s.tx_bytes), (1000, 2000));
    }
}
