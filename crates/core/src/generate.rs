//! Generación de un mundo por etapas, al estilo de Dwarf Fortress: primero la
//! geografía física (ver [`crate::geography`]) y luego, sobre ella, países y
//! localidades → personajes → eventos.
//!
//! Cada etapa usa su propio flujo aleatorio (ver [`crate::rng`]). Por ahora la
//! historia previa al año de inicio es solo un registro de fundaciones; la
//! simulación año a año llega en el paso 3.

use std::cmp::Reverse;
use std::collections::BinaryHeap;

use rand::seq::SliceRandom;
use rand::Rng as _;

use crate::geography;
use crate::ids::*;
use crate::model::*;
use crate::names;
use crate::rng::{Rng, WorldRng};

pub fn generate(config: &WorldConfig) -> World {
    let rng = WorldRng::new(config.seed);
    let geography = geography::build(config, &rng);

    let mut world = World {
        config: config.clone(),
        name: names::word(&mut rng.stream("world-name"), 2, 3),
        year: config.start_year,
        map: geography.map,
        regions: geography.regions,
        rivers: geography.rivers,
        countries: Vec::new(),
        localities: Vec::new(),
        people: Vec::new(),
        events: Vec::new(),
    };
    found_countries(&mut world, &mut rng.stream("countries"));
    found_villages(&mut world, &mut rng.stream("villages"));
    create_people(&mut world, &mut rng.stream("people"));
    sort_events(&mut world);
    world
}

fn habitable(biome: Biome) -> bool {
    !biome.is_water() && !matches!(biome, Biome::Mountain | Biome::Glacier)
}

/// Qué tan buen lugar es una casilla para asentarse: tierra fértil, agua dulce
/// y acceso al mar, como en la historia real.
fn site_score(map: &WorldMap, pos: Pos) -> f32 {
    let tile = map.tile(pos).unwrap();
    let mut score = match tile.biome {
        Biome::Grassland => 1.0,
        Biome::Forest | Biome::Savanna => 0.8,
        Biome::Shrubland | Biome::Hills | Biome::Jungle => 0.5,
        Biome::Taiga => 0.4,
        Biome::Swamp => 0.3,
        Biome::Desert | Biome::Tundra => 0.2,
        _ => 0.0,
    };
    let near =
        |f: &dyn Fn(&Tile) -> bool| f(tile) || map.neighbors8(pos).any(|n| f(map.tile(n).unwrap()));
    if near(&|t| t.river.is_some()) {
        score += 1.0;
    }
    if near(&|t| t.biome.is_sea()) {
        score += 0.6;
    }
    if near(&|t| t.biome == Biome::Lake) {
        score += 0.5;
    }
    score
}

/// Casillas habitables que cumplen `filter`, de mejor a peor sitio, con algo
/// de azar para que no siempre gane la misma.
fn best_sites(world: &World, rng: &mut Rng, filter: impl Fn(&Tile) -> bool) -> Vec<Pos> {
    let map = &world.map;
    let mut sites: Vec<(f32, Pos)> = (0..map.tiles.len())
        .filter(|&i| habitable(map.tiles[i].biome) && filter(&map.tiles[i]))
        .map(|i| {
            let pos = map.pos(i);
            (site_score(map, pos) + rng.gen::<f32>() * 0.8, pos)
        })
        .collect();
    sites.sort_by(|a, b| b.0.total_cmp(&a.0));
    sites.into_iter().map(|(_, p)| p).collect()
}

fn distance2(a: Pos, b: Pos) -> u32 {
    a.x.abs_diff(b.x).pow(2) + a.y.abs_diff(b.y).pow(2)
}

fn push_event(world: &mut World, year: Year, kind: EventKind, location: Option<Pos>) {
    let id = EventId(world.events.len() as u32);
    world.events.push(Event {
        id,
        year,
        kind,
        location,
    });
}

fn add_locality(
    world: &mut World,
    rng: &mut Rng,
    pos: Pos,
    kind: LocalityKind,
    country: Option<CountryId>,
    founded: Year,
) -> LocalityId {
    let id = LocalityId(world.localities.len() as u32);
    let population = match kind {
        LocalityKind::Village => rng.gen_range(50..400),
        LocalityKind::Town => rng.gen_range(400..3_000),
        LocalityKind::City => rng.gen_range(3_000..30_000),
    };
    world.localities.push(Locality {
        id,
        name: names::word(rng, 2, 3),
        kind,
        pos,
        country,
        population,
        founded,
    });
    push_event(
        world,
        founded,
        EventKind::LocalityFounded { locality: id },
        Some(pos),
    );
    id
}

fn found_countries(world: &mut World, rng: &mut Rng) {
    let candidates = best_sites(world, rng, |_| true);
    let map = &world.map;

    // Capitales separadas entre sí para que cada país tenga espacio.
    let spacing = (map.width.min(map.height) / 5).max(4).pow(2);
    let mut capitals: Vec<Pos> = Vec::new();
    for pos in candidates {
        if capitals.len() as u32 >= world.config.countries {
            break;
        }
        if capitals.iter().all(|&c| distance2(c, pos) >= spacing) {
            capitals.push(pos);
        }
    }

    let start = world.config.start_year;
    for (i, &pos) in capitals.iter().enumerate() {
        let country = CountryId(i as u32);
        let founded = start - rng.gen_range(50..400);
        let capital = add_locality(world, rng, pos, LocalityKind::City, Some(country), founded);
        let government = *[
            Government::Monarchy,
            Government::Republic,
            Government::Theocracy,
            Government::TribalCouncil,
        ]
        .choose(rng)
        .unwrap();
        world.countries.push(Country {
            id: country,
            name: names::word(rng, 2, 3),
            government,
            capital,
            founded,
            ruler: None,
            color: [
                rng.gen_range(60..230),
                rng.gen_range(60..230),
                rng.gen_range(60..230),
            ],
        });
        push_event(
            world,
            founded,
            EventKind::CountryFounded { country },
            Some(pos),
        );
    }

    claim_territory(world, &capitals, rng);

    // Pueblos y aldeas dentro de cada país.
    for c in 0..world.countries.len() {
        let country = CountryId(c as u32);
        let owned = best_sites(world, rng, |t| t.owner == Some(country));
        let wanted = rng.gen_range(2..=5);
        let mut placed = 0;
        for pos in owned {
            if placed == wanted {
                break;
            }
            if world.localities.iter().all(|l| distance2(l.pos, pos) >= 16) {
                let kind = if rng.gen_bool(0.4) {
                    LocalityKind::Town
                } else {
                    LocalityKind::Village
                };
                let founded = world.countries[c].founded
                    + rng.gen_range(0..=(start - world.countries[c].founded));
                add_locality(world, rng, pos, kind, Some(country), founded);
                placed += 1;
            }
        }
    }
}

/// Coste de extender el dominio a una casilla: las montañas, pantanos y
/// desiertos frenan la expansión, así que las fronteras siguen el relieve.
fn travel_cost(tile: &Tile) -> u32 {
    let base = match tile.biome {
        Biome::Grassland | Biome::Savanna => 10,
        Biome::Forest | Biome::Shrubland => 14,
        Biome::Hills | Biome::Taiga | Biome::Jungle => 20,
        Biome::Desert | Biome::Tundra | Biome::Swamp => 25,
        Biome::Mountain => 45,
        Biome::Glacier => 60,
        _ => 10,
    };
    // Cruzar un río grande cuesta un poco más: los ríos suelen ser fronteras.
    base + if tile.river.is_some() { 6 } else { 0 }
}

/// Expansión simultánea desde las capitales por tierra firme (Dijkstra), con un
/// alcance distinto para cada país. Las casillas sin reclamar quedan libres.
fn claim_territory(world: &mut World, capitals: &[Pos], rng: &mut Rng) {
    let map = &mut world.map;
    let size = map.width.max(map.height);
    let reach: Vec<u32> = capitals
        .iter()
        .map(|_| rng.gen_range(size..size * 2))
        .collect();
    let mut cost = vec![u32::MAX; map.tiles.len()];
    let mut open = BinaryHeap::new();
    for (c, &pos) in capitals.iter().enumerate() {
        let i = map.index(pos);
        cost[i] = 0;
        open.push(Reverse((0, i, c)));
    }
    while let Some(Reverse((spent, i, c))) = open.pop() {
        if spent > cost[i] || map.tiles[i].owner.is_some() {
            continue;
        }
        map.tiles[i].owner = Some(CountryId(c as u32));
        let here = map.pos(i);
        let neighbors: Vec<Pos> = map.neighbors8(here).collect();
        for nb in neighbors {
            let j = map.index(nb);
            let tile = &map.tiles[j];
            if tile.biome.is_water() || tile.owner.is_some() {
                continue;
            }
            let diagonal = nb.x != here.x && nb.y != here.y;
            let step = travel_cost(tile);
            let next = spent + if diagonal { step * 14 / 10 } else { step };
            if next <= reach[c] && next < cost[j] {
                cost[j] = next;
                open.push(Reverse((next, j, c)));
            }
        }
    }
}

/// Algunas aldeas independientes en tierras sin dueño.
fn found_villages(world: &mut World, rng: &mut Rng) {
    let free = best_sites(world, rng, |t| t.owner.is_none());
    let wanted = (world.config.countries as usize).max(2);
    let mut placed = 0;
    for pos in free {
        if placed == wanted {
            break;
        }
        if world.localities.iter().all(|l| distance2(l.pos, pos) >= 25) {
            let founded = world.config.start_year - rng.gen_range(1..150);
            add_locality(world, rng, pos, LocalityKind::Village, None, founded);
            placed += 1;
        }
    }
}

fn create_people(world: &mut World, rng: &mut Rng) {
    let start = world.config.start_year;
    for c in 0..world.countries.len() {
        let country = CountryId(c as u32);
        let homes: Vec<LocalityId> = world
            .localities
            .iter()
            .filter(|l| l.country == Some(country))
            .map(|l| l.id)
            .collect();
        let capital = world.countries[c].capital;

        let ruler = add_person(world, rng, Role::Ruler, capital, Some(country));
        world.countries[c].ruler = Some(ruler);
        let crowned = (world.people[ruler.index()].born + rng.gen_range(18..40)).min(start);
        let pos = world.localities[capital.index()].pos;
        push_event(
            world,
            crowned,
            EventKind::RulerCrowned {
                person: ruler,
                country,
            },
            Some(pos),
        );

        for _ in 0..rng.gen_range(3..=6) {
            let role = *[
                Role::Noble,
                Role::General,
                Role::Priest,
                Role::Merchant,
                Role::Scholar,
            ]
            .choose(rng)
            .unwrap();
            let home = *homes.choose(rng).unwrap();
            add_person(world, rng, role, home, Some(country));
        }
    }

    // Aventureros sin patria, uno por aldea independiente.
    let free: Vec<LocalityId> = world
        .localities
        .iter()
        .filter(|l| l.country.is_none())
        .map(|l| l.id)
        .collect();
    for home in free {
        add_person(world, rng, Role::Adventurer, home, None);
    }
}

fn add_person(
    world: &mut World,
    rng: &mut Rng,
    role: Role,
    home: LocalityId,
    country: Option<CountryId>,
) -> PersonId {
    let id = PersonId(world.people.len() as u32);
    let born = world.config.start_year - rng.gen_range(18..70);
    world.people.push(Person {
        id,
        name: names::person(rng),
        role,
        born,
        died: None,
        home,
        country,
    });
    let pos = world.localities[home.index()].pos;
    push_event(world, born, EventKind::PersonBorn { person: id }, Some(pos));
    id
}

/// Ordena los eventos cronológicamente y renumera sus ids.
fn sort_events(world: &mut World) {
    world.events.sort_by_key(|e| (e.year, e.id));
    for (i, e) in world.events.iter_mut().enumerate() {
        e.id = EventId(i as u32);
    }
}
