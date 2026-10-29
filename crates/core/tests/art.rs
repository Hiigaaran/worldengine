use resvg::usvg::{self, TreeParsing};
use worldengine_core::art::{self, Motif};
use worldengine_core::*;

fn world(seed: u64, age: Age) -> World {
    generate(&WorldConfig {
        seed,
        age,
        ..Default::default()
    })
}

/// El SVG se puede leer y dibuja algo.
fn renders(svg: &str) {
    let tree = usvg::Tree::from_str(svg, &usvg::Options::default())
        .unwrap_or_else(|e| panic!("SVG inválido ({e}):\n{svg}"));
    let tree = resvg::Tree::from_usvg(&tree);
    let mut pixmap = resvg::tiny_skia::Pixmap::new(art::SIZE, art::SIZE).unwrap();
    tree.render(resvg::tiny_skia::Transform::default(), &mut pixmap.as_mut());
    assert!(pixmap.pixels().iter().any(|p| p.alpha() > 0));
}

#[test]
fn every_motif_renders_in_every_age() {
    for age in Age::ALL {
        for motif in Motif::all() {
            renders(&motif.svg(age));
        }
    }
}

#[test]
fn every_portrait_renders() {
    for age in Age::ALL {
        let w = world(5, age);
        assert!(!w.people.is_empty());
        for person in &w.people {
            renders(&art::portrait(&w, person.id));
        }
    }
}

#[test]
fn portraits_differ_between_people_and_ages() {
    let w = world(5, Age::Realistic);
    let a = art::portrait(&w, PersonId(0));
    assert_eq!(a, art::portrait(&w, PersonId(0)));
    assert!(w.people.iter().any(|p| art::portrait(&w, p.id) != a));
    let f = world(5, Age::Fantasy);
    assert_ne!(a, art::portrait(&f, PersonId(0)));
}

#[test]
fn stories_have_images() {
    let w = world(3, Age::Cyberpunk);
    for story in w.stories() {
        renders(&art::story(&w, &story));
    }
}

#[test]
fn every_event_has_a_place() {
    let w = world(2, Age::Fantasy);
    for event in &w.events {
        let place = w.event_place(event).expect("todo evento tiene lugar");
        assert!(w.map.tile(place).is_some());
        if let Some(location) = event.location {
            assert_eq!(place, location);
        }
    }
}

#[test]
fn central_is_the_place_nearest_the_average() {
    let p = |x, y| Pos { x, y };
    assert_eq!(central(&[]), None);
    assert_eq!(central(&[p(3, 4)]), Some(p(3, 4)));
    assert_eq!(
        central(&[p(0, 0), p(10, 0), p(4, 0), p(10, 10)]),
        Some(p(4, 0))
    );
}

#[test]
fn story_focus_is_one_of_its_places() {
    let w = world(4, Age::Realistic);
    let stories = w.stories();
    assert!(!stories.is_empty());
    for story in &stories {
        let places = w.places_of(story.events.iter().map(|&e| &w.events[e.index()]));
        assert!(!places.is_empty(), "«{}» sin lugares", story.title);
        assert!(places.contains(&central(&places).unwrap()));
    }
}
