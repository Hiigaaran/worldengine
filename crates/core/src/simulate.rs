//! Simulación de la historia año a año, al estilo de Dwarf Fortress: el mundo
//! recién fundado evoluciona y cada cambio importante queda como evento.
//!
//! Cada año, en este orden:
//! 1. Las localidades crecen hacia la capacidad de su entorno; las aldeas se
//!    hacen pueblos y ciudades, y las despobladas quedan en ruinas.
//! 2. Los países extienden sus fronteras y fundan nuevas aldeas.
//! 3. Los personajes envejecen y mueren; nacen herederos y nuevas figuras. Al
//!    morir un gobernante, la sucesión depende del tipo de gobierno.
//! 4. Las relaciones cambian: los vecinos acumulan tensiones, se forman y rompen
//!    alianzas, y estallan guerras.
//! 5. Las guerras se libran batalla a batalla; se conquistan localidades, caen
//!    capitales y desaparecen países.
//! 6. Los países inestables sufren rebeliones y las pestes diezman poblaciones.
//!
//! Cada año usa su propio flujo aleatorio, así que simular 100 años y luego 50
//! más da exactamente el mismo mundo que simular 150 de una vez.

use std::collections::BTreeSet;

use rand::seq::SliceRandom;
use rand::Rng as _;

use crate::generate::{
    add_locality, add_person, best_sites, distance2, new_country, push_event, random_government,
    site_score, travel_cost,
};
use crate::ids::*;
use crate::model::*;
use crate::names;
use crate::rng::{Rng, WorldRng};

/// Simula `years` años a partir del año actual del mundo.
pub fn run(world: &mut World, years: u32) {
    for _ in 0..years {
        step(world);
    }
}

/// Avanza el mundo un año.
pub fn step(world: &mut World) {
    world.year += 1;
    let mut rng = WorldRng::new(world.config.seed).stream(&format!("year-{}", world.year));
    grow_localities(world, &mut rng);
    expand_borders(world, &mut rng);
    colonize(world, &mut rng);
    age_people(world, &mut rng);
    births(world, &mut rng);
    diplomacy(world, &mut rng);
    wage_wars(world, &mut rng);
    rebellions(world, &mut rng);
    plagues(world, &mut rng);
    for c in 0..world.countries.len() {
        let country = &mut world.countries[c];
        if country.exists() {
            country.stability =
                (country.stability + (0.75 - country.stability) * 0.05).clamp(0.0, 1.0);
        }
    }
    // Los imperios grandes son difíciles de mantener unidos.
    for c in living_countries(world) {
        let size = holdings(world, c).len() as f32;
        let country = &mut world.countries[c.index()];
        country.stability = (country.stability - (size - 8.0).max(0.0) * 0.0008).clamp(0.0, 1.0);
    }
    ensure_rulers(world, &mut rng);
    for country in &mut world.countries {
        country.stability = country.stability.clamp(0.0, 1.0);
    }
}

// ---------------------------------------------------------------- utilidades

fn event(world: &mut World, kind: EventKind, location: Option<Pos>) {
    let year = world.year;
    push_event(world, year, kind, location);
}

fn living_countries(world: &World) -> Vec<CountryId> {
    world
        .countries
        .iter()
        .filter(|c| c.exists())
        .map(|c| c.id)
        .collect()
}

/// Localidades habitadas de un país.
fn holdings(world: &World, country: CountryId) -> Vec<LocalityId> {
    world
        .localities
        .iter()
        .filter(|l| l.country == Some(country) && l.ruined.is_none())
        .map(|l| l.id)
        .collect()
}

/// Fuerza militar aproximada: población total del país.
fn strength(world: &World, country: CountryId) -> f32 {
    holdings(world, country)
        .iter()
        .map(|&l| world.localities[l.index()].population as f32)
        .sum::<f32>()
        .max(1.0)
}

fn capital_pos(world: &World, country: CountryId) -> Pos {
    world.localities[world.countries[country.index()].capital.index()].pos
}

fn age(world: &World, person: PersonId) -> i32 {
    world.year - world.people[person.index()].born
}

fn relation_index(world: &World, a: CountryId, b: CountryId) -> Option<usize> {
    let (a, b) = if a < b { (a, b) } else { (b, a) };
    world.relations.iter().position(|r| r.a == a && r.b == b)
}

fn at_war(world: &World, a: CountryId, b: CountryId) -> bool {
    world.wars.iter().any(|w| {
        w.ended.is_none()
            && ((w.attacker == a && w.defender == b) || (w.attacker == b && w.defender == a))
    })
}

fn in_any_war(world: &World, c: CountryId) -> bool {
    world
        .wars
        .iter()
        .any(|w| w.ended.is_none() && (w.attacker == c || w.defender == c))
}

/// Pares de países que comparten frontera terrestre.
fn borders(world: &World) -> BTreeSet<(CountryId, CountryId)> {
    let map = &world.map;
    let mut pairs = BTreeSet::new();
    for (i, tile) in map.tiles.iter().enumerate() {
        let Some(a) = tile.owner else { continue };
        let pos = map.pos(i);
        for nb in [
            Pos {
                x: pos.x + 1,
                y: pos.y,
            },
            Pos {
                x: pos.x,
                y: pos.y + 1,
            },
        ] {
            if let Some(b) = map.tile(nb).and_then(|t| t.owner) {
                if a != b {
                    pairs.insert(if a < b { (a, b) } else { (b, a) });
                }
            }
        }
    }
    pairs
}

fn kill(world: &mut World, person: PersonId, cause: DeathCause) {
    let p = &mut world.people[person.index()];
    if p.died.is_some() {
        return;
    }
    p.died = Some(world.year);
    p.cause_of_death = Some(cause);
    let pos = world.localities[p.home.index()].pos;
    event(world, EventKind::PersonDied { person, cause }, Some(pos));
}

// ---------------------------------------------------------------- población

fn grow_localities(world: &mut World, rng: &mut Rng) {
    for l in 0..world.localities.len() {
        if world.localities[l].ruined.is_some() {
            continue;
        }
        let pos = world.localities[l].pos;
        let is_capital = world.localities[l]
            .country
            .is_some_and(|c| world.countries[c.index()].capital == LocalityId(l as u32));
        let capacity =
            (2_000.0 + 8_000.0 * site_score(&world.map, pos)) * if is_capital { 1.6 } else { 1.0 };
        let locality = &mut world.localities[l];
        let pop = locality.population as f32;
        let rate = rng.gen_range(0.008..0.025);
        let next = (pop + rate * pop * (1.0 - pop / capacity)).max(0.0);
        locality.population = next.round() as u32;

        let grown = match locality.kind {
            LocalityKind::Village if locality.population >= 1_000 => Some(LocalityKind::Town),
            LocalityKind::Town if locality.population >= 6_000 => Some(LocalityKind::City),
            _ => None,
        };
        if let Some(kind) = grown {
            locality.kind = kind;
            event(
                world,
                EventKind::LocalityGrew {
                    locality: LocalityId(l as u32),
                    kind,
                },
                Some(pos),
            );
        } else if locality.population < 25 {
            locality.ruined = Some(world.year);
            event(
                world,
                EventKind::LocalityRuined {
                    locality: LocalityId(l as u32),
                },
                Some(pos),
            );
        }
    }
}

/// Cada país reclama algunas casillas libres en su frontera, empezando por las
/// más fáciles de ocupar. Las aldeas independientes alcanzadas se le unen.
fn expand_borders(world: &mut World, rng: &mut Rng) {
    let map = &world.map;
    let mut candidates: Vec<(f32, CountryId, usize)> = Vec::new();
    for (i, tile) in map.tiles.iter().enumerate() {
        if tile.owner.is_some() || tile.biome.is_water() {
            continue;
        }
        let mut owners: Vec<CountryId> = map
            .neighbors(map.pos(i))
            .filter_map(|n| map.tile(n).unwrap().owner)
            .collect();
        owners.dedup();
        for owner in owners {
            candidates.push((
                travel_cost(tile) as f32 + rng.gen_range(0.0..15.0),
                owner,
                i,
            ));
        }
    }
    candidates.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.2.cmp(&b.2)));

    for c in living_countries(world) {
        let quota = 1 + (strength(world, c) / 5_000.0).min(6.0) as usize;
        let chosen: Vec<usize> = candidates
            .iter()
            .filter(|(cost, owner, _)| *owner == c && *cost < 40.0)
            .take(quota)
            .map(|t| t.2)
            .collect();
        for i in chosen {
            if world.map.tiles[i].owner.is_some() {
                continue;
            }
            world.map.tiles[i].owner = Some(c);
            let pos = world.map.pos(i);
            let independent = world
                .localities
                .iter()
                .position(|l| l.pos == pos && l.country.is_none());
            if let Some(l) = independent {
                world.localities[l].country = Some(c);
                for p in world.people.iter_mut() {
                    if p.home.index() == l && p.died.is_none() {
                        p.country = Some(c);
                    }
                }
                event(
                    world,
                    EventKind::LocalityAnnexed {
                        locality: LocalityId(l as u32),
                        country: c,
                    },
                    Some(pos),
                );
            }
        }
    }
}

/// Fundación de aldeas en zonas despobladas del propio territorio.
fn colonize(world: &mut World, rng: &mut Rng) {
    for c in living_countries(world) {
        let stability = world.countries[c.index()].stability;
        if !rng.gen_bool((0.08 * stability as f64).clamp(0.0, 1.0)) {
            continue;
        }
        let territory = world.territory_size(c);
        if holdings(world, c).len() * 120 > territory {
            continue;
        }
        let sites = best_sites(world, rng, |t| t.owner == Some(c));
        let site = sites
            .into_iter()
            .take(200)
            .find(|&pos| world.localities.iter().all(|l| distance2(l.pos, pos) >= 25));
        if let Some(pos) = site {
            let year = world.year;
            add_locality(world, rng, pos, LocalityKind::Village, Some(c), year);
            let l = world.localities.len() - 1;
            world.localities[l].population = rng.gen_range(40..150);
        }
    }
}

// ---------------------------------------------------------------- personajes

fn age_people(world: &mut World, rng: &mut Rng) {
    for p in 0..world.people.len() {
        let person = PersonId(p as u32);
        if world.people[p].died.is_some() {
            continue;
        }
        let years = age(world, person) as f32;
        let mortality = if years < 16.0 {
            0.004
        } else {
            0.003 + 0.0004 * (0.09 * (years - 20.0)).exp()
        };
        if rng.gen_bool(mortality.min(1.0) as f64) {
            let cause = if years > 60.0 {
                DeathCause::OldAge
            } else {
                DeathCause::Illness
            };
            kill(world, person, cause);
            continue;
        }
        // Los gobernantes de países inestables pueden ser asesinados.
        let ruled = world
            .countries
            .iter()
            .find(|c| c.exists() && c.ruler == Some(person))
            .map(|c| c.stability);
        if let Some(stability) = ruled {
            if stability < 0.4 && rng.gen_bool(0.03) {
                kill(world, person, DeathCause::Assassination);
            }
        }
    }
}

fn births(world: &mut World, rng: &mut Rng) {
    let year = world.year;
    for c in living_countries(world) {
        let capital = world.countries[c.index()].capital;
        if let Some(ruler) = world.countries[c.index()].ruler {
            let children = world
                .people
                .iter()
                .filter(|p| p.parent == Some(ruler))
                .count();
            let fertile = (18..50).contains(&age(world, ruler));
            if fertile && children < 4 && rng.gen_bool(0.12) {
                let pos = world.localities[capital.index()].pos;
                let child =
                    add_person(world, rng, Role::Noble, capital, Some(c), year, Some(ruler));
                event(world, EventKind::PersonBorn { person: child }, Some(pos));
            }
        }

        let homes = holdings(world, c);
        let living = world
            .people
            .iter()
            .filter(|p| p.country == Some(c) && p.died.is_none())
            .count();
        let target = 5 + homes.len() / 2;
        if living < target && rng.gen_bool(0.35) {
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
            // A veces la nueva figura desciende de otra ya conocida de su ciudad.
            let parent = world
                .people
                .iter()
                .filter(|p| p.home == home && p.role != Role::Ruler && year - p.born >= 40)
                .map(|p| p.id)
                .collect::<Vec<_>>()
                .choose(rng)
                .copied()
                .filter(|_| rng.gen_bool(0.5));
            let pos = world.localities[home.index()].pos;
            let born = year - rng.gen_range(16..35);
            let person = add_person(world, rng, role, home, Some(c), born, parent);
            event(world, EventKind::PersonRose { person }, Some(pos));
        }
    }

    // Aventureros que surgen en tierras sin rey.
    let free: Vec<LocalityId> = world
        .localities
        .iter()
        .filter(|l| l.country.is_none() && l.ruined.is_none())
        .map(|l| l.id)
        .collect();
    if let Some(&home) = free.choose(rng) {
        if rng.gen_bool(0.05) {
            let pos = world.localities[home.index()].pos;
            let born = year - rng.gen_range(16..30);
            let person = add_person(world, rng, Role::Adventurer, home, None, born, None);
            event(world, EventKind::PersonRose { person }, Some(pos));
        }
    }
}

/// Elige al nuevo gobernante según el tipo de gobierno.
fn succession(world: &mut World, rng: &mut Rng, c: CountryId) {
    let country = &world.countries[c.index()];
    let previous = country.ruler;
    let government = country.government;
    let capital = country.capital;
    let candidates: Vec<PersonId> = world
        .people
        .iter()
        .filter(|p| p.country == Some(c) && p.died.is_none() && world.year - p.born >= 16)
        .filter(|p| Some(p.id) != previous)
        .map(|p| p.id)
        .collect();
    let with_role = |roles: &[Role]| -> Vec<PersonId> {
        candidates
            .iter()
            .copied()
            .filter(|&p| roles.contains(&world.people[p.index()].role))
            .collect()
    };
    let eldest = |list: Vec<PersonId>| {
        list.into_iter()
            .min_by_key(|p| (world.people[p.index()].born, p.0))
    };

    let heir = match government {
        Government::Monarchy => {
            let children: Vec<PersonId> = candidates
                .iter()
                .copied()
                .filter(|&p| previous.is_some() && world.people[p.index()].parent == previous)
                .collect();
            eldest(children).or_else(|| eldest(with_role(&[Role::Noble])))
        }
        Government::Republic => with_role(&[Role::Noble, Role::Merchant, Role::Scholar])
            .choose(rng)
            .copied(),
        Government::Theocracy => eldest(with_role(&[Role::Priest])),
        Government::TribalCouncil => with_role(&[Role::General, Role::Adventurer])
            .choose(rng)
            .copied(),
    }
    .or_else(|| candidates.choose(rng).copied());

    let year = world.year;
    let heir = match heir {
        Some(h) => h,
        None => {
            let born = year - rng.gen_range(25..45);
            let person = add_person(world, rng, Role::Noble, capital, Some(c), born, None);
            let pos = world.localities[capital.index()].pos;
            event(world, EventKind::PersonRose { person }, Some(pos));
            person
        }
    };
    let dynastic = government == Government::Monarchy
        && world.people[heir.index()].parent == previous
        && previous.is_some();
    let person = &mut world.people[heir.index()];
    person.role = Role::Ruler;
    person.home = capital;
    let country = &mut world.countries[c.index()];
    country.ruler = Some(heir);
    country.stability -= if dynastic { 0.03 } else { 0.12 };
    let pos = world.localities[capital.index()].pos;
    event(
        world,
        EventKind::RulerCrowned {
            person: heir,
            country: c,
        },
        Some(pos),
    );
}

/// Todo país vivo tiene un gobernante vivo al acabar el año.
fn ensure_rulers(world: &mut World, rng: &mut Rng) {
    for c in living_countries(world) {
        let ruler = world.countries[c.index()].ruler;
        if ruler.is_none_or(|r| world.people[r.index()].died.is_some()) {
            succession(world, rng, c);
        }
    }
}

// ---------------------------------------------------------------- diplomacia

fn diplomacy(world: &mut World, rng: &mut Rng) {
    let neighbors = borders(world);
    for r in 0..world.relations.len() {
        let (a, b) = (world.relations[r].a, world.relations[r].b);
        if !world.countries[a.index()].exists() || !world.countries[b.index()].exists() {
            continue;
        }
        let adjacent = neighbors.contains(&(a, b));
        let fighting = at_war(world, a, b);
        let same_government =
            world.countries[a.index()].government == world.countries[b.index()].government;
        let common_enemy = world.wars.iter().filter(|w| w.ended.is_none()).any(|w| {
            let enemies_of = |c: CountryId| {
                (w.attacker == c)
                    .then_some(w.defender)
                    .or((w.defender == c).then_some(w.attacker))
            };
            enemies_of(a).is_some() && enemies_of(a) == enemies_of(b)
        });

        let relation = &mut world.relations[r];
        if adjacent {
            relation.opinion -= 0.015 + rng.gen_range(-0.025..0.025);
        } else {
            relation.opinion += (0.2 - relation.opinion) * 0.05 + rng.gen_range(-0.02..0.02);
        }
        if same_government {
            relation.opinion += 0.01;
        }
        if common_enemy {
            relation.opinion += 0.05;
        }
        if fighting {
            relation.opinion = relation.opinion.min(-0.6);
        }
        relation.opinion = relation.opinion.clamp(-1.0, 1.0);
        let opinion = relation.opinion;
        let allied = relation.allied;
        let pos = Some(capital_pos(world, a));

        if !allied && !fighting && opinion > 0.35 && rng.gen_bool(0.08) {
            world.relations[r].allied = true;
            event(world, EventKind::AllianceFormed { a, b }, pos);
        } else if allied && opinion < 0.1 {
            world.relations[r].allied = false;
            event(world, EventKind::AllianceBroken { a, b }, pos);
        } else if adjacent && !allied && !fighting && opinion < -0.35 {
            let (sa, sb) = (strength(world, a), strength(world, b));
            let (attacker, defender, ratio) = if sa >= sb {
                (a, b, sa / sb)
            } else {
                (b, a, sb / sa)
            };
            if ratio > 0.9 && !in_any_war(world, attacker) && rng.gen_bool(0.06) {
                declare_war(world, rng, attacker, defender);
            }
        }
    }
}

fn declare_war(world: &mut World, rng: &mut Rng, attacker: CountryId, defender: CountryId) {
    let id = WarId(world.wars.len() as u32);
    world.wars.push(War {
        id,
        name: format!("Guerra de {}", names::word(rng, 2, 3)),
        attacker,
        defender,
        started: world.year,
        ended: None,
        winner: None,
        conquests: Vec::new(),
        exhaustion: 0.0,
    });
    let pos = capital_pos(world, defender);
    event(world, EventKind::WarDeclared { war: id }, Some(pos));
}

// ---------------------------------------------------------------- guerras

fn wage_wars(world: &mut World, rng: &mut Rng) {
    for w in 0..world.wars.len() {
        if world.wars[w].ended.is_some() {
            continue;
        }
        let war = WarId(w as u32);
        let (attacker, defender) = (world.wars[w].attacker, world.wars[w].defender);
        if !world.countries[attacker.index()].exists()
            || !world.countries[defender.index()].exists()
        {
            end_war(world, war, None);
            continue;
        }
        let targets = holdings(world, defender);
        let from = capital_pos(world, attacker);
        let Some(&target) = targets
            .iter()
            .min_by_key(|&&l| (distance2(world.localities[l.index()].pos, from), l.0))
        else {
            end_war(world, war, Some(attacker));
            continue;
        };

        let sa = strength(world, attacker) * rng.gen_range(0.6..1.4);
        let sd = strength(world, defender) * rng.gen_range(0.6..1.4) * 1.15;
        let (winner, loser) = if sa > sd {
            (attacker, defender)
        } else {
            (defender, attacker)
        };
        let pos = world.localities[target.index()].pos;
        event(
            world,
            EventKind::Battle {
                war,
                locality: target,
                winner,
                loser,
            },
            Some(pos),
        );
        let locality = &mut world.localities[target.index()];
        locality.population = (locality.population as f32 * rng.gen_range(0.85..0.97)) as u32;

        // Los generales pueden caer en combate, sobre todo en el bando perdedor.
        for (side, risk) in [(winner, 0.08), (loser, 0.2)] {
            let generals: Vec<PersonId> = world
                .people
                .iter()
                .filter(|p| p.country == Some(side) && p.died.is_none() && p.role == Role::General)
                .map(|p| p.id)
                .collect();
            if let Some(&general) = generals.choose(rng) {
                if rng.gen_bool(risk) {
                    kill(world, general, DeathCause::Battle);
                }
            }
        }

        if winner == attacker && rng.gen_bool(0.4) {
            world.wars[w].conquests.push(target);
            event(
                world,
                EventKind::LocalityConquered {
                    war,
                    locality: target,
                    from: defender,
                    to: attacker,
                },
                Some(pos),
            );
            transfer_locality(world, target, attacker);
            if !world.countries[defender.index()].exists() {
                end_war(world, war, Some(attacker));
                continue;
            }
        }

        for c in [attacker, defender] {
            world.countries[c.index()].stability -= 0.03;
        }
        world.wars[w].exhaustion += rng.gen_range(0.1..0.25);
        if world.wars[w].exhaustion >= 1.0 {
            let winner = if world.wars[w].conquests.is_empty() {
                defender
            } else {
                attacker
            };
            end_war(world, war, Some(winner));
        }
    }
}

fn end_war(world: &mut World, war: WarId, winner: Option<CountryId>) {
    let w = &mut world.wars[war.index()];
    w.ended = Some(world.year);
    w.winner = winner;
    let (a, b) = (w.attacker, w.defender);
    if let Some(r) = relation_index(world, a, b) {
        world.relations[r].opinion = -0.3;
    }
    let pos = capital_pos(world, b);
    event(world, EventKind::PeaceSigned { war }, Some(pos));
}

/// Cambia de dueño una localidad junto con las casillas que la rodean (las que
/// tenían esa localidad como la más cercana de su antiguo país).
fn transfer_locality(world: &mut World, locality: LocalityId, to: CountryId) {
    let Some(from) = world.localities[locality.index()].country else {
        world.localities[locality.index()].country = Some(to);
        return;
    };
    let others: Vec<LocalityId> = holdings(world, from)
        .into_iter()
        .filter(|&l| l != locality)
        .collect();

    // Si cae la capital, el gobierno se traslada a la localidad más poblada.
    if world.countries[from.index()].capital == locality {
        if let Some(&new_capital) = others
            .iter()
            .max_by_key(|&&l| (world.localities[l.index()].population, l.0))
        {
            world.countries[from.index()].capital = new_capital;
            if let Some(ruler) = world.countries[from.index()].ruler {
                world.people[ruler.index()].home = new_capital;
            }
            let pos = world.localities[new_capital.index()].pos;
            event(
                world,
                EventKind::CapitalMoved {
                    country: from,
                    locality: new_capital,
                },
                Some(pos),
            );
        }
    }

    let sites: Vec<(LocalityId, Pos)> = holdings(world, from)
        .iter()
        .map(|&l| (l, world.localities[l.index()].pos))
        .collect();
    let map = &mut world.map;
    for i in 0..map.tiles.len() {
        if map.tiles[i].owner != Some(from) {
            continue;
        }
        let pos = map.pos(i);
        let nearest = sites
            .iter()
            .min_by_key(|(l, p)| (distance2(*p, pos), l.0))
            .map(|s| s.0);
        if nearest.is_none_or(|l| l == locality) {
            map.tiles[i].owner = Some(to);
        }
    }

    world.localities[locality.index()].country = Some(to);
    for p in world.people.iter_mut() {
        if p.home == locality && p.died.is_none() && p.role != Role::Ruler {
            p.country = Some(to);
        }
    }

    if others.is_empty() {
        let country = &mut world.countries[from.index()];
        country.fell = Some(world.year);
        if let Some(ruler) = country.ruler.take() {
            let person = &mut world.people[ruler.index()];
            person.role = Role::Noble;
            person.country = Some(to);
        }
        let pos = world.localities[locality.index()].pos;
        event(
            world,
            EventKind::CountryFell {
                country: from,
                conqueror: Some(to),
            },
            Some(pos),
        );
        // Sus localidades en ruinas pasan también al vencedor.
        for l in world.localities.iter_mut() {
            if l.country == Some(from) {
                l.country = Some(to);
            }
        }
        for p in world.people.iter_mut() {
            if p.country == Some(from) && p.died.is_none() {
                p.country = Some(to);
            }
        }
    }
}

// ---------------------------------------------------------------- crisis

fn rebellions(world: &mut World, rng: &mut Rng) {
    let far = (world.map.width.max(world.map.height) / 8).pow(2);
    for c in living_countries(world) {
        if world.countries[c.index()].stability >= 0.35 {
            continue;
        }
        let capital = capital_pos(world, c);
        let capital_id = world.countries[c.index()].capital;
        let rebels: Vec<LocalityId> = holdings(world, c)
            .into_iter()
            .filter(|&l| {
                let loc = &world.localities[l.index()];
                l != capital_id && loc.population >= 400 && distance2(loc.pos, capital) >= far
            })
            .collect();
        let Some(&rebel) = rebels.choose(rng) else {
            continue;
        };
        if !rng.gen_bool(0.035) {
            continue;
        }
        independence(world, rng, rebel, c);
    }
}

fn independence(world: &mut World, rng: &mut Rng, locality: LocalityId, from: CountryId) {
    let year = world.year;
    let id = CountryId(world.countries.len() as u32);
    let government = random_government(rng);
    let country = new_country(rng, id, locality, year, government);
    world.countries.push(country);
    for other in 0..id.0 {
        let opinion = if CountryId(other) == from {
            -0.8
        } else {
            rng.gen_range(-0.2..0.4)
        };
        world.relations.push(Relation {
            a: CountryId(other),
            b: id,
            opinion,
            allied: false,
        });
    }
    let pos = world.localities[locality.index()].pos;
    event(
        world,
        EventKind::Independence {
            locality,
            from,
            country: id,
        },
        Some(pos),
    );
    event(world, EventKind::CountryFounded { country: id }, Some(pos));
    transfer_locality(world, locality, id);

    // El líder de la revuelta: un general o noble local, o alguien nuevo.
    let leader = world
        .people
        .iter()
        .filter(|p| {
            p.home == locality && p.died.is_none() && matches!(p.role, Role::General | Role::Noble)
        })
        .filter(|p| year - p.born >= 16)
        .map(|p| p.id)
        .next();
    let leader = match leader {
        Some(p) => p,
        None => {
            let born = year - rng.gen_range(25..50);
            let person = add_person(world, rng, Role::General, locality, Some(id), born, None);
            event(world, EventKind::PersonRose { person }, Some(pos));
            person
        }
    };
    world.people[leader.index()].role = Role::Ruler;
    world.people[leader.index()].country = Some(id);
    world.countries[id.index()].ruler = Some(leader);
    world.countries[id.index()].stability = 0.6;
    world.countries[from.index()].stability += 0.15;
    event(
        world,
        EventKind::RulerCrowned {
            person: leader,
            country: id,
        },
        Some(pos),
    );
}

fn plagues(world: &mut World, rng: &mut Rng) {
    if !rng.gen_bool(0.02) {
        return;
    }
    let Some(&c) = living_countries(world).choose(rng) else {
        return;
    };
    let mut deaths = 0;
    for l in holdings(world, c) {
        let locality = &mut world.localities[l.index()];
        let lost = (locality.population as f32 * rng.gen_range(0.1..0.35)) as u32;
        locality.population -= lost;
        deaths += lost;
    }
    let pos = capital_pos(world, c);
    event(world, EventKind::Plague { country: c, deaths }, Some(pos));
    let victims: Vec<PersonId> = world
        .people
        .iter()
        .filter(|p| p.country == Some(c) && p.died.is_none())
        .map(|p| p.id)
        .collect();
    for p in victims {
        if rng.gen_bool(0.1) {
            kill(world, p, DeathCause::Plague);
        }
    }
    world.countries[c.index()].stability -= 0.15;
}
