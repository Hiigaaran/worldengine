//! Línea de tiempo: los eventos del mundo en orden cronológico. Por ahora es una
//! lista; en el paso 4 será un control deslizable que cambia el mapa por año.

use eframe::egui::{self, RichText};
use worldengine_core::{Pos, World};

/// Muestra la línea de tiempo y devuelve la ubicación del evento clicado.
pub fn show(ui: &mut egui::Ui, world: &World, selected: Option<Pos>) -> Option<Pos> {
    let mut go_to = None;
    let first = world.events.first().map_or(world.year, |e| e.year);
    ui.horizontal(|ui| {
        ui.strong("Línea de tiempo");
        ui.label(
            RichText::new(format!(
                "del año {first} al {} · {} eventos",
                world.year,
                world.events.len()
            ))
            .weak(),
        );
    });
    ui.separator();
    egui::ScrollArea::vertical()
        .auto_shrink(false)
        .stick_to_bottom(true)
        .show(ui, |ui| {
            for event in &world.events {
                let here = event.location.is_some() && event.location == selected;
                let text =
                    RichText::new(format!("Año {:>5}  {}", event.year, world.describe(event)))
                        .monospace();
                let text = if here { text.strong() } else { text };
                if ui.selectable_label(here, text).clicked() {
                    go_to = event.location;
                }
            }
        });
    go_to
}
