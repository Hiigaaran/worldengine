//! Panel de detalles: todo lo que se sabe de la casilla seleccionada, de lo
//! general (región, país) a lo particular (localidad, personajes, eventos).

use eframe::egui::{self, RichText};
use worldengine_core::{Biome, Country, Person, Pos, Subject, Tile, World};

use crate::nav::{Nav, Page};

/// Muestra el panel y devuelve la navegación que pidió el usuario, si alguna.
pub fn show(ui: &mut egui::Ui, world: &World, selected: Option<Pos>) -> Option<Nav> {
    let mut nav = None;
    let tile = selected.and_then(|p| world.map.tile(p).map(|t| (p, t)));
    egui::ScrollArea::vertical().show(ui, |ui| match tile {
        None => overview(ui, world, &mut nav),
        Some((pos, tile)) => tile_details(ui, world, pos, tile, &mut nav),
    });
    nav
}

fn link_to(ui: &mut egui::Ui, text: impl Into<egui::WidgetText>, pos: Pos, nav: &mut Option<Nav>) {
    if ui.link(text).clicked() {
        *nav = Some(Nav::GoTo(pos));
    }
}

fn open(ui: &mut egui::Ui, page: Page, nav: &mut Option<Nav>) {
    if ui
        .small_button("Leyendas")
        .on_hover_text("Abrir su ficha en Leyendas")
        .clicked()
    {
        *nav = Some(Nav::Open(page));
    }
}

fn overview(ui: &mut egui::Ui, world: &World, nav: &mut Option<Nav>) {
    ui.heading(&world.name);
    ui.label(format!("Año {}", world.year));
    ui.label(format!(
        "{} regiones · {} ríos · {} localidades · {} personajes vivos",
        world.regions.len(),
        world.rivers.len(),
        world
            .localities
            .iter()
            .filter(|l| l.ruined.is_none())
            .count(),
        world.people.iter().filter(|p| p.died.is_none()).count(),
    ));
    ui.separator();
    ui.label("Haz clic en el mapa para ver los detalles de un lugar.");
    ui.label(RichText::new("Rueda: zoom · Arrastrar: mover").weak());
    ui.separator();

    let mut living: Vec<&Country> = world.countries.iter().filter(|c| c.exists()).collect();
    living.sort_by_key(|c| std::cmp::Reverse(world.territory_size(c.id)));
    ui.strong(format!("Países en pie ({})", living.len()));
    for country in living {
        let text = format!(
            "{} ({}) · {} casillas",
            country.name,
            country.government.name(world.config.age),
            world.territory_size(country.id)
        );
        link_to(ui, text, world.locality(country.capital).pos, nav);
    }

    let fallen: Vec<&Country> = world.countries.iter().filter(|c| !c.exists()).collect();
    ui.collapsing(format!("Países desaparecidos ({})", fallen.len()), |ui| {
        for c in fallen {
            ui.horizontal(|ui| {
                ui.label(format!("{} ({} – {})", c.name, c.founded, c.fell.unwrap()));
                open(ui, Page::Country(c.id), nav);
            });
        }
    });
    ui.collapsing(format!("Guerras ({})", world.wars.len()), |ui| {
        for war in world.wars.iter().rev() {
            let end = war.ended.map_or("en curso".to_string(), |y| y.to_string());
            let mut text = format!(
                "{} ({} – {end}): {} contra {}",
                war.name,
                war.started,
                world.country(war.attacker).name,
                world.country(war.defender).name
            );
            if let Some(winner) = war.winner {
                text += &format!(", vence {}", world.country(winner).name);
            }
            if ui.link(text).clicked() {
                *nav = Some(Nav::Open(Page::War(war.id)));
            }
        }
    });
}

fn tile_details(ui: &mut egui::Ui, world: &World, pos: Pos, tile: &Tile, nav: &mut Option<Nav>) {
    if ui.link("Volver a la vista general").clicked() {
        *nav = Some(Nav::Overview);
    }
    ui.heading(format!("Casilla ({}, {})", pos.x, pos.y));
    egui::Grid::new("tile").num_columns(2).show(ui, |ui| {
        for (label, value) in [
            ("Bioma", tile.biome.name().to_string()),
            ("Elevación", format!("{:.0}%", tile.elevation * 100.0)),
            ("Temperatura", format!("{:.0}%", tile.temperature * 100.0)),
            ("Lluvia", format!("{:.0}%", tile.rainfall * 100.0)),
        ] {
            ui.label(label);
            ui.label(value);
            ui.end_row();
        }
    });

    if let Some(river) = tile.river.map(|r| world.river(r)) {
        ui.separator();
        ui.strong(river.name.as_str());
        ui.label(format!(
            "{} casillas de largo · caudal {:.0}",
            river.path.len(),
            river.discharge
        ));
        link_to(ui, "Ir al nacimiento", river.path[0], nav);
        let mouth = match river.flows_into.map(|r| world.river(r)) {
            Some(main) => format!("Afluente del {}", main.name),
            None if world
                .map
                .tile(river.mouth)
                .is_some_and(|t| t.biome == Biome::Lake) =>
            {
                "Desemboca en un lago".to_string()
            }
            None => "Desemboca en el mar".to_string(),
        };
        link_to(ui, mouth, river.mouth, nav);
    }

    if let Some(region) = tile.region.map(|r| world.region(r)) {
        ui.separator();
        ui.strong(format!("Región: {}", region.name));
        ui.label(format!(
            "{} · {} casillas",
            region.biome.name(),
            region.area
        ));
    }

    match tile.owner.map(|c| world.country(c)) {
        Some(country) => country_details(ui, world, country, nav),
        None if !tile.biome.is_water() => {
            ui.separator();
            ui.label(RichText::new("Tierra sin reclamar").italics());
        }
        None => {}
    }

    if let Some(locality) = world.locality_at(pos) {
        ui.separator();
        ui.horizontal(|ui| {
            ui.strong(format!(
                "{}: {}",
                locality.kind.name(world.config.age),
                locality.name
            ));
            open(ui, Page::Locality(locality.id), nav);
        });
        match locality.ruined {
            Some(year) => ui.label(format!(
                "En ruinas desde el año {year} · fundada en el año {}",
                locality.founded
            )),
            None => ui.label(format!(
                "{} habitantes · fundada en el año {}",
                locality.population, locality.founded
            )),
        };
        let (living, dead): (Vec<&Person>, Vec<&Person>) =
            world.residents(locality.id).partition(|p| p.died.is_none());
        ui.label(RichText::new("Personajes de interés").underline());
        for person in living {
            person_details(ui, world, person, nav);
        }
        if !dead.is_empty() {
            ui.collapsing(format!("Personajes del pasado ({})", dead.len()), |ui| {
                for person in dead {
                    person_details(ui, world, person, nav);
                }
            });
        }
        ui.collapsing("Historia de la localidad", |ui| {
            for event in world.events_about(Subject::Locality(locality.id)) {
                ui.label(format!("Año {}: {}", event.year, world.describe(event)));
            }
        });
    }
}

fn country_details(ui: &mut egui::Ui, world: &World, country: &Country, nav: &mut Option<Nav>) {
    ui.separator();
    ui.horizontal(|ui| {
        ui.strong(format!("País: {}", country.name));
        open(ui, Page::Country(country.id), nav);
    });
    ui.label(format!(
        "{} · fundado en el año {}",
        country.government.name(world.config.age),
        country.founded
    ));
    let capital = world.locality(country.capital);
    link_to(ui, format!("Capital: {}", capital.name), capital.pos, nav);
    if let Some(ruler) = country.ruler.map(|p| world.person(p)) {
        ui.label(format!(
            "Gobierna: {} ({} años)",
            ruler.name,
            world.year - ruler.born
        ));
    }
    ui.label(format!(
        "Territorio: {} casillas · estabilidad {:.0}%",
        world.territory_size(country.id),
        country.stability * 100.0
    ));
    let allies: Vec<&str> = world
        .allies_of(country.id)
        .map(|c| world.country(c).name.as_str())
        .collect();
    if !allies.is_empty() {
        ui.label(format!("Aliados: {}", allies.join(", ")));
    }
    for war in world.wars_of(country.id).filter(|w| w.ended.is_none()) {
        let enemy = if war.attacker == country.id {
            war.defender
        } else {
            war.attacker
        };
        let text = format!("En guerra con {} ({})", world.country(enemy).name, war.name);
        ui.label(RichText::new(text).color(egui::Color32::from_rgb(230, 120, 100)));
    }

    ui.collapsing("Localidades", |ui| {
        for l in world
            .localities_of(country.id)
            .filter(|l| l.ruined.is_none())
        {
            let text = format!(
                "{} ({}, {} hab.)",
                l.name,
                l.kind.name(world.config.age),
                l.population
            );
            link_to(ui, text, l.pos, nav);
        }
    });
    ui.collapsing("Gobernantes", |ui| {
        for (year, person) in world.rulers_of(country.id) {
            let p = world.person(person);
            let until = p.died.map_or(String::new(), |d| format!(", murió en {d}"));
            ui.label(format!("Año {year}: {}{until}", p.name));
        }
    });
    ui.collapsing("Guerras", |ui| {
        for war in world.wars_of(country.id) {
            let end = war.ended.map_or("en curso".to_string(), |y| y.to_string());
            let result = match war.winner {
                Some(w) if w == country.id => "· victoria",
                Some(_) => "· derrota",
                None if war.ended.is_some() => "· tablas",
                None => "",
            };
            ui.label(format!("{} ({} – {end}) {result}", war.name, war.started));
        }
    });
    ui.collapsing("Crónica del país", |ui| {
        for event in world
            .events_about(Subject::Country(country.id))
            .filter(|e| world.is_major(e))
        {
            ui.label(format!("Año {}: {}", event.year, world.describe(event)));
        }
    });
}

fn person_details(ui: &mut egui::Ui, world: &World, person: &Person, nav: &mut Option<Nav>) {
    let title = match person.died {
        Some(died) => format!(
            "{} ({}, {} – {died})",
            person.name,
            person.role.name(world.config.age),
            person.born
        ),
        None => format!(
            "{} ({}, {} años)",
            person.name,
            person.role.name(world.config.age),
            world.year - person.born
        ),
    };
    ui.collapsing(title, |ui| {
        open(ui, Page::Person(person.id), nav);
        if let Some(parent) = person.parent.map(|p| world.person(p)) {
            ui.label(format!("Hijo de {}", parent.name));
        }
        let children: Vec<&str> = world
            .children_of(person.id)
            .map(|c| c.name.as_str())
            .collect();
        if !children.is_empty() {
            ui.label(format!("Descendencia: {}", children.join(", ")));
        }
        if let Some(country) = person.country {
            ui.label(format!("Súbdito de {}", world.country(country).name));
        }
        for event in world.events_about(Subject::Person(person.id)) {
            ui.label(format!("Año {}: {}", event.year, world.describe(event)));
        }
    });
}
