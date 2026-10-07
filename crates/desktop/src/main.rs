fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_min_inner_size([480.0, 360.0])
            .with_inner_size([980.0, 800.0]),
        ..Default::default()
    };
    eframe::run_native("M3Top", options, Box::new(|_cc| Ok(Box::new(m3top::App::default()))))
}
