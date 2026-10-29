//! Panel de Leyendas: una enciclopedia del mundo, como el modo Leyendas de
//! Dwarf Fortress. Lista países, localidades, personajes y guerras de toda la
//! historia, y abre la ficha de cada uno con sus vínculos y sus hechos.
//! Las historias agrupan los hechos que se provocaron unos a otros en relatos
//! con título y contexto.

use eframe::egui::{self, RichText};
use worldengine_core::{Event, Story, StoryId, StoryKind, Subject, World};

use crate::nav::{Nav, Page};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Category {
    Countries,
    Localities,
    People,
    Wars,
    Stories,
}

impl Category {
    const ALL: [Category; 5] = [
        Category::Stories,
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
            Category::Stories => "Historias",
        }
    }
}

pub struct Legends {
    category: Category,
    query: String,
    page: Option<Page>,
    /// Fichas visitadas, para el botón «Atrás».
    back: Vec<Page>,
    /// Filtro por tipo de historia (`None`: todas).
    story_kind: Option<StoryKind>,
    /// Historias del mundo mostrado y la huella del mundo del que salieron.
    stories: Vec<Story>,
    woven_for: Option<(u64, usize)>,
}

impl Default for Legends {
    fn default() -> Self {
        Self {
            category: Category::Stories,
            query: String::new(),
            page: None,
            back: Vec::new(),
            story_kind: None,
            stories: Vec::new(),
            woven_for: None,
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
        // Tejer las historias es caro: solo se rehace si cambia el mundo.
        let key = (world.config.seed, world.events.len());
        if self.woven_for != Some(key) {
            self.stories = world.stories();
            self.woven_for = Some(key);
            if matches!(self.page, Some(Page::Story(id)) if id.index() >= self.stories.len()) {
                self.reset();
            }
        }
        let stories = &self.stories;
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
                    Page::Country(id) => country_page(ui, world, stories, id, &mut nav),
                    Page::Locality(id) => locality_page(ui, world, stories, id, &mut nav),
                    Page::Person(id) => person_page(ui, world, stories, id, &mut nav),
                    Page::War(id) => war_page(ui, world, stories, id, &mut nav),
                    Page::Story(id) => story_page(ui, world, stories, id, &mut nav),
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
        if self.category == Category::Stories {
            ui.horizontal_wrapped(|ui| {
                if ui
                    .selectable_label(self.story_kind.is_none(), "Todas")
                    .clicked()
                {
                    self.story_kind = None;
                }
                for kind in StoryKind::ALL {
                    if ui
                        .selectable_label(self.story_kind == Some(kind), kind.name())
                        .clicked()
                    {
                        self.story_kind = Some(kind);
                    }
                }
            });
        }
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
            Category::Stories => self
                .stories
                .iter()
                .filter(|s| self.story_kind.is_none_or(|k| s.kind == k))
                .filter(|s| matches(&s.title) || matches(&s.summary))
                .map(|s| (story_label(s), Page::Story(s.id)))
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

fn years(story: &Story) -> String {
    if story.ongoing {
        format!("{} – hoy", story.start)
    } else if story.start == story.end {
        format!("año {}", story.start)
    } else {
        format!("{} – {}", story.start, story.end)
    }
}

fn story_label(story: &Story) -> String {
    format!(
        "{} ({}, {})",
        story.title,
        story.kind.name().to_lowercase(),
        years(story)
    )
}

/// Las historias en que aparece una entidad, como enlaces.
fn stories_about(ui: &mut egui::Ui, stories: &[Story], subject: Subject, nav: &mut Option<Nav>) {
    let mine: Vec<&Story> = stories.iter().filter(|s| s.involves(subject)).collect();
    if mine.is_empty() {
        return;
    }
    ui.collapsing(format!("Historias ({})", mine.len()), |ui| {
        for story in mine {
            page_link(ui, story_label(story), Page::Story(story.id), nav);
        }
    });
}

fn story_links(
    ui: &mut egui::Ui,
    stories: &[Story],
    label: &str,
    ids: &[StoryId],
    nav: &mut Option<Nav>,
) {
    if ids.is_empty() {
        return;
    }
    ui.label(RichText::new(label).strong());
    for &id in ids {
        let story = &stories[id.index()];
        page_link(
            ui,
            format!("  {}", story_label(story)),
            Page::Story(id),
            nav,
        );
    }
}

fn story_page(
    ui: &mut egui::Ui,
    world: &World,
    stories: &[Story],
    id: StoryId,
    nav: &mut Option<Nav>,
) {
    let Some(story) = stories.get(id.index()) else {
        return;
    };
    ui.heading(&story.title);
    ui.label(format!("{} · {}", story.kind.name(), years(story)));
    ui.add_space(4.0);
    ui.label(&story.summary);
    ui.add_space(4.0);

    if let Some(whole) = story.part_of {
        story_links(ui, stories, "Forma parte de", &[whole], nav);
    }
    story_links(ui, stories, "Capítulos", &story.chapters, nav);
    story_links(ui, stories, "Causas", &story.causes, nav);
    story_links(ui, stories, "Consecuencias", &story.consequences, nav);

    ui.collapsing("Protagonistas", |ui| {
        for subject in &story.subjects {
            match *subject {
                Subject::Country(c) => page_link(
                    ui,
                    format!("País: {}", world.country(c).name),
                    Page::Country(c),
                    nav,
                ),
                Subject::Person(p) => {
                    let person = world.person(p);
                    page_link(
                        ui,
                        format!("{}, {}", person.name, person.role.name().to_lowercase()),
                        Page::Person(p),
                        nav,
                    );
                }
                Subject::War(w) => page_link(ui, &world.war(w).name, Page::War(w), nav),
                Subject::Locality(_) => {}
            }
        }
    });
    let places: Vec<_> = story
        .subjects
        .iter()
        .filter_map(|s| match *s {
            Subject::Locality(l) => Some(l),
            _ => None,
        })
        .collect();
    if !places.is_empty() {
        ui.collapsing(format!("Lugares ({})", places.len()), |ui| {
            for l in places {
                page_link(ui, &world.locality(l).name, Page::Locality(l), nav);
            }
        });
    }
    ui.separator();
    ui.strong("Hechos");
    events(
        ui,
        world,
        story.events.iter().map(|&e| &world.events[e.index()]),
        nav,
    );
}

fn country_page(
    ui: &mut egui::Ui,
    world: &World,
    stories: &[Story],
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
    stories_about(ui, stories, Subject::Country(id), nav);
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
    stories: &[Story],
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
    stories_about(ui, stories, Subject::Locality(id), nav);
    ui.separator();
    ui.strong("Historia");
    events(ui, world, world.events_about(Subject::Locality(id)), nav);
}

fn person_page(
    ui: &mut egui::Ui,
    world: &World,
    stories: &[Story],
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
    if let Some(spouse) = person.spouse {
        ui.horizontal(|ui| {
            ui.label("Cónyuge:");
            page_link(ui, &world.person(spouse).name, Page::Person(spouse), nav);
        });
    }
    if let Some(grudge) = person.grudge {
        ui.horizontal(|ui| {
            ui.label("Ha jurado vengarse de");
            page_link(
                ui,
                &world.person(grudge.against).name,
                Page::Person(grudge.against),
                nav,
            );
        });
    }
    let children: Vec<_> = world.children_of(id).collect();
    if !children.is_empty() {
        ui.label("Descendencia:");
        for child in children {
            page_link(ui, format!("  {}", child.name), Page::Person(child.id), nav);
        }
    }
    stories_about(ui, stories, Subject::Person(id), nav);
    ui.separator();
    ui.strong("Vida");
    events(ui, world, world.events_about(Subject::Person(id)), nav);
}

fn war_page(
    ui: &mut egui::Ui,
    world: &World,
    stories: &[Story],
    id: worldengine_core::WarId,
    nav: &mut Option<Nav>,
) {
    let war = world.war(id);
    ui.heading(&war.name);
    // La guerra como relato, con su contexto.
    let story = stories
        .iter()
        .find(|s| s.kind == StoryKind::War && s.involves(Subject::War(id)));
    if let Some(story) = story {
        ui.label(&story.summary);
        page_link(ui, "Abrir su historia", Page::Story(story.id), nav);
    }
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
