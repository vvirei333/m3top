use std::fs;
use std::path::Path;

#[derive(Debug, Clone, PartialEq)]
pub struct ThermalZone {
    pub name: String,
    pub celsius: f32,
}

pub fn parse_millidegrees(text: &str) -> Option<f32> {
    text.trim().parse::<i64>().ok().map(|m| m as f32 / 1000.0)
}

/// Reads all zones from a directory like /sys/class/thermal. Skips unreadable
/// ones. These zones often only give a generic ACPI sensor (e.g. `acpitz`),
/// which on many machines doesn't reflect the real CPU package temperature —
/// hence [`read_zones`] prioritizes hwmon data (see [`read_hwmon_from`]).
pub fn read_zones_from(dir: &Path) -> Vec<ThermalZone> {
    let Ok(rd) = fs::read_dir(dir) else { return Vec::new() };
    let mut zones: Vec<(String, ThermalZone)> = rd
        .flatten()
        .filter_map(|e| {
            let dname = e.file_name().to_string_lossy().into_owned();
            if !dname.starts_with("thermal_zone") {
                return None;
            }
            let p = e.path();
            let celsius = parse_millidegrees(&fs::read_to_string(p.join("temp")).ok()?)?;
            let name = fs::read_to_string(p.join("type")).map(|s| s.trim().to_string()).unwrap_or_else(|_| dname.clone());
            Some((dname, ThermalZone { name, celsius }))
        })
        .collect();
    zones.sort_by_key(|a| natural_key(&a.0));
    zones.into_iter().map(|(_, z)| z).collect()
}

/// Chips that usually give the most useful readings (CPU package/die,
/// GPU, NVMe). Other chips are shown too, just lower in the list.
const PRIORITY_CHIPS: &[&str] = &["k10temp", "coretemp", "zenpower", "amdgpu", "nvme"];

fn chip_priority(chip: &str) -> usize {
    PRIORITY_CHIPS.iter().position(|&c| c == chip).unwrap_or(PRIORITY_CHIPS.len())
}

/// Reads sensors from /sys/class/hwmon/hwmon*/tempN_input (+ tempN_label),
/// e.g. `k10temp Tctl` — the real CPU temperature, as opposed to the
/// generic `acpitz` from /sys/class/thermal. Sorts: known "useful" chips
/// (CPU/GPU/NVMe) first, everything else after, within a chip by input number.
pub fn read_hwmon_from(dir: &Path) -> Vec<ThermalZone> {
    let Ok(rd) = fs::read_dir(dir) else { return Vec::new() };
    let mut entries: Vec<(usize, usize, ThermalZone)> = Vec::new();
    for hwmon_dir in rd.flatten().map(|e| e.path()) {
        let chip = fs::read_to_string(hwmon_dir.join("name")).map(|s| s.trim().to_string()).unwrap_or_else(|_| "hwmon".to_string());
        let Ok(files) = fs::read_dir(&hwmon_dir) else { continue };
        for f in files.flatten() {
            let fname = f.file_name().to_string_lossy().into_owned();
            let Some(n) = fname.strip_prefix("temp").and_then(|r| r.strip_suffix("_input")) else { continue };
            let Ok(idx) = n.parse::<usize>() else { continue };
            let Some(celsius) = fs::read_to_string(f.path()).ok().and_then(|t| parse_millidegrees(&t)) else { continue };
            let label = fs::read_to_string(hwmon_dir.join(format!("temp{idx}_label")))
                .ok()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| format!("temp{idx}"));
            entries.push((chip_priority(&chip), idx, ThermalZone { name: format!("{chip} {label}"), celsius }));
        }
    }
    entries.sort_by_key(|a| (a.0, a.1));
    entries.into_iter().map(|(_, _, z)| z).collect()
}

fn natural_key(s: &str) -> (usize, String) {
    let n = s.trim_start_matches(|c: char| !c.is_ascii_digit()).parse().unwrap_or(usize::MAX);
    (n, s.to_string())
}

/// All available zones: more accurate hwmon sensors first (CPU/GPU/NVMe),
/// then the generic /sys/class/thermal ones (in case hwmon is unavailable).
pub fn read_zones() -> Vec<ThermalZone> {
    let mut zones = read_hwmon_from(Path::new("/sys/class/hwmon"));
    zones.extend(read_zones_from(Path::new("/sys/class/thermal")));
    zones
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn millidegrees() {
        assert_eq!(parse_millidegrees("45500\n"), Some(45.5));
        assert_eq!(parse_millidegrees("x"), None);
    }

    #[test]
    fn missing_dir_is_empty() {
        assert!(read_zones_from(Path::new("/nonexistent/zzz")).is_empty());
        assert!(read_hwmon_from(Path::new("/nonexistent/zzz")).is_empty());
    }

    #[test]
    fn hwmon_parses_and_prioritizes_known_chips() {
        let base = std::env::temp_dir().join(format!("m3top-hwmon-test-{}", std::process::id()));
        let acpitz = base.join("hwmon0");
        let k10 = base.join("hwmon1");
        fs::create_dir_all(&acpitz).unwrap();
        fs::create_dir_all(&k10).unwrap();
        fs::write(acpitz.join("name"), "acpitz\n").unwrap();
        fs::write(acpitz.join("temp1_input"), "16800\n").unwrap();
        fs::write(k10.join("name"), "k10temp\n").unwrap();
        fs::write(k10.join("temp1_input"), "43250\n").unwrap();
        fs::write(k10.join("temp1_label"), "Tctl\n").unwrap();

        let zones = read_hwmon_from(&base);
        fs::remove_dir_all(&base).ok();

        assert_eq!(zones.len(), 2);
        // k10temp (a known CPU chip) should come before acpitz.
        assert_eq!(zones[0].name, "k10temp Tctl");
        assert!((zones[0].celsius - 43.25).abs() < 0.01);
        assert_eq!(zones[1].name, "acpitz temp1");
    }
}
