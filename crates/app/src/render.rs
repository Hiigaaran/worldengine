//! Pinta el mapa del mundo en una imagen con varios píxeles por casilla. La
//! elevación se interpola entre casillas, así que las costas salen suaves y el
//! relieve sombreado se aprecia al hacer zoom.

use eframe::egui::{Color32, ColorImage};
use worldengine_core::geography::SEA_LEVEL;
use worldengine_core::{Biome, World, WorldMap, YearClimate};

use crate::map_view::MapMode;

/// Píxeles por casilla en la textura del mapa.
pub const SUBPIXELS: usize = 4;

/// Con `climate`, el mapa de lluvia muestra la del año en vez de la normal.
pub fn render(world: &World, mode: MapMode, climate: Option<&YearClimate>) -> ColorImage {
    let map = &world.map;
    let (w, h) = (
        map.width as usize * SUBPIXELS,
        map.height as usize * SUBPIXELS,
    );
    let mut pixels = Vec::with_capacity(w * h);
    for py in 0..h {
        for px in 0..w {
            // Centro del píxel en coordenadas de casilla.
            let fx = (px as f32 + 0.5) / SUBPIXELS as f32;
            let fy = (py as f32 + 0.5) / SUBPIXELS as f32;
            let (tx, ty) = (px / SUBPIXELS, py / SUBPIXELS);
            let tile = &map.tiles[ty * map.width as usize + tx];
            let rgb = match mode {
                MapMode::Physical => physical(map, fx, fy),
                MapMode::Political => {
                    political(world, tx, ty, px % SUBPIXELS, py % SUBPIXELS, fx, fy)
                }
                MapMode::Elevation if tile.biome == Biome::Lake => biome_color(Biome::Lake),
                MapMode::Elevation => elevation_color(sample(map, fx, fy, |t| t.elevation)),
                MapMode::Temperature => ramp(
                    sample(map, fx, fy, |t| t.temperature),
                    &[
                        [40, 60, 200],
                        [90, 190, 230],
                        [240, 230, 120],
                        [220, 70, 40],
                    ],
                ),
                MapMode::Rainfall => {
                    if tile.biome.is_sea() {
                        [30, 40, 60]
                    } else {
                        let rain = match climate {
                            Some(c) => sample_index(map, fx, fy, |i| c.rainfall(map, i)),
                            None => sample(map, fx, fy, |t| t.rainfall),
                        };
                        ramp(
                            rain,
                            &[
                                [200, 170, 110],
                                [150, 190, 90],
                                [40, 140, 90],
                                [30, 80, 170],
                            ],
                        )
                    }
                }
            };
            pixels.push(Color32::from_rgb(rgb[0], rgb[1], rgb[2]));
        }
    }
    ColorImage {
        size: [w, h],
        pixels,
    }
}

pub fn biome_color(biome: Biome) -> [u8; 3] {
    match biome {
        Biome::Ocean => [28, 58, 110],
        Biome::Coast => [52, 100, 160],
        Biome::Lake => [60, 115, 175],
        Biome::Desert => [222, 198, 140],
        Biome::Shrubland => [176, 170, 110],
        Biome::Grassland => [140, 182, 88],
        Biome::Savanna => [190, 180, 90],
        Biome::Forest => [58, 124, 64],
        Biome::Jungle => [30, 98, 50],
        Biome::Swamp => [80, 110, 85],
        Biome::Taiga => [70, 110, 90],
        Biome::Tundra => [168, 172, 150],
        Biome::Hills => [140, 140, 95],
        Biome::Mountain => [132, 118, 108],
        Biome::Glacier => [232, 240, 246],
    }
}

/// Valor interpolado (bilineal) entre los centros de las casillas.
fn sample(map: &WorldMap, fx: f32, fy: f32, value: impl Fn(&worldengine_core::Tile) -> f32) -> f32 {
    sample_index(map, fx, fy, |i| value(&map.tiles[i]))
}

/// Como [`sample`], pero el valor se pide por índice de casilla.
fn sample_index(map: &WorldMap, fx: f32, fy: f32, value: impl Fn(usize) -> f32) -> f32 {
    let (w, h) = (map.width as i64, map.height as i64);
    let (x, y) = (fx - 0.5, fy - 0.5);
    let (x0, y0) = (x.floor(), y.floor());
    let (tx, ty) = (x - x0, y - y0);
    let at = |x: i64, y: i64| value((y.clamp(0, h - 1) * w + x.clamp(0, w - 1)) as usize);
    let (x0, y0) = (x0 as i64, y0 as i64);
    let top = at(x0, y0) * (1.0 - tx) + at(x0 + 1, y0) * tx;
    let bottom = at(x0, y0 + 1) * (1.0 - tx) + at(x0 + 1, y0 + 1) * tx;
    top * (1.0 - ty) + bottom * ty
}

/// Sombreado de relieve con luz desde el noroeste.
fn hillshade(map: &WorldMap, fx: f32, fy: f32) -> f32 {
    let d = 0.5;
    let e = |x, y| sample(map, x, y, |t| t.elevation.max(SEA_LEVEL));
    let dx = e(fx + d, fy) - e(fx - d, fy);
    let dy = e(fx, fy + d) - e(fx, fy - d);
    (1.0 - (dx + dy) * 6.0).clamp(0.55, 1.35)
}

/// Pequeña variación por píxel para que las zonas planas no se vean lisas.
fn grain(fx: f32, fy: f32) -> f32 {
    let n = ((fx * 127.1 + fy * 311.7).sin() * 43_758.547).fract().abs();
    0.96 + n * 0.08
}

/// Desplazamiento suave y determinista para que los límites entre biomas no
/// sigan la cuadrícula de casillas.
fn jitter(fx: f32, fy: f32) -> (f32, f32) {
    let a = (fx * 0.9 + fy * 1.7).sin()
        + (fx * 2.3 - fy * 1.1).sin() * 0.5
        + (fx * 4.7 + fy * 3.9).sin() * 0.25;
    let b = (fx * 1.3 - fy * 0.8).cos()
        + (fx * 1.9 + fy * 2.6).cos() * 0.5
        + (fx * 5.1 - fy * 4.3).cos() * 0.25;
    (a * 0.3, b * 0.3)
}

fn tile_at(map: &WorldMap, fx: f32, fy: f32) -> &worldengine_core::Tile {
    let x = (fx.max(0.0) as usize).min(map.width as usize - 1);
    let y = (fy.max(0.0) as usize).min(map.height as usize - 1);
    &map.tiles[y * map.width as usize + x]
}

fn physical(map: &WorldMap, fx: f32, fy: f32) -> [u8; 3] {
    let lake = sample(
        map,
        fx,
        fy,
        |t| if t.biome == Biome::Lake { 1.0 } else { 0.0 },
    );
    if lake > 0.5 {
        return biome_color(Biome::Lake);
    }
    let e = sample(map, fx, fy, |t| {
        if t.biome == Biome::Lake {
            SEA_LEVEL
        } else {
            t.elevation
        }
    });
    if e < SEA_LEVEL {
        // Agua: más clara cerca de la costa.
        let depth = 1.0 - e / SEA_LEVEL;
        return mix([70, 130, 185], [18, 40, 90], depth.powf(0.6));
    }
    let (jx, jy) = jitter(fx, fy);
    let mut biome = tile_at(map, fx + jx, fy + jy).biome;
    if biome.is_water() {
        biome = tile_at(map, fx, fy).biome;
    }
    let base = if biome.is_water() {
        // La interpolación dice tierra pero la casilla es agua: playa u orilla.
        [205, 195, 150]
    } else {
        biome_color(biome)
    };
    scale(base, hillshade(map, fx, fy) * grain(fx, fy))
}

fn political(
    world: &World,
    tx: usize,
    ty: usize,
    sx: usize,
    sy: usize,
    fx: f32,
    fy: f32,
) -> [u8; 3] {
    let map = &world.map;
    let tile = &map.tiles[ty * map.width as usize + tx];
    let physical = physical(map, fx, fy);
    if tile.biome.is_water() {
        return scale(physical, 0.8);
    }
    let gray = ((physical[0] as u32 + physical[1] as u32 + physical[2] as u32) / 3) as u8;
    let Some(owner) = tile.owner else {
        return scale([gray, gray, gray], 0.85);
    };
    let color = world.country(owner).color;
    // Borde: el último píxel de la casilla si la vecina tiene otro dueño.
    let differs = |dx: i64, dy: i64| {
        let (x, y) = (tx as i64 + dx, ty as i64 + dy);
        if x < 0 || y < 0 || x >= map.width as i64 || y >= map.height as i64 {
            return false;
        }
        let other = &map.tiles[y as usize * map.width as usize + x as usize];
        !other.biome.is_water() && other.owner != tile.owner
    };
    let edge = (sx == 0 && differs(-1, 0))
        || (sx == SUBPIXELS - 1 && differs(1, 0))
        || (sy == 0 && differs(0, -1))
        || (sy == SUBPIXELS - 1 && differs(0, 1));
    if edge {
        return scale(color, 0.6);
    }
    mix([gray, gray, gray], color, 0.6)
}

fn elevation_color(e: f32) -> [u8; 3] {
    if e < SEA_LEVEL {
        mix([10, 20, 60], [60, 110, 180], e / SEA_LEVEL)
    } else {
        ramp(
            (e - SEA_LEVEL) / (1.0 - SEA_LEVEL),
            &[
                [60, 130, 70],
                [190, 190, 110],
                [150, 110, 80],
                [250, 250, 250],
            ],
        )
    }
}

fn ramp(t: f32, stops: &[[u8; 3]]) -> [u8; 3] {
    let t = t.clamp(0.0, 1.0) * (stops.len() - 1) as f32;
    let i = (t.floor() as usize).min(stops.len() - 2);
    mix(stops[i], stops[i + 1], t - i as f32)
}

fn mix(a: [u8; 3], b: [u8; 3], t: f32) -> [u8; 3] {
    let t = t.clamp(0.0, 1.0);
    [0, 1, 2].map(|k| (a[k] as f32 * (1.0 - t) + b[k] as f32 * t) as u8)
}

fn scale(c: [u8; 3], f: f32) -> [u8; 3] {
    c.map(|v| (v as f32 * f).clamp(0.0, 255.0) as u8)
}
