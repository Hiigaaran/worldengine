//! Las imágenes vectoriales del núcleo (retratos y dibujos de eventos) dentro
//! de la interfaz.

use eframe::egui::{self, Vec2};
use worldengine_core::rng::fnv1a;

use crate::nav::Nav;

/// Una imagen SVG de `size` puntos de lado. La URI sale del contenido y del
/// tamaño: cada dibujo se rasteriza una sola vez por tamaño (egui guarda las
/// texturas por URI, así que un icono pequeño no debe servir para una ficha).
pub fn image(svg: String, size: f32) -> egui::Image<'static> {
    let uri = format!(
        "bytes://art-{:016x}-{}.svg",
        fnv1a(svg.as_bytes()),
        size.round()
    );
    egui::Image::from_bytes(uri, svg.into_bytes()).fit_to_exact_size(Vec2::splat(size))
}

/// Botón para guardar la imagen como archivo SVG.
pub fn save_button(ui: &mut egui::Ui, name: &str, svg: String, nav: &mut Option<Nav>) {
    if ui
        .small_button("Guardar imagen")
        .on_hover_text("Guardar como SVG, editable en cualquier programa de dibujo")
        .clicked()
    {
        *nav = Some(Nav::SaveImage(format!("{}.svg", file_name(name)), svg));
    }
}

/// «Aldric de Vel» → «aldric-de-vel».
fn file_name(name: &str) -> String {
    name.to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| !w.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}
