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
}

impl EventKind {
    pub fn involves(&self, subject: Subject) -> bool {
        match (*self, subject) {
            (EventKind::LocalityFounded { locality }, Subject::Locality(l)) => locality == l,
            (EventKind::CountryFounded { country }, Subject::Country(c)) => country == c,
            (EventKind::PersonBorn { person }, Subject::Person(p)) => person == p,
            (EventKind::RulerCrowned { person, .. }, Subject::Person(p)) => person == p,
            (EventKind::RulerCrowned { country, .. }, Subject::Country(c)) => country == c,
            _ => false,
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

    pub fn events_about(&self, subject: Subject) -> impl Iterator<Item = &Event> {
        self.events.iter().filter(move |e| e.kind.involves(subject))
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
        match event.kind {
            EventKind::LocalityFounded { locality } => {
                let l = self.locality(locality);
                format!("Se funda {} ({})", l.name, l.kind.name().to_lowercase())
            }
            EventKind::CountryFounded { country } => {
                let c = self.country(country);
                format!("Nace {}, {}", c.name, c.government.name().to_lowercase())
            }
            EventKind::PersonBorn { person } => {
                let p = self.person(person);
                format!("Nace {} en {}", p.name, self.locality(p.home).name)
            }
            EventKind::RulerCrowned { person, country } => {
                format!(
                    "{} asume el gobierno de {}",
                    self.person(person).name,
                    self.country(country).name
                )
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
