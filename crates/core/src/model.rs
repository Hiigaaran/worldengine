//! Modelo de datos del mundo, organizado por capas de detalle:
//! Mundo → Regiones → Países → Localidades → Personajes → Eventos.
//!
//! El mapa es la puerta de entrada: cada casilla sabe a qué región y país
//! pertenece, y desde ahí se llega al resto de la información.

use serde::{Deserialize, Serialize};

use crate::ids::*;

/// Año del calendario del mundo. Puede ser negativo (antes de la era).
pub type Year = i32;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Pos {
    pub x: u32,
    pub y: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorldConfig {
    pub seed: u64,
    pub width: u32,
    pub height: u32,
    /// Fracción aproximada de la superficie cubierta por agua (0.0 a 1.0).
    pub sea_level: f32,
    pub countries: u32,
    pub start_year: Year,
}

impl Default for WorldConfig {
    fn default() -> Self {
        Self {
            seed: 1,
            width: 160,
            height: 100,
            sea_level: 0.45,
            countries: 6,
            start_year: 0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Biome {
    Ocean,
    Coast,
    Desert,
    Grassland,
    Forest,
    Jungle,
    Tundra,
    Mountain,
    Glacier,
}

impl Biome {
    pub fn is_water(self) -> bool {
        matches!(self, Biome::Ocean | Biome::Coast)
    }

    pub fn name(self) -> &'static str {
        match self {
            Biome::Ocean => "Océano",
            Biome::Coast => "Costa",
            Biome::Desert => "Desierto",
            Biome::Grassland => "Pradera",
            Biome::Forest => "Bosque",
            Biome::Jungle => "Selva",
            Biome::Tundra => "Tundra",
            Biome::Mountain => "Montaña",
            Biome::Glacier => "Glaciar",
        }
    }
}

/// Una casilla del mapa del mundo.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Tile {
    /// 0.0 = fondo marino, 1.0 = cumbre más alta.
    pub elevation: f32,
    /// 0.0 = polar, 1.0 = ecuatorial.
    pub temperature: f32,
    /// 0.0 = árido, 1.0 = muy lluvioso.
    pub rainfall: f32,
    pub biome: Biome,
    pub region: Option<RegionId>,
    pub owner: Option<CountryId>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorldMap {
    pub width: u32,
    pub height: u32,
    /// Casillas por filas: índice = y * width + x.
    pub tiles: Vec<Tile>,
}

impl WorldMap {
    pub fn index(&self, pos: Pos) -> usize {
        (pos.y * self.width + pos.x) as usize
    }

    pub fn pos(&self, index: usize) -> Pos {
        let i = index as u32;
        Pos {
            x: i % self.width,
            y: i / self.width,
        }
    }

    pub fn tile(&self, pos: Pos) -> Option<&Tile> {
        (pos.x < self.width && pos.y < self.height).then(|| &self.tiles[self.index(pos)])
    }

    /// Vecinos ortogonales dentro del mapa.
    pub fn neighbors(&self, pos: Pos) -> impl Iterator<Item = Pos> + '_ {
        const DIRS: [(i32, i32); 4] = [(1, 0), (-1, 0), (0, 1), (0, -1)];
        DIRS.iter().filter_map(move |(dx, dy)| {
            let x = pos.x as i32 + dx;
            let y = pos.y as i32 + dy;
            (x >= 0 && y >= 0 && (x as u32) < self.width && (y as u32) < self.height).then_some(
                Pos {
                    x: x as u32,
                    y: y as u32,
                },
            )
        })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Region {
    pub id: RegionId,
    pub name: String,
    pub biome: Biome,
    pub area: u32,
    /// Casilla representativa, donde se dibuja la etiqueta.
    pub center: Pos,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Government {
    Monarchy,
    Republic,
    Theocracy,
    TribalCouncil,
}

impl Government {
    pub fn name(self) -> &'static str {
        match self {
            Government::Monarchy => "Monarquía",
            Government::Republic => "República",
            Government::Theocracy => "Teocracia",
            Government::TribalCouncil => "Consejo tribal",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Country {
    pub id: CountryId,
    pub name: String,
    pub government: Government,
    pub capital: LocalityId,
    pub founded: Year,
    pub ruler: Option<PersonId>,
    /// Color en formato RGB para el mapa político.
    pub color: [u8; 3],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum LocalityKind {
    Village,
    Town,
    City,
}

impl LocalityKind {
    pub fn name(self) -> &'static str {
        match self {
            LocalityKind::Village => "Aldea",
            LocalityKind::Town => "Pueblo",
            LocalityKind::City => "Ciudad",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Locality {
    pub id: LocalityId,
    pub name: String,
    pub kind: LocalityKind,
    pub pos: Pos,
    pub country: Option<CountryId>,
    pub population: u32,
    pub founded: Year,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Role {
    Ruler,
    Noble,
    General,
    Priest,
    Merchant,
    Scholar,
    Adventurer,
}

impl Role {
    pub fn name(self) -> &'static str {
        match self {
            Role::Ruler => "Gobernante",
            Role::Noble => "Noble",
            Role::General => "General",
            Role::Priest => "Sacerdote",
            Role::Merchant => "Mercader",
            Role::Scholar => "Erudito",
            Role::Adventurer => "Aventurero",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Person {
    pub id: PersonId,
    pub name: String,
    pub role: Role,
    pub born: Year,
    pub died: Option<Year>,
    pub home: LocalityId,
    pub country: Option<CountryId>,
}

impl Person {
    pub fn alive_in(&self, year: Year) -> bool {
        self.born <= year && self.died.is_none_or(|d| d > year)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EventKind {
    LocalityFounded {
        locality: LocalityId,
    },
    CountryFounded {
        country: CountryId,
    },
    PersonBorn {
        person: PersonId,
    },
    RulerCrowned {
        person: PersonId,
        country: CountryId,
    },
}

/// Un hecho histórico. Los eventos son la columna vertebral de la línea de tiempo
/// y del futuro modo Leyendas.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Event {
    pub id: EventId,
    pub year: Year,
    pub kind: EventKind,
    pub location: Option<Pos>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct World {
    pub config: WorldConfig,
    pub name: String,
    /// Año actual de la simulación.
    pub year: Year,
    pub map: WorldMap,
    pub regions: Vec<Region>,
    pub countries: Vec<Country>,
    pub localities: Vec<Locality>,
    pub people: Vec<Person>,
    pub events: Vec<Event>,
}
