//! Spring-mass-damper animation in the spirit of Material 3 Expressive.
//!
//! Important: springs here only move COSMETIC quantities: the content
//! width inside an already precisely computed column (`ui.columns`), and
//! the displayed metric numbers. They never participate in egui's wrap
//! decision. Mixing an animated value with wrapping (`horizontal_wrapped`
//! plus an animated column width) used to cause flicker on resize.
//! `ui.columns` always gets the exact, non-animated value; the spring
//! only visually lags behind it inside an already-fixed column, which is
//! where the "bounce" comes from.

use eframe::egui;

#[derive(Clone, Copy)]
pub struct Spring {
    pub value: f32,
    velocity: f32,
    stiffness: f32,
    damping: f32,
}

impl Spring {
    pub fn new(value: f32, stiffness: f32, damping: f32) -> Self {
        Self { value, velocity: 0.0, stiffness, damping }
    }

    /// Underdamped spring — one clear "bounce" on resize and a quick
    /// settle (ζ≈0.8 — used to be ≈0.6, several visible oscillations
    /// stretched over a couple of seconds; now one overshoot and done).
    pub fn bouncy(value: f32) -> Self {
        Self::new(value, 260.0, 26.0)
    }

    /// Close to critical damping — moves to the target quickly and
    /// smoothly with almost no overshoot. For metric numbers (percentages, bars).
    pub fn smooth(value: f32) -> Self {
        Self::new(value, 170.0, 24.0)
    }

    /// Integration step. Returns true while the motion hasn't settled yet
    /// (need to ask egui for more frames via `ctx.request_repaint()`).
    pub fn step(&mut self, target: f32, dt: f32) -> bool {
        let dt = dt.clamp(0.0, 1.0 / 30.0); // guard against a dt spike after a pause/freeze
        let accel = -self.stiffness * (self.value - target) - self.damping * self.velocity;
        self.velocity += accel * dt;
        self.value += self.velocity * dt;
        (self.value - target).abs() > 0.05 || self.velocity.abs() > 0.05
    }
}

/// All of the app's animation state in one place.
#[derive(Default)]
pub struct Anim {
    pub cpu_w: Option<Spring>,
    pub ram_w: Option<Spring>,
    pub temp_w: Option<Spring>,
    pub net_w: Option<Spring>,
    pub cpu_pct: Option<Spring>,
    pub ram_pct: Option<Spring>,
    pub swap_pct: Option<Spring>,
    pub cores: Vec<Spring>,
    /// Height of the processes card: without this, when toggling
    /// fullscreen the card widths bounced nicely while the process list
    /// below just instantly snapped to its new height — looked disjointed.
    pub table_h: Option<Spring>,
}

impl Anim {
    /// Card content width + collision "squish".
    ///
    /// The spring may want to be WIDER than the actually allotted column
    /// (`target`) — that's exactly what used to draw the card on top of
    /// its neighbor. So the visible width is hard-clamped from above by
    /// `target` (a card never physically spills past its column), and all
    /// the spring's "excess" energy (overshoot) is returned separately as
    /// `squish` 0..1 — the card doesn't overlap its neighbor, it visually
    /// squashes instead, as if it bumped into it / the window edge.
    pub fn width(slot: &mut Option<Spring>, target: f32, dt: f32, ctx: &egui::Context) -> (f32, f32) {
        let s = slot.get_or_insert_with(|| Spring::bouncy(target));
        if s.step(target, dt) {
            ctx.request_repaint();
        }
        let safe_target = target.max(1.0);
        let overshoot = (s.value - safe_target).max(0.0);
        let width = s.value.min(safe_target).max(1.0);
        let squish = (overshoot / safe_target).clamp(0.0, 1.0);
        (width, squish)
    }

    pub fn value(slot: &mut Option<Spring>, target: f32, dt: f32, ctx: &egui::Context) -> f32 {
        let s = slot.get_or_insert_with(|| Spring::smooth(target));
        if s.step(target, dt) {
            ctx.request_repaint();
        }
        s.value
    }

    /// Smoothed per-core CPU percentages. Rebuilds the springs if the
    /// core count changed (e.g. the first frame before the first sample).
    pub fn cores(&mut self, targets: &[f32], dt: f32, ctx: &egui::Context) -> Vec<f32> {
        if self.cores.len() != targets.len() {
            self.cores = targets.iter().map(|&v| Spring::smooth(v)).collect();
        }
        self.cores
            .iter_mut()
            .zip(targets)
            .map(|(s, &t)| {
                if s.step(t, dt) {
                    ctx.request_repaint();
                }
                s.value.clamp(0.0, 100.0)
            })
            .collect()
    }
}
