//! Muestra las razas de un mundo y la descripción de algunos personajes, con
//! el prompt para generar su retrato con IA.
//!
//! `cargo run -p worldengine-core --example races -- <semilla> <edad> [personajes]`

use worldengine_core::*;

fn main() {
    let mut args = std::env::args().skip(1);
    let seed = args.next().map_or(7, |s| rng::seed_from_text(&s));
    let age = args
        .next()
        .and_then(|a| Age::from_text(&a))
        .unwrap_or(Age::Fantasy);
    let shown: usize = args.next().and_then(|n| n.parse().ok()).unwrap_or(4);
    let world = generate(&WorldConfig {
        seed,
        age,
        ..Default::default()
    });
    println!("# {} ({})\n", world.name, age.name());
    for race in &world.races {
        println!("## {}\n", race.title());
        println!("{}\n", race.describe());
        for custom in &race.customs {
            println!("- **{}**: {}", custom.aspect, custom.text);
        }
        let words: Vec<String> = race
            .language
            .lexicon
            .iter()
            .map(|p| format!("{} ({})", p.word, p.meaning))
            .collect();
        println!("- **Vocabulario**: {}", words.join(", "));
        let homes: Vec<&str> = world
            .homelands_of(race.id)
            .map(|c| c.name.as_str())
            .collect();
        println!("- **Países**: {}", homes.join(", "));
        println!("- **Personajes**: {}\n", world.members_of(race.id).count());
        println!("> {}\n", race.image_prompt(age));
    }
    println!("## Personajes\n");
    for person in world.people.iter().filter(|p| p.died.is_none()).take(shown) {
        println!("### {}\n", person.name);
        println!("{}\n", world.describe_person(person.id));
        println!("> {}\n", world.image_prompt(person.id));
    }
}
