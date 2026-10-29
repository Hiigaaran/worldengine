//! Consultas sobre el mundo, pensadas para la interfaz: qué hay en una casilla,
//! quién vive en una localidad y qué eventos mencionan a una entidad.

use crate::age::Age;
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
            EventKind::Marriage { a, b, union } => {
                let mut v = vec![S::Person(a), S::Person(b)];
                if let Some((x, y)) = union {
                    v.extend([S::Country(x), S::Country(y)]);
                }
                v
            }
            EventKind::Assassinated {
                victim, plotter, ..
            }
            | EventKind::PlotFoiled {
                target: victim,
                plotter,
                ..
            } => vec![S::Person(victim), S::Person(plotter)],
            EventKind::Calamity { country, .. } => vec![S::Country(country)],
            EventKind::Feat { person, target, .. } => {
                let mut v = vec![S::Person(person)];
                v.extend(target.map(S::Country));
                v
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

    /// Dónde ocurrió un evento. Si no tiene lugar propio (una alianza, una
    /// boda), se toma el de lo que menciona: la localidad, el hogar del
    /// personaje o la capital del país.
    pub fn event_place(&self, event: &Event) -> Option<Pos> {
        event.location.or_else(|| {
            event
                .kind
                .subjects()
                .into_iter()
                .map(|s| match s {
                    Subject::Locality(l) => self.locality(l).pos,
                    Subject::Person(p) => self.locality(self.person(p).home).pos,
                    Subject::Country(c) => self.locality(self.country(c).capital).pos,
                    Subject::War(w) => {
                        self.locality(self.country(self.war(w).defender).capital)
                            .pos
                    }
                })
                .next()
        })
    }

    /// Los lugares de un grupo de eventos, sin repetir y en orden.
    pub fn places_of<'a>(&self, events: impl IntoIterator<Item = &'a Event>) -> Vec<Pos> {
        let mut places = Vec::new();
        for place in events.into_iter().filter_map(|e| self.event_place(e)) {
            if !places.contains(&place) {
                places.push(place);
            }
        }
        places
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
            EventKind::Marriage { a, b, union } => {
                union.is_some()
                    || [a, b].iter().any(|&p| {
                        self.person(p).role == Role::Ruler
                            || self.countries.iter().any(|c| c.ruler == Some(p))
                    })
            }
            EventKind::PersonDied { person, cause } => {
                cause == DeathCause::Battle
                    || cause == DeathCause::Assassination
                    || cause == DeathCause::Execution
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

    /// Las historias que se tejen con los eventos del mundo (ver
    /// [`crate::stories`]).
    pub fn stories(&self) -> Vec<crate::Story> {
        crate::stories::weave(self)
    }

    /// Descripción en una línea de un evento, para la línea de tiempo.
    pub fn describe(&self, event: &Event) -> String {
        let country = |c: CountryId| self.country(c).name.as_str();
        let locality = |l: LocalityId| self.locality(l).name.as_str();
        let person = |p: PersonId| self.person(p).name.as_str();
        let age = self.config.age;
        let place = || {
            event
                .location
                .and_then(|pos| self.locality_at(pos))
                .map_or("", |l| l.name.as_str())
        };
        match event.kind {
            EventKind::LocalityFounded { locality: l } => {
                format!(
                    "Se funda {} ({})",
                    locality(l),
                    self.locality(l).kind.name(age).to_lowercase()
                )
            }
            EventKind::CountryFounded { country: c } => {
                format!(
                    "Nace {}, {}",
                    country(c),
                    self.country(c).government.name(age).to_lowercase()
                )
            }
            EventKind::PersonBorn { person: p } => {
                let who = self.person(p);
                let couple = event.cause.and_then(|c| match self.events[c.index()].kind {
                    EventKind::Marriage { a, b, .. } => Some((a, b)),
                    _ => None,
                });
                if let Some((a, b)) = couple {
                    return format!(
                        "Nace {}, hijo de {} y {}, en {}",
                        who.name,
                        person(a),
                        person(b),
                        locality(who.home)
                    );
                }
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
                    who.role.name(age).to_lowercase(),
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
                let years = event.year - who.born;
                format!(
                    "Muere {} ({}) {} a los {years} años",
                    who.name,
                    who.role.name(age).to_lowercase(),
                    cause.name(age)
                )
            }
            EventKind::LocalityGrew { locality: l, kind } => {
                format!(
                    "{} crece y se convierte en {}",
                    locality(l),
                    kind.name(age).to_lowercase()
                )
            }
            EventKind::LocalityRuined { locality: l } => {
                format!("{} queda abandonada y en ruinas", locality(l))
            }
            EventKind::LocalityAnnexed {
                locality: l,
                country: c,
            } => {
                format!("{} se une a {}", locality(l), country(c))
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
                format!(
                    "La {} asola {}: mueren {deaths} personas",
                    age.plague(),
                    country(c)
                )
            }
            EventKind::Marriage { a, b, union } => {
                let (pa, pb) = (self.person(a), self.person(b));
                match union {
                    Some((x, y)) => format!(
                        "Boda de {} ({}) y {} ({}), que une a {} y {}",
                        pa.name,
                        pa.role.name(age).to_lowercase(),
                        pb.name,
                        pb.role.name(age).to_lowercase(),
                        country(x),
                        country(y)
                    ),
                    None => format!("{} y {} se casan", pa.name, pb.name),
                }
            }
            EventKind::Assassinated {
                victim,
                plotter,
                motive,
                exposed,
            } => {
                let v = self.person(victim);
                let mut text = format!(
                    "{} ({}) es asesinado en {}",
                    v.name,
                    v.role.name(age).to_lowercase(),
                    locality(v.home)
                );
                if exposed {
                    text += &format!("; detrás estaba {}, por {}", person(plotter), motive.name());
                } else {
                    text += "; nadie sabe quién dio la orden";
                }
                text
            }
            EventKind::PlotFoiled {
                target,
                plotter,
                motive,
            } => {
                format!(
                    "Se descubre la conjura de {} para asesinar a {} (por {})",
                    person(plotter),
                    person(target),
                    motive.name()
                )
            }
            EventKind::Calamity {
                country: c,
                calamity,
                deaths,
            } => {
                let what = match calamity {
                    Calamity::Famine => format!("La hambruna azota {}", country(c)),
                    Calamity::Flood => format!("Las crecidas inundan {}", country(c)),
                    Calamity::Earthquake => {
                        format!("Un terremoto sacude {} ({})", place(), country(c))
                    }
                    Calamity::Fire => format!("Un gran incendio arrasa {}", place()),
                    Calamity::Dragon => format!(
                        "El dragón {} ataca {}",
                        self.calamity_name(event).unwrap_or_default(),
                        place()
                    ),
                    Calamity::Undead => {
                        format!("Los muertos se alzan de sus tumbas en {}", place())
                    }
                    Calamity::ArcaneStorm => {
                        format!("Una tormenta arcana desgarra el cielo de {}", country(c))
                    }
                    Calamity::DemonGate => format!("Se abre un portal demoníaco en {}", place()),
                    Calamity::Blackout => format!("Un gran apagón deja a oscuras {}", place()),
                    Calamity::RogueAi => format!(
                        "La IA {} se rebela en {}",
                        self.calamity_name(event).unwrap_or_default(),
                        place()
                    ),
                    Calamity::AcidRain => format!("La lluvia ácida corroe {}", country(c)),
                    Calamity::Meltdown => format!("Se funde el reactor de {}", place()),
                };
                format!("{what}: mueren {deaths} personas")
            }
            EventKind::Feat {
                person: p,
                feat,
                target,
            } => {
                let who = self.person(p);
                let doer = format!("{}, {},", who.name, who.role.name(age).to_lowercase());
                let detail = self.feat_detail(event);
                match feat {
                    Feat::Invention => format!("{doer} inventa {detail}"),
                    Feat::Treatise => format!("{doer} escribe {detail}"),
                    Feat::Expedition if age == Age::Fantasy => {
                        format!("{doer} explora las ruinas olvidadas de {detail}")
                    }
                    Feat::Expedition => format!("{doer} encabeza una expedición a {detail}"),
                    Feat::Artifact => format!("{doer} forja {detail}"),
                    Feat::Grimoire => format!("{doer} descifra {detail}"),
                    Feat::ArtificialMind => {
                        format!("{doer} crea {detail}, una inteligencia artificial")
                    }
                    Feat::Implant => format!("{doer} desarrolla {detail}"),
                    Feat::Heist => match target {
                        Some(t) => format!("{doer} roba los datos secretos de {}", country(t)),
                        None => format!("{doer} da un golpe informático"),
                    },
                    Feat::Rescue => {
                        let deed = event
                            .cause
                            .and_then(|c| match self.events[c.index()].kind {
                                EventKind::Calamity { calamity, .. } => Some(calamity.rescue().0),
                                _ => None,
                            })
                            .unwrap_or("salva a su pueblo");
                        format!("{doer} {deed} en {}", place())
                    }
                }
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

/// El lugar que mejor resume un grupo de lugares: el más cercano al promedio
/// de todos. Es siempre uno de ellos, así que cae en tierra y no en mitad del
/// mar entre dos costas.
pub fn central(places: &[Pos]) -> Option<Pos> {
    let n = places.len() as f32;
    let (sx, sy) = places
        .iter()
        .fold((0.0, 0.0), |(x, y), p| (x + p.x as f32, y + p.y as f32));
    let (mx, my) = (sx / n, sy / n);
    places.iter().copied().min_by(|a, b| {
        let d = |p: &Pos| (p.x as f32 - mx).powi(2) + (p.y as f32 - my).powi(2);
        d(a).total_cmp(&d(b))
    })
}
