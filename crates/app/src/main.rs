//! Aplicación visual de WorldEngine.

mod app;
mod art;
mod details;
mod legends;
mod map_view;
mod nav;
mod overlays;
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
        Box::new(|cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);
            Ok(Box::new(app::App::default()))
        }),
    )
}
