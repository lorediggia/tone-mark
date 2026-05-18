mod app;
mod data;
mod midi;
mod ui;

use app::MarkApp;

fn main() -> eframe::Result<()> {
    eframe::run_native(
        "Tone Mark II",
        eframe::NativeOptions {
            viewport: eframe::egui::ViewportBuilder::default()
                .with_inner_size([1100.0, 720.0])
                .with_min_inner_size([900.0, 600.0]),
            ..Default::default()
        },
        Box::new(|cc| Box::new(MarkApp::new(cc))),
    )
}