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
        for c in w.countries.iter().filter(|c| c.exists()) {
            let capital = w.locality(c.capital);
            assert_eq!(capital.country, Some(c.id));
            assert_eq!(w.map.tile(capital.pos).unwrap().owner, Some(c.id));
            let ruler = w.person(c.ruler.unwrap());
            assert_eq!(ruler.role, Role::Ruler);
            assert!(ruler.alive_in(w.year), "{} sin gobernante vivo", c.name);
            assert!((0.0..=1.0).contains(&c.stability));
        }
        for c in w.countries.iter().filter(|c| !c.exists()) {
            assert_eq!(
                w.territory_size(c.id),
                0,
                "{} cayó pero conserva tierras",
                c.name
            );
        }
        for p in &w.people {
            match p.died {
                None => assert_eq!(w.locality(p.home).country, p.country),
                Some(year) => assert!(p.born <= year && year <= w.year),
            }
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

#[test]
fn history_continues_deterministically() {
    let config = WorldConfig {
        seed: 5,
        history_years: 120,
        ..Default::default()
    };
    let mut short = generate(&WorldConfig {
        history_years: 80,
        ..config.clone()
    });
    simulate::run(&mut short, 40);
    short.config.history_years = 120;
    assert_eq!(short, generate(&config));
}

#[test]
fn history_is_eventful() {
    let worlds: Vec<World> = (0..5).map(world).collect();
    let count = |f: &dyn Fn(&EventKind) -> bool| {
        worlds
            .iter()
            .flat_map(|w| &w.events)
            .filter(|e| f(&e.kind))
            .count()
    };
    assert!(
        count(&|k| matches!(k, EventKind::WarDeclared { .. })) > 0,
        "sin guerras"
    );
    assert!(
        count(&|k| matches!(k, EventKind::LocalityConquered { .. })) > 0,
        "sin conquistas"
    );
    assert!(
        count(&|k| matches!(k, EventKind::PersonDied { .. })) > 0,
        "sin muertes"
    );
    assert!(
        count(&|k| matches!(k, EventKind::LocalityGrew { .. })) > 0,
        "sin crecimiento"
    );
    assert!(
        count(&|k| matches!(k, EventKind::AllianceFormed { .. })) > 0,
        "sin alianzas"
    );
    for w in &worlds {
        assert_eq!(w.year, w.config.start_year + w.config.history_years as i32);
        assert!(
            w.countries.iter().any(|c| c.exists()),
            "no queda ningún país"
        );
        // Los países fundados al inicio llevan siglos: han tenido sucesiones.
        let founders = w
            .countries
            .iter()
            .filter(|c| c.exists() && c.founded <= w.config.start_year);
        for c in founders {
            assert!(
                w.rulers_of(c.id).len() >= 2,
                "{} nunca cambió de gobernante",
                c.name
            );
        }
        for war in &w.wars {
            if let Some(end) = war.ended {
                assert!(war.started <= end);
            }
        }
    }
}

#[test]
fn chronicle_rebuilds_any_year() {
    let config = WorldConfig {
        seed: 9,
        history_years: 60,
        ..Default::default()
    };
    let mut chronicle = Chronicle::new(&config);
    assert_eq!(chronicle.at(60), generate(&config));
    for year in [0, 7, 23, 41] {
        let expected = generate(&WorldConfig {
            history_years: year as u32,
            ..config.clone()
        });
        let mut past = chronicle.at(year);
        past.config.history_years = year as u32;
        assert_eq!(past, expected, "año {year}");
    }
}

#[test]
fn chronicle_text_covers_history() {
    let w = world(4);
    let text = w.chronicle_markdown();
    assert!(text.starts_with(&format!("# Crónica de {}", w.name)));
    for c in &w.countries {
        assert!(text.contains(&c.name), "falta {}", c.name);
    }
    for war in &w.wars {
        assert!(text.contains(&war.name));
    }
}

#[test]
fn causes_point_backwards() {
    for seed in 0..5 {
        let w = world(seed);
        for e in &w.events {
            if let Some(c) = e.cause {
                assert!(c < e.id, "{:?} causado por un evento posterior", e.kind);
                assert!(w.events[c.index()].year <= e.year);
            }
        }
        for war in &w.wars {
            let declared = war.declared.expect("toda guerra tiene declaración");
            assert!(matches!(
                w.events[declared.index()].kind,
                EventKind::WarDeclared { war: id } if id == war.id
            ));
        }
    }
}

#[test]
fn intrigues_happen() {
    let worlds: Vec<World> = (0..8).map(world).collect();
    let count = |f: &dyn Fn(&EventKind) -> bool| {
        worlds
            .iter()
            .flat_map(|w| &w.events)
            .filter(|e| f(&e.kind))
            .count()
    };
    assert!(
        count(&|k| matches!(k, EventKind::Marriage { .. })) > 0,
        "sin bodas"
    );
    assert!(
        count(&|k| matches!(
            k,
            EventKind::Assassinated { .. } | EventKind::PlotFoiled { .. }
        )) > 0,
        "sin conjuras"
    );
    for w in &worlds {
        for p in &w.people {
            if let Some(s) = p.spouse {
                assert!(w.person(s).spouse.is_some());
                assert_ne!(s, p.id, "{} casado consigo mismo", p.name);
            }
        }
    }
}

#[test]
fn stories_are_woven() {
    for seed in 0..5 {
        let w = world(seed);
        let stories = w.stories();
        assert!(!stories.is_empty(), "seed {seed} sin historias");
        // Cada guerra tiene su historia.
        for war in &w.wars {
            assert!(
                stories
                    .iter()
                    .any(|s| s.kind == StoryKind::War && s.title == war.name),
                "{} sin historia",
                war.name
            );
        }
        for (i, s) in stories.iter().enumerate() {
            assert_eq!(s.id.index(), i);
            assert!(!s.title.is_empty() && !s.summary.is_empty());
            assert!(s.start <= s.end, "{}", s.title);
            assert!(!s.events.is_empty(), "{} sin hechos", s.title);
            assert!(s.events.windows(2).all(|p| p[0] < p[1]));
            for &c in &s.consequences {
                assert!(stories[c.index()].causes.contains(&s.id));
            }
            for &c in &s.chapters {
                assert_eq!(stories[c.index()].part_of, Some(s.id));
            }
        }
        if w.wars.len() >= 2 {
            assert!(stories.iter().any(|s| s.kind != StoryKind::War));
        }
    }
}

#[test]
fn chronicle_text_tells_stories() {
    let w = world(4);
    let text = w.chronicle_markdown();
    assert!(text.contains("## Historias"));
    for s in w.stories() {
        assert!(text.contains(&s.title), "falta {}", s.title);
    }
}

#[test]
fn json_round_trip_keeps_causes() {
    let w = world(2);
    let back = World::from_json(&w.to_json().unwrap()).unwrap();
    assert_eq!(back, w);
}

fn aged(seed: u64, age: Age) -> World {
    generate(&WorldConfig {
        seed,
        age,
        ..Default::default()
    })
}

#[test]
fn ages_share_geography_but_not_history() {
    let worlds: Vec<World> = Age::ALL.iter().map(|&a| aged(5, a)).collect();
    for w in &worlds[1..] {
        assert_eq!(
            w.map.tiles.iter().map(|t| t.biome).collect::<Vec<_>>(),
            worlds[0]
                .map
                .tiles
                .iter()
                .map(|t| t.biome)
                .collect::<Vec<_>>()
        );
        assert_ne!(w.name, worlds[0].name);
        assert_ne!(w.countries[0].name, worlds[0].countries[0].name);
    }
}

#[test]
fn each_age_has_its_own_calamities_and_feats() {
    for age in Age::ALL {
        let (mut calamities, mut feats) = (0, 0);
        for seed in [1, 2, 3] {
            let w = aged(seed, age);
            for e in &w.events {
                assert!(!w.describe(e).is_empty());
                match e.kind {
                    EventKind::Calamity { calamity, .. } => {
                        assert!(
                            age.calamities().contains(&calamity),
                            "{age:?}: {calamity:?}"
                        );
                        calamities += 1;
                    }
                    EventKind::Feat { feat, .. } => {
                        assert!(
                            feat == Feat::Rescue || age.feats().contains(&feat),
                            "{age:?}: {feat:?}"
                        );
                        feats += 1;
                    }
                    _ => {}
                }
            }
            for s in w.stories() {
                assert!(!s.title.is_empty() && !s.summary.is_empty());
            }
        }
        assert!(calamities >= 3, "{age:?}: solo {calamities} calamidades");
        assert!(feats >= 5, "{age:?}: solo {feats} hazañas");
    }
}

#[test]
fn texts_speak_the_language_of_the_age() {
    let fantasy = aged(3, Age::Fantasy).chronicle_markdown();
    assert!(fantasy.contains("Edad: Fantasía"));
    let cyber = aged(3, Age::Cyberpunk);
    let text = cyber.chronicle_markdown();
    assert!(text.contains("Edad: Cyberpunk"));
    assert!(
        !text.contains("peste"),
        "en cyberpunk hay epidemias, no pestes"
    );
    let names = Government::Monarchy.name(Age::Cyberpunk);
    assert_eq!(names, "Megacorporación");
    assert_eq!(Age::from_text(" fantasia "), Some(Age::Fantasy));
    assert_eq!(Age::from_text("Cyberpunk"), Some(Age::Cyberpunk));
}

#[test]
fn worlds_saved_before_ages_load_as_realistic() {
    let w = world(2);
    let mut value: serde_json::Value = serde_json::from_str(&w.to_json().unwrap()).unwrap();
    value["config"].as_object_mut().unwrap().remove("age");
    let json = value.to_string();
    assert!(!json.contains("\"age\""));
    assert_eq!(World::from_json(&json).unwrap().config.age, Age::Realistic);
}

#[test]
fn chronicle_rebuilds_any_age() {
    let config = WorldConfig {
        seed: 6,
        history_years: 40,
        age: Age::Cyberpunk,
        ..Default::default()
    };
    let mut chronicle = Chronicle::new(&config);
    assert_eq!(chronicle.at(40), generate(&config));
}
