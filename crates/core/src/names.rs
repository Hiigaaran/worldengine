//! Generador de nombres por sílabas. Provisional: más adelante cada cultura
//! tendrá su propio idioma.

use rand::seq::SliceRandom;
use rand::Rng as _;

use crate::rng::Rng;

const ONSETS: &[&str] = &[
    "b", "c", "d", "f", "g", "l", "m", "n", "p", "r", "s", "t", "v", "z", "br", "dr", "gr", "th",
    "kh", "al", "el", "or", "",
];
const VOWELS: &[&str] = &["a", "e", "i", "o", "u", "ae", "ia", "ou"];
const CODAS: &[&str] = &["", "", "n", "r", "s", "l", "th", "nd", "m"];

pub fn word(rng: &mut Rng, min_syllables: usize, max_syllables: usize) -> String {
    let count = rng.gen_range(min_syllables..=max_syllables);
    let mut out = String::new();
    for i in 0..count {
        out.push_str(ONSETS.choose(rng).unwrap());
        out.push_str(VOWELS.choose(rng).unwrap());
        if i + 1 == count {
            out.push_str(CODAS.choose(rng).unwrap());
        }
    }
    capitalize(&out)
}

pub fn person(rng: &mut Rng) -> String {
    format!("{} {}", word(rng, 1, 2), word(rng, 2, 3))
}

fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().chain(chars).collect(),
        None => String::new(),
    }
}
