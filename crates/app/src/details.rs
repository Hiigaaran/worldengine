//! Panel de detalles: todo lo que se sabe de la casilla seleccionada, de lo
//! general (región, país) a lo particular (localidad, personajes, eventos).

use eframe::egui::{self, RichText};
use worldengine_core::{Pos, Subject, World};

/// Muestra el panel y devuelve una posición si el usuario pidió ir a ella.
pub fn show(ui: &mut egui::Ui, world: &World, selected: Option<Pos>) -> Option<Pos> {
    let mut go_to = None;
    let Some(pos) = selected else {
        ui.heading(&world.name);
        ui.label(format!("Año {}", world.year));
        ui.label(format!(
            "{} regiones · {} ríos · {} países · {} localidades · {} personajes",
            world.regions.len(),
            world.rivers.len(),
            world.countries.len(),
            world.localities.len(),
            world.people.len()
        ));
        ui.separator();
        ui.label("Haz clic en el mapa para ver los detalles de un lugar.");
        ui.label(RichText::new("Rueda: zoom · Arrastrar: mover").weak());
        ui.separator();
        ui.strong("Países");
        for country in &world.countries {
            let capital = world.locality(country.capital);
            if ui
                .link(format!("{} ({})", country.name, country.government.name()))
                .clicked()
            {
                go_to = Some(capital.pos);
            }
        }
        return go_to;
    };
    let tile = world.map.tile(pos)?;

    egui::ScrollArea::vertical().show(ui, |ui| {
        ui.heading(format!("Casilla ({}, {})", pos.x, pos.y));
        egui::Grid::new("tile").num_columns(2).show(ui, |ui| {
            ui.label("Bioma");
            ui.label(tile.biome.name());
            ui.end_row();
            ui.label("Elevación");
            ui.label(format!("{:.0}%", tile.elevation * 100.0));
            ui.end_row();
            ui.label("Temperatura");
            ui.label(format!("{:.0}%", tile.temperature * 100.0));
            ui.end_row();
            ui.label("Lluvia");
            ui.label(format!("{:.0}%", tile.rainfall * 100.0));
            ui.end_row();
        });

        if let Some(river) = tile.river.map(|r| world.river(r)) {
            ui.separator();
            ui.strong(river.name.as_str());
            ui.label(format!(
                "{} casillas de largo · caudal {:.0}",
                river.path.len(),
                river.discharge
            ));
            let source = river.path[0];
            if ui.link("Ir al nacimiento").clicked() {
                go_to = Some(source);
            }
            match river.flows_into.map(|r| world.river(r)) {
                Some(main) => {
                    if ui.link(format!("Afluente del {}", main.name)).clicked() {
                        go_to = Some(river.mouth);
                    }
                }
                None => {
                    let into = world.map.tile(river.mouth).map(|t| t.biome);
                    let label = match into {
                        Some(worldengine_core::Biome::Lake) => "Desemboca en un lago",
                        _ => "Desemboca en el mar",
                    };
                    if ui.link(label).clicked() {
                        go_to = Some(river.mouth);
                    }
                }
            }
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
            Some(country) => {
                ui.separator();
                ui.strong(format!("País: {}", country.name));
                ui.label(format!(
                    "{} · fundado en el año {}",
                    country.government.name(),
                    country.founded
                ));
                let capital = world.locality(country.capital);
                if ui.link(format!("Capital: {}", capital.name)).clicked() {
                    go_to = Some(capital.pos);
                }
                if let Some(ruler) = country.ruler.map(|p| world.person(p)) {
                    ui.label(format!("Gobierna: {}", ruler.name));
                }
                ui.label(format!(
                    "Territorio: {} casillas",
                    world.territory_size(country.id)
                ));
                ui.collapsing("Localidades", |ui| {
                    for l in world.localities_of(country.id) {
                        if ui
                            .link(format!(
                                "{} ({}, {} hab.)",
                                l.name,
                                l.kind.name(),
                                l.population
                            ))
                            .clicked()
                        {
                            go_to = Some(l.pos);
                        }
                    }
                });
            }
            None if !tile.biome.is_water() => {
                ui.separator();
                ui.label(RichText::new("Tierra sin reclamar").italics());
            }
            None => {}
        }

        if let Some(locality) = world.locality_at(pos) {
            ui.separator();
            ui.strong(format!("{}: {}", locality.kind.name(), locality.name));
            ui.label(format!(
                "{} habitantes · fundada en el año {}",
                locality.population, locality.founded
            ));
            ui.label(RichText::new("Personajes de interés").underline());
            for person in world.residents(locality.id) {
                ui.collapsing(format!("{} ({})", person.name, person.role.name()), |ui| {
                    ui.label(format!(
                        "Nacido en el año {} · {} años",
                        person.born,
                        world.year - person.born
                    ));
                    for event in world.events_about(Subject::Person(person.id)) {
                        ui.label(format!("Año {}: {}", event.year, world.describe(event)));
                    }
                });
            }
            ui.label(RichText::new("Historia").underline());
            for event in world.events_about(Subject::Locality(locality.id)) {
                ui.label(format!("Año {}: {}", event.year, world.describe(event)));
            }
        }
    });
    go_to
}
