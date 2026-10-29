//! Simulación de la historia año a año, al estilo de Dwarf Fortress: el mundo
//! recién fundado evoluciona y cada cambio importante queda como evento.
//!
//! Cada año, en este orden:
//! 1. Las localidades crecen hacia la capacidad de su entorno; las aldeas se
//!    hacen pueblos y ciudades, y las despobladas quedan en ruinas.
//! 2. Los países extienden sus fronteras y fundan nuevas aldeas.
//! 3. Los personajes envejecen y mueren; nacen herederos y nuevas figuras. Al
//!    morir un gobernante, la sucesión depende del tipo de gobierno.
//!    Los nobles se casan (a veces con casas extranjeras, acercando a sus
//!    países) y se traman conjuras: usurpadores, enemigos y vengadores.
//! 4. Las relaciones cambian: los vecinos acumulan tensiones, se forman y rompen
//!    alianzas, y estallan guerras.
//! 5. Las guerras se libran batalla a batalla; se conquistan localidades, caen
//!    capitales y desaparecen países.
//! 6. Los países inestables sufren rebeliones y las pestes diezman poblaciones.
//! 7. Las calamidades propias de la edad (hambrunas, dragones, apagones...)
//!    golpean a un país, y a veces surge alguien que las vence. Los personajes
//!    logran hazañas: inventos, artefactos, IAs, robos de datos.
//!
//! Cada evento recuerda, cuando lo hay, el hecho que lo provocó (`Event::cause`):
//! la batalla que costó una ciudad, el asesinato que desató una guerra. Con
//! esos vínculos se tejen las historias de [`crate::stories`].
//!
//! Cada año usa su propio flujo aleatorio, así que simular 100 años y luego 50
//! más da exactamente el mismo mundo que simular 150 de una vez.

use std::collections::BTreeSet;

use rand::seq::SliceRandom;
use rand::Rng as _;

use crate::age::Age;
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
    marriages(world, &mut rng);
    intrigues(world, &mut rng);
    diplomacy(world, &mut rng);
    wage_wars(world, &mut rng);
    rebellions(world, &mut rng);
    plagues(world, &mut rng);
    // Las calamidades y hazañas usan su propio flujo: así añadirlas no cambia
    // el resto de la historia de cada año.
    let mut fate = WorldRng::new(world.config.seed)
        .stream(&format!("fate-{:?}-{}", world.config.age, world.year));
    calamities(world, &mut fate);
    feats(world, &mut fate);
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

fn event(world: &mut World, kind: EventKind, location: Option<Pos>) -> EventId {
    caused(world, kind, location, None)
}

/// Registra un evento provocado por otro.
fn caused(
    world: &mut World,
    kind: EventKind,
    location: Option<Pos>,
    cause: Option<EventId>,
) -> EventId {
    let year = world.year;
    let id = push_event(world, year, kind, location);
    world.events[id.index()].cause = cause;
    id
}

fn home_pos(world: &World, person: PersonId) -> Pos {
    world.localities[world.people[person.index()].home.index()].pos
}

fn ruler_of(world: &World, person: PersonId) -> Option<CountryId> {
    world
        .countries
        .iter()
        .find(|c| c.exists() && c.ruler == Some(person))
        .map(|c| c.id)
}

/// El evento en que murió un personaje.
fn death_event(world: &World, person: PersonId) -> Option<EventId> {
    world.events.iter().rev().find_map(|e| match e.kind {
        EventKind::PersonDied { person: p, .. } | EventKind::Assassinated { victim: p, .. }
            if p == person =>
        {
            Some(e.id)
        }
        _ => None,
    })
}

/// El golpe reciente más duro que sufrió un país: una peste, una calamidad, el
/// asesinato de su gobernante o una guerra perdida. Explica rebeliones y crisis.
fn recent_blow(world: &World, country: CountryId, years: i32) -> Option<EventId> {
    let since = world.year - years;
    world
        .events
        .iter()
        .rev()
        .take_while(|e| e.year >= since)
        .find(|e| match e.kind {
            EventKind::Plague { country: c, .. } | EventKind::Calamity { country: c, .. } => {
                c == country
            }
            EventKind::Assassinated { victim, .. } => {
                world.rulers_of(country).iter().any(|&(_, p)| p == victim)
            }
            EventKind::PeaceSigned { war } => {
                let w = &world.wars[war.index()];
                (w.attacker == country || w.defender == country)
                    && w.winner.is_some_and(|winner| winner != country)
            }
            _ => false,
        })
        .map(|e| e.id)
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

fn kill(
    world: &mut World,
    person: PersonId,
    cause: DeathCause,
    because: Option<EventId>,
) -> Option<EventId> {
    let p = &mut world.people[person.index()];
    if p.died.is_some() {
        return None;
    }
    p.died = Some(world.year);
    p.cause_of_death = Some(cause);
    let pos = world.localities[p.home.index()].pos;
    Some(caused(
        world,
        EventKind::PersonDied { person, cause },
        Some(pos),
        because,
    ))
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
            kill(world, person, cause, None);
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
            let marriage = world.people[ruler.index()]
                .spouse
                .filter(|s| world.people[s.index()].died.is_none())
                .and_then(|_| marriage_of(world, ruler));
            let chance = if marriage.is_some() { 0.18 } else { 0.06 };
            if fertile && children < 4 && rng.gen_bool(chance) {
                let pos = world.localities[capital.index()].pos;
                let child =
                    add_person(world, rng, Role::Noble, capital, Some(c), year, Some(ruler));
                caused(
                    world,
                    EventKind::PersonBorn { person: child },
                    Some(pos),
                    marriage,
                );
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

/// Elige al nuevo gobernante según el tipo de gobierno. `because` es lo que
/// dejó vacío el trono (normalmente, la muerte del anterior).
fn succession(world: &mut World, rng: &mut Rng, c: CountryId, because: Option<EventId>) {
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
            // Sin hijos, hereda el consorte viudo; si no, el noble de más edad.
            let consort = previous
                .and_then(|p| world.people[p.index()].spouse)
                .filter(|s| candidates.contains(s));
            eldest(children)
                .or(consort)
                .or_else(|| eldest(with_role(&[Role::Noble])))
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
    crown(world, c, heir, if dynastic { 0.03 } else { 0.12 }, because);
}

/// Pone a `heir` al frente de `c`, con el coste en estabilidad indicado.
fn crown(world: &mut World, c: CountryId, heir: PersonId, unrest: f32, because: Option<EventId>) {
    let capital = world.countries[c.index()].capital;
    let person = &mut world.people[heir.index()];
    person.role = Role::Ruler;
    person.home = capital;
    person.country = Some(c);
    let country = &mut world.countries[c.index()];
    country.ruler = Some(heir);
    country.stability -= unrest;
    let pos = world.localities[capital.index()].pos;
    caused(
        world,
        EventKind::RulerCrowned {
            person: heir,
            country: c,
        },
        Some(pos),
        because,
    );
}

/// Todo país vivo tiene un gobernante vivo al acabar el año.
fn ensure_rulers(world: &mut World, rng: &mut Rng) {
    for c in living_countries(world) {
        let ruler = world.countries[c.index()].ruler;
        if ruler.is_none_or(|r| world.people[r.index()].died.is_some()) {
            let because = ruler.and_then(|r| death_event(world, r));
            succession(world, rng, c, because);
        }
    }
}

// ---------------------------------------------------------------- intrigas

/// Puede casarse: adulto joven, sin cónyuge vivo y de buena posición.
fn marriageable(world: &World, p: &Person) -> bool {
    p.died.is_none()
        && (18..=45).contains(&(world.year - p.born))
        && p.spouse
            .is_none_or(|s| world.people[s.index()].died.is_some())
        && matches!(
            p.role,
            Role::Ruler | Role::Noble | Role::General | Role::Merchant | Role::Scholar
        )
}

fn related(a: &Person, b: &Person) -> bool {
    a.parent == Some(b.id) || b.parent == Some(a.id) || (a.parent.is_some() && a.parent == b.parent)
}

/// La boda más reciente de un personaje.
fn marriage_of(world: &World, person: PersonId) -> Option<EventId> {
    world.events.iter().rev().find_map(|e| match e.kind {
        EventKind::Marriage { a, b, .. } if a == person || b == person => Some(e.id),
        _ => None,
    })
}

/// Gobernantes y nobles buscan pareja. Los de la familia gobernante prefieren
/// casas extranjeras amigas: esas bodas acercan a los países y pueden sellar
/// alianzas.
fn marriages(world: &mut World, rng: &mut Rng) {
    let seekers: Vec<PersonId> = world
        .people
        .iter()
        .filter(|p| matches!(p.role, Role::Ruler | Role::Noble) && marriageable(world, p))
        .map(|p| p.id)
        .collect();
    for s in seekers {
        let seeker = &world.people[s.index()];
        if !marriageable(world, seeker) {
            continue;
        }
        let Some(country) = seeker.country else {
            continue;
        };
        let royal = seeker.role == Role::Ruler
            || seeker
                .parent
                .is_some_and(|p| world.people[p.index()].role == Role::Ruler);
        if !rng.gen_bool(if seeker.role == Role::Ruler {
            0.25
        } else {
            0.06
        }) {
            continue;
        }
        let foreign = royal && rng.gen_bool(0.5);
        let partners: Vec<PersonId> = world
            .people
            .iter()
            .filter(|p| p.id != s && marriageable(world, p) && !related(p, seeker))
            .filter(|p| match p.country {
                Some(other) if foreign => {
                    other != country
                        && matches!(p.role, Role::Ruler | Role::Noble)
                        && !at_war(world, country, other)
                        && relation_index(world, country, other)
                            .is_some_and(|r| world.relations[r].opinion > -0.2)
                }
                other => !foreign && other == Some(country),
            })
            // Dos gobernantes no se casan: cada uno sigue en su corte.
            .filter(|p| !(p.role == Role::Ruler && seeker.role == Role::Ruler))
            .map(|p| p.id)
            .collect();
        if let Some(&partner) = partners.choose(rng) {
            marry(world, rng, s, partner);
        }
    }
}

fn marry(world: &mut World, rng: &mut Rng, a: PersonId, b: PersonId) {
    // Si uno gobierna, el otro se muda a su corte.
    let (a, b) = if world.people[b.index()].role == Role::Ruler {
        (b, a)
    } else {
        (a, b)
    };
    let (ca, cb) = (
        world.people[a.index()].country,
        world.people[b.index()].country,
    );
    let union = match (ca, cb) {
        (Some(x), Some(y)) if x != y => Some((x, y)),
        _ => None,
    };
    world.people[a.index()].spouse = Some(b);
    world.people[b.index()].spouse = Some(a);
    if world.people[a.index()].role == Role::Ruler {
        world.people[b.index()].home = world.people[a.index()].home;
        world.people[b.index()].country = ca;
    }
    let pos = home_pos(world, a);
    let wedding = event(world, EventKind::Marriage { a, b, union }, Some(pos));

    if let Some((x, y)) = union {
        if let Some(r) = relation_index(world, x, y) {
            let relation = &mut world.relations[r];
            relation.opinion = (relation.opinion + 0.3).min(1.0);
            if !relation.allied && relation.opinion > 0.2 && rng.gen_bool(0.5) {
                relation.allied = true;
                let pos = Some(capital_pos(world, x));
                caused(
                    world,
                    EventKind::AllianceFormed { a: x, b: y },
                    pos,
                    Some(wedding),
                );
            }
        }
    }
}

/// Conjuras: usurpadores en países inestables, gobernantes enemigos que
/// mandan asesinos y vengadores que saldan deudas de sangre.
fn intrigues(world: &mut World, rng: &mut Rng) {
    for p in 0..world.people.len() {
        let Some(grudge) = world.people[p].grudge else {
            continue;
        };
        let avenger = PersonId(p as u32);
        if world.people[p].died.is_some() || world.people[grudge.against.index()].died.is_some() {
            world.people[p].grudge = None;
            continue;
        }
        if age(world, avenger) < 18 || !rng.gen_bool(0.05) {
            continue;
        }
        world.people[p].grudge = None;
        plot(
            world,
            rng,
            avenger,
            grudge.against,
            Motive::Revenge,
            Some(grudge.since),
        );
    }

    for c in living_countries(world) {
        let Some(ruler) = world.countries[c.index()].ruler else {
            continue;
        };
        if world.people[ruler.index()].died.is_some() {
            continue;
        }
        let stability = world.countries[c.index()].stability;
        if stability < 0.4 && rng.gen_bool(0.035) {
            let ambitious: Vec<PersonId> = world
                .people
                .iter()
                .filter(|p| p.country == Some(c) && p.died.is_none() && world.year - p.born >= 20)
                .filter(|p| matches!(p.role, Role::Noble | Role::General))
                .filter(|p| {
                    p.parent != Some(ruler) && Some(p.id) != world.people[ruler.index()].spouse
                })
                .map(|p| p.id)
                .collect();
            if let Some(&plotter) = ambitious.choose(rng) {
                let because = recent_blow(world, c, 10);
                plot(world, rng, plotter, ruler, Motive::Ambition, because);
                continue;
            }
        }
        for other in living_countries(world) {
            if other == c {
                continue;
            }
            let Some(enemy) = world.countries[other.index()].ruler else {
                continue;
            };
            let war = world.wars.iter().find(|w| {
                w.ended.is_none()
                    && ((w.attacker == c && w.defender == other)
                        || (w.attacker == other && w.defender == c))
            });
            let hatred =
                relation_index(world, c, other).is_some_and(|r| world.relations[r].opinion < -0.6);
            let (motive, chance, because) = match war {
                Some(w) => (Motive::War, 0.012, w.declared),
                None if hatred => (Motive::Rivalry, 0.004, None),
                None => continue,
            };
            if rng.gen_bool(chance) {
                plot(world, rng, enemy, ruler, motive, because);
                break;
            }
        }
    }
}

/// Una conjura de `plotter` contra `target` y todas sus consecuencias.
fn plot(
    world: &mut World,
    rng: &mut Rng,
    plotter: PersonId,
    target: PersonId,
    motive: Motive,
    because: Option<EventId>,
) {
    let pos = Some(home_pos(world, target));
    let target_country = world.people[target.index()].country;
    let plotter_country = world.people[plotter.index()].country;
    let foreign = plotter_country.is_some() && plotter_country != target_country;
    let ruled = ruler_of(world, target);
    let success = match motive {
        Motive::Revenge => 0.55,
        Motive::Ambition => 0.5,
        Motive::War | Motive::Rivalry => 0.4,
    };

    let (deed, exposed) = if rng.gen_bool(success) {
        let exposed = rng.gen_bool(0.55);
        let person = &mut world.people[target.index()];
        person.died = Some(world.year);
        person.cause_of_death = Some(DeathCause::Assassination);
        let deed = caused(
            world,
            EventKind::Assassinated {
                victim: target,
                plotter,
                motive,
                exposed,
            },
            pos,
            because,
        );
        // Los hijos del muerto juran venganza contra el culpable, si se sabe.
        if exposed {
            let orphans: Vec<usize> = world
                .people
                .iter()
                .filter(|p| p.parent == Some(target) && p.died.is_none() && p.id != plotter)
                .map(|p| p.id.index())
                .collect();
            for o in orphans {
                world.people[o].grudge = Some(Grudge {
                    against: plotter,
                    since: deed,
                });
            }
        }
        if let Some(c) = ruled {
            world.countries[c.index()].stability -= 0.1;
            // El conspirador de la corte se queda con el trono vacante.
            let seizes = matches!(motive, Motive::Ambition | Motive::Revenge)
                && plotter_country == Some(c)
                && ruler_of(world, plotter).is_none()
                && rng.gen_bool(0.6);
            if seizes {
                crown(world, c, plotter, 0.15, Some(deed));
                return;
            }
        }
        (deed, exposed)
    } else {
        let deed = caused(
            world,
            EventKind::PlotFoiled {
                target,
                plotter,
                motive,
            },
            pos,
            because,
        );
        if let Some(c) = target_country {
            world.countries[c.index()].stability += 0.03;
        }
        (deed, true)
    };

    if !exposed {
        return;
    }
    if foreign {
        // Un asesino extranjero descubierto es motivo de guerra.
        let (Some(victim_side), Some(enemy)) = (target_country, plotter_country) else {
            return;
        };
        if let Some(r) = relation_index(world, victim_side, enemy) {
            world.relations[r].opinion = -1.0;
            world.relations[r].allied = false;
        }
        let ready = world.countries[enemy.index()].exists()
            && world.countries[victim_side.index()].exists()
            && !at_war(world, victim_side, enemy)
            && !in_any_war(world, victim_side);
        if ready && rng.gen_bool(0.6) {
            declare_war(world, rng, victim_side, enemy, Some(deed));
        }
    } else if ruler_of(world, plotter).is_none() && rng.gen_bool(0.7) {
        kill(world, plotter, DeathCause::Execution, Some(deed));
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
            let because = bond(world, a, b);
            caused(world, EventKind::AllianceFormed { a, b }, pos, because);
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
                let because = casus_belli(world, attacker, defender);
                declare_war(world, rng, attacker, defender, because);
            }
        }
    }
}

/// Lo que acerca a dos países: una boda entre sus casas cuyos cónyuges viven,
/// o una guerra contra un enemigo común.
fn bond(world: &World, a: CountryId, b: CountryId) -> Option<EventId> {
    let pair = |x: CountryId, y: CountryId| (x == a && y == b) || (x == b && y == a);
    let alive = |p: PersonId| world.people[p.index()].died.is_none();
    let wedding = world.events.iter().rev().find(|e| match e.kind {
        EventKind::Marriage {
            a: pa,
            b: pb,
            union: Some((x, y)),
        } => pair(x, y) && alive(pa) && alive(pb),
        _ => false,
    });
    if let Some(e) = wedding {
        return Some(e.id);
    }
    let enemy = |c: CountryId, w: &War| {
        (w.attacker == c)
            .then_some(w.defender)
            .or((w.defender == c).then_some(w.attacker))
    };
    let fronts = |c: CountryId| {
        world
            .wars
            .iter()
            .filter(|w| w.ended.is_none())
            .filter_map(move |w| enemy(c, w).map(|e| (e, w.declared)))
    };
    fronts(a)
        .find(|(e, _)| fronts(b).any(|(other, _)| other == *e))
        .and_then(|(_, declared)| declared)
}

/// El agravio que lleva a `attacker` a la guerra contra `defender`: un
/// asesinato o conjura descubierta, una derrota que vengar, una secesión, una
/// alianza traicionada o un robo de datos, de las últimas décadas.
fn casus_belli(world: &World, attacker: CountryId, defender: CountryId) -> Option<EventId> {
    let since = world.year - 60;
    let pair = |x: Option<CountryId>, y: Option<CountryId>| {
        (x == Some(attacker) && y == Some(defender)) || (x == Some(defender) && y == Some(attacker))
    };
    let country_of = |p: PersonId| world.people[p.index()].country;
    world
        .events
        .iter()
        .rev()
        .take_while(|e| e.year >= since)
        .find(|e| match e.kind {
            EventKind::Assassinated {
                victim,
                plotter,
                exposed: true,
                ..
            }
            | EventKind::PlotFoiled {
                target: victim,
                plotter,
                ..
            } => pair(country_of(victim), country_of(plotter)),
            EventKind::PeaceSigned { war } => {
                let w = &world.wars[war.index()];
                pair(Some(w.attacker), Some(w.defender)) && w.winner == Some(defender)
            }
            EventKind::Independence { from, country, .. } => pair(Some(from), Some(country)),
            EventKind::AllianceBroken { a, b } => pair(Some(a), Some(b)),
            EventKind::Feat {
                person,
                feat: Feat::Heist,
                target,
            } => pair(country_of(person), target),
            _ => false,
        })
        .map(|e| e.id)
}

fn declare_war(
    world: &mut World,
    rng: &mut Rng,
    attacker: CountryId,
    defender: CountryId,
    because: Option<EventId>,
) {
    let id = WarId(world.wars.len() as u32);
    let name = war_name(world, rng, attacker, defender, because);
    world.wars.push(War {
        id,
        name,
        attacker,
        defender,
        started: world.year,
        ended: None,
        winner: None,
        conquests: Vec::new(),
        exhaustion: 0.0,
        declared: None,
    });
    let pos = capital_pos(world, defender);
    let declared = caused(
        world,
        EventKind::WarDeclared { war: id },
        Some(pos),
        because,
    );
    world.wars[id.index()].declared = Some(declared);
}

const ORDINALS: [&str; 8] = [
    "Segunda", "Tercera", "Cuarta", "Quinta", "Sexta", "Séptima", "Octava", "Novena",
];

/// Nombre con que los cronistas recuerdan una guerra: por su causa, por el
/// tiempo que hizo aquel año en el frente o por el paisaje donde se libró.
/// Las guerras de revancha heredan el nombre de la anterior («Segunda…»).
fn war_name(
    world: &World,
    rng: &mut Rng,
    attacker: CountryId,
    defender: CountryId,
    because: Option<EventId>,
) -> String {
    let taken = |name: &str| world.wars.iter().any(|w| w.name == name);
    // Prefiere un nombre de la lista que nadie haya usado.
    let pick = |rng: &mut Rng, options: &[&str]| {
        let mut options = options.to_vec();
        options.shuffle(rng);
        options
            .iter()
            .find(|o| !taken(&format!("Guerra {o}")))
            .unwrap_or(&options[0])
            .to_string()
    };
    if let Some(EventKind::PeaceSigned { war }) = because.map(|e| world.events[e.index()].kind) {
        // Revancha: la misma guerra, un capítulo más.
        let previous = &world.wars[war.index()].name;
        let base = match previous.split_once("Guerra ") {
            Some((_, rest)) => format!("Guerra {rest}"),
            None => previous.clone(),
        };
        let pair = |w: &War| {
            (w.attacker == attacker && w.defender == defender)
                || (w.attacker == defender && w.defender == attacker)
        };
        let earlier = world
            .wars
            .iter()
            .filter(|w| pair(w) && (w.name == base || w.name.ends_with(&format!(" {base}"))))
            .count()
            .max(1);
        return format!("{} {base}", ORDINALS[(earlier - 1).min(ORDINALS.len() - 1)]);
    }
    let age = world.config.age;
    let by_age = |realistic: &'static [&'static str],
                  fantasy: &'static [&'static str],
                  cyberpunk: &'static [&'static str]| match age {
        Age::Realistic => realistic,
        Age::Fantasy => fantasy,
        Age::Cyberpunk => cyberpunk,
    };
    let theme = match because.map(|e| world.events[e.index()].kind) {
        Some(EventKind::Assassinated { .. }) => pick(
            rng,
            by_age(
                &["del Puñal", "de la Sangre Real", "del Veneno", "de la Daga"],
                &[
                    "del Puñal Maldito",
                    "de la Sangre Real",
                    "del Veneno Élfico",
                    "de la Daga Negra",
                ],
                &[
                    "del Código Rojo",
                    "de la Bala Fantasma",
                    "del Veneno Digital",
                    "del Contrato",
                ],
            ),
        ),
        Some(EventKind::PlotFoiled { .. }) => pick(
            rng,
            by_age(
                &["de la Conjura", "de los Traidores", "de la Daga Rota"],
                &["de la Conjura", "de los Traidores", "del Hechizo Roto"],
                &["de la Conjura", "de los Topos", "de la Filtración"],
            ),
        ),
        Some(EventKind::Independence { locality, .. }) => {
            format!(
                "de la Secesión de {}",
                world.localities[locality.index()].name
            )
        }
        Some(EventKind::AllianceBroken { .. }) => pick(
            rng,
            by_age(
                &["de la Alianza Rota", "del Juramento Roto"],
                &[
                    "de la Alianza Rota",
                    "del Juramento Roto",
                    "del Pacto de Sangre",
                ],
                &["de la Alianza Rota", "del Contrato Roto"],
            ),
        ),
        Some(EventKind::Feat {
            feat: Feat::Heist, ..
        }) => pick(
            rng,
            &[
                "de los Datos Robados",
                "del Cortafuegos",
                "de la Brecha",
                "del Servidor",
            ],
        ),
        _ => {
            let pos = capital_pos(world, defender);
            let climate = crate::climate::year_climate(world, world.year);
            let rain = climate.at(&world.map, pos);
            let index = world.map.index(pos);
            let stormy = climate.stormy[index];
            let strange = climate.anomaly[index];
            let near_river = world
                .map
                .neighbors8(pos)
                .find_map(|n| world.map.tile(n).unwrap().river);
            let biome = world.map.tile(pos).unwrap().biome;
            if strange && age == Age::Fantasy {
                pick(
                    rng,
                    &[
                        "de las Tormentas Arcanas",
                        "del Cielo Violeta",
                        "de los Relámpagos Arcanos",
                    ],
                )
            } else if strange && age == Age::Cyberpunk {
                pick(
                    rng,
                    &[
                        "de la Lluvia Ácida",
                        "del Cielo Tóxico",
                        "de la Lluvia Negra",
                    ],
                )
            } else if stormy {
                pick(
                    rng,
                    &["de las Tormentas", "del Trueno", "de los Relámpagos"],
                )
            } else if rain > 1.25 {
                pick(rng, &["de la Lluvia", "de las Lluvias", "del Barro"])
            } else if rain < 0.75 {
                pick(rng, &["de la Sequía", "del Polvo", "de la Sed"])
            } else if let (Some(river), true) = (near_river, rng.gen_bool(0.4)) {
                format!("del {}", world.rivers[river.index()].name)
            } else {
                let by_land: &[&str] = match (age, biome) {
                    (Age::Cyberpunk, Biome::Forest | Biome::Jungle | Biome::Taiga) => {
                        &["de la Reserva", "de los Bosques Muertos"]
                    }
                    (Age::Fantasy, Biome::Forest | Biome::Jungle | Biome::Taiga) => {
                        &["de los Bosques", "de la Espesura", "de los Árboles Vivos"]
                    }
                    (_, Biome::Forest | Biome::Jungle | Biome::Taiga) => {
                        &["de los Bosques", "de los Robles", "de la Espesura"]
                    }
                    (Age::Cyberpunk, Biome::Hills | Biome::Mountain) => {
                        &["de las Antenas", "de las Minas"]
                    }
                    (Age::Fantasy, Biome::Hills | Biome::Mountain) => &[
                        "de las Colinas",
                        "de los Pasos",
                        "de los Túmulos",
                        "de las Minas Enanas",
                    ],
                    (_, Biome::Hills | Biome::Mountain) => &["de las Colinas", "de los Pasos"],
                    (Age::Cyberpunk, Biome::Desert | Biome::Savanna | Biome::Shrubland) => {
                        &["del Yermo", "de las Placas Solares", "del Polvo Rojo"]
                    }
                    (_, Biome::Desert | Biome::Savanna | Biome::Shrubland) => {
                        &["de las Arenas", "del Sol", "de las Cenizas"]
                    }
                    (_, Biome::Tundra | Biome::Glacier) => &["del Invierno", "de la Escarcha"],
                    (Age::Cyberpunk, Biome::Swamp) => &["de los Vertederos", "de la Niebla Tóxica"],
                    (_, Biome::Swamp) => &["de las Ciénagas", "de la Niebla"],
                    _ => by_age(
                        &[
                            "de los Cuervos",
                            "de los Estandartes",
                            "de las Dos Coronas",
                            "de las Cosechas",
                            "de las Lanzas",
                        ],
                        &[
                            "de los Cuervos",
                            "de los Estandartes",
                            "de las Dos Coronas",
                            "de los Grifos",
                            "de las Runas",
                            "de los Dragones",
                        ],
                        &[
                            "de los Drones",
                            "de los Datos",
                            "de las Dos Torres",
                            "del Silicio",
                            "del Neón",
                            "de los Mercenarios",
                        ],
                    ),
                };
                if rng.gen_bool(0.25) {
                    format!("de {}", names::word(rng, age, 2, 3))
                } else {
                    pick(rng, by_land)
                }
            }
        }
    };
    let name = format!("Guerra {theme}");
    if taken(&name) {
        format!("Guerra de {}", names::word(rng, age, 2, 3))
    } else {
        name
    }
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
        let declared = world.wars[w].declared;
        let battle = caused(
            world,
            EventKind::Battle {
                war,
                locality: target,
                winner,
                loser,
            },
            Some(pos),
            declared,
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
                    kill(world, general, DeathCause::Battle, Some(battle));
                }
            }
        }

        if winner == attacker && rng.gen_bool(0.4) {
            world.wars[w].conquests.push(target);
            let conquest = caused(
                world,
                EventKind::LocalityConquered {
                    war,
                    locality: target,
                    from: defender,
                    to: attacker,
                },
                Some(pos),
                Some(battle),
            );
            transfer_locality(world, target, attacker, Some(conquest));
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
    let declared = world.wars[war.index()].declared;
    caused(world, EventKind::PeaceSigned { war }, Some(pos), declared);
}

/// Cambia de dueño una localidad junto con las casillas que la rodean (las que
/// tenían esa localidad como la más cercana de su antiguo país).
fn transfer_locality(
    world: &mut World,
    locality: LocalityId,
    to: CountryId,
    because: Option<EventId>,
) {
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
            caused(
                world,
                EventKind::CapitalMoved {
                    country: from,
                    locality: new_capital,
                },
                Some(pos),
                because,
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
        caused(
            world,
            EventKind::CountryFell {
                country: from,
                conqueror: Some(to),
            },
            Some(pos),
            because,
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
    let country = new_country(rng, world.config.age, id, locality, year, government);
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
    let because = recent_blow(world, from, 15);
    let revolt = caused(
        world,
        EventKind::Independence {
            locality,
            from,
            country: id,
        },
        Some(pos),
        because,
    );
    caused(
        world,
        EventKind::CountryFounded { country: id },
        Some(pos),
        Some(revolt),
    );
    transfer_locality(world, locality, id, Some(revolt));

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
            caused(
                world,
                EventKind::PersonRose { person },
                Some(pos),
                Some(revolt),
            );
            person
        }
    };
    world.people[leader.index()].role = Role::Ruler;
    world.people[leader.index()].country = Some(id);
    world.countries[id.index()].ruler = Some(leader);
    world.countries[id.index()].stability = 0.6;
    world.countries[from.index()].stability += 0.15;
    caused(
        world,
        EventKind::RulerCrowned {
            person: leader,
            country: id,
        },
        Some(pos),
        Some(revolt),
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
    let plague = event(world, EventKind::Plague { country: c, deaths }, Some(pos));
    let victims: Vec<PersonId> = world
        .people
        .iter()
        .filter(|p| p.country == Some(c) && p.died.is_none())
        .map(|p| p.id)
        .collect();
    for p in victims {
        if rng.gen_bool(0.1) {
            kill(world, p, DeathCause::Plague, Some(plague));
        }
    }
    world.countries[c.index()].stability -= 0.15;
}

// ---------------------------------------------------------------- edades

/// Una calamidad propia de la edad golpea a un país. Cuál depende del clima
/// del año y de lo que ese país hizo antes: un grimorio prohibido puede abrir
/// un portal, una IA puede rebelarse, los muertos de una guerra pueden alzarse.
fn calamities(world: &mut World, rng: &mut Rng) {
    if !rng.gen_bool(0.03) {
        return;
    }
    let Some(&c) = living_countries(world).choose(rng) else {
        return;
    };
    let homes = holdings(world, c);
    if homes.is_empty() {
        return;
    }
    let age = world.config.age;
    let climate = crate::climate::year_climate(world, world.year);
    let map = &world.map;
    let rain = homes
        .iter()
        .map(|&l| climate.at(map, world.localities[l.index()].pos))
        .sum::<f32>()
        / homes.len() as f32;
    // El fenómeno extraño del año solo es catastrófico si cae sobre la capital.
    let strange = climate.anomaly[map.index(capital_pos(world, c))] && rng.gen_bool(0.5);
    let rugged = homes.iter().any(|&l| {
        let pos = world.localities[l.index()].pos;
        map.neighbors8(pos)
            .any(|n| matches!(map.tile(n).unwrap().biome, Biome::Mountain | Biome::Hills))
    });
    // Lo que la provocó, si hay algo: una hazaña peligrosa o una matanza.
    let since = |years: i32| world.year - years;
    let recent_feat = |feat: Feat| {
        world
            .events
            .iter()
            .rev()
            .take_while(|e| e.year >= since(60))
            .find(|e| {
                matches!(e.kind, EventKind::Feat { person, feat: f, .. }
                    if f == feat && world.people[person.index()].country == Some(c))
            })
            .map(|e| e.id)
    };
    let carnage = world
        .events
        .iter()
        .rev()
        .take_while(|e| e.year >= since(15))
        .find(|e| match e.kind {
            EventKind::Battle { locality, .. } => homes.contains(&locality),
            EventKind::Plague { country, .. } => country == c,
            _ => false,
        })
        .map(|e| e.id);

    let (calamity, because) = match age {
        Age::Realistic => {
            let calamity = if rain < 0.85 {
                Calamity::Famine
            } else if rain > 1.15 {
                Calamity::Flood
            } else if rugged && rng.gen_bool(0.4) {
                Calamity::Earthquake
            } else {
                // Una mala cosecha no necesita sequía: plagas, heladas, langostas.
                *[Calamity::Fire, Calamity::Famine].choose(rng).unwrap()
            };
            (calamity, None)
        }
        Age::Fantasy => {
            let grimoire = recent_feat(Feat::Grimoire);
            if strange {
                (Calamity::ArcaneStorm, None)
            } else if grimoire.is_some() && rng.gen_bool(0.6) {
                (Calamity::DemonGate, grimoire)
            } else if carnage.is_some() && rng.gen_bool(0.5) {
                (Calamity::Undead, carnage)
            } else if rugged || rng.gen_bool(0.5) {
                (Calamity::Dragon, None)
            } else {
                (
                    *[Calamity::Undead, Calamity::DemonGate].choose(rng).unwrap(),
                    None,
                )
            }
        }
        Age::Cyberpunk => {
            let mind = recent_feat(Feat::ArtificialMind);
            if strange {
                (Calamity::AcidRain, None)
            } else if mind.is_some() && rng.gen_bool(0.6) {
                (Calamity::RogueAi, mind)
            } else {
                (
                    *[Calamity::Blackout, Calamity::Meltdown, Calamity::RogueAi]
                        .choose(rng)
                        .unwrap(),
                    None,
                )
            }
        }
    };

    // Unas calamidades caen sobre una ciudad; otras, sobre todo el país.
    let local = !matches!(
        calamity,
        Calamity::Famine | Calamity::Flood | Calamity::ArcaneStorm | Calamity::AcidRain
    );
    let target = if local {
        // Los dragones prefieren aldeas y pueblos; los reactores, las grandes ciudades.
        let pick_big = matches!(
            calamity,
            Calamity::Meltdown | Calamity::Blackout | Calamity::RogueAi | Calamity::Fire
        );
        if pick_big {
            *homes
                .iter()
                .max_by_key(|&&l| (world.localities[l.index()].population, l.0))
                .unwrap()
        } else {
            *homes.choose(rng).unwrap()
        }
    } else {
        world.countries[c.index()].capital
    };
    let struck: Vec<LocalityId> = if local { vec![target] } else { homes };
    let mut deaths = 0;
    for &l in &struck {
        let locality = &mut world.localities[l.index()];
        let share = if local {
            rng.gen_range(0.15..0.4)
        } else {
            rng.gen_range(0.04..0.14)
        };
        let lost = (locality.population as f32 * share) as u32;
        locality.population -= lost;
        deaths += lost;
    }
    let pos = world.localities[target.index()].pos;
    let disaster = caused(
        world,
        EventKind::Calamity {
            country: c,
            calamity,
            deaths,
        },
        Some(pos),
        because,
    );
    world.countries[c.index()].stability -= 0.08;

    let victims: Vec<PersonId> = world
        .people
        .iter()
        .filter(|p| p.died.is_none() && struck.contains(&p.home))
        .map(|p| p.id)
        .collect();
    let risk = if local { 0.15 } else { 0.05 };
    for p in victims {
        if rng.gen_bool(risk) {
            kill(world, p, DeathCause::Calamity, Some(disaster));
        }
    }

    // A veces alguien se alza y vence a la calamidad.
    let chance = match age {
        Age::Realistic => 0.35,
        Age::Fantasy => 0.6,
        Age::Cyberpunk => 0.5,
    };
    if !rng.gen_bool(chance) {
        return;
    }
    let year = world.year;
    let heroes: Vec<PersonId> = world
        .people
        .iter()
        .filter(|p| p.country == Some(c) && p.died.is_none() && year - p.born >= 18)
        .filter(|p| Feat::Rescue.roles().contains(&p.role))
        .map(|p| p.id)
        .collect();
    let hero = match heroes.choose(rng) {
        Some(&h) => h,
        None => {
            let born = year - rng.gen_range(18..35);
            let person = add_person(world, rng, Role::Adventurer, target, Some(c), born, None);
            caused(
                world,
                EventKind::PersonRose { person },
                Some(pos),
                Some(disaster),
            );
            person
        }
    };
    caused(
        world,
        EventKind::Feat {
            person: hero,
            feat: Feat::Rescue,
            target: None,
        },
        Some(pos),
        Some(disaster),
    );
    world.countries[c.index()].stability += 0.08;
}

/// Los personajes logran hazañas propias de su edad: inventos y expediciones,
/// artefactos y grimorios, IAs, implantes y robos de datos.
fn feats(world: &mut World, rng: &mut Rng) {
    let age = world.config.age;
    let year = world.year;
    for c in living_countries(world) {
        if !rng.gen_bool(0.012) {
            continue;
        }
        let feat = *age.feats().choose(rng).unwrap();
        // El robo de datos va contra el país peor visto.
        let target = if feat == Feat::Heist {
            let rival = living_countries(world)
                .into_iter()
                .filter(|&o| o != c)
                .filter_map(|o| {
                    relation_index(world, c, o).map(|r| (o, world.relations[r].opinion))
                })
                .min_by(|a, b| a.1.total_cmp(&b.1).then(a.0.cmp(&b.0)));
            match rival {
                Some((o, _)) => Some(o),
                None => continue,
            }
        } else {
            None
        };
        let doers: Vec<PersonId> = world
            .people
            .iter()
            .filter(|p| p.country == Some(c) && p.died.is_none() && year - p.born >= 18)
            .filter(|p| feat.roles().contains(&p.role))
            .map(|p| p.id)
            .collect();
        let Some(&person) = doers.choose(rng) else {
            continue;
        };
        let pos = home_pos(world, person);
        event(
            world,
            EventKind::Feat {
                person,
                feat,
                target,
            },
            Some(pos),
        );
        world.countries[c.index()].stability += 0.03;
        if let Some(t) = target {
            if let Some(r) = relation_index(world, c, t) {
                world.relations[r].opinion = (world.relations[r].opinion - 0.35).max(-1.0);
            }
        }
    }
}
