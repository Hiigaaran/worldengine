use eframe::egui;
use worldengine_core::{generate, rng, simulate, Pos, World, WorldConfig};

use crate::details::{self, Nav};
use crate::map_view::{MapMode, MapView};
use crate::timeline::Timeline;

pub struct App {
    seed_text: String,
    config: WorldConfig,
    world: World,
    map: MapView,
    timeline: Timeline,
    /// Casilla seleccionada en el mapa; todo el panel de detalles parte de aquí.
    selected: Option<Pos>,
    status: String,
}

impl Default for App {
    fn default() -> Self {
        let config = WorldConfig::default();
        Self {
            seed_text: config.seed.to_string(),
            world: generate(&config),
            config,
            map: MapView::default(),
            timeline: Timeline::default(),
            selected: None,
            status: String::new(),
        }
    }
}

impl App {
    fn regenerate(&mut self) {
        self.config.seed = rng::seed_from_text(&self.seed_text);
        self.world = generate(&self.config);
        self.map.invalidate();
        self.selected = None;
        self.status = format!("Mundo «{}» generado", self.world.name);
    }

    fn advance(&mut self, years: u32) {
        simulate::run(&mut self.world, years);
        self.map.refresh();
        self.status = format!(
            "Simulados {years} años más: ahora es el año {}",
            self.world.year
        );
    }

    fn export_json(&mut self) {
        let path = format!("{}.worldengine.json", self.world.name.to_lowercase());
        self.status = match self.world.to_json().map(|json| std::fs::write(&path, json)) {
            Ok(Ok(())) => format!("Exportado a {path}"),
            Ok(Err(e)) => format!("No se pudo escribir {path}: {e}"),
            Err(e) => format!("No se pudo serializar: {e}"),
        };
    }

    fn toolbar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.heading(format!("Mundo de {}", self.world.name));
            ui.separator();
            ui.label("Semilla:");
            let seed = ui.add(egui::TextEdit::singleline(&mut self.seed_text).desired_width(140.0));
            let enter = seed.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
            if ui.button("Generar").clicked() || enter {
                self.regenerate();
            }
            ui.add(egui::Slider::new(&mut self.config.countries, 1..=16).text("países"));
            ui.add(egui::Slider::new(&mut self.config.sea_level, 0.2..=0.8).text("agua"));
            ui.add(
                egui::Slider::new(&mut self.config.history_years, 0..=1000)
                    .text("años de historia"),
            );
        });
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new(format!("Año {}", self.world.year)).strong());
            if ui.button("Simular +10 años").clicked() {
                self.advance(10);
            }
            if ui.button("Simular +50 años").clicked() {
                self.advance(50);
            }
            ui.separator();
            ui.label("Mapa:");
            for mode in MapMode::ALL {
                if ui
                    .selectable_label(self.map.mode == mode, mode.label())
                    .clicked()
                {
                    self.map.set_mode(mode);
                }
            }
            ui.separator();
            if ui.button("Exportar JSON").clicked() {
                self.export_json();
            }
            ui.label(egui::RichText::new(&self.status).weak());
        });
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("toolbar").show(ctx, |ui| self.toolbar(ui));

        egui::TopBottomPanel::bottom("timeline")
            .resizable(true)
            .default_height(200.0)
            .show(ctx, |ui| {
                if let Some(pos) = self.timeline.show(ui, &self.world, self.selected) {
                    self.selected = Some(pos);
                    self.map.center_on(pos);
                }
            });

        egui::SidePanel::right("details")
            .resizable(true)
            .default_width(340.0)
            .show(ctx, |ui| {
                match details::show(ui, &self.world, self.selected) {
                    Some(Nav::GoTo(pos)) => {
                        self.selected = Some(pos);
                        self.map.center_on(pos);
                    }
                    Some(Nav::Overview) => self.selected = None,
                    None => {}
                }
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(pos) = self.map.show(ui, &self.world, self.selected) {
                self.selected = Some(pos);
            }
        });
    }
}
