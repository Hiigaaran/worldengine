//! Generación de un mundo por etapas, al estilo de Dwarf Fortress:
//! terreno → biomas → regiones → países y localidades → personajes → eventos.
//!
//! Cada etapa usa su propio flujo aleatorio (ver [`crate::rng`]). Por ahora la
//! historia previa al año de inicio es solo un registro de fundaciones; la
//! simulación año a año llega en el paso 3.

use std::collections::VecDeque;

use rand::seq::SliceRandom;
use rand::Rng as _;

use crate::ids::*;
use crate::model::*;
use crate::names;
use crate::noise::ValueNoise;
use crate::rng::{Rng, WorldRng};

/// Elevación que separa el mar de la tierra firme.
pub const SEA_LEVEL: f32 = 0.4;

/// Las regiones más pequeñas que esto se funden con una vecina.
const MIN_REGION_AREA: u32 = 6;

pub fn generate(config: &WorldConfig) -> World {
    let rng = WorldRng::new(config.seed);
    let mut map = terrain(config, &mut rng.stream("terrain"));
    assign_biomes(&mut map);
    let regions = build_regions(&mut map, &mut rng.stream("regions"));

    let mut world = World {
        config: config.clone(),
        name: names::word(&mut rng.stream("world-name"), 2, 3),
        year: config.start_year,
        map,
        regions,
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

fn terrain(config: &WorldConfig, rng: &mut Rng) -> WorldMap {
    let (w, h) = (config.width, config.height);
    let elevation_noise = ValueNoise::new(rng, 64);
    let rain_noise = ValueNoise::new(rng, 64);
    let temp_noise = ValueNoise::new(rng, 64);

    let scale = 6.0 / w.max(h) as f32;
    let mut raw: Vec<f32> = Vec::with_capacity((w * h) as usize);
    for y in 0..h {
        for x in 0..w {
            let (fx, fy) = (x as f32 * scale, y as f32 * scale);
            // Hundir los bordes para que los continentes queden dentro del mapa.
            let dx = x as f32 / w as f32 * 2.0 - 1.0;
            let dy = y as f32 / h as f32 * 2.0 - 1.0;
            let edge = (dx * dx + dy * dy).sqrt().min(1.0);
            raw.push(elevation_noise.fractal(fx, fy, 5) - edge.powi(3) * 0.35);
        }
    }

    // El nivel del mar es el cuantil pedido, así la proporción de agua es estable.
    let mut sorted = raw.clone();
    sorted.sort_by(f32::total_cmp);
    let sea = sorted[((sorted.len() - 1) as f32 * config.sea_level.clamp(0.0, 1.0)) as usize];
    let (min, max) = (sorted[0], sorted[sorted.len() - 1]);

    let tiles = raw
        .iter()
        .enumerate()
        .map(|(i, &e)| {
            let elevation = if e < sea {
                (e - min) / (sea - min).max(f32::EPSILON) * SEA_LEVEL
            } else {
                SEA_LEVEL + (e - sea) / (max - sea).max(f32::EPSILON) * (1.0 - SEA_LEVEL)
            };
            let (x, y) = ((i as u32 % w) as f32, (i as u32 / w) as f32);
            let latitude = 1.0 - ((y / h as f32) - 0.5).abs() * 2.0;
            let altitude_cooling = (elevation - SEA_LEVEL).max(0.0) * 0.6;
            let temperature = (latitude * 0.85 + temp_noise.fractal(x * scale, y * scale, 3) * 0.3
                - 0.1
                - altitude_cooling)
                .clamp(0.0, 1.0);
            let rainfall = rain_noise.fractal(x * scale * 1.5, y * scale * 1.5, 4);
            Tile {
                elevation,
                temperature,
                rainfall,
                biome: Biome::Ocean,
                region: None,
                owner: None,
            }
        })
        .collect();

    WorldMap {
        width: w,
        height: h,
        tiles,
    }
}

fn assign_biomes(map: &mut WorldMap) {
    for tile in &mut map.tiles {
        let (e, t, r) = (tile.elevation, tile.temperature, tile.rainfall);
        tile.biome = if e < SEA_LEVEL {
            if e > SEA_LEVEL * 0.85 {
                Biome::Coast
            } else {
                Biome::Ocean
            }
        } else if t < 0.12 {
            Biome::Glacier
        } else if e > 0.82 {
            Biome::Mountain
        } else if t < 0.28 {
            Biome::Tundra
        } else if r < 0.35 && t > 0.45 {
            Biome::Desert
        } else if r > 0.6 && t > 0.65 {
            Biome::Jungle
        } else if r > 0.5 {
            Biome::Forest
        } else {
            Biome::Grassland
        };
    }
}

/// Agrupa casillas contiguas del mismo bioma en regiones con nombre.
fn build_regions(map: &mut WorldMap, rng: &mut Rng) -> Vec<Region> {
    let n = map.tiles.len();
    let mut component = vec![u32::MAX; n];
    let mut areas: Vec<u32> = Vec::new();
    for start in 0..n {
        if component[start] != u32::MAX {
            continue;
        }
        let id = areas.len() as u32;
        let biome = map.tiles[start].biome;
        let mut area = 0;
        let mut queue = VecDeque::from([start]);
        component[start] = id;
        while let Some(i) = queue.pop_front() {
            area += 1;
            for nb in map.neighbors(map.pos(i)) {
                let j = map.index(nb);
                if component[j] == u32::MAX && map.tiles[j].biome == biome {
                    component[j] = id;
                    queue.push_back(j);
                }
            }
        }
        areas.push(area);
    }

    // Fundir regiones diminutas con la vecina más grande, repitiendo hasta que
    // no quede ninguna (una diminuta puede rodear a otra diminuta).
    let mut parent: Vec<u32> = (0..areas.len() as u32).collect();
    loop {
        let mut changed = false;
        for i in 0..n {
            let c = find(&mut parent, component[i]);
            if areas[c as usize] >= MIN_REGION_AREA {
                continue;
            }
            let best = map
                .neighbors(map.pos(i))
                .map(|nb| find(&mut parent, component[map.index(nb)]))
                .filter(|&o| o != c)
                .max_by_key(|&o| areas[o as usize]);
            if let Some(o) = best {
                parent[c as usize] = o;
                areas[o as usize] += areas[c as usize];
                areas[c as usize] = 0;
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }

    // Asignar ids compactos y tomar el bioma más común como bioma de la región.
    let mut compact = vec![u32::MAX; areas.len()];
    let mut regions: Vec<Region> = Vec::new();
    let mut biome_counts: Vec<Vec<(Biome, u32)>> = Vec::new();
    let mut sums: Vec<(u64, u64)> = Vec::new();
    for (i, &comp) in component.iter().enumerate() {
        let root = find(&mut parent, comp) as usize;
        if compact[root] == u32::MAX {
            compact[root] = regions.len() as u32;
            regions.push(Region {
                id: RegionId(compact[root]),
                name: String::new(),
                biome: map.tiles[i].biome,
                area: 0,
                center: map.pos(i),
            });
            biome_counts.push(Vec::new());
            sums.push((0, 0));
        }
        let r = compact[root] as usize;
        let pos = map.pos(i);
        regions[r].area += 1;
        sums[r].0 += u64::from(pos.x);
        sums[r].1 += u64::from(pos.y);
        let biome = map.tiles[i].biome;
        match biome_counts[r].iter_mut().find(|(b, _)| *b == biome) {
            Some((_, c)) => *c += 1,
            None => biome_counts[r].push((biome, 1)),
        }
        map.tiles[i].region = Some(RegionId(r as u32));
    }

    for (r, region) in regions.iter_mut().enumerate() {
        region.biome = biome_counts[r].iter().max_by_key(|(_, c)| *c).unwrap().0;
        region.name = region_name(region.biome, &names::word(rng, 2, 3));
        // La casilla de la región más cercana a su centroide.
        let (cx, cy) = (
            sums[r].0 / u64::from(region.area),
            sums[r].1 / u64::from(region.area),
        );
        region.center = map
            .tiles
            .iter()
            .enumerate()
            .filter(|(_, t)| t.region == Some(region.id))
            .map(|(i, _)| map.pos(i))
            .min_by_key(|p| p.x.abs_diff(cx as u32).pow(2) + p.y.abs_diff(cy as u32).pow(2))
            .unwrap();
    }
    regions
}

fn find(parent: &mut [u32], mut x: u32) -> u32 {
    while parent[x as usize] != x {
        parent[x as usize] = parent[parent[x as usize] as usize];
        x = parent[x as usize];
    }
    x
}

fn region_name(biome: Biome, word: &str) -> String {
    let prefix = match biome {
        Biome::Ocean => "Mar de",
        Biome::Coast => "Bahía de",
        Biome::Desert => "Desierto de",
        Biome::Grassland => "Llanuras de",
        Biome::Forest => "Bosque de",
        Biome::Jungle => "Selva de",
        Biome::Tundra => "Tundra de",
        Biome::Mountain => "Montes de",
        Biome::Glacier => "Hielos de",
    };
    format!("{prefix} {word}")
}

fn habitable(biome: Biome) -> bool {
    matches!(
        biome,
        Biome::Grassland | Biome::Forest | Biome::Jungle | Biome::Desert | Biome::Tundra
    )
}

fn fertile(biome: Biome) -> bool {
    matches!(biome, Biome::Grassland | Biome::Forest)
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
    let map = &world.map;
    let mut candidates: Vec<Pos> = (0..map.tiles.len())
        .filter(|&i| fertile(map.tiles[i].biome))
        .map(|i| map.pos(i))
        .collect();
    candidates.shuffle(rng);

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
        let mut owned: Vec<Pos> = (0..world.map.tiles.len())
            .filter(|&i| {
                world.map.tiles[i].owner == Some(country) && habitable(world.map.tiles[i].biome)
            })
            .map(|i| world.map.pos(i))
            .collect();
        owned.shuffle(rng);
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

/// Expansión simultánea desde las capitales por tierra firme, con un radio
/// máximo distinto para cada país. Las casillas sin reclamar quedan libres.
fn claim_territory(world: &mut World, capitals: &[Pos], rng: &mut Rng) {
    let map = &mut world.map;
    let reach: Vec<u32> = capitals.iter().map(|_| rng.gen_range(8..18)).collect();
    let mut dist = vec![u32::MAX; map.tiles.len()];
    let mut queue = VecDeque::new();
    for (c, &pos) in capitals.iter().enumerate() {
        let i = map.index(pos);
        dist[i] = 0;
        map.tiles[i].owner = Some(CountryId(c as u32));
        queue.push_back(i);
    }
    while let Some(i) = queue.pop_front() {
        let owner = map.tiles[i].owner.unwrap();
        if dist[i] >= reach[owner.index()] {
            continue;
        }
        let neighbors: Vec<Pos> = map.neighbors(map.pos(i)).collect();
        for nb in neighbors {
            let j = map.index(nb);
            if dist[j] == u32::MAX && !map.tiles[j].biome.is_water() {
                dist[j] = dist[i] + 1;
                map.tiles[j].owner = Some(owner);
                queue.push_back(j);
            }
        }
    }
}

/// Algunas aldeas independientes en tierras sin dueño.
fn found_villages(world: &mut World, rng: &mut Rng) {
    let mut free: Vec<Pos> = (0..world.map.tiles.len())
        .filter(|&i| world.map.tiles[i].owner.is_none() && habitable(world.map.tiles[i].biome))
        .map(|i| world.map.pos(i))
        .collect();
    free.shuffle(rng);
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
