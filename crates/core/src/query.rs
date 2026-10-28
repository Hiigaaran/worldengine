//! Consultas sobre el mundo, pensadas para la interfaz: qué hay en una casilla,
//! quién vive en una localidad y qué eventos mencionan a una entidad.

use crate::ids::*;
use crate::model::*;

/// Una entidad a la que puede referirse un evento.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Subject {
    Country(CountryId),
    Locality(LocalityId),
    Person(PersonId),
    War(WarId),
}

impl EventKind {
    /// Entidades mencionadas por el evento.
    pub fn subjects(&self) -> Vec<Subject> {
        use Subject as S;
        match *self {
            EventKind::LocalityFounded { locality }
            | EventKind::LocalityGrew { locality, .. }
            | EventKind::LocalityRuined { locality } => vec![S::Locality(locality)],
            EventKind::CountryFounded { country } | EventKind::Plague { country, .. } => {
                vec![S::Country(country)]
            }
            EventKind::PersonBorn { person }
            | EventKind::PersonRose { person }
            | EventKind::PersonDied { person, .. } => {
                vec![S::Person(person)]
            }
            EventKind::RulerCrowned { person, country } => {
                vec![S::Person(person), S::Country(country)]
            }
            EventKind::LocalityAnnexed { locality, country }
            | EventKind::CapitalMoved { country, locality } => {
                vec![S::Locality(locality), S::Country(country)]
            }
            EventKind::AllianceFormed { a, b } | EventKind::AllianceBroken { a, b } => {
                vec![S::Country(a), S::Country(b)]
            }
            EventKind::WarDeclared { war } | EventKind::PeaceSigned { war } => vec![S::War(war)],
            EventKind::Battle {
                war,
                locality,
                winner,
                loser,
            } => {
                vec![
                    S::War(war),
                    S::Locality(locality),
                    S::Country(winner),
                    S::Country(loser),
                ]
            }
            EventKind::LocalityConquered {
                war,
                locality,
                from,
                to,
            } => {
                vec![
                    S::War(war),
                    S::Locality(locality),
                    S::Country(from),
                    S::Country(to),
                ]
            }
            EventKind::CountryFell { country, conqueror } => {
                let mut v = vec![S::Country(country)];
                v.extend(conqueror.map(S::Country));
                v
            }
            EventKind::Independence {
                locality,
                from,
                country,
            } => {
                vec![S::Locality(locality), S::Country(from), S::Country(country)]
            }
        }
    }
}

impl World {
    pub fn region(&self, id: RegionId) -> &Region {
        &self.regions[id.index()]
    }

    pub fn river(&self, id: RiverId) -> &River {
        &self.rivers[id.index()]
    }

    pub fn country(&self, id: CountryId) -> &Country {
        &self.countries[id.index()]
    }

    pub fn locality(&self, id: LocalityId) -> &Locality {
        &self.localities[id.index()]
    }

    pub fn person(&self, id: PersonId) -> &Person {
        &self.people[id.index()]
    }

    pub fn locality_at(&self, pos: Pos) -> Option<&Locality> {
        self.localities.iter().find(|l| l.pos == pos)
    }

    pub fn localities_of(&self, country: CountryId) -> impl Iterator<Item = &Locality> {
        self.localities
            .iter()
            .filter(move |l| l.country == Some(country))
    }

    pub fn residents(&self, locality: LocalityId) -> impl Iterator<Item = &Person> {
        self.people.iter().filter(move |p| p.home == locality)
    }

    pub fn war(&self, id: WarId) -> &War {
        &self.wars[id.index()]
    }

    /// Eventos que mencionan a una entidad. Las guerras incluyen sus batallas, y
    /// un país, las guerras en que participó.
    pub fn events_about(&self, subject: Subject) -> impl Iterator<Item = &Event> {
        self.events.iter().filter(move |e| {
            let subjects = e.kind.subjects();
            subjects.contains(&subject)
                || match subject {
                    Subject::Country(c) => subjects.iter().any(|s| match *s {
                        Subject::War(w) => self.war(w).attacker == c || self.war(w).defender == c,
                        _ => false,
                    }),
                    _ => false,
                }
        })
    }

    /// Los eventos menores (nacimientos, aldeas que crecen, muertes de personajes
    /// secundarios) se pueden ocultar en la línea de tiempo para leer solo la
    /// gran historia.
    pub fn is_major(&self, event: &Event) -> bool {
        match event.kind {
            EventKind::PersonBorn { .. }
            | EventKind::PersonRose { .. }
            | EventKind::LocalityFounded { .. }
            | EventKind::Battle { .. }
            | EventKind::LocalityGrew {
                kind: LocalityKind::Town,
                ..
            } => false,
            EventKind::PersonDied { person, cause } => {
                cause == DeathCause::Battle
                    || cause == DeathCause::Assassination
                    || self.person(person).role == Role::Ruler
                    || self.countries.iter().any(|c| c.ruler == Some(person))
            }
            _ => true,
        }
    }

    /// Gobernantes de un país en orden, con el año en que asumieron.
    pub fn rulers_of(&self, country: CountryId) -> Vec<(Year, PersonId)> {
        self.events
            .iter()
            .filter_map(|e| match e.kind {
                EventKind::RulerCrowned { person, country: c } if c == country => {
                    Some((e.year, person))
                }
                _ => None,
            })
            .collect()
    }

    pub fn children_of(&self, person: PersonId) -> impl Iterator<Item = &Person> {
        self.people.iter().filter(move |p| p.parent == Some(person))
    }

    pub fn allies_of(&self, country: CountryId) -> impl Iterator<Item = CountryId> + '_ {
        self.relations
            .iter()
            .filter(|r| r.allied)
            .filter_map(move |r| {
                if r.a == country {
                    Some(r.b)
                } else if r.b == country {
                    Some(r.a)
                } else {
                    None
                }
            })
    }

    pub fn wars_of(&self, country: CountryId) -> impl Iterator<Item = &War> {
        self.wars
            .iter()
            .filter(move |w| w.attacker == country || w.defender == country)
    }

    /// Superficie de un país en casillas.
    pub fn territory_size(&self, country: CountryId) -> usize {
        self.map
            .tiles
            .iter()
            .filter(|t| t.owner == Some(country))
            .count()
    }

    /// Descripción en una línea de un evento, para la línea de tiempo.
    pub fn describe(&self, event: &Event) -> String {
        let country = |c: CountryId| self.country(c).name.as_str();
        let locality = |l: LocalityId| self.locality(l).name.as_str();
        let person = |p: PersonId| self.person(p).name.as_str();
        match event.kind {
            EventKind::LocalityFounded { locality: l } => {
                format!(
                    "Se funda {} ({})",
                    locality(l),
                    self.locality(l).kind.name().to_lowercase()
                )
            }
            EventKind::CountryFounded { country: c } => {
                format!(
                    "Nace {}, {}",
                    country(c),
                    self.country(c).government.name().to_lowercase()
                )
            }
            EventKind::PersonBorn { person: p } => {
                let who = self.person(p);
                match who.parent {
                    Some(parent) => format!(
                        "Nace {}, hijo de {}, en {}",
                        who.name,
                        person(parent),
                        locality(who.home)
                    ),
                    None => format!("Nace {} en {}", who.name, locality(who.home)),
                }
            }
            EventKind::PersonRose { person: p } => {
                let who = self.person(p);
                format!(
                    "{}, {}, gana renombre en {}",
                    who.name,
                    who.role.name().to_lowercase(),
                    locality(who.home)
                )
            }
            EventKind::RulerCrowned {
                person: p,
                country: c,
            } => {
                format!("{} asume el gobierno de {}", person(p), country(c))
            }
            EventKind::PersonDied { person: p, cause } => {
                let who = self.person(p);
                let age = event.year - who.born;
                format!(
                    "Muere {} ({}) {} a los {age} años",
                    who.name,
                    who.role.name().to_lowercase(),
                    cause.name()
                )
            }
            EventKind::LocalityGrew { locality: l, kind } => {
                format!(
                    "{} crece y se convierte en {}",
                    locality(l),
                    kind.name().to_lowercase()
                )
            }
            EventKind::LocalityRuined { locality: l } => {
                format!("{} queda abandonada y en ruinas", locality(l))
            }
            EventKind::LocalityAnnexed {
                locality: l,
                country: c,
            } => {
                format!("La aldea de {} se une a {}", locality(l), country(c))
            }
            EventKind::AllianceFormed { a, b } => {
                format!("{} y {} sellan una alianza", country(a), country(b))
            }
            EventKind::AllianceBroken { a, b } => {
                format!("Se rompe la alianza entre {} y {}", country(a), country(b))
            }
            EventKind::WarDeclared { war } => {
                let w = self.war(war);
                format!(
                    "{} declara la guerra a {}: comienza la {}",
                    country(w.attacker),
                    country(w.defender),
                    w.name
                )
            }
            EventKind::Battle {
                war,
                locality: l,
                winner,
                ..
            } => {
                format!(
                    "Batalla de {} ({}): vence {}",
                    locality(l),
                    self.war(war).name,
                    country(winner)
                )
            }
            EventKind::LocalityConquered {
                locality: l,
                from,
                to,
                ..
            } => {
                format!(
                    "{} conquista {}, que pertenecía a {}",
                    country(to),
                    locality(l),
                    country(from)
                )
            }
            EventKind::PeaceSigned { war } => {
                let w = self.war(war);
                match w.winner {
                    Some(c) => format!("Termina la {}: vence {}", w.name, country(c)),
                    None => format!("Termina la {} sin vencedor", w.name),
                }
            }
            EventKind::CapitalMoved {
                country: c,
                locality: l,
            } => {
                format!("{} traslada su capital a {}", country(c), locality(l))
            }
            EventKind::CountryFell {
                country: c,
                conqueror,
            } => match conqueror {
                Some(by) => format!("Cae {}, absorbido por {}", country(c), country(by)),
                None => format!("Cae {}", country(c)),
            },
            EventKind::Independence {
                locality: l,
                from,
                country: c,
            } => {
                format!(
                    "{} se rebela contra {} y se proclama independiente como {}",
                    locality(l),
                    country(from),
                    country(c)
                )
            }
            EventKind::Plague { country: c, deaths } => {
                format!("La peste asola {}: mueren {deaths} personas", country(c))
            }
        }
    }

    pub fn to_json(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(self)
    }

    pub fn from_json(json: &str) -> serde_json::Result<World> {
        serde_json::from_str(json)
    }
}
