//! Imágenes vectoriales (SVG) para los personajes, los eventos y las historias.
//!
//! Todo se dibuja con formas simples sobre un lienzo de 128 × 128 y depende de
//! la edad del mundo: en la edad realista los retratos parecen grabados sobre
//! pergamino; en la de fantasía brillan sobre un cielo estrellado, y en la
//! cyberpunk relucen en neón sobre una cuadrícula. Los retratos combinan rasgos
//! sacados del id del personaje (piel, pelo, barba) con los atributos de su rol:
//! la corona del gobernante, el yelmo del general o el sombrero del mago.
//!
//! Las imágenes son texto SVG, así que pueden guardarse y editarse en
//! cualquier programa de dibujo vectorial.

use std::f32::consts::TAU;
use std::fmt::Write;

use crate::age::Age;
use crate::ids::*;
use crate::model::*;
use crate::rng::fnv1a;
use crate::stories::{Story, StoryKind};

/// Lado del lienzo de todas las imágenes.
pub const SIZE: u32 = 128;

/// Escribe en una cadena sin preocuparse del `Result` (escribir en un `String`
/// no falla).
macro_rules! put {
    ($s:expr, $($arg:tt)*) => {{
        let _ = write!($s, $($arg)*);
    }};
}

/// El dibujo que representa un tipo de evento o de historia.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Motif {
    Settlement,
    Banner,
    Birth,
    Rise,
    Crown,
    Grave,
    Fallen,
    Dagger,
    Skull,
    Growth,
    Ruins,
    Annex,
    Alliance,
    BrokenAlliance,
    Swords,
    Battle,
    Fortress,
    Peace,
    Capital,
    Fall,
    BrokenChain,
    Plague,
    Rings,
    Plot,
    Lineage,
    Sun,
    Rivalry,
    Calamity(Calamity),
    Feat(Feat),
}

impl Motif {
    pub fn of_event(kind: &EventKind) -> Motif {
        match *kind {
            EventKind::LocalityFounded { .. } => Motif::Settlement,
            EventKind::CountryFounded { .. } => Motif::Banner,
            EventKind::PersonBorn { .. } => Motif::Birth,
            EventKind::PersonRose { .. } => Motif::Rise,
            EventKind::RulerCrowned { .. } => Motif::Crown,
            EventKind::PersonDied { cause, .. } => match cause {
                DeathCause::OldAge | DeathCause::Illness => Motif::Grave,
                DeathCause::Battle => Motif::Fallen,
                DeathCause::Assassination | DeathCause::Execution => Motif::Dagger,
                DeathCause::Plague => Motif::Plague,
                DeathCause::Calamity => Motif::Skull,
            },
            EventKind::LocalityGrew { .. } => Motif::Growth,
            EventKind::LocalityRuined { .. } => Motif::Ruins,
            EventKind::LocalityAnnexed { .. } => Motif::Annex,
            EventKind::AllianceFormed { .. } => Motif::Alliance,
            EventKind::AllianceBroken { .. } => Motif::BrokenAlliance,
            EventKind::WarDeclared { .. } => Motif::Swords,
            EventKind::Battle { .. } => Motif::Battle,
            EventKind::LocalityConquered { .. } => Motif::Fortress,
            EventKind::PeaceSigned { .. } => Motif::Peace,
            EventKind::CapitalMoved { .. } => Motif::Capital,
            EventKind::CountryFell { .. } => Motif::Fall,
            EventKind::Independence { .. } => Motif::BrokenChain,
            EventKind::Plague { .. } => Motif::Plague,
            EventKind::Marriage { .. } => Motif::Rings,
            EventKind::Assassinated { .. } => Motif::Dagger,
            EventKind::PlotFoiled { .. } => Motif::Plot,
            EventKind::Calamity { calamity, .. } => Motif::Calamity(calamity),
            EventKind::Feat { feat, .. } => Motif::Feat(feat),
        }
    }

    /// El dibujo de una historia: el de su tipo o, en calamidades y hazañas,
    /// el de la calamidad o la hazaña que la abre.
    pub fn of_story(world: &World, story: &Story) -> Motif {
        let first = |pick: fn(&EventKind) -> Option<Motif>| {
            story
                .events
                .iter()
                .find_map(|&e| pick(&world.events[e.index()].kind))
        };
        match story.kind {
            StoryKind::War => Motif::Battle,
            StoryKind::Rivalry => Motif::Rivalry,
            StoryKind::Intrigue => Motif::Plot,
            StoryKind::Marriage => Motif::Rings,
            StoryKind::Plague => Motif::Plague,
            StoryKind::Rebellion => Motif::BrokenChain,
            StoryKind::Dynasty => Motif::Lineage,
            StoryKind::GoldenAge => Motif::Sun,
            StoryKind::Calamity => first(|k| match *k {
                EventKind::Calamity { calamity, .. } => Some(Motif::Calamity(calamity)),
                _ => None,
            })
            .unwrap_or(Motif::Skull),
            StoryKind::Feat => first(|k| match *k {
                EventKind::Feat { feat, .. } => Some(Motif::Feat(feat)),
                _ => None,
            })
            .unwrap_or(Motif::Rise),
        }
    }

    /// Todos los dibujos, para probarlos y para el catálogo de ejemplo.
    pub fn all() -> Vec<Motif> {
        let mut all = vec![
            Motif::Settlement,
            Motif::Banner,
            Motif::Birth,
            Motif::Rise,
            Motif::Crown,
            Motif::Grave,
            Motif::Fallen,
            Motif::Dagger,
            Motif::Skull,
            Motif::Growth,
            Motif::Ruins,
            Motif::Annex,
            Motif::Alliance,
            Motif::BrokenAlliance,
            Motif::Swords,
            Motif::Battle,
            Motif::Fortress,
            Motif::Peace,
            Motif::Capital,
            Motif::Fall,
            Motif::BrokenChain,
            Motif::Plague,
            Motif::Rings,
            Motif::Plot,
            Motif::Lineage,
            Motif::Sun,
            Motif::Rivalry,
        ];
        for age in Age::ALL {
            all.extend(age.calamities().iter().map(|&c| Motif::Calamity(c)));
            all.extend(age.feats().iter().map(|&f| Motif::Feat(f)));
        }
        all.push(Motif::Feat(Feat::Rescue));
        let mut unique = Vec::new();
        for motif in all {
            if !unique.contains(&motif) {
                unique.push(motif);
            }
        }
        unique
    }

    /// El dibujo como SVG, con el estilo de la edad.
    pub fn svg(self, age: Age) -> String {
        let p = Palette::of(age);
        card(age, &p, &motif(self, age, &p), false)
    }
}

/// Imagen de un evento.
pub fn event(world: &World, event: &Event) -> String {
    Motif::of_event(&event.kind).svg(world.config.age)
}

/// Imagen de una historia.
pub fn story(world: &World, story: &Story) -> String {
    Motif::of_story(world, story).svg(world.config.age)
}

/// Colores de una edad.
struct Palette {
    top: &'static str,
    bottom: &'static str,
    ink: &'static str,
    /// Dorado, magenta...: coronas, monedas, estrellas.
    accent: &'static str,
    /// Rojo, turquesa, cian: gemas, banderas, luces.
    accent2: &'static str,
    metal: &'static str,
    light: &'static str,
    stone: &'static str,
    ground: &'static str,
    blood: &'static str,
    /// Si las luces brillan (fantasía y cyberpunk).
    glow: bool,
}

impl Palette {
    fn of(age: Age) -> Palette {
        match age {
            Age::Realistic => Palette {
                top: "#f1e6c8",
                bottom: "#cdb68a",
                ink: "#3a2a1a",
                accent: "#c8962e",
                accent2: "#9b2d20",
                metal: "#a3abb3",
                light: "#fbf4e2",
                stone: "#a49c8c",
                ground: "#9c8458",
                blood: "#9b1f16",
                glow: false,
            },
            Age::Fantasy => Palette {
                top: "#3d2b72",
                bottom: "#140d30",
                ink: "#120a26",
                accent: "#f2c94c",
                accent2: "#4fe0c4",
                metal: "#c9d3f2",
                light: "#fff6dc",
                stone: "#8a86a8",
                ground: "#2b2150",
                blood: "#c02a3a",
                glow: true,
            },
            Age::Cyberpunk => Palette {
                top: "#1d0f33",
                bottom: "#05060d",
                ink: "#05060d",
                accent: "#ff2fb9",
                accent2: "#29f0ff",
                metal: "#8b97a6",
                light: "#e8fbff",
                stone: "#4a4f63",
                ground: "#141526",
                blood: "#ff3355",
                glow: true,
            },
        }
    }

    /// Atributo para que una forma brille, si la edad tiene luces.
    fn glow(&self) -> &'static str {
        if self.glow {
            r##" filter="url(#glow)""##
        } else {
            ""
        }
    }
}

/// El marco común: fondo de la edad, decoración y el dibujo encima. `faded`
/// pasa el dibujo a escala de grises (personajes muertos).
fn card(age: Age, p: &Palette, body: &str, faded: bool) -> String {
    let mut s = String::with_capacity(body.len() + 2000);
    put!(
        s,
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="{SIZE}" height="{SIZE}" viewBox="0 0 128 128">"##
    );
    put!(
        s,
        r##"<defs><linearGradient id="bg" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="{}"/><stop offset="1" stop-color="{}"/></linearGradient>"##,
        p.top,
        p.bottom
    );
    s += r##"<filter id="glow" x="-50%" y="-50%" width="200%" height="200%"><feGaussianBlur stdDeviation="2.2" result="b"/><feMerge><feMergeNode in="b"/><feMergeNode in="SourceGraphic"/></feMerge></filter>"##;
    s += r##"<filter id="faded"><feColorMatrix type="saturate" values="0.12"/></filter>"##;
    s += r##"<clipPath id="card"><rect width="128" height="128" rx="14"/></clipPath></defs>"##;
    s += r##"<g clip-path="url(#card)"><rect width="128" height="128" fill="url(#bg)"/>"##;
    match age {
        Age::Realistic => put!(
            s,
            r##"<rect x="6" y="6" width="116" height="116" rx="9" fill="none" stroke="{}" stroke-opacity="0.3" stroke-width="1.5"/>"##,
            p.ink
        ),
        Age::Fantasy => {
            for (x, y, r) in [
                (14, 18, 1.4),
                (30, 9, 0.9),
                (104, 14, 1.6),
                (116, 34, 1.0),
                (22, 44, 1.0),
                (96, 28, 0.8),
                (110, 58, 1.2),
                (12, 70, 1.3),
                (50, 12, 0.8),
                (78, 8, 1.1),
            ] {
                put!(
                    s,
                    r##"<circle cx="{x}" cy="{y}" r="{r}" fill="{}" fill-opacity="0.8"/>"##,
                    p.light
                );
            }
        }
        Age::Cyberpunk => {
            let mut grid = String::new();
            for y in [92, 100, 109, 119] {
                put!(grid, "M0 {y} L128 {y} ");
            }
            for x in [-64, -24, 16, 56, 72, 112, 152, 192] {
                put!(grid, "M64 84 L{x} 128 ");
            }
            put!(
                s,
                r##"<path d="{grid}" stroke="{}" stroke-opacity="0.25" stroke-width="1" fill="none"/>"##,
                p.accent2
            );
        }
    }
    if faded {
        put!(s, r##"<g filter="url(#faded)">{body}</g>"##);
    } else {
        s += body;
    }
    s += "</g>";
    let border = if age == Age::Realistic {
        p.ink
    } else {
        p.accent2
    };
    put!(
        s,
        r##"<rect x="1" y="1" width="126" height="126" rx="13.5" fill="none" stroke="{border}" stroke-opacity="0.6" stroke-width="2"/></svg>"##
    );
    s
}

// --- Retratos ---------------------------------------------------------------

const SKINS: [&str; 6] = [
    "#f3d5bd", "#e8bf9a", "#d39f74", "#b07b52", "#8a5a3b", "#5f3b26",
];
const HAIR: [&str; 6] = [
    "#2b1d14", "#5a3a22", "#a8662f", "#d8b25a", "#8c8c8c", "#161616",
];
const FANTASY_HAIR: [&str; 3] = ["#e6e6f2", "#7b2d26", "#2d4a7b"];
const NEON_HAIR: [&str; 3] = ["#ff2fb9", "#29f0ff", "#b6ff3b"];

/// Retrato de un personaje, según su rol, la edad del mundo y rasgos propios
/// que salen de su id. Los muertos se ven en tonos apagados.
pub fn portrait(world: &World, id: PersonId) -> String {
    let person = world.person(id);
    let age = world.config.age;
    let p = Palette::of(age);
    let trait_of = |salt: &str| -> usize {
        let text = format!("{}-{}-{salt}", world.config.seed, id.index());
        fnv1a(text.as_bytes()) as usize
    };
    let skin = SKINS[trait_of("piel") % SKINS.len()];
    let pick = trait_of("pelo");
    let hair = match age {
        Age::Fantasy if pick % 4 == 0 => FANTASY_HAIR[pick / 4 % FANTASY_HAIR.len()],
        Age::Cyberpunk if pick % 3 == 0 => NEON_HAIR[pick / 3 % NEON_HAIR.len()],
        _ => HAIR[pick % HAIR.len()],
    };
    let role = person.role;
    let wizard = age == Age::Fantasy && role == Role::Scholar;
    let mohawk = age == Age::Cyberpunk && role == Role::Adventurer;
    let style = if mohawk { 2 } else { trait_of("peinado") % 4 };
    let beard = wizard || trait_of("barba") % 3 == 0;
    let attire = attire(age, role);
    let ink = p.ink;

    let mut b = String::new();
    if style == 1 {
        put!(
            b,
            r##"<path d="M40 60 C37 30 91 30 88 60 L90 94 L38 94 Z" fill="{hair}" stroke="{ink}" stroke-width="1.5"/>"##
        );
    }
    put!(
        b,
        r##"<path d="M16 130 C18 100 38 90 64 90 C90 90 110 100 112 130 Z" fill="{attire}" stroke="{ink}" stroke-width="2"/>"##
    );
    put!(
        b,
        r##"<path d="M56 74 L56 90 Q64 95 72 90 L72 74 Z" fill="{skin}" stroke="{ink}" stroke-width="1.5"/>"##
    );
    for x in [41, 87] {
        put!(
            b,
            r##"<ellipse cx="{x}" cy="60" rx="4" ry="6" fill="{skin}" stroke="{ink}" stroke-width="1.5"/>"##
        );
    }
    put!(
        b,
        r##"<ellipse cx="64" cy="58" rx="23" ry="25" fill="{skin}" stroke="{ink}" stroke-width="2"/>"##
    );
    put!(
        b,
        r##"<circle cx="55" cy="58" r="2.6" fill="{ink}"/><circle cx="73" cy="58" r="2.6" fill="{ink}"/>"##
    );
    put!(
        b,
        r##"<path d="M50 52 L59 51 M69 51 L78 52" stroke="{ink}" stroke-width="2" stroke-linecap="round"/>"##
    );
    put!(
        b,
        r##"<path d="M64 60 L62 67 L66 67" fill="none" stroke="{ink}" stroke-width="1.4" stroke-linecap="round"/>"##
    );
    if beard {
        let tip = if wizard {
            "C42 98 60 116 64 118 C68 116 86 98 86 62"
        } else {
            "C44 88 84 88 86 62"
        };
        put!(
            b,
            r##"<path d="M42 62 {tip} C80 76 70 78 64 78 C58 78 48 76 42 62 Z" fill="{hair}" stroke="{ink}" stroke-width="1.5"/>"##
        );
    }
    put!(
        b,
        r##"<path d="M58 73 Q64 76 70 73" fill="none" stroke="#2a1410" stroke-width="1.6" stroke-linecap="round"/>"##
    );
    match style {
        2 => put!(
            b,
            r##"<path d="M41 64 C40 54 42 48 46 46 L45 62 Z M87 64 C88 54 86 48 82 46 L83 62 Z" fill="{hair}" stroke="{ink}" stroke-width="1"/>"##
        ),
        _ => {
            put!(
                b,
                r##"<path d="M41 58 C38 29 90 29 87 58 C82 44 72 40 64 40 C56 40 46 44 41 58 Z" fill="{hair}" stroke="{ink}" stroke-width="1.5"/>"##
            );
            if style == 3 {
                put!(
                    b,
                    r##"<circle cx="64" cy="31" r="8" fill="{hair}" stroke="{ink}" stroke-width="1.5"/>"##
                );
            }
        }
    }
    b += &accessory(age, role, attire, &p);
    if person.died.is_some() {
        b += r##"<path d="M94 0 L128 0 L128 34 Z" fill="#111" fill-opacity="0.85"/>"##;
    }
    card(age, &p, &b, person.died.is_some())
}

/// Color de la ropa según el rol y la edad.
fn attire(age: Age, role: Role) -> &'static str {
    match (age, role) {
        (Age::Realistic, Role::Ruler) => "#7a1f2b",
        (Age::Realistic, Role::Noble) => "#2f4f7a",
        (Age::Realistic, Role::General) => "#4a5a3a",
        (Age::Realistic, Role::Priest) => "#5b4632",
        (Age::Realistic, Role::Merchant) => "#8a5a2b",
        (Age::Realistic, Role::Scholar) => "#2b2b3a",
        (Age::Realistic, Role::Adventurer) => "#6b4a2b",
        (Age::Fantasy, Role::Ruler) => "#5a2f9a",
        (Age::Fantasy, Role::Noble) => "#2d6a5a",
        (Age::Fantasy, Role::General) => "#c9d3e8",
        (Age::Fantasy, Role::Priest) => "#efeaf7",
        (Age::Fantasy, Role::Merchant) => "#a0442a",
        (Age::Fantasy, Role::Scholar) => "#2a3f9a",
        (Age::Fantasy, Role::Adventurer) => "#3d6b35",
        (Age::Cyberpunk, Role::Ruler) => "#1d1d2b",
        (Age::Cyberpunk, Role::Noble) => "#262a36",
        (Age::Cyberpunk, Role::General) => "#2e3a2e",
        (Age::Cyberpunk, Role::Priest) => "#2a1838",
        (Age::Cyberpunk, Role::Merchant) => "#30254a",
        (Age::Cyberpunk, Role::Scholar) => "#1c2430",
        (Age::Cyberpunk, Role::Adventurer) => "#3a2b2b",
    }
}

/// La corona, con la base a la altura de la frente de un retrato.
fn crown(fill: &str, gem: &str, p: &Palette) -> String {
    let ink = p.ink;
    format!(
        r##"<path d="M43 44 L42 24 L52 33 L58 18 L64 31 L70 18 L76 33 L86 24 L85 44 Z" fill="{fill}" stroke="{ink}" stroke-width="2" stroke-linejoin="round"/><rect x="42" y="40" width="44" height="6" rx="2" fill="{fill}" stroke="{ink}" stroke-width="1.5"/><circle cx="64" cy="37" r="3.2" fill="{gem}" stroke="{ink}"/><circle cx="52" cy="38" r="2.2" fill="{gem}"/><circle cx="76" cy="38" r="2.2" fill="{gem}"/>"##
    )
}

/// Capucha que deja ver la cara.
fn hood(fill: &str, ink: &str) -> String {
    format!(
        r##"<path d="M34 98 C26 56 38 22 64 22 C90 22 102 56 94 98 L82 93 C86 66 80 39 64 39 C48 39 42 66 46 93 Z" fill="{fill}" stroke="{ink}" stroke-width="2"/>"##
    )
}

/// Solapas y corbata de un traje.
fn suit(p: &Palette, tie: &str) -> String {
    format!(
        r##"<path d="M50 91 L64 112 L78 91" fill="none" stroke="{}" stroke-width="3"/><path d="M64 95 L60 101 L64 124 L68 101 Z" fill="{tie}"/>"##,
        p.light
    )
}

fn coins(p: &Palette) -> String {
    let mut s = String::new();
    for (x, y) in [(96, 116), (104, 108), (92, 104)] {
        put!(
            s,
            r##"<circle cx="{x}" cy="{y}" r="7" fill="{}" stroke="{}" stroke-width="1.5"/><circle cx="{x}" cy="{y}" r="3.5" fill="none" stroke="{}" stroke-width="1"/>"##,
            p.accent,
            p.ink,
            p.ink
        );
    }
    s
}

/// Lo que distingue a cada rol en cada edad.
fn accessory(age: Age, role: Role, attire: &str, p: &Palette) -> String {
    let (ink, a, b2, metal, light, glow) = (p.ink, p.accent, p.accent2, p.metal, p.light, p.glow());
    match (age, role) {
        (Age::Realistic | Age::Fantasy, Role::Ruler) => {
            let mut s = format!(
                r##"<path d="M26 104 Q64 84 102 104 L102 113 Q64 94 26 113 Z" fill="#f4efe4" stroke="{ink}" stroke-width="1.5"/>"##
            );
            for x in [36, 50, 78, 92] {
                put!(s, r##"<circle cx="{x}" cy="103" r="1.6" fill="#222"/>"##);
            }
            let gem = if age == Age::Fantasy { b2 } else { p.blood };
            put!(s, "<g{glow}>{}</g>", crown(a, gem, p));
            s
        }
        (Age::Cyberpunk, Role::Ruler) => format!(
            r##"{}<ellipse cx="64" cy="26" rx="26" ry="7" fill="none" stroke="{b2}" stroke-width="3"{glow}/><ellipse cx="64" cy="26" rx="17" ry="4" fill="none" stroke="{a}" stroke-width="1.5"{glow}/>"##,
            suit(p, a)
        ),
        (Age::Realistic, Role::Noble) => {
            let mut s = format!(
                r##"<ellipse cx="64" cy="90" rx="20" ry="6" fill="#f7f3ea" stroke="{ink}" stroke-width="1.5"/>"##
            );
            for x in (48..=80).step_by(4) {
                put!(
                    s,
                    r##"<path d="M{x} 86 L{x} 94" stroke="{ink}" stroke-width="0.8"/>"##
                );
            }
            put!(
                s,
                r##"<ellipse cx="60" cy="36" rx="25" ry="9" fill="#1f2f4a" stroke="{ink}" stroke-width="1.5"/><path d="M78 34 C92 22 100 20 106 10 C98 26 92 32 80 39 Z" fill="{b2}" stroke="{ink}"/>"##
            );
            s
        }
        (Age::Fantasy, Role::Noble) => format!(
            r##"<path d="M34 98 L42 82 L52 92 Z M94 98 L86 82 L76 92 Z" fill="{attire}" stroke="{ink}" stroke-width="1.5"/><path d="M41 47 Q64 38 87 47" fill="none" stroke="{metal}" stroke-width="3"/><path d="M64 37 L68 42 L64 47 L60 42 Z" fill="{b2}" stroke="{ink}"{glow}/>"##
        ),
        (Age::Cyberpunk, Role::Noble) => format!(
            r##"{}<path d="M46 53 L82 53 L80 62 Q72 64 66 58 L62 58 Q56 64 48 62 Z" fill="{ink}"/><path d="M50 56 L57 56 M70 56 L77 56" stroke="{b2}" stroke-width="1.5"{glow}/><circle cx="87" cy="66" r="2" fill="{a}"{glow}/>"##,
            suit(p, b2)
        ),
        (Age::Realistic, Role::General) => format!(
            r##"<ellipse cx="31" cy="100" rx="12" ry="5" fill="{a}" stroke="{ink}"/><ellipse cx="97" cy="100" rx="12" ry="5" fill="{a}" stroke="{ink}"/><path d="M48 106 L52 116 M56 106 L52 116" stroke="{b2}" stroke-width="3"/><circle cx="52" cy="119" r="4" fill="{a}" stroke="{ink}"/><path d="M38 54 C38 26 90 26 90 54 Z" fill="{metal}" stroke="{ink}" stroke-width="2"/><path d="M34 54 L94 54 L90 59 L38 59 Z" fill="{metal}" stroke="{ink}" stroke-width="1.5"/><path d="M64 28 L64 54" stroke="{ink}" stroke-width="2"/>"##
        ),
        (Age::Fantasy, Role::General) => format!(
            r##"<ellipse cx="32" cy="100" rx="14" ry="8" fill="{metal}" stroke="{ink}" stroke-width="1.5"/><ellipse cx="96" cy="100" rx="14" ry="8" fill="{metal}" stroke="{ink}" stroke-width="1.5"/><circle cx="64" cy="112" r="7" fill="{a}" stroke="{ink}"{glow}/><path d="M64 30 C70 12 88 8 100 14 C86 14 76 20 70 33 Z" fill="{b2}" stroke="{ink}"{glow}/><path d="M38 57 C38 26 90 26 90 57 L88 77 L80 75 L80 59 L48 59 L48 75 L40 77 Z" fill="{metal}" stroke="{ink}" stroke-width="2"/><path d="M64 30 L64 50" stroke="{ink}" stroke-width="1.5"/>"##
        ),
        (Age::Cyberpunk, Role::General) => format!(
            r##"<path d="M44 92 L52 130 M84 92 L76 130" stroke="#0d0f12" stroke-width="6"/><rect x="58" y="106" width="12" height="5" rx="1" fill="{a}"{glow}/><path d="M38 53 C38 25 90 25 90 53 L90 57 L38 57 Z" fill="#2b3036" stroke="{ink}" stroke-width="2"/><rect x="42" y="52" width="44" height="10" rx="5" fill="{b2}" fill-opacity="0.85"{glow}/>"##
        ),
        (Age::Realistic, Role::Priest) => format!(
            r##"{}<path d="M56 92 L64 104 L72 92" fill="none" stroke="{a}" stroke-width="1.5"/><circle cx="64" cy="109" r="6" fill="{a}" stroke="{ink}"/><path d="M64 99 L64 119 M54 109 L74 109" stroke="{a}" stroke-width="1.5"/>"##,
            hood(attire, ink)
        ),
        (Age::Fantasy, Role::Priest) => format!(
            r##"<ellipse cx="64" cy="16" rx="20" ry="5" fill="none" stroke="{a}" stroke-width="3"{glow}/>{}<path d="M56 92 L64 104 L72 92" fill="none" stroke="{a}" stroke-width="1.5"/><path d="M64 101 L70 109 L64 117 L58 109 Z" fill="{b2}" stroke="{ink}"{glow}/>"##,
            hood(attire, ink)
        ),
        (Age::Cyberpunk, Role::Priest) => format!(
            r##"{}<path d="M40 86 L44 74 L44 52 L52 40 M88 86 L84 74 L84 52 L76 40" fill="none" stroke="{b2}" stroke-width="1.5"{glow}/><circle cx="44" cy="74" r="2" fill="{b2}"/><circle cx="84" cy="74" r="2" fill="{b2}"/><circle cx="55" cy="58" r="3.4" fill="{b2}"{glow}/><circle cx="73" cy="58" r="3.4" fill="{b2}"{glow}/><circle cx="64" cy="110" r="6" fill="none" stroke="{a}" stroke-width="2"{glow}/>"##,
            hood(attire, ink)
        ),
        (Age::Realistic, Role::Merchant) => format!(
            r##"{}<ellipse cx="64" cy="40" rx="32" ry="7" fill="#3b2a1e" stroke="{ink}" stroke-width="1.5"/><path d="M46 40 L48 18 Q64 12 80 18 L82 40 Z" fill="#3b2a1e" stroke="{ink}" stroke-width="1.5"/><rect x="47" y="31" width="34" height="5" fill="{a}"/>"##,
            coins(p)
        ),
        (Age::Fantasy, Role::Merchant) => format!(
            r##"{}<ellipse cx="64" cy="36" rx="27" ry="13" fill="{b2}" stroke="{ink}" stroke-width="1.5"/><path d="M40 34 Q64 46 88 34 M44 28 Q64 38 84 28" fill="none" stroke="{ink}" stroke-opacity="0.5"/><circle cx="64" cy="38" r="4" fill="{a}" stroke="{ink}"{glow}/><path d="M66 34 C70 20 78 14 84 12 C78 20 74 26 70 36 Z" fill="{light}" stroke="{ink}"/>"##,
            coins(p)
        ),
        (Age::Cyberpunk, Role::Merchant) => format!(
            r##"<path d="M40 46 C40 25 88 25 88 46 Z" fill="#20232b" stroke="{ink}" stroke-width="1.5"/><path d="M84 43 L106 46 L86 49 Z" fill="#20232b" stroke="{ink}"/><path d="M42 40 L86 40" stroke="{a}" stroke-width="2"{glow}/><circle cx="40" cy="69" r="2.5" fill="{b2}"{glow}/><rect x="84" y="104" width="22" height="14" rx="2" fill="#111" stroke="{b2}" stroke-width="1.5"{glow}/><path d="M88 109 L100 109 M88 113 L96 113" stroke="{a}" stroke-width="1.5"/>"##
        ),
        (Age::Realistic, Role::Scholar) => format!(
            r##"<circle cx="55" cy="58" r="6" fill="none" stroke="{ink}" stroke-width="1.8"/><circle cx="73" cy="58" r="6" fill="none" stroke="{ink}" stroke-width="1.8"/><path d="M61 58 L67 58" stroke="{ink}" stroke-width="1.8"/><rect x="80" y="102" width="24" height="17" rx="2" fill="{b2}" stroke="{ink}" stroke-width="1.5"/><path d="M84 102 L84 119" stroke="{light}" stroke-width="2"/><path d="M28 126 C32 106 40 96 52 90 C46 102 40 114 32 126 Z" fill="{light}" stroke="{ink}"/>"##
        ),
        (Age::Fantasy, Role::Scholar) => format!(
            r##"<path d="M104 130 L104 72" stroke="#6b4a2b" stroke-width="4" stroke-linecap="round"/><circle cx="104" cy="66" r="7" fill="{b2}" stroke="{ink}"{glow}/><path d="M36 47 L60 5 Q66 0 69 8 L92 47 Z" fill="{attire}" stroke="{ink}" stroke-width="2"/><ellipse cx="64" cy="47" rx="35" ry="6" fill="{attire}" stroke="{ink}" stroke-width="2"/>{}{}"##,
            star(60.0, 32.0, 5.0, 5, 0.45, a, p),
            star(72.0, 20.0, 3.5, 5, 0.45, a, p)
        ),
        (Age::Cyberpunk, Role::Scholar) => format!(
            r##"{}<path d="M40 58 C40 28 88 28 88 58" fill="none" stroke="{metal}" stroke-width="4"/><rect x="35" y="52" width="9" height="16" rx="3" fill="{metal}" stroke="{ink}"/><rect x="84" y="52" width="9" height="16" rx="3" fill="{metal}" stroke="{ink}"/><path d="M39 67 Q42 78 56 77" fill="none" stroke="{metal}" stroke-width="2"/><rect x="46" y="53" width="36" height="9" rx="4" fill="{b2}" fill-opacity="0.55"{glow}/>"##,
            hood(attire, ink)
        ),
        (Age::Realistic, Role::Adventurer) => format!(
            r##"<path d="M36 96 L90 130" stroke="#3b2614" stroke-width="5"/><path d="M47 86 Q64 97 81 86 L83 95 Q64 105 45 95 Z" fill="{b2}" stroke="{ink}" stroke-width="1.5"/><ellipse cx="64" cy="40" rx="35" ry="8" fill="#5a3d22" stroke="{ink}" stroke-width="1.5"/><path d="M44 40 Q46 17 64 19 Q82 17 84 40 Z" fill="#5a3d22" stroke="{ink}" stroke-width="1.5"/><path d="M45 34 Q64 38 83 34" stroke="{a}" stroke-width="2.5" fill="none"/>"##
        ),
        (Age::Fantasy, Role::Adventurer) => format!(
            r##"<path d="M100 74 Q122 104 100 134" fill="none" stroke="#8a5d33" stroke-width="4"/><path d="M100 74 L100 134" stroke="{light}" stroke-width="1"/><path d="M22 96 L30 74 M28 96 L36 76" stroke="#8a5d33" stroke-width="2.5"/><path d="M27 72 L30 64 L33 72 Z M33 74 L36 66 L39 74 Z" fill="{b2}"/>{}"##,
            hood(attire, ink)
        ),
        (Age::Cyberpunk, Role::Adventurer) => format!(
            r##"<path d="M54 38 L56 14 L61 30 L64 8 L67 30 L72 14 L74 38 Q64 34 54 38 Z" fill="{a}" stroke="{ink}"{glow}/><circle cx="73" cy="58" r="6" fill="#1a1a1a" stroke="{metal}" stroke-width="2"/><circle cx="73" cy="58" r="2.6" fill="#ff3344"{glow}/><path d="M80 52 L86 50 M80 64 L86 66" stroke="{metal}" stroke-width="1.5"/><path d="M49 62 L55 71" stroke="#b33" stroke-width="1.5"/><path d="M22 104 Q30 92 44 94 L42 108 Z" fill="{metal}" stroke="{ink}"/>"##
        ),
    }
}

// --- Piezas de los dibujos ---------------------------------------------------

/// Estrella de `points` puntas centrada en (`cx`, `cy`).
fn star(cx: f32, cy: f32, r: f32, points: usize, inner: f32, fill: &str, p: &Palette) -> String {
    let mut d = String::new();
    for i in 0..points * 2 {
        let radius = if i % 2 == 0 { r } else { r * inner };
        let angle = i as f32 / (points * 2) as f32 * TAU - TAU / 4.0;
        let (x, y) = (cx + radius * angle.cos(), cy + radius * angle.sin());
        put!(d, "{}{x:.1} {y:.1} ", if i == 0 { "M" } else { "L" });
    }
    format!(
        r##"<path d="{d}Z" fill="{fill}" stroke="{}" stroke-width="1" stroke-linejoin="round"{}/>"##,
        p.ink,
        p.glow()
    )
}

/// Una espada (o daga) con la guarda en (`x`, `y`), girada `angle` grados.
fn sword(x: f32, y: f32, len: f32, angle: f32, blade: &str, p: &Palette, glow: bool) -> String {
    let (ink, a) = (p.ink, p.accent);
    let tip = -len;
    let shoulder = -len + 10.0;
    let glow = if glow { p.glow() } else { "" };
    format!(
        r##"<g transform="translate({x} {y}) rotate({angle})"><path d="M-4 0 L-4 {shoulder} L0 {tip} L4 {shoulder} L4 0 Z" fill="{blade}" stroke="{ink}" stroke-width="1.5"{glow}/><rect x="-13" y="0" width="26" height="5" rx="2" fill="{a}" stroke="{ink}" stroke-width="1.2"/><rect x="-3" y="5" width="6" height="14" fill="#5a3a22" stroke="{ink}" stroke-width="1.2"/><circle cx="0" cy="22" r="4" fill="{a}" stroke="{ink}" stroke-width="1.2"/></g>"##
    )
}

fn crossed_swords(p: &Palette, age: Age) -> String {
    let blade = if age == Age::Cyberpunk {
        p.accent2
    } else {
        p.metal
    };
    let neon = age == Age::Cyberpunk;
    sword(38.0, 96.0, 74.0, 42.0, blade, p, neon) + &sword(90.0, 96.0, 74.0, -42.0, blade, p, neon)
}

fn skull(cx: f32, cy: f32, scale: f32, p: &Palette) -> String {
    let ink = p.ink;
    format!(
        r##"<g transform="translate({cx} {cy}) scale({scale})"><path d="M-20 4 C-25 -24 25 -24 20 4 L14 8 L14 18 L-14 18 L-14 8 Z" fill="#ece6d6" stroke="{ink}" stroke-width="2"/><circle cx="-8" cy="-2" r="5" fill="{ink}"/><circle cx="8" cy="-2" r="5" fill="{ink}"/><path d="M0 4 L-3 10 L3 10 Z" fill="{ink}"/><path d="M-8 18 L-8 12 M0 18 L0 12 M8 18 L8 12" stroke="{ink}" stroke-width="1.5"/></g>"##
    )
}

fn ground(p: &Palette) -> String {
    format!(
        r##"<path d="M0 108 Q64 96 128 108 L128 128 L0 128 Z" fill="{}" stroke="{}" stroke-width="1.5"/>"##,
        p.ground, p.ink
    )
}

/// Una casa de pie sobre `base`, de `w` de ancho.
fn house(x: f32, base: f32, w: f32, h: f32, p: &Palette) -> String {
    let (ink, roof, wall) = (p.ink, p.accent2, p.light);
    let top = base - h;
    let peak = top - w * 0.45;
    let (right, mid) = (x + w, x + w / 2.0);
    let door = w * 0.22;
    format!(
        r##"<path d="M{x} {base} L{x} {top} L{mid} {peak} L{right} {top} L{right} {base} Z" fill="{wall}" stroke="{ink}" stroke-width="2"/><path d="M{} {} L{mid} {peak} L{} {}" fill="none" stroke="{roof}" stroke-width="5" stroke-linejoin="round" stroke-linecap="round"/><rect x="{}" y="{}" width="{door}" height="{}" fill="{ink}"/>"##,
        x - 3.0,
        top + 2.0,
        right + 3.0,
        top + 2.0,
        mid - door / 2.0,
        base - h * 0.45,
        h * 0.45
    )
}

/// Una torre de oficinas con ventanas encendidas (cyberpunk).
fn tower(x: f32, base: f32, w: f32, h: f32, p: &Palette, seed: u32) -> String {
    let mut s = format!(
        r##"<rect x="{x}" y="{}" width="{w}" height="{h}" fill="#151826" stroke="{}" stroke-width="1.5"/>"##,
        base - h,
        p.accent2
    );
    let mut i = seed;
    let mut y = base - h + 5.0;
    while y < base - 6.0 {
        let mut wx = x + 4.0;
        while wx < x + w - 6.0 {
            i = i.wrapping_mul(1_103_515_245).wrapping_add(12_345);
            if !(i >> 16).is_multiple_of(3) {
                let color = if (i >> 20).is_multiple_of(4) {
                    p.accent
                } else {
                    p.accent2
                };
                put!(
                    s,
                    r##"<rect x="{wx}" y="{y}" width="3" height="4" fill="{color}" fill-opacity="0.85"/>"##
                );
            }
            wx += 6.0;
        }
        y += 8.0;
    }
    s
}

fn town(p: &Palette, age: Age) -> String {
    if age == Age::Cyberpunk {
        tower(18.0, 112.0, 26.0, 60.0, p, 1)
            + &tower(48.0, 112.0, 30.0, 86.0, p, 2)
            + &tower(82.0, 112.0, 28.0, 48.0, p, 3)
    } else {
        ground(p) + &house(18.0, 106.0, 36.0, 30.0, p) + &house(60.0, 104.0, 48.0, 40.0, p)
    }
}

fn flag(x: f32, y: f32, scale: f32, p: &Palette) -> String {
    format!(
        r##"<g transform="translate({x} {y}) scale({scale})"><path d="M0 0 L0 -40" stroke="{ink}" stroke-width="3" stroke-linecap="round"/><path d="M1 -40 C9 -44 15 -34 26 -38 L26 -22 C15 -18 9 -28 1 -24 Z" fill="{}" stroke="{ink}" stroke-width="1.5"{}/></g>"##,
        p.accent,
        p.glow(),
        ink = p.ink
    )
}

fn cloud(cx: f32, cy: f32, fill: &str, p: &Palette) -> String {
    format!(
        r##"<g transform="translate({cx} {cy})" fill="{fill}" stroke="{}" stroke-width="2"><circle cx="-22" cy="4" r="15"/><circle cx="22" cy="4" r="15"/><circle cx="0" cy="-8" r="20"/><rect x="-22" y="4" width="44" height="15" stroke="none"/></g>"##,
        p.ink
    )
}

fn shield(x: f32, y: f32, angle: f32, fill: &str, p: &Palette) -> String {
    format!(
        r##"<path transform="translate({x} {y}) rotate({angle})" d="M0 -26 L24 -18 L22 8 Q16 26 0 32 Q-16 26 -22 8 L-24 -18 Z" fill="{fill}" stroke="{}" stroke-width="2.5"/>"##,
        p.ink
    )
}

fn gear(cx: f32, cy: f32, r: f32, teeth: usize, fill: &str, p: &Palette) -> String {
    let mut d = String::new();
    let step = TAU / teeth as f32;
    for i in 0..teeth {
        let a = i as f32 * step;
        for (offset, radius) in [(0.0, r * 0.8), (0.18, r), (0.5 - 0.18, r), (0.5, r * 0.8)] {
            let angle = a + offset * step;
            put!(
                d,
                "{}{:.1} {:.1} ",
                if d.is_empty() { "M" } else { "L" },
                cx + radius * angle.cos(),
                cy + radius * angle.sin()
            );
        }
        let angle = a + step;
        put!(
            d,
            "L{:.1} {:.1} ",
            cx + r * 0.8 * angle.cos(),
            cy + r * 0.8 * angle.sin()
        );
    }
    format!(
        r##"<path d="{d}Z" fill="{fill}" stroke="{ink}" stroke-width="2" stroke-linejoin="round"/><circle cx="{cx}" cy="{cy}" r="{}" fill="{}" stroke="{ink}" stroke-width="2"/>"##,
        r * 0.32,
        p.bottom,
        ink = p.ink
    )
}

fn sparkles(p: &Palette, at: &[(f32, f32, f32)]) -> String {
    at.iter()
        .map(|&(x, y, r)| star(x, y, r, 4, 0.35, p.light, p))
        .collect()
}

// --- Dibujos de eventos -----------------------------------------------------

fn motif(m: Motif, age: Age, p: &Palette) -> String {
    let (ink, a, b2, metal, light, glow) = (p.ink, p.accent, p.accent2, p.metal, p.light, p.glow());
    match m {
        Motif::Settlement => town(p, age) + &flag(84.0, 50.0, 0.8, p),
        Motif::Banner => format!(
            r##"<path d="M40 114 L40 20" stroke="{ink}" stroke-width="4" stroke-linecap="round"/><circle cx="40" cy="18" r="4" fill="{a}" stroke="{ink}"/><path d="M42 24 C62 16 74 34 98 26 L98 66 C74 74 62 56 42 64 Z" fill="{b2}" stroke="{ink}" stroke-width="2"{glow}/>{}"##,
            star(70.0, 45.0, 11.0, 5, 0.45, a, p)
        ),
        Motif::Birth => format!(
            r##"{}<path d="M30 76 Q64 118 98 76 Z" fill="{b2}" stroke="{ink}" stroke-width="2"/><path d="M26 104 Q64 120 102 104" fill="none" stroke="{ink}" stroke-width="3" stroke-linecap="round"/><circle cx="54" cy="72" r="9" fill="#f1d0b5" stroke="{ink}" stroke-width="1.5"/><path d="M60 77 Q78 64 94 76 Z" fill="{light}" stroke="{ink}" stroke-width="1.5"/>"##,
            star(64.0, 34.0, 15.0, 5, 0.45, a, p)
        ),
        Motif::Rise => {
            let leaf = match age {
                Age::Realistic => "#5c8a3a",
                Age::Fantasy => a,
                Age::Cyberpunk => b2,
            };
            let mut s = String::new();
            for i in 0..7 {
                for side in [-1.0_f32, 1.0] {
                    let deg = 90.0 + side * (40.0 + i as f32 * 19.0);
                    let rad = deg.to_radians();
                    let (x, y) = (64.0 + 36.0 * rad.cos(), 64.0 + 36.0 * rad.sin());
                    put!(
                        s,
                        r##"<ellipse cx="{x:.1}" cy="{y:.1}" rx="8" ry="3.4" transform="rotate({:.0} {x:.1} {y:.1})" fill="{leaf}" stroke="{ink}" stroke-width="1"/>"##,
                        deg + 90.0 + side * 25.0
                    );
                }
            }
            s + &star(64.0, 62.0, 20.0, 5, 0.45, a, p)
        }
        Motif::Crown if age == Age::Cyberpunk => format!(
            r##"<ellipse cx="64" cy="98" rx="40" ry="9" fill="none" stroke="{b2}" stroke-width="2"{glow}/><ellipse cx="64" cy="98" rx="26" ry="5" fill="none" stroke="{a}" stroke-width="1.5"{glow}/><path d="M64 98 L64 86" stroke="{b2}" stroke-width="1" stroke-dasharray="2 2"/><g transform="translate(64 60) scale(1.8) translate(-64 -32)" fill-opacity="0.35">{}</g>"##,
            crown(b2, a, p)
        ),
        Motif::Crown => format!(
            r##"<ellipse cx="64" cy="102" rx="40" ry="10" fill="{b2}" stroke="{ink}" stroke-width="2"/><g transform="translate(64 64) scale(2) translate(-64 -32)"{glow}>{}</g>"##,
            crown(a, if age == Age::Fantasy { b2 } else { p.blood }, p)
        ),
        Motif::Grave if age == Age::Cyberpunk => format!(
            r##"<rect x="20" y="30" width="88" height="60" rx="6" fill="#0a0f14" stroke="{metal}" stroke-width="3"/><path d="M26 62 L50 62 L56 46 L62 80 L68 62 L102 62" fill="none" stroke="{b2}" stroke-width="3" stroke-linejoin="round"{glow}/><path d="M56 90 L52 104 L76 104 L72 90" fill="{metal}" stroke="{ink}"/>"##
        ),
        Motif::Grave => format!(
            r##"{}<path d="M42 106 L42 58 Q42 34 64 34 Q86 34 86 58 L86 106 Z" fill="{}" stroke="{ink}" stroke-width="2"/><path d="M64 46 L64 70 M55 54 L73 54" stroke="{ink}" stroke-width="3" stroke-linecap="round"/><path d="M52 82 L76 82 M54 90 L74 90" stroke="{ink}" stroke-opacity="0.5" stroke-width="2"/>"##,
            ground(p),
            p.stone
        ),
        Motif::Fallen => format!(
            r##"{}{}<path d="M20 106 C20 78 58 78 58 106 Z" fill="{metal}" stroke="{ink}" stroke-width="2"/><path d="M16 106 L62 106" stroke="{ink}" stroke-width="3"/><path d="M39 81 L39 106" stroke="{ink}" stroke-width="1.5"/>"##,
            sword(76.0, 38.0, 66.0, 180.0, metal, p, false),
            ground(p)
        ),
        Motif::Dagger => format!(
            r##"{}<path d="M88 92 Q80 104 88 110 Q96 104 88 92 Z" fill="{}" stroke="{ink}"/><path d="M98 104 Q94 110 98 113 Q102 110 98 104 Z" fill="{}"/>"##,
            sword(54.0, 84.0, 52.0, 35.0, metal, p, age == Age::Cyberpunk),
            p.blood,
            p.blood
        ),
        Motif::Skull => ground(p) + &skull(64.0, 66.0, 1.7, p),
        Motif::Growth => {
            let (small, big) = if age == Age::Cyberpunk {
                (
                    tower(16.0, 112.0, 24.0, 34.0, p, 4),
                    tower(80.0, 112.0, 32.0, 84.0, p, 5),
                )
            } else {
                (
                    ground(p) + &house(14.0, 106.0, 26.0, 20.0, p),
                    house(76.0, 104.0, 40.0, 46.0, p),
                )
            };
            format!(
                r##"{small}{big}<path d="M54 90 L54 46 M42 58 L54 44 L66 58" stroke="{a}" stroke-width="6" fill="none" stroke-linecap="round" stroke-linejoin="round"{glow}/>"##
            )
        }
        Motif::Ruins => format!(
            r##"{}<circle cx="44" cy="44" r="9" fill="{metal}" fill-opacity="0.45"/><circle cx="54" cy="28" r="12" fill="{metal}" fill-opacity="0.35"/><circle cx="68" cy="14" r="14" fill="{metal}" fill-opacity="0.25"/><path d="M20 108 L20 64 L30 58 L34 70 L42 52 L48 66 L58 60 L58 108 Z" fill="{stone}" stroke="{ink}" stroke-width="2"/><path d="M70 108 L70 80 L80 74 L86 86 L96 76 L98 108 Z" fill="{stone}" stroke="{ink}" stroke-width="2"/><path d="M20 78 L58 78 M20 92 L58 92 M70 92 L98 92 M38 64 L38 78 M30 78 L30 92 M46 78 L46 92 M84 92 L84 108" stroke="{ink}" stroke-opacity="0.5"/><path d="M60 108 L64 100 L68 108 Z M100 108 L106 102 L110 108 Z" fill="{stone}" stroke="{ink}"/>"##,
            ground(p),
            stone = p.stone
        ),
        Motif::Annex => format!(
            "{}{}{}",
            town(p, age),
            flag(84.0, 50.0, 0.8, p),
            flag(34.0, 66.0, 0.6, p)
        ),
        Motif::Alliance => format!(
            r##"<circle cx="50" cy="64" r="22" fill="none" stroke="{a}" stroke-width="8"{glow}/><circle cx="78" cy="64" r="22" fill="none" stroke="{b2}" stroke-width="8"{glow}/><path d="M72 43 A22 22 0 0 1 72 85" fill="none" stroke="{a}" stroke-width="8"/>{}"##,
            sparkles(p, &[(24.0, 30.0, 5.0), (104.0, 96.0, 6.0)])
        ),
        Motif::BrokenAlliance => format!(
            r##"<circle cx="38" cy="64" r="20" fill="none" stroke="{a}" stroke-width="8"/><circle cx="90" cy="64" r="20" fill="none" stroke="{b2}" stroke-width="8"/><path d="M64 26 L57 50 L70 60 L58 84 L72 104" stroke="{}" stroke-width="4" fill="none" stroke-linejoin="round"{glow}/>"##,
            p.blood
        ),
        Motif::Swords => crossed_swords(p, age),
        Motif::Battle => {
            let burst = match age {
                Age::Realistic => "#e8892a",
                Age::Fantasy => b2,
                Age::Cyberpunk => a,
            };
            star(64.0, 62.0, 48.0, 14, 0.55, burst, p)
                + &star(64.0, 62.0, 30.0, 10, 0.55, light, p)
                + &crossed_swords(p, age)
        }
        Motif::Fortress if age == Age::Cyberpunk => format!(
            "{}<path d=\"M62 26 L62 8\" stroke=\"{metal}\" stroke-width=\"2\"/><circle cx=\"62\" cy=\"7\" r=\"3\" fill=\"{a}\"{glow}/>{}",
            tower(36.0, 116.0, 52.0, 90.0, p, 6),
            flag(62.0, 26.0, 0.6, p)
        ),
        Motif::Fortress => {
            let mut s = ground(p);
            put!(
                s,
                r##"<rect x="36" y="42" width="56" height="66" fill="{}" stroke="{ink}" stroke-width="2"/>"##,
                p.stone
            );
            for i in 0..4 {
                put!(
                    s,
                    r##"<rect x="{}" y="30" width="9" height="13" fill="{}" stroke="{ink}" stroke-width="2"/>"##,
                    36 + i * 15,
                    p.stone
                );
            }
            put!(
                s,
                r##"<path d="M54 108 L54 88 Q64 76 74 88 L74 108 Z" fill="{ink}"/><path d="M36 62 L92 62 M36 80 L54 80 M74 80 L92 80" stroke="{ink}" stroke-opacity="0.4"/>"##
            );
            s + &flag(64.0, 30.0, 0.7, p)
        }
        Motif::Peace if age == Age::Cyberpunk => format!(
            r##"<rect x="30" y="22" width="68" height="88" rx="8" fill="#0d121c" stroke="{metal}" stroke-width="3"/><path d="M40 40 L88 40 M40 52 L80 52 M40 64 L86 64" stroke="{b2}" stroke-width="3" stroke-linecap="round"{glow}/><path d="M48 86 L58 96 L80 76" fill="none" stroke="{a}" stroke-width="5" stroke-linecap="round" stroke-linejoin="round"{glow}/>"##
        ),
        Motif::Peace => format!(
            r##"<rect x="30" y="32" width="68" height="64" fill="{light}" stroke="{ink}" stroke-width="2"/><rect x="24" y="26" width="80" height="11" rx="5.5" fill="{light}" stroke="{ink}" stroke-width="2"/><rect x="24" y="91" width="80" height="11" rx="5.5" fill="{light}" stroke="{ink}" stroke-width="2"/><path d="M40 48 L88 48 M40 58 L84 58 M40 68 L70 68" stroke="{ink}" stroke-opacity="0.6" stroke-width="2"/><circle cx="84" cy="80" r="8" fill="{}" stroke="{ink}" stroke-width="1.5"/><path d="M96 20 C104 30 104 40 98 52 L92 60 C94 46 94 34 96 20 Z" fill="{light}" stroke="{ink}"/>"##,
            p.blood
        ),
        Motif::Capital => format!(
            r##"<circle cx="64" cy="64" r="44" fill="none" stroke="{b2}" stroke-width="4"{glow}/><circle cx="64" cy="64" r="36" fill="none" stroke="{a}" stroke-width="1.5"/>{}"##,
            star(64.0, 66.0, 32.0, 5, 0.42, a, p)
        ),
        Motif::Fall => format!(
            r##"{}<g transform="translate(62 74) rotate(22) scale(1.7) translate(-64 -32)">{}</g><path d="M74 38 L64 58 L76 72 L66 94" stroke="{ink}" stroke-width="3" fill="none" stroke-linejoin="round"/><path d="M28 30 L34 40 L24 40 Z M98 24 L104 30 L96 34 Z M20 64 L28 66 L22 72 Z" fill="{a}" stroke="{ink}"/>"##,
            ground(p),
            crown(a, p.blood, p)
        ),
        Motif::BrokenChain => {
            let mut s = String::new();
            for (x, angle) in [(8.0, -12.0), (30.0, 10.0), (72.0, -10.0), (94.0, 12.0)] {
                put!(
                    s,
                    r##"<rect x="{x}" y="56" width="28" height="16" rx="8" fill="none" stroke="{metal}" stroke-width="6" transform="rotate({angle} {} 64)"/>"##,
                    x + 14.0
                );
            }
            s + &star(64.0, 62.0, 16.0, 8, 0.4, a, p)
        }
        Motif::Plague if age == Age::Cyberpunk => {
            let virus = "#9cff3b";
            let mut s = String::new();
            for i in 0..12 {
                let angle = i as f32 / 12.0 * TAU;
                let (c, si) = (angle.cos(), angle.sin());
                put!(
                    s,
                    r##"<path d="M{:.1} {:.1} L{:.1} {:.1}" stroke="{virus}" stroke-width="3"/><circle cx="{:.1}" cy="{:.1}" r="4.5" fill="{virus}"{glow}/>"##,
                    64.0 + 24.0 * c,
                    64.0 + 24.0 * si,
                    64.0 + 38.0 * c,
                    64.0 + 38.0 * si,
                    64.0 + 40.0 * c,
                    64.0 + 40.0 * si
                );
            }
            put!(
                s,
                r##"<circle cx="64" cy="64" r="26" fill="#1f3a12" stroke="{virus}" stroke-width="3"{glow}/><circle cx="56" cy="58" r="5" fill="{virus}" fill-opacity="0.6"/><circle cx="72" cy="70" r="7" fill="{virus}" fill-opacity="0.4"/>"##
            );
            s
        }
        Motif::Plague => {
            let miasma = if age == Age::Fantasy {
                "#7dff8a"
            } else {
                "#7a9a3a"
            };
            let mut s = String::new();
            for (x, y, r) in [(30, 40, 14), (96, 36, 16), (24, 84, 12), (104, 82, 14)] {
                put!(
                    s,
                    r##"<circle cx="{x}" cy="{y}" r="{r}" fill="{miasma}" fill-opacity="0.4"{glow}/>"##
                );
            }
            s + &skull(64.0, 62.0, 1.6, p)
        }
        Motif::Rings => format!(
            r##"<circle cx="52" cy="72" r="20" fill="none" stroke="{a}" stroke-width="7"{glow}/><circle cx="76" cy="72" r="20" fill="none" stroke="{}" stroke-width="7"{glow}/><path d="M76 40 L84 48 L76 56 L68 48 Z" fill="{light}" stroke="{ink}" stroke-width="1.5"{glow}/>{}"##,
            if age == Age::Cyberpunk { b2 } else { a },
            sparkles(p, &[(30.0, 36.0, 6.0), (102.0, 30.0, 5.0)])
        ),
        Motif::Plot => format!(
            r##"{}<path d="M18 60 Q64 20 110 60 Q64 100 18 60 Z" fill="{light}" stroke="{ink}" stroke-width="2.5"/><circle cx="64" cy="60" r="15" fill="{b2}" stroke="{ink}" stroke-width="2"{glow}/><circle cx="64" cy="60" r="6" fill="{ink}"/><circle cx="60" cy="56" r="2.5" fill="{light}"/>"##,
            sword(34.0, 112.0, 60.0, 60.0, metal, p, false)
        ),
        Motif::Lineage => {
            let mut s = format!(
                r##"<path d="M64 46 L40 74 M64 46 L88 74 M40 74 L28 104 M40 74 L52 104 M88 74 L76 104 M88 74 L100 104" stroke="{ink}" stroke-width="3" fill="none"/>"##
            );
            for (x, y) in [
                (40, 74),
                (88, 74),
                (28, 104),
                (52, 104),
                (76, 104),
                (100, 104),
            ] {
                put!(
                    s,
                    r##"<circle cx="{x}" cy="{y}" r="7" fill="{b2}" stroke="{ink}" stroke-width="1.5"/>"##
                );
            }
            put!(
                s,
                r##"<circle cx="64" cy="46" r="10" fill="{a}" stroke="{ink}" stroke-width="2"{glow}/><g transform="translate(64 26) scale(0.7) translate(-64 -32)">{}</g>"##,
                crown(a, b2, p)
            );
            s
        }
        Motif::Sun => {
            let mut s = String::new();
            for i in 0..12 {
                let angle = i as f32 / 12.0 * TAU;
                let side = TAU / 48.0;
                let point = |r: f32, d: f32| {
                    (
                        64.0 + r * (angle + d).cos(),
                        64.0 + r * (angle + d).sin(),
                    )
                };
                let ((x1, y1), (x2, y2), (x3, y3)) =
                    (point(30.0, -side), point(50.0, 0.0), point(30.0, side));
                put!(
                    s,
                    r##"<path d="M{x1:.1} {y1:.1} L{x2:.1} {y2:.1} L{x3:.1} {y3:.1} Z" fill="{a}" stroke="{ink}" stroke-width="1"{glow}/>"##
                );
            }
            put!(
                s,
                r##"<circle cx="64" cy="64" r="26" fill="{a}" stroke="{ink}" stroke-width="2"{glow}/><circle cx="64" cy="64" r="18" fill="{light}" fill-opacity="0.5"/>"##
            );
            s
        }
        Motif::Rivalry => format!(
            "{}{}<path d=\"M66 22 L56 58 L70 58 L60 104\" stroke=\"{}\" stroke-width=\"4\" fill=\"none\" stroke-linejoin=\"round\"{glow}/>",
            shield(36.0, 64.0, -12.0, a, p),
            shield(92.0, 64.0, 12.0, b2, p),
            p.blood
        ),
        Motif::Calamity(c) => calamity(c, p),
        Motif::Feat(f) => feat(f, p),
    }
}

fn calamity(c: Calamity, p: &Palette) -> String {
    let (ink, a, b2, metal, light, glow) = (p.ink, p.accent, p.accent2, p.metal, p.light, p.glow());
    match c {
        Calamity::Famine => format!(
            r##"<circle cx="98" cy="28" r="14" fill="#e8a33a" stroke="{ink}" stroke-width="1.5"/>{}<path d="M20 116 L30 108 L26 100 M58 118 L64 108 L72 112 M96 116 L102 106" stroke="{ink}" stroke-width="1.5" fill="none"/><path d="M36 108 Q34 80 44 62 M58 108 Q60 78 52 58 M80 108 Q78 84 90 66" stroke="#a08850" stroke-width="3" fill="none"/><ellipse cx="46" cy="60" rx="4" ry="9" transform="rotate(130 46 60)" fill="#b89a5a" stroke="{ink}"/><ellipse cx="50" cy="58" rx="4" ry="9" transform="rotate(200 50 58)" fill="#b89a5a" stroke="{ink}"/><ellipse cx="92" cy="66" rx="4" ry="9" transform="rotate(140 92 66)" fill="#b89a5a" stroke="{ink}"/>"##,
            ground(p)
        ),
        Calamity::Flood => {
            let mut s = format!(
                r##"<path d="M44 58 L64 40 L84 58 Z" fill="{}" stroke="{ink}" stroke-width="2"/><rect x="48" y="58" width="32" height="20" fill="{light}" stroke="{ink}" stroke-width="2"/><path d="M22 14 L16 28 M42 10 L36 24 M94 12 L88 26 M112 18 L106 32" stroke="#6aa6d8" stroke-width="2"/>"##,
                p.accent2
            );
            for (y, color) in [(74.0, "#2f6fa8"), (90.0, "#3f86c4"), (106.0, "#5aa0d8")] {
                put!(
                    s,
                    r##"<path d="M0 {y} Q16 {} 32 {y} T64 {y} T96 {y} T128 {y} L128 128 L0 128 Z" fill="{color}" stroke="{ink}" stroke-width="1.5"/>"##,
                    y - 8.0
                );
            }
            s
        }
        Calamity::Earthquake => format!(
            r##"<path d="M0 80 L58 80 L66 92 L56 104 L62 128 L0 128 Z" fill="{g}" stroke="{ink}" stroke-width="2"/><path d="M128 88 L70 88 L76 98 L68 110 L74 128 L128 128 Z" fill="{g}" stroke="{ink}" stroke-width="2"/><g transform="rotate(-14 30 80)">{}</g><path d="M86 40 Q92 50 86 60 M96 34 Q106 50 96 66 M106 28 Q120 50 106 72" stroke="{ink}" stroke-width="2" fill="none"/>"##,
            house(16.0, 80.0, 30.0, 24.0, p),
            g = p.ground
        ),
        Calamity::Fire => {
            let flame = "M64 16 C80 40 98 52 94 80 C92 100 78 110 64 110 C50 110 36 100 34 80 C32 62 46 52 50 36 C54 50 58 54 62 56 C60 42 60 30 64 16 Z";
            format!(
                r##"<path d="M30 112 L98 104 M34 104 L96 114" stroke="#5a3a22" stroke-width="7" stroke-linecap="round"/><path d="{flame}" fill="#e24a1b" stroke="{ink}" stroke-width="2"/><path transform="translate(64 110) scale(0.7) translate(-64 -110)" d="{flame}" fill="#f59a23"/><path transform="translate(64 110) scale(0.4) translate(-64 -110)" d="{flame}" fill="#ffe066"/>"##
            )
        }
        Calamity::Dragon => format!(
            r##"<circle cx="80" cy="44" r="30" fill="{a}" fill-opacity="0.9"{glow}/><g fill="#120a1f" stroke="#120a1f"><path d="M14 94 C36 80 56 86 70 76 C78 70 84 62 96 60 L106 54 L102 66 C96 72 90 80 80 86 C64 98 40 102 14 94 Z"/><path d="M56 80 C48 50 38 36 20 26 C30 40 32 50 30 58 C26 52 20 50 12 50 C22 60 28 72 44 84 Z"/><path d="M66 76 C70 46 82 30 104 20 C94 36 94 46 96 56 Z"/><path d="M14 94 C8 100 10 108 18 108"/></g><path d="M106 58 Q114 54 122 62 Q114 60 118 70 Q110 62 106 64 Z" fill="#ff7a1a"{glow}/>"##
        ),
        Calamity::Undead => format!(
            r##"{}<path d="M86 104 L86 70 Q86 56 100 56 Q114 56 114 70 L114 104 Z" fill="{}" stroke="{ink}" stroke-width="2"/><ellipse cx="54" cy="104" rx="28" ry="8" fill="#7dff8a" fill-opacity="0.35"{glow}/><g stroke="#ece6d6" stroke-linecap="round" fill="none"><path d="M54 110 L56 76" stroke-width="7"/><path d="M48 70 L42 46 M54 66 L52 38 M60 66 L64 40 M64 72 L74 52 M50 76 L36 66" stroke-width="4"/></g><ellipse cx="56" cy="72" rx="10" ry="8" fill="#ece6d6" stroke="{ink}"/>"##,
            ground(p),
            p.stone
        ),
        Calamity::ArcaneStorm => format!(
            r##"{}<path d="M66 70 L52 96 L64 96 L56 122 L82 88 L68 88 L78 70 Z" fill="#c78bff" stroke="{ink}" stroke-width="1.5"{glow}/>{}"##,
            cloud(64.0, 44.0, "#4a3a7a", p),
            sparkles(
                p,
                &[(24.0, 82.0, 6.0), (104.0, 92.0, 7.0), (98.0, 18.0, 5.0)]
            )
        ),
        Calamity::DemonGate => format!(
            r##"<ellipse cx="64" cy="64" rx="32" ry="44" fill="#1a0505" stroke="#ff5a1f" stroke-width="8"{glow}/><ellipse cx="64" cy="64" rx="20" ry="30" fill="none" stroke="#ff9a3c" stroke-width="2" stroke-dasharray="6 4"/><path d="M50 50 L46 38 L56 46 M78 50 L82 38 L72 46" fill="#3a0a0a" stroke="#ff5a1f" stroke-width="2"/><ellipse cx="57" cy="60" rx="4" ry="2.5" fill="#ff3a1a"{glow}/><ellipse cx="71" cy="60" rx="4" ry="2.5" fill="#ff3a1a"{glow}/>"##
        ),
        Calamity::Blackout => format!(
            r##"<rect x="10" y="62" width="24" height="54" fill="#0e1018" stroke="{metal}"/><rect x="38" y="42" width="30" height="74" fill="#0e1018" stroke="{metal}"/><rect x="72" y="70" width="22" height="46" fill="#0e1018" stroke="{metal}"/><rect x="98" y="54" width="22" height="62" fill="#0e1018" stroke="{metal}"/><rect x="44" y="50" width="4" height="5" fill="{a}" fill-opacity="0.6"/><path d="M68 14 L52 44 L64 44 L56 72 L80 36 L68 36 L76 14 Z" fill="{b2}" stroke="{ink}" stroke-width="1.5"{glow}/><path d="M38 20 L98 70" stroke="{}" stroke-width="6" stroke-linecap="round"/>"##,
            p.blood
        ),
        Calamity::RogueAi => format!(
            r##"<path d="M10 64 L24 64 M104 64 L118 64 M64 10 L64 24 M64 104 L64 118" stroke="{b2}" stroke-width="2"/><circle cx="64" cy="64" r="40" fill="#111" stroke="{metal}" stroke-width="6"/><circle cx="64" cy="64" r="26" fill="#3a0000"/><circle cx="64" cy="64" r="16" fill="#b00000"{glow}/><circle cx="64" cy="64" r="8" fill="#ff3030"{glow}/><circle cx="60" cy="60" r="3" fill="#ffd0d0"/>"##
        ),
        Calamity::AcidRain => {
            let mut s = cloud(64.0, 36.0, "#2a3a2a", p);
            for (x, y) in [(36, 70), (54, 84), (72, 68), (90, 82), (44, 100), (80, 102)] {
                put!(
                    s,
                    r##"<path d="M{x} {y} q-5 9 0 11 q5 -2 0 -11 Z" fill="#9cff3b"{glow}/>"##
                );
            }
            put!(
                s,
                r##"<ellipse cx="40" cy="118" rx="18" ry="4" fill="#9cff3b" fill-opacity="0.5"{glow}/><ellipse cx="90" cy="120" rx="16" ry="3.5" fill="#9cff3b" fill-opacity="0.5"{glow}/>"##
            );
            s
        }
        Calamity::Meltdown => {
            let mut s = format!(
                r##"<circle cx="64" cy="64" r="44" fill="#ffd21f" stroke="{b2}" stroke-width="3"{glow}/>"##
            );
            let at = |r: f32, deg: f32| {
                let rad = deg.to_radians();
                (64.0 + r * rad.cos(), 64.0 + r * rad.sin())
            };
            for k in 0..3 {
                let mid = -90.0 + k as f32 * 120.0;
                let (a0, a1) = (mid - 30.0, mid + 30.0);
                let ((x1, y1), (x2, y2), (x3, y3), (x4, y4)) =
                    (at(11.0, a0), at(36.0, a0), at(36.0, a1), at(11.0, a1));
                put!(
                    s,
                    r##"<path d="M{x1:.1} {y1:.1} L{x2:.1} {y2:.1} A36 36 0 0 1 {x3:.1} {y3:.1} L{x4:.1} {y4:.1} A11 11 0 0 0 {x1:.1} {y1:.1} Z" fill="#111"/>"##
                );
            }
            s + r##"<circle cx="64" cy="64" r="7" fill="#111"/>"##
        }
    }
}

fn feat(f: Feat, p: &Palette) -> String {
    let (ink, a, b2, metal, light, glow) = (p.ink, p.accent, p.accent2, p.metal, p.light, p.glow());
    match f {
        Feat::Invention => format!(
            r##"{}<circle cx="88" cy="40" r="17" fill="#ffe680" stroke="{ink}" stroke-width="2"{glow}/><path d="M82 42 L86 48 L90 42 L94 48" stroke="{ink}" fill="none" stroke-width="1.5"/><rect x="81" y="56" width="14" height="10" rx="2" fill="{metal}" stroke="{ink}"/>"##,
            gear(48.0, 78.0, 32.0, 10, metal, p)
        ),
        Feat::Expedition => format!(
            r##"<circle cx="64" cy="64" r="44" fill="{light}" stroke="{ink}" stroke-width="3"/><circle cx="64" cy="64" r="36" fill="none" stroke="{ink}" stroke-opacity="0.4"/><path d="M64 22 L72 64 L56 64 Z" fill="{}" stroke="{ink}" stroke-width="1.5"/><path d="M64 106 L72 64 L56 64 Z" fill="{metal}" stroke="{ink}" stroke-width="1.5"/><path d="M22 64 L64 58 L64 70 Z M106 64 L64 58 L64 70 Z" fill="{ink}" fill-opacity="0.5"/><circle cx="64" cy="64" r="4" fill="{a}" stroke="{ink}"/>"##,
            p.blood
        ),
        Feat::Treatise => format!(
            r##"<path d="M64 98 C50 90 34 90 18 94 L18 38 C34 34 50 34 64 42 Z" fill="{light}" stroke="{ink}" stroke-width="2"/><path d="M64 98 C78 90 94 90 110 94 L110 38 C94 34 78 34 64 42 Z" fill="{light}" stroke="{ink}" stroke-width="2"/><path d="M26 50 L56 52 M26 60 L56 62 M26 70 L56 72 M72 52 L102 50 M72 62 L102 60 M72 72 L96 70" stroke="{ink}" stroke-opacity="0.5" stroke-width="2"/><path d="M96 14 C104 24 104 36 98 48 L90 62 C92 46 92 30 96 14 Z" fill="{a}" stroke="{ink}"/>"##
        ),
        Feat::Artifact => format!(
            r##"<ellipse cx="64" cy="110" rx="30" ry="7" fill="{}" stroke="{ink}" stroke-width="2"/><path d="M64 16 L90 46 L78 102 L50 102 L38 46 Z" fill="{b2}" stroke="{ink}" stroke-width="2"{glow}/><path d="M38 46 L90 46 M64 16 L56 46 L64 102 L72 46 Z" fill="none" stroke="{light}" stroke-opacity="0.7" stroke-width="1.5"/>{}"##,
            p.stone,
            sparkles(
                p,
                &[(22.0, 30.0, 7.0), (106.0, 40.0, 6.0), (100.0, 92.0, 5.0)]
            )
        ),
        Feat::Grimoire => format!(
            r##"<rect x="34" y="24" width="60" height="80" rx="5" fill="#4a1d2a" stroke="{ink}" stroke-width="2"/><rect x="34" y="24" width="10" height="80" fill="#2e0f1a"/><circle cx="68" cy="60" r="18" fill="none" stroke="{b2}" stroke-width="2.5"{glow}/>{}<path d="M28 84 L100 76" stroke="{metal}" stroke-width="4" stroke-dasharray="8 3"/>"##,
            star(68.0, 60.0, 15.0, 5, 0.38, "none", p).replace(
                &format!(r##"stroke="{ink}" stroke-width="1""##),
                &format!(r##"stroke="{b2}" stroke-width="2""##)
            )
        ),
        Feat::ArtificialMind => {
            let mut s = format!(
                r##"<rect x="34" y="34" width="60" height="60" rx="8" fill="#101826" stroke="{b2}" stroke-width="3"{glow}/>"##
            );
            for i in 0..5 {
                let o = 42 + i * 11;
                put!(
                    s,
                    r##"<path d="M{o} 34 L{o} 22 M{o} 94 L{o} 106 M34 {o} L22 {o} M94 {o} L106 {o}" stroke="{metal}" stroke-width="3"/>"##
                );
            }
            put!(
                s,
                r##"<path d="M46 64 C46 48 56 44 64 50 C72 44 82 48 82 64 C82 78 72 84 64 78 C56 84 46 78 46 64 Z M64 50 L64 78 M52 60 L58 64 M76 60 L70 64 M54 72 L60 70 M74 72 L68 70" fill="none" stroke="{a}" stroke-width="2"{glow}/>"##
            );
            s
        }
        Feat::Implant => {
            let mut s = format!(
                r##"<rect x="52" y="98" width="26" height="24" rx="3" fill="{metal}" stroke="{ink}" stroke-width="2"/><rect x="42" y="62" width="44" height="40" rx="9" fill="{metal}" stroke="{ink}" stroke-width="2"/>"##
            );
            for (i, top) in [30, 24, 26, 34].into_iter().enumerate() {
                let x = 43 + i * 11;
                put!(
                    s,
                    r##"<rect x="{x}" y="{top}" width="9" height="{}" rx="4" fill="{metal}" stroke="{ink}" stroke-width="1.5"/><circle cx="{}" cy="{}" r="2" fill="{b2}"{glow}/>"##,
                    66 - top,
                    x as f32 + 4.5,
                    (top + 66) / 2
                );
            }
            put!(
                s,
                r##"<rect x="24" y="66" width="22" height="9" rx="4.5" transform="rotate(-35 42 72)" fill="{metal}" stroke="{ink}" stroke-width="1.5"/><path d="M50 80 L60 80 L64 88 L78 88" stroke="{a}" stroke-width="2" fill="none"{glow}/>"##
            );
            s
        }
        Feat::Heist => format!(
            r##"<rect x="62" y="20" width="44" height="30" rx="4" fill="#101826" stroke="{b2}" stroke-width="2"{glow}/><path d="M68 30 L76 30 M80 30 L84 30 M68 40 L72 40 M76 40 L90 40 M88 30 L100 30" stroke="{b2}" stroke-width="2"/><circle cx="40" cy="80" r="16" fill="none" stroke="{a}" stroke-width="7"{glow}/><path d="M54 74 L104 74 L104 86 L98 86 L98 96 L90 96 L90 86 L84 86 L84 94 L76 94 L76 86 L54 86 Z" fill="{a}" stroke="{ink}" stroke-width="1.5"{glow}/>"##
        ),
        Feat::Rescue => format!(
            r##"<g transform="translate(64 64) scale(1.6) translate(-64 -62)">{}{}</g>"##,
            shield(64.0, 60.0, 0.0, b2, p),
            star(64.0, 60.0, 16.0, 5, 0.45, a, p)
        ),
    }
}
