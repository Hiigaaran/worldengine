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

use crate::age::Age;
use crate::geography;
use crate::ids::*;
use crate::model::*;
use crate::names;
use crate::rng::{Rng, WorldRng};
use crate::simulate;

/// Genera un mundo completo: geografía, fundación y `history_years` años de historia.
pub fn generate(config: &WorldConfig) -> World {
    let mut world = found(config);
    simulate::run(&mut world, config.history_years);
    world
}

/// Genera el mundo tal como era en el año de inicio, antes de simular la historia.
pub fn found(config: &WorldConfig) -> World {
    let rng = WorldRng::new(config.seed);
    let geography = geography::build(config, &rng);

    let mut world = World {
        config: config.clone(),
        name: names::world(&mut rng.stream("world-name"), config.age),
        year: config.start_year,
        map: geography.map,
        regions: geography.regions,
        rivers: geography.rivers,
        countries: Vec::new(),
        localities: Vec::new(),
        people: Vec::new(),
        relations: Vec::new(),
        wars: Vec::new(),
        events: Vec::new(),
    };
    found_countries(&mut world, &mut rng.stream("countries"));
    found_villages(&mut world, &mut rng.stream("villages"));
    create_people(&mut world, &mut rng.stream("people"));
    init_relations(&mut world, &mut rng.stream("relations"));
    sort_events(&mut world);
    world
}

fn init_relations(world: &mut World, rng: &mut Rng) {
    let n = world.countries.len() as u32;
    for a in 0..n {
        for b in a + 1..n {
            world.relations.push(Relation {
                a: CountryId(a),
                b: CountryId(b),
                opinion: rng.gen_range(-0.2..0.4),
                allied: false,
            });
        }
    }
}

pub(crate) fn habitable(biome: Biome) -> bool {
    !biome.is_water() && !matches!(biome, Biome::Mountain | Biome::Glacier)
}

/// Qué tan buen lugar es una casilla para asentarse: tierra fértil, agua dulce
/// y acceso al mar, como en la historia real.
pub(crate) fn site_score(map: &WorldMap, pos: Pos) -> f32 {
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
pub(crate) fn best_sites(world: &World, rng: &mut Rng, filter: impl Fn(&Tile) -> bool) -> Vec<Pos> {
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

pub(crate) fn distance2(a: Pos, b: Pos) -> u32 {
    a.x.abs_diff(b.x).pow(2) + a.y.abs_diff(b.y).pow(2)
}

pub(crate) fn push_event(
    world: &mut World,
    year: Year,
    kind: EventKind,
    location: Option<Pos>,
) -> EventId {
    let id = EventId(world.events.len() as u32);
    world.events.push(Event {
        id,
        year,
        kind,
        location,
        cause: None,
    });
    id
}

pub(crate) fn add_locality(
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
        name: names::place(rng, world.config.age),
        kind,
        pos,
        country,
        population,
        founded,
        ruined: None,
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
        let government = random_government(rng);
        let age = world.config.age;
        world
            .countries
            .push(new_country(rng, age, country, capital, founded, government));
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

pub(crate) fn random_government(rng: &mut Rng) -> Government {
    *[
        Government::Monarchy,
        Government::Republic,
        Government::Theocracy,
        Government::TribalCouncil,
    ]
    .choose(rng)
    .unwrap()
}

pub(crate) fn new_country(
    rng: &mut Rng,
    age: Age,
    id: CountryId,
    capital: LocalityId,
    founded: Year,
    government: Government,
) -> Country {
    let mut color = [
        rng.gen_range(60..230),
        rng.gen_range(60..230),
        rng.gen_range(60..230),
    ];
    if age == Age::Cyberpunk {
        // Colores de neón: un canal encendido y otro apagado.
        let bright = rng.gen_range(0..3);
        let dark = (bright + rng.gen_range(1..3)) % 3;
        color[bright] = rng.gen_range(225..=255);
        color[dark] = rng.gen_range(20..70);
    }
    Country {
        id,
        name: names::country(rng, age, government),
        government,
        capital,
        founded,
        ruler: None,
        color,
        stability: 0.7,
        fell: None,
    }
}

/// Coste de extender el dominio a una casilla: las montañas, pantanos y
/// desiertos frenan la expansión, así que las fronteras siguen el relieve.
pub(crate) fn travel_cost(tile: &Tile) -> u32 {
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

        let born = start - rng.gen_range(25..65);
        let ruler = add_founder(world, rng, Role::Ruler, capital, Some(country), born);
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
            let born = start - rng.gen_range(18..70);
            add_founder(world, rng, role, home, Some(country), born);
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
        let born = start - rng.gen_range(18..50);
        add_founder(world, rng, Role::Adventurer, home, None, born);
    }
}

/// Crea un personaje sin registrar su nacimiento: quien llama decide cómo entra
/// en la historia (nacimiento, ascenso, rebelión...).
#[allow(clippy::too_many_arguments)]
pub(crate) fn add_person(
    world: &mut World,
    rng: &mut Rng,
    role: Role,
    home: LocalityId,
    country: Option<CountryId>,
    born: Year,
    parent: Option<PersonId>,
) -> PersonId {
    let id = PersonId(world.people.len() as u32);
    // Los hijos heredan el apellido de su progenitor.
    let name = match parent {
        Some(p) => {
            let family = world.people[p.index()]
                .name
                .split(' ')
                .next_back()
                .unwrap_or("")
                .to_string();
            format!("{} {family}", names::given(rng, world.config.age))
        }
        None => names::person(rng, world.config.age),
    };
    world.people.push(Person {
        id,
        name,
        role,
        born,
        died: None,
        home,
        country,
        parent,
        cause_of_death: None,
        spouse: None,
        grudge: None,
    });
    id
}

/// Personaje de la época de fundación, con su nacimiento en la historia.
fn add_founder(
    world: &mut World,
    rng: &mut Rng,
    role: Role,
    home: LocalityId,
    country: Option<CountryId>,
    born: Year,
) -> PersonId {
    let id = add_person(world, rng, role, home, country, born, None);
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
