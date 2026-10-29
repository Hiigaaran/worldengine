//! Línea de tiempo: un control para viajar a cualquier año (el mapa y los
//! detalles muestran el mundo de ese momento) y la lista de eventos hasta ese
//! año, con filtros. Cada evento lleva el icono de su tipo; al elegirlo se
//! viaja a su año, se centra el mapa en su lugar y se abre su ficha.

use eframe::egui::{self, RichText};
use worldengine_core::{art, Pos, World, Year};

use crate::art::image;
use crate::nav::Nav;

pub struct Timeline {
    /// Oculta nacimientos, fundaciones de aldeas y muertes de personajes menores.
    pub major_only: bool,
    pub query: String,
    /// Avanza un año cada pocos instantes, como una película de la historia.
    pub playing: bool,
}

impl Default for Timeline {
    fn default() -> Self {
        Self {
            major_only: true,
            query: String::new(),
            playing: false,
        }
    }
}

impl Timeline {
    /// `world` es el mundo en el presente; `shown` es el año que se está viendo.
    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        world: &World,
        shown: Year,
        selected: Option<Pos>,
    ) -> Option<Nav> {
        let mut nav = None;
        let first = world.config.start_year;
        let present = world.year;

        ui.horizontal(|ui| {
            ui.strong("Año mostrado");
            let mut year = shown;
            ui.spacing_mut().slider_width = (ui.available_width() - 420.0).clamp(150.0, 700.0);
            let slider = ui.add(egui::Slider::new(&mut year, first..=present));
            if slider.changed() && year != shown {
                nav = Some(Nav::Travel(year, None));
            }
            for (label, delta) in [("-10", -10), ("-1", -1), ("+1", 1), ("+10", 10)] {
                if ui.button(label).clicked() {
                    nav = Some(Nav::Travel((shown + delta).clamp(first, present), None));
                }
            }
            let play = if self.playing { "Pausa" } else { "Reproducir" };
            if ui.button(play).clicked() {
                self.playing = !self.playing;
                if self.playing && shown >= present {
                    nav = Some(Nav::Travel(first, None));
                }
            }
            if shown != present && ui.button("Volver al presente").clicked() {
                self.playing = false;
                nav = Some(Nav::Present);
            }
        });

        let query = self.query.trim().to_lowercase();
        // Las descripciones se calculan una vez por cuadro para poder filtrar por texto.
        let rows: Vec<(usize, String)> = world
            .events
            .iter()
            .enumerate()
            .take_while(|(_, e)| e.year <= shown)
            .filter(|(_, e)| !self.major_only || world.is_major(e))
            .map(|(i, e)| (i, world.describe(e)))
            .filter(|(_, text)| query.is_empty() || text.to_lowercase().contains(&query))
            .collect();

        ui.horizontal(|ui| {
            ui.strong("Línea de tiempo");
            ui.label(
                RichText::new(format!(
                    "del año {} al {shown} · {} eventos",
                    world.events.first().map_or(first, |e| e.year),
                    rows.len(),
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
                    let this_year = event.year == shown && shown != present;
                    let here = event.location.is_some() && event.location == selected;
                    let text = RichText::new(format!("Año {:>5}  {text}", event.year)).monospace();
                    let text = if this_year || here {
                        text.strong()
                    } else {
                        text
                    };
                    ui.horizontal(|ui| {
                        ui.add(image(art::event(world, event), row_height - 2.0));
                        if ui.selectable_label(here, text).clicked() {
                            nav = Some(Nav::Event {
                                id: event.id,
                                travel: true,
                            });
                        }
                    });
                }
            });
        nav
    }
}
