use std::fs;

#[derive(Debug, Clone, PartialEq)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub uid: u32,
    /// Usage in % of a single core (can exceed 100 for multi-threaded processes).
    pub cpu_percent: f32,
    /// Resident memory, in bytes.
    pub rss: u64,
    pub cmdline: String,
}

/// Process time (utime+stime) in ticks — for delta computation.
pub type ProcTicks = u64;

pub struct ParsedStat {
    pub name: String,
    pub ticks: ProcTicks,
}

/// Parses /proc/[pid]/stat. The name in parens can contain spaces and
/// parens itself, so we look for the LAST ')'.
pub fn parse_pid_stat(text: &str) -> Option<ParsedStat> {
    let open = text.find('(')?;
    let close = text.rfind(')')?;
    if close < open {
        return None;
    }
    let name = text[open + 1..close].to_string();
    // after ')': state(0) ppid(1) ... utime(11) stime(12)
    let f: Vec<&str> = text[close + 1..].split_whitespace().collect();
    let utime: u64 = f.get(11)?.parse().ok()?;
    let stime: u64 = f.get(12)?.parse().ok()?;
    Some(ParsedStat { name, ticks: utime + stime })
}

/// (uid, rss in bytes) from /proc/[pid]/status.
pub fn parse_pid_status(text: &str) -> (u32, u64) {
    let (mut uid, mut rss) = (0, 0);
    for line in text.lines() {
        if let Some(v) = line.strip_prefix("Uid:") {
            uid = v.split_whitespace().next().and_then(|x| x.parse().ok()).unwrap_or(0);
        } else if let Some(v) = line.strip_prefix("VmRSS:") {
            rss = v.split_whitespace().next().and_then(|x| x.parse::<u64>().ok()).unwrap_or(0) * 1024;
        }
    }
    (uid, rss)
}

pub fn parse_cmdline(raw: &[u8]) -> String {
    let parts: Vec<String> = raw
        .split(|&b| b == 0)
        .filter(|p| !p.is_empty())
        .map(|p| String::from_utf8_lossy(p).into_owned())
        .collect();
    parts.join(" ")
}

/// A single process: (info without cpu_percent, ticks). None if the process vanished/unavailable.
pub fn read_process(pid: u32) -> Option<(ProcessInfo, ProcTicks)> {
    let base = format!("/proc/{pid}");
    let st = parse_pid_stat(&fs::read_to_string(format!("{base}/stat")).ok()?)?;
    let (uid, rss) = fs::read_to_string(format!("{base}/status")).map(|t| parse_pid_status(&t)).unwrap_or((0, 0));
    let mut cmdline = fs::read(format!("{base}/cmdline")).map(|r| parse_cmdline(&r)).unwrap_or_default();
    if cmdline.is_empty() {
        cmdline = format!("[{}]", st.name); // kernel thread
    }
    Some((ProcessInfo { pid, name: st.name, uid, cpu_percent: 0.0, rss, cmdline }, st.ticks))
}

pub fn list_pids() -> Vec<u32> {
    fs::read_dir("/proc")
        .map(|rd| rd.flatten().filter_map(|e| e.file_name().to_str()?.parse().ok()).collect())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stat_with_tricky_name() {
        let t = "42 (my (weird) app) S 1 42 42 0 -1 4194560 100 0 0 0 150 50 0 0 20 0 1 0 100 1000 10 0";
        let p = parse_pid_stat(t).unwrap();
        assert_eq!(p.name, "my (weird) app");
        assert_eq!(p.ticks, 200);
    }

    #[test]
    fn status() {
        assert_eq!(parse_pid_status("Name:\tx\nUid:\t1000\t1000\t1000\t1000\nVmRSS:\t  2048 kB\n"), (1000, 2048 * 1024));
    }

    #[test]
    fn cmdline() {
        assert_eq!(parse_cmdline(b"/bin/sh\0-c\0echo\0"), "/bin/sh -c echo");
        assert_eq!(parse_cmdline(b""), "");
    }

    #[test]
    fn reads_self() {
        let (p, _) = read_process(std::process::id()).unwrap();
        assert!(p.rss > 0);
    }
}
