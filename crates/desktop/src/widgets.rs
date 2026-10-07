use eframe::egui::{self, RichText, Ui};
use egui_extras::{Column, TableBuilder};
use egui_plot::{Line, Plot, PlotPoints};
use m3top_core::Snapshot;

use crate::anim::Anim;
use crate::{filtered_processes, theme, CpuHistory, SortBy};

/// How many temperature sensors to show in the card — there can be more
/// than a dozen (chipset, VRM, etc.), but usually only the first few are
/// useful (see chip priority in m3top_core::thermal).
const MAX_THERMAL_ROWS: usize = 4;

fn unavailable(ui: &mut Ui, label: &str) {
    ui.label(RichText::new(format!("{label}: unavailable")).color(theme::ON_SURFACE_VARIANT));
}

fn history_line(dq: &std::collections::VecDeque<f32>) -> Line<'static> {
    let pts: PlotPoints = dq.iter().enumerate().map(|(i, &v)| [i as f64, v as f64]).collect();
    Line::new(pts)
}

/// A large metric number + a colored level-badge pill to the right —
/// an Expressive-style accent (like the colored FAB circles in the
/// reference), not just plain text.
fn metric_headline(ui: &mut Ui, value_text: &str, level_badge: Option<(&str, egui::Color32)>) {
    ui.horizontal(|ui| {
        ui.label(RichText::new(value_text).font(theme::bold_font(34.0)).color(theme::ON_SURFACE));
        if let Some((text, color)) = level_badge {
            egui::Frame::new().fill(color).corner_radius(theme::SHAPE_FULL).inner_margin(egui::Margin::symmetric(10, 4)).show(ui, |ui| {
                ui.label(RichText::new(text).font(theme::bold_font(11.0)).color(theme::ON_PRIMARY));
            });
        }
    });
}

fn level_label(percent: f32, warn_at: f32, bad_at: f32) -> &'static str {
    if percent >= bad_at {
        "high"
    } else if percent >= warn_at {
        "moderate"
    } else {
        "normal"
    }
}

/// A 2×2 grid assembled row by row (CPU+Temperatures in one row,
/// RAM+Network in the other): row height = the tallest card IN THAT ROW,
/// not the entire opposite column — meaning short cards no longer leave
/// a huge empty area below them, as happened with two independent
/// columns of different total height.
///
/// `ui.columns` divides the width exactly, with no guessing about
/// wrapping — so the layout itself never flickers. Springs (`Anim`)
/// only animate what gets passed INSIDE an already-fixed column (card
/// content width, displayed numbers) — a real "bounce" with no risk of
/// desync.
pub fn metric_row(ui: &mut Ui, snap: &Snapshot, history: &CpuHistory, anim: &mut Anim) {
    let dt = ui.ctx().input(|i| i.stable_dt);
    let ctx = ui.ctx().clone();

    ui.columns(2, |cols| {
        let w0 = cols[0].available_width();
        let w1 = cols[1].available_width();
        let (cpu_w, cpu_sq) = Anim::width(&mut anim.cpu_w, w0, dt, &ctx);
        cpu_card(&mut cols[0], snap, history, cpu_w, cpu_sq, anim, dt, &ctx);
        let (temp_w, temp_sq) = Anim::width(&mut anim.temp_w, w1, dt, &ctx);
        thermal_card(&mut cols[1], snap, temp_w, temp_sq);
    });
    ui.columns(2, |cols| {
        let w0 = cols[0].available_width();
        let w1 = cols[1].available_width();
        let (ram_w, ram_sq) = Anim::width(&mut anim.ram_w, w0, dt, &ctx);
        mem_card(&mut cols[0], snap, ram_w, ram_sq, anim, dt, &ctx);
        let (net_w, net_sq) = Anim::width(&mut anim.net_w, w1, dt, &ctx);
        net_card(&mut cols[1], snap, net_w, net_sq);
    });
}

#[allow(clippy::too_many_arguments)]
pub fn cpu_card(ui: &mut Ui, snap: &Snapshot, history: &CpuHistory, width: f32, squish: f32, anim: &mut Anim, dt: f32, ctx: &egui::Context) {
    theme::card_squish(ui, squish, |ui| {
        ui.set_width(width);
        theme::card_title(ui, "CPU");
        match &snap.cpu {
            Some(cpu) => {
                let shown_total = Anim::value(&mut anim.cpu_pct, cpu.total, dt, ctx).clamp(0.0, 100.0);
                let color = theme::level_color(shown_total, 70.0, 90.0);
                metric_headline(ui, &format!("{:.1}%", shown_total), Some((level_label(shown_total, 70.0, 90.0), color)));
                ui.add_space(4.0);

                egui::Frame::new().fill(theme::SURFACE_CONTAINER_LOWEST).corner_radius(theme::SHAPE_LARGE).inner_margin(5.0).show(ui, |ui| {
                    Plot::new("cpu_plot")
                        .height(52.0)
                        .width((width - 44.0).max(40.0))
                        .show_axes([false, false])
                        .show_grid([false, false])
                        .allow_drag(false)
                        .allow_zoom(false)
                        .allow_scroll(false)
                        .include_y(0.0)
                        .include_y(100.0)
                        .show(ui, |pui| {
                            pui.line(history_line(&history.total).color(theme::PRIMARY).fill(0.0_f32));
                        });
                });

                ui.add_space(4.0);
                let shown_cores = anim.cores(&cpu.cores, dt, ctx);
                let cols = ((width / 92.0) as usize).clamp(2, 8);
                egui::Grid::new("cpu_cores").num_columns(cols).spacing([6.0, 4.0]).show(ui, |ui| {
                    for (i, &v) in shown_cores.iter().enumerate() {
                        let c = theme::level_color(v, 70.0, 90.0);
                        core_pill(ui, i, v, c);
                        if i % cols == cols - 1 {
                            ui.end_row();
                        }
                    }
                });
            }
            None => unavailable(ui, "CPU"),
        }
    });
}

/// A colored dot indicator drawn as a primitive (not a text glyph!) —
/// symbols like "●"/"🔍" aren't in egui's default font and render as
/// empty tofu squares. For indicators like this, the whole app draws a
/// filled `Frame`/circle instead of a unicode icon.
fn color_dot(ui: &mut Ui, color: egui::Color32, diameter: f32) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(diameter, diameter), egui::Sense::hover());
    ui.painter().circle_filled(rect.center(), diameter / 2.0, color);
}

fn core_pill(ui: &mut Ui, idx: usize, percent: f32, color: egui::Color32) {
    egui::Frame::new()
        .fill(theme::SURFACE_CONTAINER_HIGH)
        .corner_radius(theme::SHAPE_FULL)
        .inner_margin(egui::Margin::symmetric(8, 3))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                color_dot(ui, color, 8.0);
                ui.label(RichText::new(format!("{idx} {percent:>4.0}%")).color(theme::ON_SURFACE).size(11.0).monospace());
            });
        });
}

#[allow(clippy::too_many_arguments)]
pub fn mem_card(ui: &mut Ui, snap: &Snapshot, width: f32, squish: f32, anim: &mut Anim, dt: f32, ctx: &egui::Context) {
    theme::card_squish(ui, squish, |ui| {
        ui.set_width(width);
        theme::card_title(ui, "RAM");
        match &snap.mem {
            Some(m) => {
                let pct = Anim::value(&mut anim.ram_pct, m.used_percent(), dt, ctx).clamp(0.0, 100.0);
                let color = theme::level_color(pct, 75.0, 90.0);
                metric_headline(ui, &format!("{:.0}%", pct), Some((level_label(pct, 75.0, 90.0), color)));
                ui.add_space(4.0);
                ui.add(egui::ProgressBar::new(pct / 100.0).fill(color).corner_radius(6).desired_width(width - 28.0).desired_height(10.0));
                ui.add_space(4.0);
                ui.label(
                    RichText::new(format!(
                        "{:.1} / {:.1} GiB",
                        m.used() as f64 / 1_073_741_824.0,
                        m.total as f64 / 1_073_741_824.0
                    ))
                    .color(theme::ON_SURFACE_VARIANT)
                    .size(12.0),
                );
                if m.swap_total > 0 {
                    let swap_pct = Anim::value(&mut anim.swap_pct, m.swap_percent(), dt, ctx).clamp(0.0, 100.0);
                    ui.add_space(4.0);
                    ui.label(
                        RichText::new(format!(
                            "Swap: {:.0}% ({:.1} / {:.1} GiB)",
                            swap_pct,
                            m.swap_used() as f64 / 1_073_741_824.0,
                            m.swap_total as f64 / 1_073_741_824.0
                        ))
                        .color(theme::ON_SURFACE_VARIANT)
                        .size(12.0),
                    );
                    ui.add(egui::ProgressBar::new(swap_pct / 100.0).fill(theme::PRIMARY).corner_radius(4).desired_width(width - 28.0).desired_height(6.0));
                } else {
                    ui.add_space(4.0);
                    ui.label(RichText::new("Swap: none").color(theme::ON_SURFACE_VARIANT).size(12.0));
                }
            }
            None => unavailable(ui, "RAM"),
        }
    });
}

pub fn thermal_card(ui: &mut Ui, snap: &Snapshot, width: f32, squish: f32) {
    theme::card_squish(ui, squish, |ui| {
        ui.set_width(width);
        theme::card_title(ui, "Temperatures");
        if snap.thermal.is_empty() {
            unavailable(ui, "Temperatures");
            return;
        }
        ui.add_space(4.0);
        for z in snap.thermal.iter().take(MAX_THERMAL_ROWS) {
            let color = theme::level_color(z.celsius, 70.0, 85.0);
            ui.horizontal(|ui| {
                ui.label(RichText::new(&z.name).color(theme::ON_SURFACE_VARIANT).size(12.0));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    egui::Frame::new().fill(color).corner_radius(theme::SHAPE_FULL).inner_margin(egui::Margin::symmetric(8, 2)).show(ui, |ui| {
                        ui.label(RichText::new(format!("{:.1}°C", z.celsius)).font(theme::bold_font(12.0)).color(theme::ON_PRIMARY));
                    });
                });
            });
        }
    });
}

pub fn net_card(ui: &mut Ui, snap: &Snapshot, width: f32, squish: f32) {
    theme::card_squish(ui, squish, |ui| {
        ui.set_width(width);
        theme::card_title(ui, "Network");
        match &snap.net {
            Some(n) => {
                ui.add_space(4.0);
                ui.label(RichText::new(format!("Down: {}", human_bytes_per_sec(n.rx_per_sec))).font(theme::bold_font(16.0)).color(theme::GOOD));
                ui.label(RichText::new(format!("Up: {}", human_bytes_per_sec(n.tx_per_sec))).font(theme::bold_font(16.0)).color(theme::WARN));
            }
            None => unavailable(ui, "Network"),
        }
    });
}

fn human_bytes_per_sec(v: f64) -> String {
    const UNITS: [&str; 4] = ["B/s", "KiB/s", "MiB/s", "GiB/s"];
    let mut v = v;
    let mut i = 0;
    while v >= 1024.0 && i < UNITS.len() - 1 {
        v /= 1024.0;
        i += 1;
    }
    format!("{v:.1} {}", UNITS[i])
}

pub fn process_card(
    ui: &mut Ui,
    snap: &Snapshot,
    search: &mut String,
    sort_by: &mut SortBy,
    sort_desc: &mut bool,
    my_only: &mut bool,
    available_height: f32,
) {
    theme::card(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.horizontal(|ui| {
            theme::card_title(ui, "Processes");
            ui.add_space(8.0);
            theme::search_bar(ui, search, "search by name, pid, command...", 260.0);
            ui.add_space(4.0);
            if theme::chip(ui, "my processes only", *my_only) {
                *my_only = !*my_only;
            }
        });
        ui.add_space(6.0);

        let rows = filtered_processes(snap, search, *sort_by, *sort_desc, *my_only);

        let row_h = 26.0;
        let header_h = row_h + 6.0;
        // The table scrolls ITSELF within the remaining space — no need to stretch the window.
        let table_height = (available_height - header_h - 46.0).max(row_h * 3.0);
        TableBuilder::new(ui)
            .striped(true)
            .column(Column::exact(72.0))
            .column(Column::initial(170.0).at_least(90.0).clip(true))
            .column(Column::exact(96.0))
            .column(Column::exact(100.0))
            .column(Column::remainder().clip(true))
            .min_scrolled_height(0.0)
            .max_scroll_height(table_height)
            .header(header_h, |mut header| {
                let mut cell = |label: &str, this: SortBy| {
                    header.col(|ui| {
                        let active = *sort_by == this;
                        let arrow = active.then_some(*sort_desc);
                        if theme::sort_chip(ui, label, active, arrow) {
                            if active {
                                *sort_desc = !*sort_desc;
                            } else {
                                *sort_by = this;
                                *sort_desc = true;
                            }
                        }
                    });
                };
                cell("PID", SortBy::Pid);
                cell("Name", SortBy::Name);
                cell("CPU%", SortBy::Cpu);
                cell("RAM", SortBy::Mem);
                header.col(|ui| {
                    ui.label(RichText::new("Command").color(theme::ON_SURFACE_VARIANT).strong());
                });
            })
            .body(|body| {
                body.rows(row_h, rows.len(), |mut row| {
                    let p = rows[row.index()];
                    let cpu_color = theme::level_color(p.cpu_percent, 50.0, 90.0);
                    row.col(|ui| {
                        ui.label(RichText::new(p.pid.to_string()).color(theme::ON_SURFACE_VARIANT).monospace());
                    });
                    row.col(|ui| {
                        ui.label(RichText::new(&p.name).color(theme::ON_SURFACE));
                    });
                    row.col(|ui| {
                        ui.label(RichText::new(format!("{:>5.1}", p.cpu_percent)).font(egui::FontId::monospace(13.0)).color(cpu_color));
                    });
                    row.col(|ui| {
                        ui.label(RichText::new(format!("{:>5} MiB", p.rss / 1_048_576)).color(theme::ON_SURFACE_VARIANT).monospace());
                    });
                    row.col(|ui| {
                        ui.label(RichText::new(&p.cmdline).color(theme::ON_SURFACE_VARIANT).size(11.0));
                    });
                });
            });
    });
}
