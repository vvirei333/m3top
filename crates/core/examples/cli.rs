use std::{thread, time::Duration};

use m3top_core::Sampler;

fn main() {
    let mut s = Sampler::new();
    s.sample();
    thread::sleep(Duration::from_secs(1));
    let snap = s.sample();

    match &snap.cpu {
        Some(c) => {
            println!("CPU total: {:.1}%", c.total);
            for (i, v) in c.cores.iter().enumerate() {
                println!("  cpu{i}: {v:.1}%");
            }
        }
        None => println!("CPU: unavailable"),
    }
    match &snap.mem {
        Some(m) => println!(
            "RAM: {:.2}/{:.2} GiB ({:.0}%), swap {:.0}%",
            m.used() as f64 / 1073741824.0,
            m.total as f64 / 1073741824.0,
            m.used_percent(),
            m.swap_percent()
        ),
        None => println!("RAM: unavailable"),
    }
    if snap.thermal.is_empty() {
        println!("Temperatures: unavailable");
    }
    for z in &snap.thermal {
        println!("  {}: {:.1}°C", z.name, z.celsius);
    }
    if let Some(n) = &snap.net {
        println!("Network: down {:.0} B/s up {:.0} B/s ({} iface)", n.rx_per_sec, n.tx_per_sec, n.interfaces.len());
    }
    let mut p = snap.processes.clone();
    p.sort_by(|a, b| b.cpu_percent.total_cmp(&a.cpu_percent));
    println!("Processes: {}. Top by CPU:", p.len());
    for x in p.iter().take(8) {
        println!("  {:>7} {:<20} {:>6.1}% {:>8} MiB  {:.50}", x.pid, x.name, x.cpu_percent, x.rss / 1048576, x.cmdline);
    }
}
