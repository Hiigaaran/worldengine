use eframe::egui;
use worldengine_core::{generate, rng, simulate, Chronicle, Pos, World, WorldConfig, Year};

use crate::details;
use crate::legends::Legends;
use crate::map_view::{MapMode, MapView};
use crate::nav::Nav;
use crate::timeline::Timeline;

/// Segundos entre años al reproducir la historia.
const PLAY_STEP: f64 = 0.35;

/// Viaje en el tiempo: el mundo tal como era en un año pasado.
#[derive(Default)]
struct TimeTravel {
    /// Se crea al primer viaje; guarda instantáneas para que moverse sea rápido.
    chronicle: Option<Chronicle>,
    /// El mundo en el año mostrado; `None` es el presente.
    view: Option<World>,
    last_step: f64,
}

pub struct App {
    seed_text: String,
    config: WorldConfig,
    /// El mundo en el presente, con toda su historia.
    world: World,
    time: TimeTravel,
    map: MapView,
    timeline: Timeline,
    legends: Legends,
    show_legends: bool,
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
            time: TimeTravel::default(),
            map: MapView::default(),
            timeline: Timeline::default(),
            legends: Legends::default(),
            show_legends: true,
            selected: None,
            status: String::new(),
        }
    }
}

impl App {
    /// El mundo que se está mostrando: el del año elegido o el presente.
    fn shown(&self) -> &World {
        self.time.view.as_ref().unwrap_or(&self.world)
    }

    fn regenerate(&mut self) {
        self.config.seed = rng::seed_from_text(&self.seed_text);
        self.world = generate(&self.config);
        self.time = TimeTravel::default();
        self.timeline.playing = false;
        self.legends.reset();
        self.map.invalidate();
        self.selected = None;
        self.status = format!("Mundo «{}» generado", self.world.name);
    }

    fn advance(&mut self, years: u32) {
        simulate::run(&mut self.world, years);
        self.time.view = None;
        self.status = format!(
            "Simulados {years} años más: ahora es el año {}",
            self.world.year
        );
    }

    fn travel(&mut self, year: Year) {
        let year = year.clamp(self.world.config.start_year, self.world.year);
        if year == self.world.year {
            self.time.view = None;
            return;
        }
        let chronicle = self
            .time
            .chronicle
            .get_or_insert_with(|| Chronicle::new(&self.world.config));
        self.time.view = Some(chronicle.at(year));
    }

    fn apply(&mut self, nav: Nav) {
        match nav {
            Nav::Select(pos) => self.selected = Some(pos),
            Nav::GoTo(pos) => {
                self.selected = Some(pos);
                self.map.center_on(pos);
            }
            Nav::Overview => self.selected = None,
            Nav::Travel(year, pos) => {
                self.travel(year);
                if let Some(pos) = pos {
                    self.selected = Some(pos);
                    self.map.center_on(pos);
                }
            }
            Nav::Present => self.time.view = None,
            Nav::Open(page) => {
                self.legends.open(page);
                self.show_legends = true;
            }
        }
    }

    fn save(&mut self, path: String, contents: Result<String, String>) {
        self.status = match contents.map(|text| std::fs::write(&path, text)) {
            Ok(Ok(())) => format!("Guardado en {path}"),
            Ok(Err(e)) => format!("No se pudo escribir {path}: {e}"),
            Err(e) => format!("No se pudo preparar {path}: {e}"),
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
            ui.toggle_value(&mut self.show_legends, "Leyendas");
            ui.separator();
            ui.label(egui::RichText::new(format!("Presente: año {}", self.world.year)).strong());
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
            let base = self.world.name.to_lowercase();
            if ui.button("Exportar crónica").clicked() {
                let text = self.world.chronicle_markdown();
                self.save(format!("{base}-cronica.md"), Ok(text));
            }
            if ui.button("Exportar JSON").clicked() {
                let json = self.world.to_json().map_err(|e| e.to_string());
                self.save(format!("{base}.worldengine.json"), json);
            }
            ui.label(egui::RichText::new(&self.status).weak());
        });
    }

    /// Avanza un año si se está reproduciendo la historia.
    fn play(&mut self, ctx: &egui::Context) {
        if !self.timeline.playing {
            return;
        }
        let now = ctx.input(|i| i.time);
        if now - self.time.last_step >= PLAY_STEP {
            self.time.last_step = now;
            let next = self.shown().year + 1;
            self.travel(next);
            if next >= self.world.year {
                self.timeline.playing = false;
            }
        }
        ctx.request_repaint();
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.play(ctx);
        egui::TopBottomPanel::top("toolbar").show(ctx, |ui| self.toolbar(ui));

        let mut navs = Vec::new();
        let shown = self.time.view.as_ref().unwrap_or(&self.world);
        let year = shown.year;

        egui::TopBottomPanel::bottom("timeline")
            .resizable(true)
            .default_height(220.0)
            .show(ctx, |ui| {
                navs.extend(self.timeline.show(ui, &self.world, year, self.selected));
            });

        if self.show_legends {
            egui::SidePanel::left("legends")
                .resizable(true)
                .default_width(320.0)
                .show(ctx, |ui| navs.extend(self.legends.show(ui, &self.world)));
        }

        egui::SidePanel::right("details")
            .resizable(true)
            .default_width(340.0)
            .show(ctx, |ui| {
                if shown.year != self.world.year {
                    ui.label(
                        egui::RichText::new(format!("Viendo el año {}", shown.year))
                            .color(egui::Color32::from_rgb(250, 200, 90)),
                    );
                }
                navs.extend(details::show(ui, shown, self.selected));
            });

        // Los grandes hechos del año mostrado se marcan en el mapa.
        let markers: Vec<Pos> = if year == self.world.year {
            Vec::new()
        } else {
            shown
                .events
                .iter()
                .rev()
                .take_while(|e| e.year == year)
                .filter(|e| shown.is_major(e))
                .filter_map(|e| e.location)
                .collect()
        };
        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(pos) = self.map.show(ui, shown, self.selected, &markers) {
                navs.push(Nav::Select(pos));
            }
        });

        for nav in navs {
            self.apply(nav);
        }
    }
}
