//! Muestra las historias que se tejen en un mundo.
//! Uso: cargo run -p worldengine-core --example stories -- [semilla]

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
    for story in world.stories() {
        println!(
            "[{}] {} ({} – {}) · {} eventos",
            story.kind.name(),
            story.title,
            story.start,
            story.end,
            story.events.len()
        );
        println!("  {}\n", story.summary);
    }
}
