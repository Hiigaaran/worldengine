//! Edades: el tono del mundo. La misma semilla da la misma geografía en todas
//! las edades, pero cambian los nombres, los títulos y oficios, el clima
//! extraño, las calamidades y hazañas, y el lenguaje de los cronistas.
//!
//! - **Realista**: reinos y repúblicas de una historia humana sin magia.
//! - **Fantasía**: magos, dragones, muertos que se alzan y tormentas arcanas.
//! - **Cyberpunk**: megacorporaciones, hackers, IAs rebeldes y lluvia ácida.

use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};

use crate::model::*;
use crate::names;
use crate::rng::{Rng, WorldRng};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum Age {
    #[default]
    Realistic,
    Fantasy,
    Cyberpunk,
}

impl Age {
    pub const ALL: [Age; 3] = [Age::Realistic, Age::Fantasy, Age::Cyberpunk];

    pub fn name(self) -> &'static str {
        match self {
            Age::Realistic => "Realista",
            Age::Fantasy => "Fantasía",
            Age::Cyberpunk => "Cyberpunk",
        }
    }

    /// Reconoce una edad por su nombre, sin importar mayúsculas ni tildes:
    /// «realista», «fantasia», «Fantasía», «cyberpunk».
    pub fn from_text(text: &str) -> Option<Age> {
        let t = text.trim().to_lowercase().replace('í', "i");
        Age::ALL
            .into_iter()
            .find(|a| a.name().to_lowercase().replace('í', "i") == t)
    }

    pub fn description(self) -> &'static str {
        match self {
            Age::Realistic => {
                "Reinos, repúblicas y señores de la guerra; hambrunas, terremotos e inventos."
            }
            Age::Fantasy => {
                "Magos y paladines, dragones, muertos que se alzan, artefactos y tormentas arcanas."
            }
            Age::Cyberpunk => {
                "Megacorporaciones y bandas, hackers, IAs rebeldes, apagones y lluvia ácida."
            }
        }
    }

    /// Cómo se llama un linaje gobernante, con artículo: «La Casa», «El Clan».
    pub fn house(self) -> &'static str {
        match self {
            Age::Realistic | Age::Fantasy => "La Casa",
            Age::Cyberpunk => "El Clan",
        }
    }

    /// Quienes rodean al gobernante: «la corte», «la junta».
    pub fn court(self) -> &'static str {
        match self {
            Age::Realistic | Age::Fantasy => "la corte",
            Age::Cyberpunk => "la junta directiva",
        }
    }

    /// La enfermedad que diezma países, en minúscula. Todas son femeninas.
    pub fn plague(self) -> &'static str {
        match self {
            Age::Realistic => "peste",
            Age::Fantasy => "plaga",
            Age::Cyberpunk => "epidemia",
        }
    }

    /// El fenómeno extraño del clima, si la edad tiene uno: (singular, plural).
    pub fn anomaly(self) -> Option<(&'static str, &'static str)> {
        match self {
            Age::Realistic => None,
            Age::Fantasy => Some(("tormenta arcana", "tormentas arcanas")),
            Age::Cyberpunk => Some(("lluvia ácida", "lluvias ácidas")),
        }
    }

    /// Calamidades posibles en esta edad.
    pub fn calamities(self) -> &'static [Calamity] {
        match self {
            Age::Realistic => &[
                Calamity::Famine,
                Calamity::Flood,
                Calamity::Earthquake,
                Calamity::Fire,
            ],
            Age::Fantasy => &[
                Calamity::Dragon,
                Calamity::Undead,
                Calamity::ArcaneStorm,
                Calamity::DemonGate,
            ],
            Age::Cyberpunk => &[
                Calamity::Blackout,
                Calamity::RogueAi,
                Calamity::AcidRain,
                Calamity::Meltdown,
            ],
        }
    }

    /// Hazañas que pueden lograr los personajes por iniciativa propia.
    pub fn feats(self) -> &'static [Feat] {
        match self {
            Age::Realistic => &[Feat::Invention, Feat::Expedition, Feat::Treatise],
            Age::Fantasy => &[Feat::Artifact, Feat::Grimoire, Feat::Expedition],
            Age::Cyberpunk => &[Feat::ArtificialMind, Feat::Implant, Feat::Heist],
        }
    }
}

/// Un sustantivo con su plural y su género, para concordar artículos.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Noun {
    pub one: &'static str,
    pub many: &'static str,
    pub feminine: bool,
}

impl Noun {
    /// «un pueblo», «una aldea».
    pub fn a(self) -> String {
        format!("{} {}", if self.feminine { "una" } else { "un" }, self.one)
    }
}

const fn noun(one: &'static str, many: &'static str, feminine: bool) -> Noun {
    Noun {
        one,
        many,
        feminine,
    }
}

impl Government {
    pub fn name(self, age: Age) -> &'static str {
        self.noun(age).one
    }

    pub fn noun(self, age: Age) -> Noun {
        match (age, self) {
            (Age::Realistic, Government::Monarchy) => noun("Monarquía", "monarquías", true),
            (Age::Realistic, Government::Republic) => noun("República", "repúblicas", true),
            (Age::Realistic, Government::Theocracy) => noun("Teocracia", "teocracias", true),
            (Age::Realistic, Government::TribalCouncil) => {
                noun("Consejo tribal", "consejos tribales", false)
            }
            (Age::Fantasy, Government::Monarchy) => noun("Reino", "reinos", false),
            (Age::Fantasy, Government::Republic) => {
                noun("Concilio de magos", "concilios de magos", false)
            }
            (Age::Fantasy, Government::Theocracy) => {
                noun("Teocracia arcana", "teocracias arcanas", true)
            }
            (Age::Fantasy, Government::TribalCouncil) => noun("Horda", "hordas", true),
            (Age::Cyberpunk, Government::Monarchy) => {
                noun("Megacorporación", "megacorporaciones", true)
            }
            (Age::Cyberpunk, Government::Republic) => noun("Ciudad libre", "ciudades libres", true),
            (Age::Cyberpunk, Government::Theocracy) => {
                noun("Culto tecnológico", "cultos tecnológicos", false)
            }
            (Age::Cyberpunk, Government::TribalCouncil) => {
                noun("Sindicato de bandas", "sindicatos de bandas", false)
            }
        }
    }
}

impl LocalityKind {
    pub fn name(self, age: Age) -> &'static str {
        self.noun(age).one
    }

    pub fn noun(self, age: Age) -> Noun {
        match (age, self) {
            (Age::Realistic, LocalityKind::Village) => noun("Aldea", "aldeas", true),
            (Age::Realistic, LocalityKind::Town) => noun("Pueblo", "pueblos", false),
            (Age::Realistic, LocalityKind::City) => noun("Ciudad", "ciudades", true),
            (Age::Fantasy, LocalityKind::Village) => noun("Aldea", "aldeas", true),
            (Age::Fantasy, LocalityKind::Town) => noun("Burgo", "burgos", false),
            (Age::Fantasy, LocalityKind::City) => noun("Ciudadela", "ciudadelas", true),
            (Age::Cyberpunk, LocalityKind::Village) => noun("Enclave", "enclaves", false),
            (Age::Cyberpunk, LocalityKind::Town) => noun("Ciudad", "ciudades", true),
            (Age::Cyberpunk, LocalityKind::City) => noun("Megaciudad", "megaciudades", true),
        }
    }
}

impl Role {
    pub fn name(self, age: Age) -> &'static str {
        match (age, self) {
            (Age::Realistic | Age::Fantasy, Role::Noble) => "Noble",
            (Age::Realistic, Role::Ruler) => "Gobernante",
            (Age::Realistic, Role::General) => "General",
            (Age::Realistic, Role::Priest) => "Sacerdote",
            (Age::Realistic, Role::Merchant) => "Mercader",
            (Age::Realistic, Role::Scholar) => "Erudito",
            (Age::Realistic, Role::Adventurer) => "Aventurero",
            (Age::Fantasy, Role::Ruler) => "Soberano",
            (Age::Fantasy, Role::General) => "Paladín",
            (Age::Fantasy, Role::Priest) => "Clérigo",
            (Age::Fantasy, Role::Merchant) => "Mercader",
            (Age::Fantasy, Role::Scholar) => "Mago",
            (Age::Fantasy, Role::Adventurer) => "Aventurero",
            (Age::Cyberpunk, Role::Ruler) => "Director",
            (Age::Cyberpunk, Role::Noble) => "Ejecutivo",
            (Age::Cyberpunk, Role::General) => "Comandante",
            (Age::Cyberpunk, Role::Priest) => "Tecnochamán",
            (Age::Cyberpunk, Role::Merchant) => "Traficante",
            (Age::Cyberpunk, Role::Scholar) => "Hacker",
            (Age::Cyberpunk, Role::Adventurer) => "Mercenario",
        }
    }
}

impl DeathCause {
    pub fn name(self, age: Age) -> &'static str {
        match (age, self) {
            (_, DeathCause::OldAge) => "de vejez",
            (_, DeathCause::Assassination) => "asesinado",
            (_, DeathCause::Execution) => "ejecutado",
            (Age::Cyberpunk, DeathCause::Illness) => "por un fallo de sus implantes",
            (_, DeathCause::Illness) => "por enfermedad",
            (Age::Cyberpunk, DeathCause::Battle) => "en combate",
            (_, DeathCause::Battle) => "en batalla",
            (Age::Realistic, DeathCause::Plague) => "por la peste",
            (Age::Fantasy, DeathCause::Plague) => "por la plaga",
            (Age::Cyberpunk, DeathCause::Plague) => "por la epidemia",
            (_, DeathCause::Calamity) => "en la catástrofe",
        }
    }
}

impl Calamity {
    pub fn name(self) -> &'static str {
        match self {
            Calamity::Famine => "Hambruna",
            Calamity::Flood => "Inundación",
            Calamity::Earthquake => "Terremoto",
            Calamity::Fire => "Gran incendio",
            Calamity::Dragon => "Ataque de dragón",
            Calamity::Undead => "Alzamiento de los muertos",
            Calamity::ArcaneStorm => "Tormenta arcana",
            Calamity::DemonGate => "Portal demoníaco",
            Calamity::Blackout => "Gran apagón",
            Calamity::RogueAi => "IA rebelde",
            Calamity::AcidRain => "Lluvia ácida",
            Calamity::Meltdown => "Fusión del reactor",
        }
    }

    /// Con artículo, en minúscula, para mitad de frase: «la hambruna».
    pub fn the(self) -> &'static str {
        match self {
            Calamity::Famine => "la hambruna",
            Calamity::Flood => "la inundación",
            Calamity::Earthquake => "el terremoto",
            Calamity::Fire => "el gran incendio",
            Calamity::Dragon => "el ataque del dragón",
            Calamity::Undead => "el alzamiento de los muertos",
            Calamity::ArcaneStorm => "la tormenta arcana",
            Calamity::DemonGate => "el portal demoníaco",
            Calamity::Blackout => "el gran apagón",
            Calamity::RogueAi => "la rebelión de la IA",
            Calamity::AcidRain => "la lluvia ácida",
            Calamity::Meltdown => "la fusión del reactor",
        }
    }

    /// Lo que hizo quien venció a la calamidad: (presente, pasado).
    pub fn rescue(self) -> (&'static str, &'static str) {
        match self {
            Calamity::Famine => (
                "organiza el reparto de grano y salva a los hambrientos",
                "organizó el reparto de grano y salvó a los hambrientos",
            ),
            Calamity::Flood => (
                "levanta diques y rescata a los damnificados",
                "levantó diques y rescató a los damnificados",
            ),
            Calamity::Earthquake => (
                "dirige la reconstrucción de las calles derrumbadas",
                "dirigió la reconstrucción de las calles derrumbadas",
            ),
            Calamity::Fire => (
                "encabeza a quienes apagan el incendio",
                "encabezó a quienes apagaron el incendio",
            ),
            Calamity::Dragon => ("da muerte al dragón", "dio muerte al dragón"),
            Calamity::Undead => (
                "devuelve a los muertos a sus tumbas",
                "devolvió a los muertos a sus tumbas",
            ),
            Calamity::ArcaneStorm => ("disipa la tormenta arcana", "disipó la tormenta arcana"),
            Calamity::DemonGate => ("sella el portal demoníaco", "selló el portal demoníaco"),
            Calamity::Blackout => (
                "restablece la red eléctrica",
                "restableció la red eléctrica",
            ),
            Calamity::RogueAi => ("desconecta a la IA rebelde", "desconectó a la IA rebelde"),
            Calamity::AcidRain => (
                "levanta escudos contra la lluvia ácida",
                "levantó escudos contra la lluvia ácida",
            ),
            Calamity::Meltdown => (
                "contiene la fusión del reactor",
                "contuvo la fusión del reactor",
            ),
        }
    }
}

impl Feat {
    pub fn name(self) -> &'static str {
        match self {
            Feat::Invention => "Invento",
            Feat::Expedition => "Expedición",
            Feat::Treatise => "Tratado",
            Feat::Artifact => "Artefacto",
            Feat::Grimoire => "Grimorio",
            Feat::ArtificialMind => "Inteligencia artificial",
            Feat::Implant => "Implante",
            Feat::Heist => "Golpe informático",
            Feat::Rescue => "Rescate",
        }
    }

    /// Quién suele lograr cada hazaña.
    pub fn roles(self) -> &'static [Role] {
        match self {
            Feat::Invention | Feat::Treatise => &[Role::Scholar, Role::Merchant],
            Feat::Expedition => &[Role::Adventurer, Role::Merchant, Role::General],
            Feat::Artifact => &[Role::Scholar, Role::Priest, Role::Adventurer],
            Feat::Grimoire => &[Role::Scholar, Role::Priest],
            Feat::ArtificialMind | Feat::Implant => &[Role::Scholar, Role::Merchant],
            Feat::Heist => &[Role::Scholar, Role::Adventurer],
            Feat::Rescue => &[Role::Adventurer, Role::General, Role::Scholar, Role::Priest],
        }
    }
}

/// Nombres de los tipos de historia según la edad.
impl crate::stories::StoryKind {
    pub fn name(self, age: Age) -> &'static str {
        use crate::stories::StoryKind as K;
        match self {
            K::War => "Guerra",
            K::Rivalry => "Rivalidad",
            K::Intrigue => "Intriga",
            K::Marriage => "Boda",
            K::Plague => match age {
                Age::Realistic => "Peste",
                Age::Fantasy => "Plaga",
                Age::Cyberpunk => "Epidemia",
            },
            K::Rebellion => "Rebelión",
            K::Dynasty => "Dinastía",
            K::GoldenAge => "Edad de oro",
            K::Calamity => "Calamidad",
            K::Feat => "Hazaña",
        }
    }
}

impl World {
    /// Azar estable para los detalles de un evento: el mismo evento recibe
    /// siempre el mismo nombre de dragón o de artefacto.
    fn lore_rng(&self, event: &Event) -> Rng {
        WorldRng::new(self.config.seed).stream(&format!("lore-{}-{}", event.year, event.id.0))
    }

    /// Lo que creó o logró una hazaña, con artículo: «la imprenta», «la espada
    /// Kal'dor», «KAIRO, una inteligencia artificial».
    pub fn feat_detail(&self, event: &Event) -> String {
        let EventKind::Feat { feat, target, .. } = event.kind else {
            return String::new();
        };
        let age = self.config.age;
        let mut rng = self.lore_rng(event);
        let pick = |rng: &mut Rng, options: &[&str]| options.choose(rng).unwrap().to_string();
        match feat {
            Feat::Invention => pick(
                &mut rng,
                &[
                    "la imprenta",
                    "el astrolabio",
                    "el arado de hierro",
                    "la brújula",
                    "el molino de viento",
                    "el reloj mecánico",
                    "el telescopio",
                    "el telar mecánico",
                ],
            ),
            Feat::Treatise => format!(
                "un tratado de {}",
                pick(
                    &mut rng,
                    &[
                        "medicina",
                        "leyes",
                        "astronomía",
                        "filosofía",
                        "matemáticas",
                        "historia"
                    ]
                )
            ),
            Feat::Expedition => {
                let regions: Vec<&Region> = self.regions.iter().filter(|r| r.area >= 30).collect();
                match regions.choose(&mut rng) {
                    Some(r) => r.name.clone(),
                    None => "tierras lejanas".to_string(),
                }
            }
            Feat::Artifact => {
                let kind = pick(
                    &mut rng,
                    &[
                        "la espada",
                        "el báculo",
                        "el anillo",
                        "la corona",
                        "el escudo",
                        "el arco",
                    ],
                );
                format!("{kind} {}", names::word(&mut rng, age, 2, 3))
            }
            Feat::Grimoire => format!(
                "el grimorio prohibido de {}",
                names::word(&mut rng, age, 2, 3)
            ),
            Feat::ArtificialMind => self.ai_name(&mut rng),
            Feat::Implant => pick(
                &mut rng,
                &[
                    "un implante neural de combate",
                    "unos ojos ópticos de visión nocturna",
                    "una columna de fibra de carbono",
                    "un chip de memoria eterna",
                    "un corazón sintético",
                ],
            ),
            Feat::Heist => match target {
                Some(t) => format!("los datos secretos de {}", self.country(t).name),
                None => "datos secretos".to_string(),
            },
            Feat::Rescue => String::new(),
        }
    }

    fn ai_name(&self, rng: &mut Rng) -> String {
        names::word(rng, Age::Cyberpunk, 2, 2).to_uppercase()
    }

    /// El nombre propio de una calamidad, si lo tiene: el dragón, la IA.
    pub fn calamity_name(&self, event: &Event) -> Option<String> {
        let EventKind::Calamity { calamity, .. } = event.kind else {
            return None;
        };
        let mut rng = self.lore_rng(event);
        match calamity {
            Calamity::Dragon => Some(names::word(&mut rng, Age::Fantasy, 2, 3)),
            // La IA rebelde es la que creó alguien años atrás, si se sabe.
            Calamity::RogueAi => Some(
                event
                    .cause
                    .map(|c| &self.events[c.index()])
                    .filter(|c| matches!(c.kind, EventKind::Feat { .. }))
                    .map(|c| self.feat_detail(c))
                    .unwrap_or_else(|| self.ai_name(&mut rng)),
            ),
            _ => None,
        }
    }
}
