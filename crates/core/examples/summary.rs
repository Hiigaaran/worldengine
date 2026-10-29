//! Genera un mundo y muestra un resumen en la terminal.
//! Uso: cargo run -p worldengine-core --example summary -- [semilla] [realista|fantasia|cyberpunk]

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
    let living = world.countries.iter().filter(|c| c.exists()).count();
    let dead = world.people.iter().filter(|p| p.died.is_some()).count();
    println!(
        "{living} países en pie de {} · {} guerras · {dead} personajes muertos",
        world.countries.len(),
        world.wars.len()
    );
    for c in world.countries.iter().filter(|c| c.exists()) {
        println!(
            "  {} ({}), capital {}, {} casillas, {} gobernantes",
            c.name,
            c.government.name(age),
            world.locality(c.capital).name,
            world.territory_size(c.id),
            world.rulers_of(c.id).len()
        );
    }
    println!("Últimos eventos importantes:");
    for e in world
        .events
        .iter()
        .rev()
        .filter(|e| world.is_major(e))
        .take(15)
    {
        println!("  año {}: {}", e.year, world.describe(e));
    }
}
