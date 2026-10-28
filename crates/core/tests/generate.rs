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
    let water = w.map.tiles.iter().filter(|t| t.biome.is_sea()).count() as f32;
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
        for l in &w.localities {
            assert!(
                !w.map.tile(l.pos).unwrap().biome.is_water(),
                "localidad en el agua"
            );
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

#[test]
fn rivers_run_downhill_and_connect() {
    for seed in 0..5 {
        let w = world(seed);
        assert!(!w.rivers.is_empty(), "seed {seed} sin ríos");
        for river in &w.rivers {
            let mut previous_flow = 0.0;
            for pair in river.path.windows(2) {
                assert!(pair[0].x.abs_diff(pair[1].x) <= 1 && pair[0].y.abs_diff(pair[1].y) <= 1);
            }
            for &p in &river.path {
                let tile = w.map.tile(p).unwrap();
                assert_eq!(tile.river, Some(river.id));
                assert!(!tile.biome.is_water());
                assert!(tile.flow >= previous_flow, "el caudal crece aguas abajo");
                previous_flow = tile.flow;
            }
            let last = *river.path.last().unwrap();
            assert!(last.x.abs_diff(river.mouth.x) <= 1 && last.y.abs_diff(river.mouth.y) <= 1);
            if let Some(main) = river.flows_into {
                assert!(
                    w.river(main).path.contains(&river.mouth),
                    "afluente sin confluencia"
                );
            }
        }
    }
}

#[test]
fn geography_is_varied() {
    let worlds: Vec<World> = (0..10).map(world).collect();
    assert!(
        worlds
            .iter()
            .any(|w| w.map.tiles.iter().any(|t| t.biome == Biome::Lake)),
        "ningún mundo tiene lagos"
    );
    for w in &worlds {
        let mut biomes: Vec<Biome> = w.map.tiles.iter().map(|t| t.biome).collect();
        biomes.sort_by_key(|b| *b as u8);
        biomes.dedup();
        assert!(biomes.len() >= 8, "solo {} biomas", biomes.len());
    }
}
