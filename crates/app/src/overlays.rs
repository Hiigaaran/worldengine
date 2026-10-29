//! Animaciones sobre el mapa: nubes y lluvia del clima del año, y explosiones
//! donde hubo batallas. Cada edad tiene su paleta: las tormentas arcanas son
//! violetas y chispean, la lluvia ácida es verdosa, y las batallas de fantasía
//! estallan en magia y las cyberpunk en neón.

use std::f32::consts::TAU;

use eframe::egui::{self, Color32, Painter, Pos2, Shape, Stroke, Vec2};
use worldengine_core::climate::Cloud;
use worldengine_core::Age;

/// Pseudoaleatorio estable a partir de un índice: así cada nube conserva su
/// forma de un cuadro al siguiente.
fn hash(i: usize, salt: u32) -> f32 {
    let mut x = (i as u32).wrapping_mul(0x9e37_79b9) ^ salt.wrapping_mul(0x85eb_ca6b);
    x ^= x >> 15;
    x = x.wrapping_mul(0x2c1b_3c6d);
    x ^= x >> 12;
    (x & 0xffff) as f32 / 65535.0
}

fn with_alpha(c: Color32, alpha: f32) -> Color32 {
    Color32::from_rgba_unmultiplied(c.r(), c.g(), c.b(), (alpha.clamp(0.0, 1.0) * 255.0) as u8)
}

/// Dibuja las nubes del año. `to_screen` pasa de casillas a pantalla y `px` es
/// el tamaño en pantalla de una casilla. Una nube bajo el cursor se vuelve casi
/// transparente para dejar ver el mapa.
#[allow(clippy::too_many_arguments)]
pub fn clouds(
    ui: &egui::Ui,
    painter: &Painter,
    clouds: &[Cloud],
    age: Age,
    year: i32,
    to_screen: impl Fn(f32, f32) -> Pos2,
    px: f32,
    cursor: Option<Pos2>,
) {
    let time = ui.input(|i| i.time) as f32;
    // De muy cerca las nubes estorban: se van desvaneciendo.
    let zoom_fade = 1.0 - ((px - 14.0) / 30.0).clamp(0.0, 0.65);
    let view = painter.clip_rect();

    for (i, cloud) in clouds.iter().enumerate() {
        let radius = cloud.radius * px;
        // Deriva lenta en la dirección del viento, de ida y vuelta.
        let drift = (time * 0.15 + hash(i, 1) * TAU).sin() * cloud.radius * 0.25;
        let center = to_screen(cloud.x + drift * cloud.wind, cloud.y);
        if !view.expand(radius * 1.6).contains(center) {
            continue;
        }
        let hovered = cursor.is_some_and(|c| c.distance(center) < radius * 1.1);
        let fade =
            ui.ctx()
                .animate_bool_with_time(egui::Id::new(("cloud", year, i)), hovered, 0.25);
        let alpha = zoom_fade * (1.0 - 0.85 * fade);

        // Lluvia: trazos inclinados por el viento que caen bajo la nube.
        let drops = 8 + (cloud.intensity * 18.0) as usize;
        let fall = radius * 1.1;
        let slant = Vec2::new(-cloud.wind * radius * 0.1, radius * 0.25);
        let rain = match (cloud.anomaly, age) {
            (true, Age::Fantasy) => Color32::from_rgb(210, 150, 255),
            (true, Age::Cyberpunk) => Color32::from_rgb(170, 255, 80),
            _ => Color32::from_rgb(170, 205, 255),
        };
        for d in 0..drops {
            let speed = 0.9 + hash(d, i as u32) * 0.6;
            let t = (time * speed + hash(d, 7 + i as u32)).fract();
            let x = (hash(d, 3 + i as u32) - 0.5) * radius * 1.5;
            let start = center + Vec2::new(x - slant.x * t * 4.0, radius * 0.35 + fall * t);
            let a = alpha * 0.8 * (1.0 - t);
            painter.line_segment(
                [start, start + slant],
                Stroke::new((px * 0.15).clamp(1.2, 2.5), with_alpha(rain, a)),
            );
        }

        // Chispas arcanas que flotan alrededor de la nube.
        if cloud.anomaly && age == Age::Fantasy {
            for k in 0..6 {
                let t = (time * 0.5 + hash(k, 31 + i as u32)).fract();
                let angle = hash(k, 33 + i as u32) * TAU + time * 0.4;
                let p = center
                    + Vec2::angled(angle) * radius * (0.4 + 0.6 * t)
                    + Vec2::new(0.0, radius * 0.3);
                let glow = if k % 2 == 0 {
                    Color32::from_rgb(120, 230, 255)
                } else {
                    Color32::from_rgb(230, 140, 255)
                };
                painter.circle_filled(
                    p,
                    (px * 0.25).clamp(1.5, 3.5) * (1.0 - t),
                    with_alpha(glow, alpha * (1.0 - t)),
                );
            }
        }

        // Rayo ocasional en las tormentas (y en las tormentas arcanas).
        let arcane = cloud.anomaly && age == Age::Fantasy;
        let flash = (cloud.storm || arcane) && (time * 0.45 + hash(i, 5)).fract() < 0.05;
        if flash {
            let mut p = center + Vec2::new((hash(i, 6) - 0.5) * radius, radius * 0.25);
            let mut points = vec![p];
            for k in 0..4 {
                p += Vec2::new((hash(k, 9 + i as u32) - 0.5) * radius * 0.4, radius * 0.22);
                points.push(p);
            }
            painter.add(Shape::line(
                points,
                Stroke::new(
                    (px * 0.25).clamp(1.5, 3.0),
                    with_alpha(
                        if arcane {
                            Color32::from_rgb(220, 160, 255)
                        } else {
                            Color32::from_rgb(255, 250, 200)
                        },
                        alpha,
                    ),
                ),
            ));
        }

        // La nube: varios copos de distinto tamaño, más oscuros abajo.
        let (top, bottom) = if cloud.anomaly && age == Age::Fantasy {
            (
                if flash {
                    Color32::from_rgb(225, 190, 255)
                } else {
                    Color32::from_rgb(150, 100, 210)
                },
                Color32::from_rgb(70, 35, 120),
            )
        } else if cloud.anomaly && age == Age::Cyberpunk {
            (
                Color32::from_rgb(140, 160, 60),
                Color32::from_rgb(60, 72, 25),
            )
        } else if cloud.storm {
            (
                if flash {
                    Color32::from_rgb(200, 200, 220)
                } else {
                    Color32::from_rgb(110, 115, 130)
                },
                Color32::from_rgb(60, 62, 75),
            )
        } else {
            (
                Color32::from_rgb(235, 238, 245),
                Color32::from_rgb(170, 175, 190),
            )
        };
        let puffs = 7;
        for (layer, color, lift) in [(0, bottom, 0.12), (1, top, -0.05)] {
            for k in 0..puffs {
                let angle = k as f32 / puffs as f32 * TAU + hash(k, i as u32) * 0.8;
                let spread = Vec2::new(0.6, 0.38) * radius;
                let wobble = (time * 0.6 + hash(k, 11 + i as u32) * TAU).sin() * 0.04;
                let offset = Vec2::new(angle.cos() * spread.x, angle.sin() * spread.y)
                    + Vec2::new(0.0, lift * radius);
                let r = radius * (0.32 + 0.18 * hash(k, 13 + i as u32) + wobble);
                let a = alpha * if layer == 0 { 0.55 } else { 0.7 };
                painter.circle_filled(center + offset, r, with_alpha(color, a));
            }
            let a = alpha * if layer == 0 { 0.55 } else { 0.75 };
            painter.circle_filled(
                center + Vec2::new(0.0, lift * radius),
                radius * 0.45,
                with_alpha(color, a),
            );
        }
    }
}

/// Explosiones animadas en las zonas de combate del año.
pub fn explosions(ui: &egui::Ui, painter: &Painter, places: &[Pos2], px: f32, age: Age) {
    let time = ui.input(|i| i.time) as f32;
    // (halo, núcleo, centro, chispas): fuego, magia o neón.
    let (halo, core, heart, sparks) = match age {
        Age::Realistic => (
            Color32::from_rgb(220, 60, 20),
            Color32::from_rgb(255, 170, 40),
            Color32::from_rgb(255, 245, 200),
            Color32::from_rgb(255, 210, 90),
        ),
        Age::Fantasy => (
            Color32::from_rgb(120, 50, 220),
            Color32::from_rgb(90, 200, 255),
            Color32::from_rgb(235, 225, 255),
            Color32::from_rgb(200, 150, 255),
        ),
        Age::Cyberpunk => (
            Color32::from_rgb(255, 30, 150),
            Color32::from_rgb(40, 230, 255),
            Color32::from_rgb(230, 255, 255),
            Color32::from_rgb(255, 240, 60),
        ),
    };
    let size = (px * 1.6).clamp(10.0, 40.0);
    let view = painter.clip_rect().expand(size * 2.0);
    for (i, &p) in places.iter().enumerate() {
        if !view.contains(p) {
            continue;
        }
        // Cada lugar con su propio ritmo, y varias detonaciones a la vez.
        for burst in 0..2 {
            let period = 1.3 + hash(i, 21 + burst) * 0.6;
            let t = (time / period + hash(i, 23 + burst)).fract();
            let jitter =
                Vec2::new(hash(i, 25 + burst) - 0.5, hash(i, 27 + burst) - 0.5) * size * 0.8;
            let c = p + jitter * (burst as f32);
            let r = size * (0.25 + 0.75 * t.sqrt()) * (1.0 - 0.3 * burst as f32);
            let fade = 1.0 - t;

            // Humo que sube tras la explosión.
            if t > 0.35 {
                let s = (t - 0.35) / 0.65;
                painter.circle_filled(
                    c + Vec2::new(0.0, -size * 0.6 * s),
                    r * (0.6 + 0.4 * s),
                    with_alpha(Color32::from_gray(70), 0.35 * (1.0 - s)),
                );
            }
            // Bola de fuego: núcleo claro dentro de un halo rojo.
            painter.circle_filled(c, r, with_alpha(halo, 0.55 * fade));
            painter.circle_filled(c, r * 0.6 * (1.0 - t * 0.6), with_alpha(core, 0.85 * fade));
            painter.circle_filled(c, r * 0.25 * fade, with_alpha(heart, fade));
            // Onda expansiva y chispas.
            painter.circle_stroke(
                c,
                r * 1.3,
                Stroke::new(2.0_f32, with_alpha(halo, 0.6 * fade)),
            );
            for k in 0..8 {
                let angle = k as f32 / 8.0 * TAU + hash(i, 29 + burst) * TAU;
                let dir = Vec2::angled(angle);
                let a = c + dir * r * 0.9;
                let b = c + dir * r * (1.2 + 0.6 * t);
                painter.line_segment([a, b], Stroke::new(1.5_f32, with_alpha(sparks, fade)));
            }
        }
    }
}
