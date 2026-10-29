//! Muestra las historias que se tejen en un mundo.
//! Uso: cargo run -p worldengine-core --example stories -- [semilla] [realista|fantasia|cyberpunk]

use worldengine_core::*;

fn main() {
    let seed = std::env::args()
        .nth(1)
        .map(|s| rng::seed_from_text(&s))
        .unwrap_or(1);
    let age = std::env::args()
        .nth(2)
        .and_then(|a| Age::from_text(&a))
        .unwrap_or_default();
    let world = generate(&WorldConfig {
        seed,
        age,
        ..Default::default()
    });
    for story in world.stories() {
        println!(
            "[{}] {} ({} – {}) · {} eventos",
            story.kind.name(age),
            story.title,
            story.start,
            story.end,
            story.events.len()
        );
        println!("  {}\n", story.summary);
    }
}
