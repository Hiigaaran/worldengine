use worldengine_core::*;

fn world(seed: u64) -> World {
    generate(&WorldConfig {
        seed,
        ..Default::default()
    })
}

#[test]
fn same_seed_same_world() {
    assert_eq!(world(7), world(7));
}

#[test]
fn different_seeds_differ() {
    assert_ne!(world(1).map, world(2).map);
}

#[test]
fn water_fraction_follows_config() {
    let w = world(3);
    let water = w.map.tiles.iter().filter(|t| t.biome.is_water()).count() as f32;
    let fraction = water / w.map.tiles.len() as f32;
    assert!(
        (fraction - w.config.sea_level).abs() < 0.02,
        "fraction = {fraction}"
    );
}

#[test]
fn references_are_consistent() {
    for seed in 0..10 {
        let w = world(seed);
        assert!(!w.countries.is_empty(), "seed {seed} sin países");
        for tile in &w.map.tiles {
            let region = tile.region.expect("toda casilla tiene región");
            assert!(region.index() < w.regions.len());
            if let Some(owner) = tile.owner {
                assert!(owner.index() < w.countries.len());
                assert!(!tile.biome.is_water(), "el agua no tiene dueño");
            }
        }
        for c in &w.countries {
            let capital = w.locality(c.capital);
            assert_eq!(capital.country, Some(c.id));
            assert_eq!(w.map.tile(capital.pos).unwrap().owner, Some(c.id));
            let ruler = w.person(c.ruler.unwrap());
            assert_eq!(ruler.role, Role::Ruler);
        }
        for p in &w.people {
            assert!(p.alive_in(w.year));
            assert_eq!(w.locality(p.home).country, p.country);
        }
        for r in &w.regions {
            assert_eq!(w.map.tile(r.center).unwrap().region, Some(r.id));
        }
        assert!(w.events.windows(2).all(|e| e[0].year <= e[1].year));
        assert!(w.events.iter().all(|e| e.year <= w.year));
        for (i, e) in w.events.iter().enumerate() {
            assert_eq!(e.id.index(), i);
            assert!(!w.describe(e).is_empty());
        }
    }
}

#[test]
fn json_round_trip() {
    let w = world(11);
    let back = World::from_json(&w.to_json().unwrap()).unwrap();
    assert_eq!(w, back);
}

#[test]
fn text_seeds_are_stable() {
    assert_eq!(rng::seed_from_text("Arda"), rng::seed_from_text(" Arda "));
    assert_eq!(rng::seed_from_text("42"), 42);
}
