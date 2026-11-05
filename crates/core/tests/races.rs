use worldengine_core::*;

fn world(seed: u64, age: Age) -> World {
    generate(&WorldConfig {
        seed,
        age,
        ..Default::default()
    })
}

#[test]
fn every_age_has_its_own_races() {
    for age in Age::ALL {
        for seed in [1, 7, 42] {
            let w = world(seed, age);
            assert!(w.races.len() >= 2, "{age:?} {seed}: pocas razas");
            for race in &w.races {
                assert_eq!(race.kind.is_culture(), age == Age::Realistic);
                assert!(!race.endonym.is_empty());
                assert!(!race.skins.is_empty() && !race.hair.is_empty() && !race.eyes.is_empty());
                assert!(race.skins.iter().all(|t| t.color.is_some()));
                assert_eq!(race.customs.len(), 7);
                assert_eq!(race.language.lexicon.len(), 10);
                assert!(race.height_cm[0] < race.height_cm[1]);
            }
            if age != Age::Realistic {
                // Una sola raza de cada clase, y siempre humanos.
                let mut kinds: Vec<RaceKind> = w.races.iter().map(|r| r.kind).collect();
                kinds.sort_by_key(|k| *k as u8);
                kinds.dedup();
                assert_eq!(kinds.len(), w.races.len());
                assert!(w
                    .races
                    .iter()
                    .any(|r| matches!(r.kind, RaceKind::Human | RaceKind::Natural)));
            }
        }
    }
}

#[test]
fn countries_and_people_belong_to_races() {
    for age in Age::ALL {
        let w = world(3, age);
        for c in &w.countries {
            let race = c.people.expect("todo país tiene un pueblo mayoritario");
            assert!(race.index() < w.races.len());
        }
        for p in &w.people {
            assert!(p.race.is_some(), "{} no tiene raza", p.name);
            if let Some(parent) = p.parent {
                assert_eq!(p.race, w.person(parent).race, "los hijos heredan la raza");
            }
        }
        // La mayoría de un país es casi siempre su raza mayoritaria.
        for race in &w.races {
            if w.homelands_of(race.id).next().is_some() {
                assert!(w.members_of(race.id).count() > 0);
            }
        }
    }
}

#[test]
fn descriptions_and_prompts_follow_the_race() {
    for age in Age::ALL {
        let w = world(11, age);
        for p in &w.people {
            let profile = w.profile(p.id).unwrap();
            let race = w.race(profile.race);
            assert!(profile.height_cm <= race.height_cm[1]);
            assert!(race.skins.contains(&profile.skin));
            let text = w.describe_person(p.id);
            assert!(text.starts_with(&p.name));
            assert!(text.contains(&race.endonym), "{text}");
            assert!(text.contains(&race.language.name));
            let prompt = w.image_prompt(p.id);
            assert!(prompt.starts_with("Portrait of"));
            assert!(prompt.contains(&profile.skin.en));
        }
    }
}

#[test]
fn profiles_are_stable() {
    let a = world(5, Age::Fantasy);
    let b = world(5, Age::Fantasy);
    assert_eq!(a.races, b.races);
    for p in &a.people {
        assert_eq!(a.profile(p.id), b.profile(p.id));
        assert_eq!(a.describe_person(p.id), b.describe_person(p.id));
    }
}

#[test]
fn image_json_is_valid() {
    let w = world(2, Age::Cyberpunk);
    let json = w.image_json(PersonId(0));
    let value: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert!(value["prompt"].as_str().unwrap().starts_with("Portrait of"));
    assert!(value["profile"]["skin"]["en"].is_string());
    assert!(value["description"].is_string());
}

#[test]
fn chronicle_lists_races_and_people() {
    let w = world(4, Age::Fantasy);
    let md = w.chronicle_markdown();
    assert!(md.contains("## Razas"));
    assert!(md.contains("## Personajes de hoy"));
    for race in &w.races {
        assert!(md.contains(&race.title()));
    }
    let r = world(4, Age::Realistic).chronicle_markdown();
    assert!(r.contains("## Pueblos"));
}

#[test]
fn old_worlds_without_races_still_load() {
    let w = world(6, Age::Realistic);
    let mut value = serde_json::to_value(&w).unwrap();
    value.as_object_mut().unwrap().remove("races");
    for p in value["people"].as_array_mut().unwrap() {
        p.as_object_mut().unwrap().remove("race");
    }
    let old: World = serde_json::from_value(value).unwrap();
    assert!(old.races.is_empty());
    assert!(old.profile(PersonId(0)).is_none());
    assert_eq!(old.describe_person(PersonId(0)), "");
    // Sin raza, el retrato se dibuja como antes.
    assert!(!art::portrait(&old, PersonId(0)).is_empty());
}
