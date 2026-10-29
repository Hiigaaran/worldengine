use eframe::egui;
use worldengine_core::{
    generate, rng, simulate, year_climate, Age, Chronicle, EventKind, Pos, World, WorldConfig,
    Year, YearClimate,
};

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
    /// Clima del año mostrado; se recalcula al cambiar de año.
    climate: Option<YearClimate>,
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
            climate: None,
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
        self.climate = None;
        self.status = format!(
            "Mundo «{}» generado ({})",
            self.world.name,
            self.config.age.name().to_lowercase()
        );
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
            ui.label(egui::RichText::new(self.world.config.age.name()).italics());
            ui.separator();
            ui.label("Semilla:");
            let seed = ui.add(egui::TextEdit::singleline(&mut self.seed_text).desired_width(140.0));
            let enter = seed.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
            let mut regenerate = ui.button("Generar").clicked() || enter;
            ui.label("Edad:");
            egui::ComboBox::from_id_salt("age")
                .selected_text(self.config.age.name())
                .show_ui(ui, |ui| {
                    for age in Age::ALL {
                        let chosen = ui
                            .selectable_value(&mut self.config.age, age, age.name())
                            .on_hover_text(age.description());
                        // Cambiar de edad genera de nuevo el mundo con la misma semilla.
                        regenerate |= chosen.clicked();
                    }
                })
                .response
                .on_hover_text(self.config.age.description());
            if regenerate {
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
            ui.toggle_value(&mut self.map.show_weather, "Clima");
            ui.toggle_value(&mut self.map.show_battles, "Batallas");
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
        // Donde se combatió ese año: batallas y conquistas, sin repetir lugar.
        let mut battles: Vec<Pos> = Vec::new();
        for e in shown.events.iter().rev().take_while(|e| e.year >= year) {
            let fight = matches!(
                e.kind,
                EventKind::Battle { .. } | EventKind::LocalityConquered { .. }
            );
            if let (true, true, Some(pos)) = (e.year == year, fight, e.location) {
                if !battles.contains(&pos) {
                    battles.push(pos);
                }
            }
        }

        if self.climate.as_ref().is_none_or(|c| c.year != year) {
            self.climate = Some(year_climate(shown, year));
        }
        let climate = self.climate.as_ref();
        let show_weather = self.map.show_weather;

        egui::CentralPanel::default().show(ctx, |ui| {
            let weather = climate.filter(|_| show_weather);
            if let Some(pos) = self
                .map
                .show(ui, shown, self.selected, &markers, &battles, weather)
            {
                navs.push(Nav::Select(pos));
            }
            if let Some(c) = weather {
                climate_summary(ui, shown, c, battles.len());
            }
        });

        for nav in navs {
            self.apply(nav);
        }
    }
}

/// Resumen del clima del año en una esquina del mapa.
fn climate_summary(ui: &egui::Ui, world: &World, climate: &YearClimate, battles: usize) {
    let percent = |r: f32| format!("{:+.0}%", (r - 1.0) * 100.0);
    let mut lines = vec![format!("Clima del año {}", climate.year)];
    if let Some(w) = climate.wettest {
        lines.push(format!(
            "Más lluvioso: {} ({})",
            world.region(w.region).name,
            percent(w.ratio)
        ));
    }
    if let Some(d) = climate.driest {
        lines.push(format!(
            "Más seco: {} ({})",
            world.region(d.region).name,
            percent(d.ratio)
        ));
    }
    lines.push(match climate.storms {
        0 => "Sin grandes tormentas".to_string(),
        1 => "1 gran tormenta".to_string(),
        n => format!("{n} grandes tormentas"),
    });
    if let Some((one, many)) = world.config.age.anomaly() {
        lines.push(match climate.anomalies {
            0 => format!("Sin {many}"),
            1 => format!("1 {one}"),
            n => format!("{n} {many}"),
        });
    }
    if battles > 0 {
        lines.push(format!("Combates en {battles} lugares"));
    }
    let rect = ui.min_rect();
    let painter = ui.painter_at(rect);
    let font = egui::FontId::proportional(13.0);
    let text = lines.join("\n");
    let galley = painter.layout_no_wrap(text, font, egui::Color32::from_gray(230));
    let pos = rect.left_top() + egui::vec2(10.0, 10.0);
    painter.rect_filled(
        egui::Rect::from_min_size(pos, galley.size()).expand(6.0),
        4.0,
        egui::Color32::from_black_alpha(150),
    );
    painter.galley(pos, galley, egui::Color32::WHITE);
}
