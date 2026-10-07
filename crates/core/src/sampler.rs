use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use crate::cpu::{self, CpuSample, CpuStats};
use crate::mem::{self, MemStats};
use crate::net::{self, NetStats};
use crate::process::{self, ProcessInfo};
use crate::thermal::{self, ThermalZone};

/// A single snapshot of all metrics. `None` / empty Vec means the source is unavailable.
#[derive(Debug, Clone, Default)]
pub struct Snapshot {
    pub cpu: Option<CpuStats>,
    pub mem: Option<MemStats>,
    pub thermal: Vec<ThermalZone>,
    pub net: Option<NetStats>,
    pub processes: Vec<ProcessInfo>,
}

/// Keeps previous samples and computes deltas.
#[derive(Default)]
pub struct Sampler {
    prev_cpu: Option<CpuSample>,
    prev_proc: HashMap<u32, u64>,
    prev_net: Option<(u64, u64, Instant)>,
}

impl Sampler {
    pub fn new() -> Self {
        Self::default()
    }

    /// The first call yields zeros for delta metrics; from the second call on — real values.
    pub fn sample(&mut self) -> Snapshot {
        let now = Instant::now();

        let cur_cpu = cpu::read_stat();
        let cpu_delta_total = match (&self.prev_cpu, &cur_cpu) {
            (Some(p), Some(c)) => Some(c.total.total().saturating_sub(p.total.total())),
            _ => None,
        };
        let cpu = match (&self.prev_cpu, &cur_cpu) {
            (Some(p), Some(c)) => Some(CpuStats::between(p, c)),
            (None, Some(c)) => Some(CpuStats { total: 0.0, cores: vec![0.0; c.cores.len()] }),
            _ => None,
        };
        let ncores = cur_cpu.as_ref().map_or(1, |c| c.cores.len().max(1)) as f32;
        self.prev_cpu = cur_cpu;

        let mut processes = Vec::new();
        let mut new_prev = HashMap::new();
        for pid in process::list_pids() {
            let Some((mut info, ticks)) = process::read_process(pid) else { continue };
            if let (Some(&old), Some(dt)) = (self.prev_proc.get(&pid), cpu_delta_total) {
                if dt > 0 {
                    // dt is the tick sum across all cores; this process's share of a single core:
                    info.cpu_percent = ticks.saturating_sub(old) as f32 / dt as f32 * 100.0 * ncores;
                }
            }
            new_prev.insert(pid, ticks);
            processes.push(info);
        }
        self.prev_proc = new_prev;

        let net = net::read_net_dev().map(|mut n| {
            if let Some((prx, ptx, pt)) = self.prev_net {
                let dt = now.duration_since(pt).as_secs_f64().max(0.001);
                n.rx_per_sec = n.rx_bytes.saturating_sub(prx) as f64 / dt;
                n.tx_per_sec = n.tx_bytes.saturating_sub(ptx) as f64 / dt;
            }
            self.prev_net = Some((n.rx_bytes, n.tx_bytes, now));
            n
        });

        Snapshot { cpu, mem: mem::read_meminfo(), thermal: thermal::read_zones(), net, processes }
    }
}

/// Shared handle: the UI reads the latest snapshot without blocking on polling.
#[derive(Clone, Default)]
pub struct SnapshotHandle(Arc<Mutex<Arc<Snapshot>>>);

impl SnapshotHandle {
    pub fn latest(&self) -> Arc<Snapshot> {
        self.0.lock().map(|g| g.clone()).unwrap_or_default()
    }

    /// Spawns a background thread with the given interval. `on_update` is e.g. egui's repaint.
    pub fn spawn(interval: Duration, on_update: impl Fn() + Send + 'static) -> SnapshotHandle {
        let handle = SnapshotHandle::default();
        let h = handle.clone();
        thread::spawn(move || {
            let mut sampler = Sampler::new();
            loop {
                let snap = Arc::new(sampler.sample());
                if let Ok(mut g) = h.0.lock() {
                    *g = snap;
                }
                on_update();
                thread::sleep(interval);
            }
        });
        handle
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn two_samples_work() {
        let mut s = Sampler::new();
        let a = s.sample();
        assert!(!a.processes.is_empty());
        std::thread::sleep(Duration::from_millis(100));
        let b = s.sample();
        assert!(b.cpu.is_some());
        assert!(b.mem.unwrap().total > 0);
    }
}
