//! Genera un mundo y muestra un resumen en la terminal.
//! Uso: cargo run -p worldengine-core --example summary -- [semilla]

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
    println!(
        "Mundo «{}» (semilla {seed}), año {}",
        world.name, world.year
    );
    println!(
        "{} regiones, {} ríos, {} países, {} localidades, {} personajes, {} eventos",
        world.regions.len(),
        world.rivers.len(),
        world.countries.len(),
        world.localities.len(),
        world.people.len(),
        world.events.len()
    );
    for c in &world.countries {
        println!(
            "  {} ({}), capital {}, {} casillas",
            c.name,
            c.government.name(),
            world.locality(c.capital).name,
            world.territory_size(c.id)
        );
    }
    println!("Últimos eventos:");
    for e in world.events.iter().rev().take(5) {
        println!("  año {}: {}", e.year, world.describe(e));
    }
}
