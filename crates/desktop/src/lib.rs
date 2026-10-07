mod anim;
mod theme;
mod widgets;

use anim::Anim;

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use eframe::egui;
use m3top_core::{ProcessInfo, Snapshot, SnapshotHandle};

const HISTORY_LEN: usize = 120; // ~2 minutes at 1 sample/second

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SortBy {
    Pid,
    Name,
    Cpu,
    Mem,
}

/// History of per-core values for the chart.
#[derive(Default)]
pub struct CpuHistory {
    pub total: VecDeque<f32>,
    pub cores: Vec<VecDeque<f32>>,
}

impl CpuHistory {
    fn push(&mut self, total: f32, cores: &[f32]) {
        push_bounded(&mut self.total, total);
        if self.cores.len() != cores.len() {
            self.cores = (0..cores.len()).map(|_| VecDeque::new()).collect();
        }
        for (slot, &v) in self.cores.iter_mut().zip(cores) {
            push_bounded(slot, v);
        }
    }
}

fn push_bounded(dq: &mut VecDeque<f32>, v: f32) {
    dq.push_back(v);
    if dq.len() > HISTORY_LEN {
        dq.pop_front();
    }
}

pub struct App {
    snapshots: SnapshotHandle,
    repaint_flag: Arc<Mutex<egui::Context>>,
    history: CpuHistory,
    last_snapshot_ptr: usize,
    search: String,
    sort_by: SortBy,
    sort_desc: bool,
    my_processes_only: bool,
    anim: Anim,
    fonts_installed: bool,
}

impl Default for App {
    fn default() -> Self {
        // placeholder context swapped in on first update() call
        let ctx_holder = Arc::new(Mutex::new(egui::Context::default()));
        let ctx_for_thread = ctx_holder.clone();
        let snapshots = SnapshotHandle::spawn(Duration::from_secs(1), move || {
            if let Ok(ctx) = ctx_for_thread.lock() {
                ctx.request_repaint();
            }
        });
        Self {
            snapshots,
            repaint_flag: ctx_holder,
            history: CpuHistory::default(),
            last_snapshot_ptr: 0,
            search: String::new(),
            sort_by: SortBy::Cpu,
            sort_desc: true,
            my_processes_only: false,
            anim: Anim::default(),
            fonts_installed: false,
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if let Ok(mut held) = self.repaint_flag.lock() {
            *held = ctx.clone();
        }
        if !self.fonts_installed {
            theme::install_fonts(ctx);
            self.fonts_installed = true;
        }
        theme::apply(ctx);

        let snap = self.snapshots.latest();
        // Only accumulate history when a new snapshot arrives (by Arc pointer).
        let ptr = Arc::as_ptr(&snap) as usize;
        if ptr != self.last_snapshot_ptr {
            self.last_snapshot_ptr = ptr;
            if let Some(cpu) = &snap.cpu {
                self.history.push(cpu.total, &cpu.cores);
            }
        }

        let panel_frame = egui::Frame::new().fill(theme::SURFACE_DIM).inner_margin(egui::Margin::symmetric(14, 12));
        egui::CentralPanel::default().frame(panel_frame).show(ctx, |ui| {
            ui.horizontal(|ui| {
                egui::Frame::new().fill(theme::PRIMARY).corner_radius(theme::SHAPE_FULL).inner_margin(egui::Margin::symmetric(10, 6)).show(ui, |ui| {
                    ui.label(egui::RichText::new("M3").color(theme::ON_PRIMARY).strong().size(16.0));
                });
                ui.label(egui::RichText::new("M3Top").color(theme::ON_SURFACE).strong().size(22.0));
            });
            ui.add_space(6.0);

            // Top metric row — a regular (non-scrolling) part of the page.
            // The processes card takes whatever height is left, and
            // scrolls ITS OWN list internally — no need to stretch the
            // window to see processes below the visible area.
            // Real egui::columns divides the width exactly, with no
            // guessing about wrapping — springs inside metric_row only
            // animate the CARD content inside already-fixed columns, not
            // the layout itself, so there's a "bounce" but no flicker.
            widgets::metric_row(ui, &snap, &self.history, &mut self.anim);
            ui.add_space(6.0);
            // Height also eases smoothly toward its new target — otherwise,
            // toggling fullscreen/windowed mode made the process list snap
            // abruptly to its new height while the cards above bounced nicely.
            let target_height = ui.available_height();
            let dt = ctx.input(|i| i.stable_dt);
            // .min(target_height): the animation never "overshoots" the
            // actually available space — it only smoothly catches up to the
            // target from below, otherwise the list could briefly spill past
            // the window's bottom edge while un-fullscreening, while the
            // spring is still catching up.
            let remaining_height = Anim::value(&mut self.anim.table_h, target_height, dt, ctx).min(target_height);
            widgets::process_card(
                ui,
                &snap,
                &mut self.search,
                &mut self.sort_by,
                &mut self.sort_desc,
                &mut self.my_processes_only,
                remaining_height,
            );
        });
    }
}

/// Filtered and sorted process list for the table.
pub fn filtered_processes<'a>(
    snap: &'a Snapshot,
    search: &str,
    sort_by: SortBy,
    desc: bool,
    my_only: bool,
) -> Vec<&'a ProcessInfo> {
    let my_uid = current_uid();
    let needle = search.to_lowercase();
    let mut v: Vec<&ProcessInfo> = snap
        .processes
        .iter()
        .filter(|p| !my_only || p.uid == my_uid)
        .filter(|p| {
            needle.is_empty()
                || p.name.to_lowercase().contains(&needle)
                || p.cmdline.to_lowercase().contains(&needle)
                || p.pid.to_string().contains(&needle)
        })
        .collect();
    v.sort_by(|a, b| {
        let ord = match sort_by {
            SortBy::Pid => a.pid.cmp(&b.pid),
            SortBy::Name => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
            SortBy::Cpu => a.cpu_percent.total_cmp(&b.cpu_percent),
            SortBy::Mem => a.rss.cmp(&b.rss),
        };
        if desc { ord.reverse() } else { ord }
    });
    v
}

#[cfg(target_os = "linux")]
fn current_uid() -> u32 {
    // SAFETY: getuid() is a plain syscall with no arguments and no memory side effects.
    unsafe { libc_getuid() }
}

#[cfg(target_os = "linux")]
extern "C" {
    #[link_name = "getuid"]
    fn libc_getuid() -> u32;
}

#[cfg(not(target_os = "linux"))]
fn current_uid() -> u32 {
    0
}
