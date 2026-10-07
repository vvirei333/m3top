//! M3Top core: system metrics collection from /proc and /sys. No UI dependencies.

pub mod cpu;
pub mod mem;
pub mod net;
pub mod process;
pub mod sampler;
pub mod thermal;

pub use cpu::{CpuStats, CpuTimes};
pub use mem::MemStats;
pub use net::{NetInterface, NetStats};
pub use process::ProcessInfo;
pub use sampler::{Sampler, Snapshot, SnapshotHandle};
pub use thermal::ThermalZone;
