//! Panel de Leyendas: una enciclopedia del mundo, como el modo Leyendas de
//! Dwarf Fortress. Lista países, localidades, personajes y guerras de toda la
//! historia, y abre la ficha de cada uno con sus vínculos y sus hechos.

use eframe::egui::{self, RichText};
use worldengine_core::{Event, Subject, World};

use crate::nav::{Nav, Page};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Category {
    Countries,
    Localities,
    People,
    Wars,
}

impl Category {
    const ALL: [Category; 4] = [
        Category::Countries,
        Category::Localities,
        Category::People,
        Category::Wars,
    ];

    fn label(self) -> &'static str {
        match self {
            Category::Countries => "Países",
            Category::Localities => "Localidades",
            Category::People => "Personajes",
            Category::Wars => "Guerras",
        }
    }
}

pub struct Legends {
    category: Category,
    query: String,
    page: Option<Page>,
    /// Fichas visitadas, para el botón «Atrás».
    back: Vec<Page>,
}

impl Default for Legends {
    fn default() -> Self {
        Self {
            category: Category::Countries,
            query: String::new(),
            page: None,
            back: Vec::new(),
        }
    }
}

impl Legends {
    pub fn open(&mut self, page: Page) {
        if let Some(current) = self.page.replace(page) {
            if current != page {
                self.back.push(current);
            }
        }
    }

    pub fn reset(&mut self) {
        self.page = None;
        self.back.clear();
    }

    pub fn show(&mut self, ui: &mut egui::Ui, world: &World) -> Option<Nav> {
        let mut nav = None;
        ui.heading("Leyendas");
        match self.page {
            Some(page) => {
                ui.horizontal(|ui| {
                    if ui.link("Lista").clicked() {
                        self.page = None;
                        self.back.clear();
                    }
                    if !self.back.is_empty() && ui.link("Atrás").clicked() {
                        self.page = self.back.pop();
                    }
                });
                ui.separator();
                egui::ScrollArea::vertical().show(ui, |ui| match page {
                    Page::Country(id) => country_page(ui, world, id, &mut nav),
                    Page::Locality(id) => locality_page(ui, world, id, &mut nav),
                    Page::Person(id) => person_page(ui, world, id, &mut nav),
                    Page::War(id) => war_page(ui, world, id, &mut nav),
                });
            }
            None => self.list(ui, world, &mut nav),
        }
        if let Some(Nav::Open(page)) = nav {
            self.open(page);
            return None;
        }
        nav
    }

    fn list(&mut self, ui: &mut egui::Ui, world: &World, nav: &mut Option<Nav>) {
        ui.horizontal_wrapped(|ui| {
            for category in Category::ALL {
                if ui
                    .selectable_label(self.category == category, category.label())
                    .clicked()
                {
                    self.category = category;
                }
            }
        });
        ui.add(egui::TextEdit::singleline(&mut self.query).hint_text("Buscar por nombre…"));
        ui.separator();

        let query = self.query.trim().to_lowercase();
        let matches = |name: &str| query.is_empty() || name.to_lowercase().contains(&query);
        let rows: Vec<(String, Page)> = match self.category {
            Category::Countries => world
                .countries
                .iter()
                .filter(|c| matches(&c.name))
                .map(|c| {
                    let end = c.fell.map_or(String::new(), |y| y.to_string());
                    (
                        format!("{} ({} – {end})", c.name, c.founded),
                        Page::Country(c.id),
                    )
                })
                .collect(),
            Category::Localities => world
                .localities
                .iter()
                .filter(|l| matches(&l.name))
                .map(|l| {
                    let state = if l.ruined.is_some() {
                        "ruinas"
                    } else {
                        l.kind.name()
                    };
                    (
                        format!("{} ({})", l.name, state.to_lowercase()),
                        Page::Locality(l.id),
                    )
                })
                .collect(),
            Category::People => world
                .people
                .iter()
                .filter(|p| matches(&p.name))
                .map(|p| {
                    let died = p.died.map_or(String::new(), |d| d.to_string());
                    (
                        format!(
                            "{}, {} ({} – {died})",
                            p.name,
                            p.role.name().to_lowercase(),
                            p.born
                        ),
                        Page::Person(p.id),
                    )
                })
                .collect(),
            Category::Wars => world
                .wars
                .iter()
                .filter(|w| matches(&w.name))
                .map(|w| (format!("{} ({})", w.name, w.started), Page::War(w.id)))
                .collect(),
        };

        ui.label(RichText::new(format!("{} resultados", rows.len())).weak());
        let row_height = ui.text_style_height(&egui::TextStyle::Body) + 4.0;
        egui::ScrollArea::vertical().auto_shrink(false).show_rows(
            ui,
            row_height,
            rows.len(),
            |ui, range| {
                for (text, page) in &rows[range] {
                    if ui.link(text).clicked() {
                        *nav = Some(Nav::Open(*page));
                    }
                }
            },
        );
    }
}

fn page_link(
    ui: &mut egui::Ui,
    text: impl Into<egui::WidgetText>,
    page: Page,
    nav: &mut Option<Nav>,
) {
    if ui.link(text).clicked() {
        *nav = Some(Nav::Open(page));
    }
}

/// Lista de eventos: el año viaja en el tiempo hasta ese momento y lugar.
fn events<'a>(
    ui: &mut egui::Ui,
    world: &World,
    events: impl Iterator<Item = &'a Event>,
    nav: &mut Option<Nav>,
) {
    for event in events {
        ui.horizontal_wrapped(|ui| {
            let year = ui
                .link(format!("Año {}", event.year))
                .on_hover_text("Ver el mapa en ese año");
            if year.clicked() {
                *nav = Some(Nav::Travel(event.year, event.location));
            }
            ui.label(world.describe(event));
        });
    }
}

fn country_page(
    ui: &mut egui::Ui,
    world: &World,
    id: worldengine_core::CountryId,
    nav: &mut Option<Nav>,
) {
    let country = world.country(id);
    ui.heading(&country.name);
    let end = country
        .fell
        .map_or("hoy".to_string(), |y| format!("año {y}"));
    ui.label(format!(
        "{} · año {} – {end}",
        country.government.name(),
        country.founded
    ));
    if country.exists() {
        let capital = world.locality(country.capital);
        ui.horizontal(|ui| {
            ui.label("Capital:");
            page_link(ui, &capital.name, Page::Locality(capital.id), nav);
            if ui.small_button("Mapa").clicked() {
                *nav = Some(Nav::GoTo(capital.pos));
            }
        });
        ui.label(format!("Territorio: {} casillas", world.territory_size(id)));
    }

    ui.collapsing("Gobernantes", |ui| {
        for (year, person) in world.rulers_of(id) {
            ui.horizontal(|ui| {
                ui.label(format!("Año {year}:"));
                page_link(ui, &world.person(person).name, Page::Person(person), nav);
            });
        }
    });
    ui.collapsing("Guerras", |ui| {
        for war in world.wars_of(id) {
            page_link(
                ui,
                format!("{} ({})", war.name, war.started),
                Page::War(war.id),
                nav,
            );
        }
    });
    ui.collapsing("Localidades", |ui| {
        for l in world.localities_of(id) {
            let state = if l.ruined.is_some() {
                "ruinas"
            } else {
                l.kind.name()
            };
            page_link(
                ui,
                format!("{} ({})", l.name, state.to_lowercase()),
                Page::Locality(l.id),
                nav,
            );
        }
    });
    ui.separator();
    ui.strong("Crónica");
    events(
        ui,
        world,
        world
            .events_about(Subject::Country(id))
            .filter(|e| world.is_major(e)),
        nav,
    );
}

fn locality_page(
    ui: &mut egui::Ui,
    world: &World,
    id: worldengine_core::LocalityId,
    nav: &mut Option<Nav>,
) {
    let locality = world.locality(id);
    ui.heading(&locality.name);
    match locality.ruined {
        Some(year) => ui.label(format!("En ruinas desde el año {year}")),
        None => ui.label(format!(
            "{} · {} habitantes",
            locality.kind.name(),
            locality.population
        )),
    };
    ui.label(format!("Fundada en el año {}", locality.founded));
    if let Some(c) = locality.country {
        ui.horizontal(|ui| {
            ui.label("Pertenece a");
            page_link(ui, &world.country(c).name, Page::Country(c), nav);
        });
    }
    if ui.small_button("Ver en el mapa").clicked() {
        *nav = Some(Nav::GoTo(locality.pos));
    }
    ui.collapsing("Personajes", |ui| {
        for p in world.residents(id) {
            let died = p.died.map_or(String::new(), |d| d.to_string());
            page_link(
                ui,
                format!("{} ({} – {died})", p.name, p.born),
                Page::Person(p.id),
                nav,
            );
        }
    });
    ui.separator();
    ui.strong("Historia");
    events(ui, world, world.events_about(Subject::Locality(id)), nav);
}

fn person_page(
    ui: &mut egui::Ui,
    world: &World,
    id: worldengine_core::PersonId,
    nav: &mut Option<Nav>,
) {
    let person = world.person(id);
    ui.heading(&person.name);
    let life = match (person.died, person.cause_of_death) {
        (Some(d), Some(cause)) => format!("año {} – {d}, murió {}", person.born, cause.name()),
        _ => format!(
            "nacido en el año {}, vive ({} años)",
            person.born,
            world.year - person.born
        ),
    };
    ui.label(format!("{} · {life}", person.role.name()));
    ui.horizontal(|ui| {
        ui.label("Hogar:");
        page_link(
            ui,
            &world.locality(person.home).name,
            Page::Locality(person.home),
            nav,
        );
    });
    if let Some(c) = person.country {
        ui.horizontal(|ui| {
            ui.label("País:");
            page_link(ui, &world.country(c).name, Page::Country(c), nav);
        });
    }
    if let Some(parent) = person.parent {
        ui.horizontal(|ui| {
            ui.label("Progenitor:");
            page_link(ui, &world.person(parent).name, Page::Person(parent), nav);
        });
    }
    let children: Vec<_> = world.children_of(id).collect();
    if !children.is_empty() {
        ui.label("Descendencia:");
        for child in children {
            page_link(ui, format!("  {}", child.name), Page::Person(child.id), nav);
        }
    }
    ui.separator();
    ui.strong("Vida");
    events(ui, world, world.events_about(Subject::Person(id)), nav);
}

fn war_page(ui: &mut egui::Ui, world: &World, id: worldengine_core::WarId, nav: &mut Option<Nav>) {
    let war = world.war(id);
    ui.heading(&war.name);
    let end = war
        .ended
        .map_or("en curso".to_string(), |y| format!("año {y}"));
    ui.label(format!("Año {} – {end}", war.started));
    ui.horizontal(|ui| {
        page_link(
            ui,
            &world.country(war.attacker).name,
            Page::Country(war.attacker),
            nav,
        );
        ui.label("contra");
        page_link(
            ui,
            &world.country(war.defender).name,
            Page::Country(war.defender),
            nav,
        );
    });
    match war.winner {
        Some(c) => {
            ui.horizontal(|ui| {
                ui.label("Vencedor:");
                page_link(ui, &world.country(c).name, Page::Country(c), nav);
            });
        }
        None if war.ended.is_some() => {
            ui.label("Sin vencedor");
        }
        None => {}
    }
    if !war.conquests.is_empty() {
        ui.label("Conquistas:");
        for &l in &war.conquests {
            page_link(
                ui,
                format!("  {}", world.locality(l).name),
                Page::Locality(l),
                nav,
            );
        }
    }
    ui.separator();
    ui.strong("Batallas y hechos");
    events(ui, world, world.events_about(Subject::War(id)), nav);
}
