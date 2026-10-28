//! Aplicación visual de WorldEngine.

mod app;
mod details;
mod legends;
mod map_view;
mod nav;
mod render;
mod timeline;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_title("WorldEngine")
            .with_inner_size([1400.0, 860.0]),
        ..Default::default()
    };
    eframe::run_native(
        "WorldEngine",
        options,
        Box::new(|_cc| Ok(Box::new(app::App::default()))),
    )
}
