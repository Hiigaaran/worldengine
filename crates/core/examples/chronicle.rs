//! Genera un mundo y escribe su crónica en Markdown por la salida estándar.
//! Uso: cargo run -p worldengine-core --example chronicle -- [semilla] > cronica.md

use worldengine_core::*;

fn main() {
    let seed = std::env::args()
        .nth(1)
        .map(|s| rng::seed_from_text(&s))
        .unwrap_or(1);
    let world = generate(&WorldConfig {
        seed,
        ..Default::default()
    });
    print!("{}", world.chronicle_markdown());
}
