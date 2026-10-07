use std::fs;

/// Memory in bytes.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct MemStats {
    pub total: u64,
    pub available: u64,
    pub swap_total: u64,
    pub swap_free: u64,
}

impl MemStats {
    pub fn used(&self) -> u64 {
        self.total.saturating_sub(self.available)
    }
    pub fn swap_used(&self) -> u64 {
        self.swap_total.saturating_sub(self.swap_free)
    }
    pub fn used_percent(&self) -> f32 {
        if self.total == 0 { 0.0 } else { self.used() as f32 / self.total as f32 * 100.0 }
    }
    pub fn swap_percent(&self) -> f32 {
        if self.swap_total == 0 { 0.0 } else { self.swap_used() as f32 / self.swap_total as f32 * 100.0 }
    }
}

pub fn parse_meminfo(text: &str) -> Option<MemStats> {
    let mut m = MemStats::default();
    let mut have_total = false;
    let mut have_avail = false;
    let mut memfree = 0;
    for line in text.lines() {
        let Some((key, val)) = line.split_once(':') else { continue };
        let mut parts = val.split_whitespace();
        let Some(n) = parts.next().and_then(|v| v.parse::<u64>().ok()) else { continue };
        let bytes = if parts.next() == Some("kB") { n * 1024 } else { n };
        match key {
            "MemTotal" => { m.total = bytes; have_total = true }
            "MemAvailable" => { m.available = bytes; have_avail = true }
            "MemFree" => memfree = bytes,
            "SwapTotal" => m.swap_total = bytes,
            "SwapFree" => m.swap_free = bytes,
            _ => {}
        }
    }
    if !have_avail {
        m.available = memfree; // very old kernels
    }
    have_total.then_some(m)
}

pub fn read_meminfo() -> Option<MemStats> {
    parse_meminfo(&fs::read_to_string("/proc/meminfo").ok()?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses() {
        let t = "MemTotal:       16000 kB\nMemFree: 1000 kB\nMemAvailable:    4000 kB\nSwapTotal: 2000 kB\nSwapFree: 500 kB\n";
        let m = parse_meminfo(t).unwrap();
        assert_eq!(m.total, 16000 * 1024);
        assert_eq!(m.used(), 12000 * 1024);
        assert_eq!(m.swap_used(), 1500 * 1024);
        assert!((m.used_percent() - 75.0).abs() < 0.01);
    }

    #[test]
    fn missing_total_is_none() {
        assert!(parse_meminfo("MemFree: 1 kB\n").is_none());
    }
}
