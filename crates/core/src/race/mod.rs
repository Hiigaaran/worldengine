//! Razas y pueblos: quiénes habitan el mundo.
//!
//! Cada mundo genera, según su edad, unas pocas razas a partir del
//! [catálogo](catalog) de arquetipos: pueblos humanos de culturas distintas en
//! la edad realista; humanos, elfos, enanos, orcos, medianos y dracónidos en la
//! de fantasía; naturales, aumentados, clones, sintéticos y mutantes en la
//! cyberpunk. Cada raza tiene un cuerpo (estatura, complexión, piel, pelo,
//! ojos, rasgos, vestimenta), un carácter, una lengua propia con la que se
//! nombra a sus miembros, y costumbres.
//!
//! Cada país tiene una raza mayoritaria y cada personaje pertenece a una. De la
//! raza sale su [`Profile`]: su aspecto y carácter concretos, con los que se
//! escribe su descripción ([`World::describe_person`]) y un prompt en inglés para
//! generar su imagen con IA ([`World::image_prompt`]). Los rasgos visibles
//! son estructurados y bilingües ([`Trait`]) para poder alimentar otras
//! herramientas.

pub(crate) mod catalog;

use rand::seq::SliceRandom;
use rand::Rng as _;
use serde::{Deserialize, Serialize};

use crate::age::Age;
use crate::generate::distance2;
use crate::ids::*;
use crate::model::*;
use crate::names;
use crate::rng::{Rng, WorldRng};

use catalog::{Archetype, C, T};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RaceKind {
    // Realista: culturas humanas.
    RiverFolk,
    Steppe,
    Seafarers,
    Highlanders,
    Woodland,
    Desert,
    Northern,
    // Fantasía.
    Human,
    Elf,
    Dwarf,
    Orc,
    Halfling,
    Draconid,
    // Cyberpunk.
    Natural,
    Augmented,
    Clone,
    Synthetic,
    Mutant,
}

impl RaceKind {
    /// Las culturas humanas de la edad realista.
    pub fn is_culture(self) -> bool {
        matches!(
            self,
            RaceKind::RiverFolk
                | RaceKind::Steppe
                | RaceKind::Seafarers
                | RaceKind::Highlanders
                | RaceKind::Woodland
                | RaceKind::Desert
                | RaceKind::Northern
        )
    }

    /// Nombre de la raza en plural, o del tipo de cultura.
    pub fn name(self) -> &'static str {
        match self {
            RaceKind::RiverFolk => "Pueblo ribereño",
            RaceKind::Steppe => "Pueblo de la estepa",
            RaceKind::Seafarers => "Pueblo marinero",
            RaceKind::Highlanders => "Pueblo montañés",
            RaceKind::Woodland => "Pueblo del bosque",
            RaceKind::Desert => "Pueblo del desierto",
            RaceKind::Northern => "Pueblo del norte helado",
            RaceKind::Human => "Humanos",
            RaceKind::Elf => "Elfos",
            RaceKind::Dwarf => "Enanos",
            RaceKind::Orc => "Orcos",
            RaceKind::Halfling => "Medianos",
            RaceKind::Draconid => "Dracónidos",
            RaceKind::Natural => "Naturales",
            RaceKind::Augmented => "Aumentados",
            RaceKind::Clone => "Clones",
            RaceKind::Synthetic => "Sintéticos",
            RaceKind::Mutant => "Mutantes",
        }
    }

    /// Un miembro, según su género: «un elfo», «una elfa».
    fn member(self, gender: Gender) -> &'static str {
        let (m, f) = match self {
            k if k.is_culture() => ("un hombre", "una mujer"),
            RaceKind::Human => ("un humano", "una humana"),
            RaceKind::Elf => ("un elfo", "una elfa"),
            RaceKind::Dwarf => ("un enano", "una enana"),
            RaceKind::Orc => ("un orco", "una orca"),
            RaceKind::Halfling => ("un mediano", "una mediana"),
            RaceKind::Draconid => ("un dracónido", "una dracónida"),
            RaceKind::Natural => ("un humano sin implantes", "una humana sin implantes"),
            RaceKind::Augmented => ("un cíborg", "una cíborg"),
            RaceKind::Clone => ("un clon", "una clon"),
            RaceKind::Synthetic => ("un androide", "una androide"),
            _ => ("un mutante", "una mutante"),
        };
        match gender {
            Gender::Male => m,
            Gender::Female => f,
        }
    }

    /// En inglés, para los prompts.
    pub fn en(self) -> &'static str {
        match self {
            k if k.is_culture() => "human",
            RaceKind::Human => "human",
            RaceKind::Elf => "elf",
            RaceKind::Dwarf => "dwarf",
            RaceKind::Orc => "orc",
            RaceKind::Halfling => "halfling",
            RaceKind::Draconid => "dragonkin",
            RaceKind::Natural => "unaugmented human",
            RaceKind::Augmented => "cyborg",
            RaceKind::Clone => "clone",
            RaceKind::Synthetic => "android",
            _ => "mutant",
        }
    }

    fn en_plural(self) -> &'static str {
        match self {
            RaceKind::Elf => "elves",
            RaceKind::Dwarf => "dwarves",
            RaceKind::Natural => "unaugmented humans",
            RaceKind::Augmented => "cyborgs",
            RaceKind::Clone => "clones",
            RaceKind::Synthetic => "androids",
            RaceKind::Mutant => "mutants",
            RaceKind::Orc => "orcs",
            RaceKind::Halfling => "halflings",
            RaceKind::Draconid => "dragonkin",
            _ => "humans",
        }
    }

    /// La cultura de los pueblos realistas, en inglés.
    fn culture_en(self) -> Option<&'static str> {
        Some(match self {
            RaceKind::RiverFolk => "a river-valley farming culture",
            RaceKind::Steppe => "a nomadic horse-riding steppe culture",
            RaceKind::Seafarers => "a seafaring coastal culture",
            RaceKind::Highlanders => "a highland clan culture",
            RaceKind::Woodland => "a forest-dwelling hunter culture",
            RaceKind::Desert => "a desert caravan culture",
            RaceKind::Northern => "an arctic reindeer-herding culture",
            _ => return None,
        })
    }

    fn archetype(self) -> &'static Archetype {
        catalog::REALISTIC
            .iter()
            .chain(catalog::FANTASY)
            .chain(catalog::CYBERPUNK)
            .find(|a| a.kind == self)
            .expect("todo tipo de raza está en el catálogo")
    }

    /// Los que se adaptan a cualquier tierra: humanos y naturales.
    fn generalist(self) -> bool {
        self.archetype().biomes.is_empty() && !self.is_culture()
    }
}

/// Cómo se forman los nombres en una lengua.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum NameStyle {
    #[default]
    Plain,
    /// Con apóstrofos que parten el nombre: «Gro'mash».
    Apostrophe,
    /// Con alias callejero: Kai «Cromo» Vex.
    Alias,
    /// Con número de serie: Tessa KX-12.
    Serial,
}

/// Rasgos del cuerpo que cambian el dibujo del retrato.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Marks {
    pub pointed_ears: bool,
    pub tusks: bool,
    pub scales: bool,
    pub horns: bool,
    pub cyber_eye: bool,
    pub synthetic: bool,
    /// Hombres y mujeres llevan barba.
    pub beard_all: bool,
    pub beardless: bool,
    /// Sin pelo: una cresta o nada.
    pub hairless: bool,
}

/// Un rasgo visible, en español y en inglés, con su color si lo tiene.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Trait {
    pub es: String,
    pub en: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
}

impl Trait {
    fn of((es, en): T) -> Self {
        Trait {
            es: es.to_string(),
            en: en.to_string(),
            color: None,
        }
    }

    fn colored((es, en, hex): C) -> Self {
        Trait {
            es: es.to_string(),
            en: en.to_string(),
            color: Some(hex.to_string()),
        }
    }
}

/// Una palabra de la lengua de una raza y lo que significa.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Phrase {
    pub word: String,
    pub meaning: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Language {
    /// Cómo llaman a su lengua, en minúscula.
    pub name: String,
    pub onsets: Vec<String>,
    pub vowels: Vec<String>,
    pub codas: Vec<String>,
    pub style: NameStyle,
    pub greeting: Phrase,
    /// Unas cuantas palabras de uso común.
    pub lexicon: Vec<Phrase>,
    /// Cómo se ponen los nombres.
    pub naming: String,
    pub writing: String,
}

impl Language {
    /// Una palabra con los sonidos de esta lengua.
    pub fn word(&self, rng: &mut Rng, min_syllables: usize, max_syllables: usize) -> String {
        let count = rng.gen_range(min_syllables..=max_syllables);
        let mut out = String::new();
        // Las palabras de una o dos letras («E», «Ia») no sirven de nombre.
        for _ in 0..6 {
            out.clear();
            for i in 0..count {
                out.push_str(self.onsets.choose(rng).map_or("", |s| s));
                out.push_str(self.vowels.choose(rng).map_or("a", |s| s));
                if i + 1 == count {
                    out.push_str(self.codas.choose(rng).map_or("", |s| s));
                }
            }
            if out.chars().count() >= 3 {
                break;
            }
        }
        if self.style == NameStyle::Apostrophe && count >= 2 && rng.gen_bool(0.3) {
            let chars: Vec<char> = out.chars().collect();
            let mid = chars.len() / 2;
            if mid > 0 {
                out = chars[..mid]
                    .iter()
                    .chain(['\''].iter())
                    .chain(chars[mid..].iter())
                    .collect();
            }
        }
        names::capitalize(&out)
    }

    /// Nombre de pila, con alias si la lengua los usa.
    pub fn given(&self, rng: &mut Rng) -> String {
        let name = self.word(rng, 1, 2);
        if self.style == NameStyle::Alias && rng.gen_bool(0.4) {
            return format!("{name} «{}»", names::ALIASES.choose(rng).unwrap());
        }
        name
    }

    /// Apellido: el de la familia, el clan o la serie.
    pub fn family(&self, rng: &mut Rng) -> String {
        if self.style == NameStyle::Serial {
            let letters: String = (0..2)
                .map(|_| (b'A' + rng.gen_range(0..26u8)) as char)
                .collect();
            return format!("{letters}-{}", rng.gen_range(1..100));
        }
        self.word(rng, 2, 3)
    }

    /// Nombre completo; el apellido es siempre la última palabra.
    pub fn person(&self, rng: &mut Rng) -> String {
        format!("{} {}", self.given(rng), self.family(rng))
    }
}

/// Una costumbre: el aspecto de la vida al que se refiere y en qué consiste.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Custom {
    pub aspect: String,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Race {
    pub id: RaceId,
    pub kind: RaceKind,
    /// «Elfos», o «Pueblo Tamaru» en la edad realista.
    pub name: String,
    /// Cómo se llaman a sí mismos.
    pub endonym: String,
    pub homeland: Option<RegionId>,
    /// Una raza sin país propio al empezar la historia: nómadas, exiliados.
    pub stateless: bool,
    /// Estatura adulta habitual, en centímetros.
    pub height_cm: [u16; 2],
    /// Años que suele vivir.
    pub lifespan: u16,
    pub builds: Vec<Trait>,
    pub skins: Vec<Trait>,
    pub hair: Vec<Trait>,
    pub eyes: Vec<Trait>,
    pub features: Vec<Trait>,
    pub attire: Vec<Trait>,
    pub marks: Marks,
    /// Rasgos de carácter, como sustantivos: «orgullo», «paciencia».
    pub temperament: Vec<String>,
    /// Lo que más valoran: «la memoria de los siglos».
    pub values: Vec<String>,
    pub language: Language,
    pub customs: Vec<Custom>,
}

impl Race {
    /// «Elfos (los Thalorim)», «Pueblo Tamaru».
    pub fn title(&self) -> String {
        if self.kind.is_culture() {
            self.name.clone()
        } else {
            format!("{} (los {})", self.name, self.endonym)
        }
    }

    /// Una descripción de la raza en varios párrafos, en español.
    pub fn describe(&self) -> String {
        let mut out = String::new();
        let who = if self.kind.is_culture() {
            format!(
                "Los {} son un {}.",
                self.endonym,
                self.kind.name().to_lowercase()
            )
        } else {
            format!(
                "Los {} se llaman a sí mismos {}.",
                self.name.to_lowercase(),
                self.endonym
            )
        };
        out += &who;
        out += &format!(
            " Miden entre {} y {}, viven unos {} años y son de complexión {}.",
            meters(self.height_cm[0]),
            meters(self.height_cm[1]),
            self.lifespan,
            list(self.builds.iter().map(|t| t.es.as_str()), "o"),
        );
        let skin = list(self.skins.iter().map(|t| t.es.as_str()), "o");
        let hair = list(self.hair.iter().map(|t| t.es.as_str()), "o");
        let eyes = list(self.eyes.iter().map(|t| t.es.as_str()), "o");
        out += &format!(" Tienen la piel {skin}; {hair}; y ojos {eyes}.");
        out += &format!(
            " Muchos tienen {}. Visten {}.",
            list(self.features.iter().map(|t| t.es.as_str()), "o"),
            list(self.attire.iter().map(|t| t.es.as_str()), "o")
        );
        out += &format!(
            "\n\nSe les conoce por {}. Valoran {}.",
            list(self.temperament.iter().map(|s| format!("su {s}")), "y"),
            list(self.values.iter().map(|s| s.as_str()), "y")
        );
        out += &format!(
            "\n\nHablan {}. Saludan con «{}» («{}»). {} {}",
            self.language.name,
            self.language.greeting.word,
            self.language.greeting.meaning,
            self.language.naming,
            self.language.writing
        );
        out
    }

    /// Prompt en inglés para generar una imagen de la raza.
    pub fn image_prompt(&self, age: Age) -> String {
        let en = |v: &[Trait]| list(v.iter().map(|t| t.en.as_str()), "or");
        let who = match self.kind.culture_en() {
            Some(c) => format!("the {} people, {c}", self.endonym),
            None => format!("the {} ({})", self.endonym, self.kind.en_plural()),
        };
        format!(
            "Concept art character sheet of {who}. Men and women, {}–{} cm tall, {} build, {} skin, {} hair, {} eyes. Distinctive features: {}. Typical clothing: {}. {}.",
            self.height_cm[0],
            self.height_cm[1],
            en(&self.builds),
            en(&self.skins),
            en(&self.hair),
            en(&self.eyes),
            en(&self.features),
            en(&self.attire),
            style_en(age)
        )
    }
}

/// Género de un personaje.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Gender {
    Female,
    Male,
}

impl Gender {
    pub fn name(self) -> &'static str {
        match self {
            Gender::Female => "mujer",
            Gender::Male => "hombre",
        }
    }
}

/// El aspecto y el carácter concretos de un personaje, derivados de su raza.
/// Es estable: el mismo personaje tiene siempre el mismo perfil.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Profile {
    pub person: PersonId,
    pub name: String,
    pub race: RaceId,
    pub race_name: String,
    pub gender: Gender,
    /// Años cumplidos (al morir, si murió).
    pub age_years: i32,
    /// Edad que aparenta, en años humanos.
    pub apparent_age: i32,
    pub role: String,
    pub height_cm: u16,
    pub build: Trait,
    pub skin: Trait,
    pub hair: Trait,
    pub hair_style: Trait,
    pub eyes: Trait,
    pub beard: bool,
    pub features: Vec<Trait>,
    pub attire: Trait,
    pub gear: Trait,
    pub personality: Vec<String>,
    pub language: String,
    pub marks: Marks,
    /// Corte de pelo para el retrato vectorial.
    #[serde(skip)]
    pub(crate) cut: u8,
}

const HAIR_STYLES: [(&str, &str, u8); 6] = [
    ("corto", "short", 0),
    ("largo y suelto", "long and loose", 1),
    ("recogido en un moño", "tied up in a bun", 3),
    ("trenzado", "braided", 1),
    ("rapado a los lados", "shaved at the sides", 2),
    ("atado en una coleta", "tied in a ponytail", 0),
];

/// Marcas personales que puede tener cualquiera.
fn personal_marks(age: Age) -> &'static [T] {
    match age {
        Age::Cyberpunk => &[
            (
                "un código QR tatuado en la mano",
                "a QR code tattooed on the hand",
            ),
            ("una cicatriz de quemadura química", "a chemical burn scar"),
            (
                "unas ojeras de no dormir",
                "dark circles from sleepless nights",
            ),
            ("un diente de metal", "a metal tooth"),
        ],
        _ => &[
            ("una cicatriz en la ceja", "a scar across one eyebrow"),
            ("la cara llena de pecas", "a freckled face"),
            ("un lunar junto a los labios", "a beauty mark near the lips"),
            ("una mirada cansada", "tired eyes"),
        ],
    }
}

/// Lo que lleva encima cada oficio: (español, inglés).
fn gear(age: Age, role: Role) -> T {
    match (age, role) {
        (Age::Realistic, Role::Ruler) => (
            "una corona de oro y un manto de armiño",
            "a gold crown and an ermine mantle",
        ),
        (Age::Realistic, Role::Noble) => {
            ("joyas y un anillo de sello", "jewelry and a signet ring")
        }
        (Age::Realistic, Role::General) => (
            "una coraza y una espada al cinto",
            "a breastplate and a sword at the hip",
        ),
        (Age::Realistic, Role::Priest) => (
            "un hábito y un símbolo sagrado al cuello",
            "a religious habit and a holy symbol",
        ),
        (Age::Realistic, Role::Merchant) => (
            "una bolsa de monedas y un libro de cuentas",
            "a coin purse and a ledger",
        ),
        (Age::Realistic, Role::Scholar) => (
            "rollos de pergamino y unas lentes",
            "parchment scrolls and spectacles",
        ),
        (Age::Realistic, Role::Adventurer) => (
            "una capa de viaje, un arco y botas gastadas",
            "a travel cloak, a bow and worn boots",
        ),
        (Age::Fantasy, Role::Ruler) => (
            "una corona enjoyada y un cetro",
            "a jeweled crown and a scepter",
        ),
        (Age::Fantasy, Role::Noble) => (
            "un manto bordado y un broche con una gema",
            "an embroidered mantle and a gem brooch",
        ),
        (Age::Fantasy, Role::General) => (
            "una armadura de placas reluciente y un escudo sagrado",
            "shining plate armor and a holy shield",
        ),
        (Age::Fantasy, Role::Priest) => {
            ("una maza y un amuleto sagrado", "a mace and a holy amulet")
        }
        (Age::Fantasy, Role::Merchant) => (
            "una bolsa de monedas y baratijas mágicas",
            "a coin purse and magical trinkets",
        ),
        (Age::Fantasy, Role::Scholar) => (
            "un báculo rúnico y un grimorio",
            "a runic staff and a spellbook",
        ),
        (Age::Fantasy, Role::Adventurer) => (
            "una espada, un arco y una mochila de aventurero",
            "a sword, a bow and an adventurer's pack",
        ),
        (Age::Cyberpunk, Role::Ruler) => (
            "un traje de diseño y un holograma corporativo",
            "a designer suit and a corporate hologram",
        ),
        (Age::Cyberpunk, Role::Noble) => (
            "un traje caro y un auricular de comunicación",
            "an expensive suit and a comm earpiece",
        ),
        (Age::Cyberpunk, Role::General) => (
            "un exoesqueleto militar y un fusil inteligente",
            "a military exoskeleton and a smart rifle",
        ),
        (Age::Cyberpunk, Role::Priest) => (
            "collares de circuitos y velas LED",
            "circuit-board necklaces and LED candles",
        ),
        (Age::Cyberpunk, Role::Merchant) => (
            "un maletín blindado y anillos de oro",
            "an armored briefcase and gold rings",
        ),
        (Age::Cyberpunk, Role::Scholar) => (
            "un ciberdeck y cables conectados a la nuca",
            "a cyberdeck with cables plugged into the neck",
        ),
        (Age::Cyberpunk, Role::Adventurer) => (
            "una chaqueta blindada y una pistola pesada",
            "an armored jacket and a heavy pistol",
        ),
    }
}

/// El rasgo de carácter que da el oficio.
fn role_trait(role: Role) -> &'static str {
    match role {
        Role::Ruler => "autoridad",
        Role::Noble => "refinamiento",
        Role::General => "disciplina",
        Role::Priest => "devoción",
        Role::Merchant => "astucia",
        Role::Scholar => "curiosidad",
        Role::Adventurer => "audacia",
    }
}

fn role_en(age: Age, role: Role) -> &'static str {
    match (age, role) {
        (Age::Realistic, Role::Ruler) => "ruler",
        (Age::Fantasy, Role::Ruler) => "sovereign",
        (Age::Cyberpunk, Role::Ruler) => "megacorporation CEO",
        (Age::Cyberpunk, Role::Noble) => "corporate executive",
        (_, Role::Noble) => "noble",
        (Age::Fantasy, Role::General) => "paladin",
        (Age::Cyberpunk, Role::General) => "military commander",
        (_, Role::General) => "general",
        (Age::Fantasy, Role::Priest) => "cleric",
        (Age::Cyberpunk, Role::Priest) => "technoshaman",
        (_, Role::Priest) => "priest",
        (Age::Cyberpunk, Role::Merchant) => "smuggler",
        (_, Role::Merchant) => "merchant",
        (Age::Fantasy, Role::Scholar) => "wizard",
        (Age::Cyberpunk, Role::Scholar) => "hacker",
        (_, Role::Scholar) => "scholar",
        (Age::Cyberpunk, Role::Adventurer) => "mercenary",
        (_, Role::Adventurer) => "adventurer",
    }
}

fn biome_en(biome: Biome) -> &'static str {
    match biome {
        Biome::Ocean | Biome::Coast => "seaside",
        Biome::Lake => "lakeside",
        Biome::Desert => "desert",
        Biome::Shrubland => "scrubland",
        Biome::Grassland => "grassland",
        Biome::Savanna => "savanna",
        Biome::Forest => "forest",
        Biome::Jungle => "jungle",
        Biome::Swamp => "swamp",
        Biome::Taiga => "taiga",
        Biome::Tundra => "tundra",
        Biome::Hills => "hills",
        Biome::Mountain => "mountain",
        Biome::Glacier => "glacier",
    }
}

fn style_en(age: Age) -> &'static str {
    match age {
        Age::Realistic => {
            "Style: realistic historical oil painting, natural light, highly detailed"
        }
        Age::Fantasy => {
            "Style: high fantasy character illustration, dramatic lighting, highly detailed"
        }
        Age::Cyberpunk => {
            "Style: cyberpunk digital art, neon lighting, rainy night, highly detailed"
        }
    }
}

fn meters(cm: u16) -> String {
    format!("{},{:02} m", cm / 100, cm % 100)
}

/// «a, b y c».
fn list<S: AsRef<str>>(items: impl IntoIterator<Item = S>, last: &str) -> String {
    let items: Vec<String> = items.into_iter().map(|s| s.as_ref().to_string()).collect();
    match items.len() {
        0 => String::new(),
        1 => items[0].clone(),
        n => format!("{} {last} {}", items[..n - 1].join(", "), items[n - 1]),
    }
}

fn some<R: Clone>(rng: &mut Rng, pool: &[R], n: usize) -> Vec<R> {
    pool.choose_multiple(rng, n.min(pool.len()))
        .cloned()
        .collect()
}

/// Una ventana de `n` elementos de `pool` centrada en `at` (0 a 1).
fn window<R: Clone>(pool: &[R], at: f32, n: usize) -> Vec<R> {
    let n = n.min(pool.len());
    let center = (at.clamp(0.0, 1.0) * (pool.len() - 1) as f32).round() as usize;
    let start = center.saturating_sub(n / 2).min(pool.len() - n);
    pool[start..start + n].to_vec()
}

const LEXICON: [&str; 10] = [
    "agua", "fuego", "tierra", "cielo", "hogar", "sangre", "amigo", "enemigo", "muerte", "honor",
];

const ASPECTS: [&str; 7] = [
    "Ritos de paso",
    "Matrimonio",
    "Funerales",
    "Cocina",
    "Creencias",
    "Tabúes",
    "Escritura",
];

fn language(rng: &mut Rng, a: &Archetype) -> Language {
    let subset = |rng: &mut Rng, pool: &[&str], min: usize| -> Vec<String> {
        let n = (pool.len() * 3 / 4).max(min).min(pool.len());
        pool.choose_multiple(rng, n)
            .map(|s| s.to_string())
            .collect()
    };
    let mut lang = Language {
        name: String::new(),
        onsets: subset(rng, a.sounds.onsets, 5),
        vowels: subset(rng, a.sounds.vowels, 3),
        codas: subset(rng, a.sounds.codas, 2),
        style: a.sounds.style,
        greeting: Phrase {
            word: String::new(),
            meaning: String::new(),
        },
        lexicon: Vec::new(),
        naming: a.naming.choose(rng).unwrap().to_string(),
        writing: a.customs.writing.choose(rng).unwrap().to_string(),
    };
    let plain = Language {
        style: NameStyle::Plain,
        ..lang.clone()
    };
    lang.name = plain.word(rng, 2, 2).to_lowercase();
    lang.greeting = Phrase {
        word: format!(
            "{} {}",
            plain.word(rng, 1, 2),
            plain.word(rng, 1, 2).to_lowercase()
        ),
        meaning: a.greetings.choose(rng).unwrap().to_string(),
    };
    lang.lexicon = LEXICON
        .iter()
        .map(|m| Phrase {
            word: plain.word(rng, 1, 2).to_lowercase(),
            meaning: m.to_string(),
        })
        .collect();
    lang
}

fn build_race(
    rng: &mut Rng,
    id: RaceId,
    a: &Archetype,
    homeland: Option<RegionId>,
    climate: f32,
    stateless: bool,
) -> Race {
    let language = language(rng, a);
    let endonym = {
        let plain = Language {
            style: NameStyle::Plain,
            ..language.clone()
        };
        plain.word(rng, 2, 3)
    };
    let (skins, hair, eyes) = if a.kind.is_culture() {
        // La piel, el pelo y los ojos siguen al clima: más oscuros cerca del
        // ecuador, más claros cerca de los polos.
        let cold = 1.0 - climate + rng.gen_range(-0.12..0.12);
        (
            window(&catalog::REAL_SKINS, cold, 2),
            window(&catalog::REAL_HAIR, cold, 3),
            window(&catalog::REAL_EYES, cold, 2),
        )
    } else {
        (
            some(rng, a.skins, 3),
            some(rng, a.hair, 3),
            some(rng, a.eyes, 3),
        )
    };
    let customs: Vec<Custom> = [
        a.customs.rites,
        a.customs.marriage,
        a.customs.funerals,
        a.customs.food,
        a.customs.beliefs,
        a.customs.taboos,
    ]
    .iter()
    .zip(ASPECTS)
    .map(|(pool, aspect)| Custom {
        aspect: aspect.to_string(),
        text: pool.choose(rng).unwrap().to_string(),
    })
    .chain(std::iter::once(Custom {
        aspect: ASPECTS[6].to_string(),
        text: language.writing.clone(),
    }))
    .collect();
    let name = if a.kind.is_culture() {
        format!("Pueblo {endonym}")
    } else {
        a.kind.name().to_string()
    };
    Race {
        id,
        kind: a.kind,
        name,
        endonym,
        homeland,
        stateless,
        height_cm: [a.height.0, a.height.1],
        lifespan: a.lifespan,
        builds: some(rng, a.builds, 2).into_iter().map(Trait::of).collect(),
        skins: skins.into_iter().map(Trait::colored).collect(),
        hair: hair.into_iter().map(Trait::colored).collect(),
        eyes: eyes.into_iter().map(Trait::of).collect(),
        features: some(rng, a.features, 3)
            .into_iter()
            .map(Trait::of)
            .collect(),
        attire: some(rng, a.attire, 2).into_iter().map(Trait::of).collect(),
        marks: a.marks,
        temperament: some(rng, a.temperament, 3)
            .into_iter()
            .map(String::from)
            .collect(),
        values: some(rng, a.values, 2)
            .into_iter()
            .map(String::from)
            .collect(),
        language,
        customs,
    }
}

fn catalog_of(age: Age) -> &'static [Archetype] {
    match age {
        Age::Realistic => catalog::REALISTIC,
        Age::Fantasy => catalog::FANTASY,
        Age::Cyberpunk => catalog::CYBERPUNK,
    }
}

/// Qué tan bien encaja un arquetipo en un lugar.
fn fitness(world: &World, a: &Archetype, pos: Pos) -> f32 {
    let map = &world.map;
    let tile = map.tile(pos).unwrap();
    let near =
        |f: &dyn Fn(&Tile) -> bool| f(tile) || map.neighbors8(pos).any(|n| f(map.tile(n).unwrap()));
    let mut score = if a.biomes.contains(&tile.biome) {
        2.0
    } else if a.biomes.is_empty() {
        1.2
    } else {
        0.0
    };
    match a.kind {
        RaceKind::Seafarers if near(&|t| t.biome.is_sea()) => score += 2.5,
        RaceKind::RiverFolk if near(&|t| t.river.is_some()) => score += 2.0,
        RaceKind::Northern if tile.temperature < 0.3 => score += 1.5,
        RaceKind::Desert if tile.rainfall < 0.25 => score += 1.0,
        _ => {}
    }
    score
}

/// La región de tierra firme de un lugar. Algunos islotes quedan dentro de
/// regiones marinas: entonces, la región de tierra más cercana.
fn land_region(world: &World, pos: Pos) -> Option<RegionId> {
    let here = world.map.tile(pos)?.region;
    match here {
        Some(r) if !world.region(r).biome.is_water() => Some(r),
        _ => world
            .regions
            .iter()
            .filter(|r| !r.biome.is_water())
            .min_by_key(|r| distance2(r.center, pos))
            .map(|r| r.id),
    }
}

/// Genera las razas del mundo y da a cada país su raza mayoritaria. Se llama
/// al fundar el mundo, antes de crear a los personajes.
pub(crate) fn populate(world: &mut World, rng: &mut Rng) {
    let age = world.config.age;
    let pool = catalog_of(age);
    let capitals: Vec<Pos> = world
        .countries
        .iter()
        .map(|c| world.locality(c.capital).pos)
        .collect();
    let anchored = capitals.len().min(4);
    let mut chosen: Vec<(&Archetype, Pos)> = Vec::new();
    let mut order: Vec<usize> = (0..capitals.len()).collect();
    order.shuffle(rng);
    for &c in order.iter().take(anchored) {
        let pos = capitals[c];
        let best = pool
            .iter()
            .filter(|a| age == Age::Realistic || chosen.iter().all(|(b, _)| b.kind != a.kind))
            .map(|a| {
                let repeat = chosen.iter().filter(|(b, _)| b.kind == a.kind).count() as f32;
                (
                    fitness(world, a, pos) - 1.5 * repeat + rng.gen_range(0.0..1.5),
                    a,
                )
            })
            .max_by(|x, y| x.0.total_cmp(&y.0))
            .map(|(_, a)| a)
            .unwrap();
        chosen.push((best, pos));
    }
    // En fantasía y cyberpunk siempre hay humanos con país propio.
    if age != Age::Realistic && !chosen.iter().any(|(a, _)| a.kind.generalist()) {
        let generalist = pool.iter().find(|a| a.kind.generalist()).unwrap();
        match chosen.last_mut() {
            Some(last) => last.0 = generalist,
            None => chosen.push((generalist, Pos { x: 0, y: 0 })),
        }
    }
    let countries_of: Vec<usize> = order.iter().take(anchored).copied().collect();

    // Una raza más, sin país: vive en tierras libres o dispersa.
    let free = world
        .localities
        .iter()
        .find(|l| l.country.is_none())
        .map(|l| l.pos)
        .or_else(|| capitals.first().copied())
        .unwrap_or(Pos { x: 0, y: 0 });
    let wanderer = pool
        .iter()
        .filter(|a| age == Age::Realistic || chosen.iter().all(|(b, _)| b.kind != a.kind))
        .map(|a| {
            let repeat = chosen.iter().filter(|(b, _)| b.kind == a.kind).count() as f32;
            (
                fitness(world, a, free) - 1.5 * repeat + rng.gen_range(0.0..1.5),
                a,
            )
        })
        .max_by(|x, y| x.0.total_cmp(&y.0))
        .map(|(_, a)| a);

    let mut races = Vec::new();
    let all = chosen
        .iter()
        .map(|&(a, pos)| (a, pos, false))
        .chain(wanderer.map(|a| (a, free, true)));
    for (i, (a, pos, stateless)) in all.enumerate() {
        let tile = world.map.tile(pos).unwrap();
        races.push(build_race(
            rng,
            RaceId(i as u32),
            a,
            land_region(world, pos),
            tile.temperature,
            stateless,
        ));
    }
    world.races = races;

    // Cada país: la raza que fundó allí, o la de la capital ancla más cercana.
    let total = world.races.len();
    for (c, &here) in capitals.iter().enumerate() {
        let race = match countries_of.iter().position(|&x| x == c) {
            Some(i) => i,
            None => {
                let nearest = (0..chosen.len())
                    .min_by_key(|&i| distance2(chosen[i].1, here))
                    .unwrap_or(0);
                if rng.gen_bool(0.75) {
                    nearest
                } else {
                    rng.gen_range(0..total)
                }
            }
        };
        world.countries[c].people = Some(RaceId(race as u32));
    }
}

/// La raza de alguien que entra en la historia: la de su progenitor, o casi
/// siempre la mayoritaria de su país; a veces, una minoría.
pub(crate) fn race_for(
    world: &World,
    id: PersonId,
    country: Option<CountryId>,
    parent: Option<PersonId>,
) -> Option<RaceId> {
    if world.races.is_empty() {
        return None;
    }
    if let Some(race) = parent.and_then(|p| world.person(p).race) {
        return Some(race);
    }
    let mut rng = WorldRng::new(world.config.seed).stream(&format!("race-{}", id.0));
    let majority = match country {
        Some(c) => world.country(c).people,
        None => world.races.iter().find(|r| r.stateless).map(|r| r.id),
    };
    match majority {
        Some(r) if rng.gen_bool(0.8) => Some(r),
        _ => Some(RaceId(rng.gen_range(0..world.races.len()) as u32)),
    }
}

/// La raza de un país que se independiza: la que más abunda entre los
/// personajes de la localidad rebelde, o la del país del que se separa.
pub(crate) fn rebel_people(world: &World, locality: LocalityId, from: CountryId) -> Option<RaceId> {
    let mut counts = vec![0u32; world.races.len()];
    for p in world.residents(locality).filter(|p| p.died.is_none()) {
        if let Some(r) = p.race {
            counts[r.index()] += 1;
        }
    }
    let best = (0..counts.len()).max_by_key(|&i| (counts[i], std::cmp::Reverse(i)));
    match best {
        Some(i) if counts[i] > 0 => Some(RaceId(i as u32)),
        _ => world.country(from).people,
    }
}

impl World {
    pub fn race(&self, id: RaceId) -> &Race {
        &self.races[id.index()]
    }

    /// Personajes de una raza.
    pub fn members_of(&self, race: RaceId) -> impl Iterator<Item = &Person> {
        self.people.iter().filter(move |p| p.race == Some(race))
    }

    /// Países donde una raza es mayoría.
    pub fn homelands_of(&self, race: RaceId) -> impl Iterator<Item = &Country> {
        self.countries
            .iter()
            .filter(move |c| c.people == Some(race))
    }

    /// El aspecto y el carácter de un personaje, si pertenece a una raza.
    pub fn profile(&self, id: PersonId) -> Option<Profile> {
        let person = self.person(id);
        let race = self.race(person.race?);
        let age = self.config.age;
        let mut rng = WorldRng::new(self.config.seed).stream(&format!("profile-{}", id.0));
        let gender = if rng.gen_bool(0.5) {
            Gender::Female
        } else {
            Gender::Male
        };
        let age_years = person.died.unwrap_or(self.year) - person.born;
        let apparent_age = (age_years as f32 * 80.0 / race.lifespan as f32).round() as i32;
        let apparent_age = apparent_age.max(age_years.min(20));
        let [lo, hi] = race.height_cm;
        let mut height_cm = rng.gen_range(lo..=hi);
        if gender == Gender::Female {
            height_cm = (height_cm as f32 * 0.94) as u16;
        }
        let pick = |rng: &mut Rng, v: &[Trait]| v.choose(rng).cloned().unwrap();
        let build = pick(&mut rng, &race.builds);
        let skin = pick(&mut rng, &race.skins);
        let mut hair = pick(&mut rng, &race.hair);
        let eyes = pick(&mut rng, &race.eyes);
        let marks = race.marks;
        if apparent_age >= 55 && !marks.synthetic && !marks.hairless {
            hair = Trait {
                es: if hair.es.starts_with("cabello") {
                    "cabello canoso".to_string()
                } else {
                    "pelo canoso".to_string()
                },
                en: "grey".to_string(),
                color: Some("#b8b8b8".to_string()),
            };
        }
        let styles: &[usize] = match gender {
            Gender::Female => &[1, 2, 3, 5, 1],
            Gender::Male => &[0, 4, 0, 3, 5],
        };
        let mut style = HAIR_STYLES[*styles.choose(&mut rng).unwrap()];
        if age == Age::Cyberpunk && person.role == Role::Adventurer {
            style = ("en cresta", "in a mohawk", 2);
        }
        let wizard = age == Age::Fantasy && person.role == Role::Scholar;
        let beard = !marks.beardless
            && (marks.beard_all || (gender == Gender::Male && (wizard || rng.gen_bool(0.4))));
        let mut features: Vec<Trait> = race
            .features
            .choose_multiple(&mut rng, 2)
            .cloned()
            .collect();
        if rng.gen_bool(0.4) && !marks.synthetic {
            features.push(Trait::of(*personal_marks(age).choose(&mut rng).unwrap()));
        }
        let attire = pick(&mut rng, &race.attire);
        let gear = Trait::of(gear(age, person.role));
        let mut personality: Vec<String> = race
            .temperament
            .choose_multiple(&mut rng, 2)
            .cloned()
            .collect();
        let own = role_trait(person.role).to_string();
        if !personality.contains(&own) {
            personality.push(own);
        }
        Some(Profile {
            person: id,
            name: person.name.clone(),
            race: race.id,
            race_name: race.name.clone(),
            gender,
            age_years,
            apparent_age,
            role: person.role.name_for(age, gender).to_string(),
            height_cm,
            build,
            skin,
            hair,
            hair_style: Trait {
                es: style.0.to_string(),
                en: style.1.to_string(),
                color: None,
            },
            eyes,
            beard,
            features,
            attire,
            gear,
            personality,
            language: race.language.name.clone(),
            marks,
            cut: style.2,
        })
    }

    /// El oficio de un personaje con el género que le corresponde.
    pub fn role_name(&self, id: PersonId) -> &'static str {
        let role = self.person(id).role;
        match self.profile(id) {
            Some(p) => role.name_for(self.config.age, p.gender),
            None => role.name(self.config.age),
        }
    }

    /// Descripción del personaje en español: quién es, su aspecto, su ropa,
    /// su carácter y su lengua, según su raza.
    pub fn describe_person(&self, id: PersonId) -> String {
        let Some(p) = self.profile(id) else {
            return String::new();
        };
        let person = self.person(id);
        let race = self.race(p.race);
        let alive = person.died.is_none();
        let v = |now: &'static str, then: &'static str| if alive { now } else { then };
        let who = if race.kind.is_culture() {
            format!("{} del pueblo {}", race.kind.member(p.gender), race.endonym)
        } else {
            format!("{} de los {}", race.kind.member(p.gender), race.endonym)
        };
        let place = match person.country {
            Some(c) => format!(" en {}", self.country(c).name),
            None => " sin patria".to_string(),
        };
        let mut out = format!(
            "{} {} {who}, {}{place}",
            p.name,
            v("es", "fue"),
            p.role.to_lowercase()
        );
        out += &match (person.died, person.cause_of_death) {
            (Some(_), Some(cause)) => format!(
                ", que murió {} a los {} años.",
                cause.name(self.config.age),
                p.age_years
            ),
            _ => format!(", de {} años.", p.age_years),
        };
        if p.apparent_age + 15 < p.age_years {
            out += &format!(" {} unos {}.", v("Aparenta", "Aparentaba"), p.apparent_age);
        }
        let hair = if p.marks.hairless {
            p.hair.es.clone()
        } else {
            format!("{} {}", p.hair.es, p.hair_style.es)
        };
        out += &format!(
            " {} {} y {} de complexión {}; {} la piel {}, {hair} y los ojos {}.",
            v("Mide", "Medía"),
            meters(p.height_cm),
            v("es", "era"),
            p.build.es,
            v("tiene", "tenía"),
            p.skin.es,
            p.eyes.es
        );
        if p.beard && !p.features.iter().any(|f| f.es.contains("barba")) {
            out += &format!(" {} barba.", v("Lleva", "Llevaba"));
        }
        if !p.features.is_empty() {
            out += &format!(
                " Se le {} por {}.",
                v("reconoce", "reconocía"),
                list(p.features.iter().map(|f| f.es.as_str()), "y")
            );
        }
        out += &format!(
            " {} {} y {} {}.",
            v("Viste", "Vestía"),
            p.attire.es,
            v("lleva", "llevaba"),
            p.gear.es
        );
        out += &format!(
            " {} por {}.",
            v("Destaca", "Destacaba"),
            list(p.personality.iter().map(|s| format!("su {s}")), "y")
        );
        if let Some(value) = race.values.first() {
            out += &format!(" Como los suyos, {} {value}.", v("valora", "valoraba"));
        }
        out += &format!(
            " {} {} y {} con «{}» («{}»).",
            v("Habla", "Hablaba"),
            race.language.name,
            v("saluda", "saludaba"),
            race.language.greeting.word,
            race.language.greeting.meaning
        );
        out
    }

    /// Todo lo que hace falta para generar la imagen de un personaje, en JSON:
    /// su perfil estructurado, su descripción y el prompt.
    pub fn image_json(&self, id: PersonId) -> String {
        #[derive(Serialize)]
        struct Request<'a> {
            world: &'a str,
            age: Age,
            year: Year,
            profile: Profile,
            description: String,
            prompt: String,
        }
        let Some(profile) = self.profile(id) else {
            return String::new();
        };
        serde_json::to_string_pretty(&Request {
            world: &self.name,
            age: self.config.age,
            year: self.year,
            profile,
            description: self.describe_person(id),
            prompt: self.image_prompt(id),
        })
        .unwrap_or_default()
    }

    /// Prompt en inglés para generar el retrato del personaje con IA.
    pub fn image_prompt(&self, id: PersonId) -> String {
        let Some(p) = self.profile(id) else {
            return String::new();
        };
        let age = self.config.age;
        let person = self.person(id);
        let race = self.race(p.race);
        let woman = p.gender == Gender::Female;
        let who = match (race.kind, race.kind.culture_en()) {
            (_, Some(culture)) => format!(
                "a {}-year-old {} of the {} people ({culture})",
                p.age_years,
                if woman { "woman" } else { "man" },
                race.endonym
            ),
            (RaceKind::Synthetic, _) => format!(
                "a {}-presenting android",
                if woman { "female" } else { "male" }
            ),
            (kind, _) => format!(
                "a {}-year-old {} {}",
                p.age_years,
                kind.en(),
                if woman { "woman" } else { "man" }
            ),
        };
        let looks = if p.apparent_age + 15 < p.age_years || race.kind == RaceKind::Synthetic {
            format!(" who looks about {}", p.apparent_age)
        } else {
            String::new()
        };
        let hair = if p.marks.hairless {
            format!("{} instead of hair", p.hair.en)
        } else {
            format!("{} hair, {}", p.hair.en, p.hair_style.en)
        };
        let beard = if p.beard { ", a beard" } else { "" };
        let features = list(p.features.iter().map(|f| f.en.as_str()), "and");
        let home = self.locality(person.home);
        let setting = match age {
            Age::Cyberpunk if home.kind == LocalityKind::Village => {
                "the outskirts of a fortified enclave in a polluted wasteland".to_string()
            }
            Age::Cyberpunk => "neon-lit megacity streets".to_string(),
            _ => {
                let biome = self
                    .map
                    .tile(home.pos)
                    .map_or(Biome::Grassland, |t| t.biome);
                format!("a {} landscape", biome_en(biome))
            }
        };
        format!(
            "Portrait of {}, {who}{looks}, a {}. {} cm tall, {} build, {} skin, {hair}, {} eyes{beard}. Distinctive features: {features}. Wearing {}, with {}. Background: {setting}. {}.",
            p.name,
            role_en(age, person.role),
            p.height_cm,
            p.build.en,
            p.skin.en,
            p.eyes.en,
            p.attire.en,
            p.gear.en,
            style_en(age)
        )
    }
}
