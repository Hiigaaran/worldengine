//! Línea de tiempo: los eventos del mundo en orden cronológico, con filtros. En
//! el paso 4 será un control deslizable que cambia el mapa por año.

use eframe::egui::{self, RichText};
use worldengine_core::{Pos, World};

pub struct Timeline {
    /// Oculta nacimientos, fundaciones de aldeas y muertes de personajes menores.
    pub major_only: bool,
    pub query: String,
}

impl Default for Timeline {
    fn default() -> Self {
        Self {
            major_only: true,
            query: String::new(),
        }
    }
}

impl Timeline {
    /// Muestra la línea de tiempo y devuelve la ubicación del evento clicado.
    pub fn show(&mut self, ui: &mut egui::Ui, world: &World, selected: Option<Pos>) -> Option<Pos> {
        let mut go_to = None;
        let first = world.events.first().map_or(world.year, |e| e.year);
        let query = self.query.trim().to_lowercase();
        // Las descripciones se calculan una vez por cuadro para poder filtrar por texto.
        let rows: Vec<(usize, String)> = world
            .events
            .iter()
            .enumerate()
            .filter(|(_, e)| !self.major_only || world.is_major(e))
            .map(|(i, e)| (i, world.describe(e)))
            .filter(|(_, text)| query.is_empty() || text.to_lowercase().contains(&query))
            .collect();

        ui.horizontal(|ui| {
            ui.strong("Línea de tiempo");
            ui.label(
                RichText::new(format!(
                    "del año {first} al {} · {} de {} eventos",
                    world.year,
                    rows.len(),
                    world.events.len()
                ))
                .weak(),
            );
            ui.separator();
            ui.checkbox(&mut self.major_only, "Solo grandes hechos");
            ui.label("Buscar:");
            ui.add(
                egui::TextEdit::singleline(&mut self.query)
                    .hint_text("nombre, guerra, peste…")
                    .desired_width(180.0),
            );
        });
        ui.separator();

        let row_height = ui.text_style_height(&egui::TextStyle::Monospace) + 4.0;
        egui::ScrollArea::vertical()
            .auto_shrink(false)
            .stick_to_bottom(true)
            .show_rows(ui, row_height, rows.len(), |ui, range| {
                for (i, text) in &rows[range] {
                    let event = &world.events[*i];
                    let here = event.location.is_some() && event.location == selected;
                    let text = RichText::new(format!("Año {:>5}  {text}", event.year)).monospace();
                    let text = if here { text.strong() } else { text };
                    if ui.selectable_label(here, text).clicked() {
                        go_to = event.location;
                    }
                }
            });
        go_to
    }
}
