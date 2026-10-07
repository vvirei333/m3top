use std::fs;

/// CPU time counters (in ticks) from a single /proc/stat line.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct CpuTimes {
    pub user: u64,
    pub nice: u64,
    pub system: u64,
    pub idle: u64,
    pub iowait: u64,
    pub irq: u64,
    pub softirq: u64,
    pub steal: u64,
}

impl CpuTimes {
    pub fn total(&self) -> u64 {
        self.user + self.nice + self.system + self.idle + self.iowait + self.irq + self.softirq + self.steal
    }

    pub fn idle_all(&self) -> u64 {
        self.idle + self.iowait
    }

    /// Usage (0..=100) between two samples.
    pub fn usage_since(&self, prev: &CpuTimes) -> f32 {
        let total = self.total().saturating_sub(prev.total());
        if total == 0 {
            return 0.0;
        }
        let idle = self.idle_all().saturating_sub(prev.idle_all());
        (total.saturating_sub(idle) as f32 / total as f32 * 100.0).clamp(0.0, 100.0)
    }
}

/// Raw counters: total + per-core.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CpuSample {
    pub total: CpuTimes,
    pub cores: Vec<CpuTimes>,
}

/// CPU usage in percent.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct CpuStats {
    pub total: f32,
    pub cores: Vec<f32>,
}

impl CpuStats {
    pub fn between(prev: &CpuSample, cur: &CpuSample) -> CpuStats {
        let cores = cur
            .cores
            .iter()
            .enumerate()
            .map(|(i, c)| prev.cores.get(i).map_or(0.0, |p| c.usage_since(p)))
            .collect();
        CpuStats { total: cur.total.usage_since(&prev.total), cores }
    }
}

fn parse_line(rest: &str) -> Option<CpuTimes> {
    let mut it = rest.split_whitespace().map(|v| v.parse::<u64>().unwrap_or(0));
    Some(CpuTimes {
        user: it.next()?,
        nice: it.next()?,
        system: it.next()?,
        idle: it.next()?,
        iowait: it.next().unwrap_or(0),
        irq: it.next().unwrap_or(0),
        softirq: it.next().unwrap_or(0),
        steal: it.next().unwrap_or(0),
    })
}

pub fn parse_stat(text: &str) -> Option<CpuSample> {
    let mut total = None;
    let mut cores = Vec::new();
    for line in text.lines() {
        let Some((name, rest)) = line.split_once(char::is_whitespace) else { continue };
        if name == "cpu" {
            total = parse_line(rest);
        } else if name.strip_prefix("cpu").is_some_and(|n| n.chars().all(|c| c.is_ascii_digit())) {
            cores.push(parse_line(rest)?);
        }
    }
    Some(CpuSample { total: total?, cores })
}

pub fn read_stat() -> Option<CpuSample> {
    parse_stat(&fs::read_to_string("/proc/stat").ok()?)
}

#[cfg(test)]
mod tests {
    use super::*;

    const A: &str = "cpu  100 0 100 800 0 0 0 0 0 0\ncpu0 50 0 50 400 0 0 0 0 0 0\ncpu1 50 0 50 400 0 0 0 0 0 0\nintr 1 2 3\n";
    const B: &str = "cpu  200 0 200 1000 0 0 0 0 0 0\ncpu0 150 0 50 400 0 0 0 0 0 0\ncpu1 50 0 150 600 0 0 0 0 0 0\n";

    #[test]
    fn parses_total_and_cores() {
        let s = parse_stat(A).unwrap();
        assert_eq!(s.cores.len(), 2);
        assert_eq!(s.total.total(), 1000);
    }

    #[test]
    fn computes_delta() {
        let st = CpuStats::between(&parse_stat(A).unwrap(), &parse_stat(B).unwrap());
        assert!((st.total - 50.0).abs() < 0.01);
        assert!((st.cores[0] - 100.0).abs() < 0.01);
        assert!((st.cores[1] - 33.333).abs() < 0.1);
    }

    #[test]
    fn garbage_is_none() {
        assert!(parse_stat("nonsense").is_none());
    }
}
