//! Generador de nombres por sílabas, con un sonido propio para cada edad:
//! nombres de aire europeo en la realista, élficos y enanos en la de fantasía,
//! y marcas, alias y sectores en la cyberpunk.

use rand::seq::SliceRandom;
use rand::Rng as _;

use crate::age::Age;
use crate::model::Government;
use crate::rng::Rng;

struct Phonology {
    onsets: &'static [&'static str],
    vowels: &'static [&'static str],
    codas: &'static [&'static str],
}

const REALISTIC: Phonology = Phonology {
    onsets: &[
        "b", "c", "d", "f", "g", "l", "m", "n", "p", "r", "s", "t", "v", "br", "cl", "gr", "pr",
        "tr", "", "",
    ],
    vowels: &["a", "e", "i", "o", "u", "a", "e", "o", "ia", "ue"],
    codas: &["", "", "", "n", "r", "s", "l"],
};

const FANTASY: Phonology = Phonology {
    onsets: &[
        "b", "d", "g", "l", "m", "n", "r", "s", "th", "v", "z", "dr", "gr", "kh", "thr", "gl",
        "el", "al", "or", "ar", "y", "",
    ],
    vowels: &[
        "a", "e", "i", "o", "u", "a", "e", "i", "ae", "ia", "ei", "y", "ë",
    ],
    codas: &["", "", "n", "r", "th", "l", "nd", "m", "rn", "dh", "s"],
};

const CYBERPUNK: Phonology = Phonology {
    onsets: &[
        "k", "z", "x", "v", "r", "n", "j", "d", "s", "t", "m", "ky", "ry", "sh", "ts", "kr", "dr",
        "",
    ],
    vowels: &["a", "e", "i", "o", "u", "y", "ai", "ei", "o"],
    codas: &["", "", "x", "k", "n", "s", "r", "z", "ko"],
};

fn phonology(age: Age) -> &'static Phonology {
    match age {
        Age::Realistic => &REALISTIC,
        Age::Fantasy => &FANTASY,
        Age::Cyberpunk => &CYBERPUNK,
    }
}

pub fn word(rng: &mut Rng, age: Age, min_syllables: usize, max_syllables: usize) -> String {
    let p = phonology(age);
    let count = rng.gen_range(min_syllables..=max_syllables);
    let mut out = String::new();
    for i in 0..count {
        out.push_str(p.onsets.choose(rng).unwrap());
        out.push_str(p.vowels.choose(rng).unwrap());
        if i + 1 == count {
            out.push_str(p.codas.choose(rng).unwrap());
        }
    }
    // En fantasía, a veces un apóstrofo parte el nombre: «Kal'dor».
    if age == Age::Fantasy && count >= 2 && rng.gen_bool(0.08) {
        let mid = out
            .char_indices()
            .map(|(i, _)| i)
            .nth(out.chars().count() / 2)
            .unwrap_or(0);
        if mid > 0 {
            out.insert(mid, '\'');
        }
    }
    capitalize(&out)
}

/// Nombre de pila. En cyberpunk muchos llevan un alias: Kai «Cromo».
pub fn given(rng: &mut Rng, age: Age) -> String {
    let name = word(rng, age, 1, 2);
    if age == Age::Cyberpunk && rng.gen_bool(0.4) {
        const ALIASES: &[&str] = &[
            "Cromo",
            "Neón",
            "Glitch",
            "Cero",
            "Chispa",
            "Vudú",
            "Kilo",
            "Ceniza",
            "Fantasma",
            "Ruido",
            "Byte",
            "Lince",
            "Óxido",
            "Estática",
            "Navaja",
            "Píxel",
        ];
        return format!("{name} «{}»", ALIASES.choose(rng).unwrap());
    }
    name
}

/// Nombre completo; el apellido es siempre la última palabra.
pub fn person(rng: &mut Rng, age: Age) -> String {
    format!("{} {}", given(rng, age), word(rng, age, 2, 3))
}

pub fn world(rng: &mut Rng, age: Age) -> String {
    word(rng, age, 2, 3)
}

/// Nombre de una localidad.
pub fn place(rng: &mut Rng, age: Age) -> String {
    let w = word(rng, age, 2, 3);
    match age {
        Age::Realistic => match rng.gen_range(0..10) {
            0 => format!("San {w}"),
            1 => format!("Villa{}", w.to_lowercase()),
            2 => format!("Puerto {w}"),
            _ => w,
        },
        Age::Fantasy => {
            if rng.gen_bool(0.25) {
                const FIRST: &[&str] = &[
                    "Roca", "Torre", "Valle", "Fuente", "Vado", "Peña", "Cumbre", "Hondo",
                ];
                const SECOND: &[&str] = &[
                    "umbría", "dragón", "niebla", "plata", "cuervo", "luna", "runa", "espino",
                    "sombra", "lobo",
                ];
                format!(
                    "{}{}",
                    FIRST.choose(rng).unwrap(),
                    SECOND.choose(rng).unwrap()
                )
            } else {
                w
            }
        }
        Age::Cyberpunk => match rng.gen_range(0..10) {
            0 | 1 => format!("Neo {w}"),
            2 => format!("Distrito {w}"),
            3 => format!("Sector {}", rng.gen_range(2..99)),
            4 => format!("{w}-{}", rng.gen_range(1..40)),
            5 => format!("Arcología {w}"),
            _ => w,
        },
    }
}

/// Nombre de un país, que en cyberpunk depende de quién manda.
pub fn country(rng: &mut Rng, age: Age, government: Government) -> String {
    let w = word(rng, age, 2, 3);
    match age {
        Age::Realistic => {
            if rng.gen_bool(0.4) && w.ends_with(|c: char| !"aeiou".contains(c)) {
                format!("{w}ia")
            } else {
                w
            }
        }
        Age::Fantasy => {
            if rng.gen_bool(0.35) {
                let end = ["dor", "wyn", "heim", "mar", "ion", "ath"]
                    .choose(rng)
                    .unwrap();
                format!(
                    "{}{end}",
                    w.trim_end_matches(|c: char| "aeiouy".contains(c))
                )
            } else {
                w
            }
        }
        Age::Cyberpunk => match government {
            Government::Monarchy => {
                let suffix = [
                    "Corp",
                    "Dynamics",
                    "Industrias",
                    "Sistemas",
                    "Biotech",
                    "Holdings",
                    "Zaibatsu",
                ]
                .choose(rng)
                .unwrap();
                format!("{w} {suffix}")
            }
            Government::Republic => format!("Zona Libre {w}"),
            Government::Theocracy => format!("Red Sagrada de {w}"),
            Government::TribalCouncil => format!("Sindicato {w}"),
        },
    }
}

pub fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().chain(chars).collect(),
        None => String::new(),
    }
}
