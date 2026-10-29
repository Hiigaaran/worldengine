//! Clima de cada año: cuánto llovió y dónde hubo tormentas.
//!
//! No se simula día a día. Un año se resume en unas decenas de frentes de lluvia
//! que cruzan el mapa empujados por los vientos dominantes; lo que cada uno
//! descarga se acumula por casilla y se compara con lo normal, que es el
//! `rainfall` de la casilla. Así sale un promedio anual: dónde llovió más que de
//! costumbre, dónde hubo sequía y por dónde pasaron las tormentas.
//!
//! El clima depende solo de la semilla, el año y el mapa, así que no se guarda:
//! se recalcula al viajar en el tiempo y siempre da lo mismo.

use rand::Rng as _;

use crate::geography::{latitude, SEA_LEVEL};
use crate::ids::*;
use crate::model::*;
use crate::noise::ValueNoise;
use crate::rng::{Rng, WorldRng};

/// Lado, en casillas, de las celdas en que se agrupan las nubes del mapa.
const CELL: u32 = 10;
/// Lluvia relativa mínima para que una celda muestre nubes.
const CLOUDY: f32 = 1.25;
/// Nubes como máximo, para no tapar el mapa: se quedan las más intensas.
const MAX_CLOUDS: usize = 32;

/// Una zona del mapa donde ese año llovió más de lo normal.
#[derive(Debug, Clone, PartialEq)]
pub struct Cloud {
    /// Centro, en coordenadas de casilla (puede tener decimales).
    pub x: f32,
    pub y: f32,
    /// Radio en casillas.
    pub radius: f32,
    /// De 0 (algo más que lo normal) a 1 (diluvio).
    pub intensity: f32,
    /// Si por ahí pasaron tormentas fuertes.
    pub storm: bool,
    /// Hacia dónde sopla el viento: 1 al este, -1 al oeste.
    pub wind: f32,
}

/// Resumen de una región en un año.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RegionRain {
    pub region: RegionId,
    /// Lluvia del año respecto a lo normal (1.0 = año normal).
    pub ratio: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct YearClimate {
    pub year: Year,
    /// Lluvia del año en cada casilla respecto a lo normal: 1.0 es un año
    /// corriente, 1.5 la mitad más, 0.5 la mitad.
    pub rain: Vec<f32>,
    /// Si cada casilla sufrió tormentas fuertes.
    pub stormy: Vec<bool>,
    pub clouds: Vec<Cloud>,
    /// Frentes que llegaron a tierra como tormentas.
    pub storms: u32,
    pub wettest: Option<RegionRain>,
    pub driest: Option<RegionRain>,
}

impl YearClimate {
    pub fn at(&self, map: &WorldMap, pos: Pos) -> f32 {
        self.rain[map.index(pos)]
    }

    /// Lluvia absoluta del año (0 a 1, como `Tile::rainfall`) en una casilla.
    pub fn rainfall(&self, map: &WorldMap, index: usize) -> f32 {
        (map.tiles[index].rainfall * self.rain[index]).clamp(0.0, 1.0)
    }
}

/// Un frente de lluvia que recorre el mapa durante unos días.
struct Front {
    x: f32,
    y: f32,
    dx: f32,
    dy: f32,
    radius: f32,
    /// Cuánta agua descarga en cada paso.
    moisture: f32,
    severe: bool,
    steps: u32,
}

pub fn year_climate(world: &World, year: Year) -> YearClimate {
    let map = &world.map;
    let (w, h) = (map.width, map.height);
    let mut rng = WorldRng::new(world.config.seed).stream(&format!("climate-{year}"));
    let n = map.tiles.len();

    // Tendencia general del año: zonas amplias algo más húmedas o secas.
    let trend = ValueNoise::new(&mut rng, 16);
    let scale = 3.0 / w.max(h) as f32;

    let mut exposure = vec![0.0f32; n];
    let mut severe = vec![0.0f32; n];
    let mut storms = 0;
    let fronts = rng.gen_range(140..200);
    for _ in 0..fronts {
        let mut front = new_front(&mut rng, w, h);
        let mut landed = false;
        for _ in 0..front.steps {
            let (cx, cy) = (front.x, front.y);
            if cx < 0.0 || cy < 0.0 || cx >= w as f32 || cy >= h as f32 {
                break;
            }
            let r = front.radius;
            let (x0, x1) = ((cx - r).max(0.0) as u32, ((cx + r) as u32).min(w - 1));
            let (y0, y1) = ((cy - r).max(0.0) as u32, ((cy + r) as u32).min(h - 1));
            for y in y0..=y1 {
                for x in x0..=x1 {
                    let d2 = (x as f32 + 0.5 - cx).powi(2) + (y as f32 + 0.5 - cy).powi(2);
                    let falloff = 1.0 - d2 / (r * r);
                    if falloff <= 0.0 {
                        continue;
                    }
                    let i = (y * w + x) as usize;
                    let fall = front.moisture * falloff;
                    exposure[i] += fall;
                    if front.severe && map.tiles[i].elevation >= SEA_LEVEL {
                        severe[i] += fall;
                        landed = true;
                    }
                }
            }
            front.x += front.dx;
            front.y += front.dy;
        }
        storms += u32::from(landed);
    }

    let land: Vec<usize> = (0..n)
        .filter(|&i| map.tiles[i].elevation >= SEA_LEVEL)
        .collect();
    let mean = land.iter().map(|&i| exposure[i]).sum::<f32>() / land.len().max(1) as f32;
    let rain: Vec<f32> = (0..n)
        .map(|i| {
            let p = map.pos(i);
            let t = trend.fractal(p.x as f32 * scale, p.y as f32 * scale, 3);
            let fronts = exposure[i] / mean.max(f32::EPSILON);
            ((0.75 + 0.5 * t) * (0.6 + 0.4 * fronts)).clamp(0.25, 2.5)
        })
        .collect();
    let max_severe = severe.iter().copied().fold(0.0, f32::max);
    let stormy: Vec<bool> = severe
        .iter()
        .map(|&s| max_severe > 0.0 && s > max_severe * 0.3)
        .collect();

    let (wettest, driest) = extremes(world, &rain);
    YearClimate {
        year,
        clouds: clouds(map, &rain, &stormy),
        rain,
        stormy,
        storms,
        wettest,
        driest,
    }
}

fn new_front(rng: &mut Rng, w: u32, h: u32) -> Front {
    let y = rng.gen_range(0.0..h as f32);
    let lat = latitude(y as u32, h);
    // Los mismos vientos que dan forma al clima normal (ver `geography`).
    let eastward = (1.0 / 3.0..2.0 / 3.0).contains(&lat);
    let speed = rng.gen_range(1.2..2.2);
    let dx = if eastward { speed } else { -speed };
    let steps = rng.gen_range(20..60);
    // Empieza viento arriba para que el recorrido caiga sobre el mapa.
    let reach = speed * steps as f32;
    let x = if eastward {
        rng.gen_range(-reach * 0.5..w as f32 - reach * 0.3)
    } else {
        rng.gen_range(reach * 0.3..w as f32 + reach * 0.5)
    }
    .clamp(0.0, w as f32 - 1.0);
    // Cerca de los polos las tormentas son raras.
    let severe = lat < 0.8 && rng.gen_bool(0.07);
    Front {
        x,
        y,
        dx,
        dy: rng.gen_range(-0.5..0.5),
        radius: rng.gen_range(4.0..10.0),
        moisture: rng.gen_range(0.6..1.2) * if severe { 1.5 } else { 1.0 },
        severe,
        steps,
    }
}

/// Agrupa la lluvia en celdas y pone nubes en las que se mojaron más que de
/// costumbre. Solo sobre tierra: la lluvia en alta mar no le importa a nadie.
fn clouds(map: &WorldMap, rain: &[f32], stormy: &[bool]) -> Vec<Cloud> {
    let mut out = Vec::new();
    for cy in 0..map.height.div_ceil(CELL) {
        for cx in 0..map.width.div_ceil(CELL) {
            let (mut sum, mut land, mut total, mut storm) = (0.0, 0, 0, 0);
            let (mut sx, mut sy, mut weight) = (0.0, 0.0, 0.0);
            for y in cy * CELL..((cy + 1) * CELL).min(map.height) {
                for x in cx * CELL..((cx + 1) * CELL).min(map.width) {
                    total += 1;
                    let i = (y * map.width + x) as usize;
                    if map.tiles[i].elevation < SEA_LEVEL {
                        continue;
                    }
                    land += 1;
                    sum += rain[i];
                    storm += usize::from(stormy[i]);
                    let wgt = (rain[i] - 1.0).max(0.01);
                    sx += (x as f32 + 0.5) * wgt;
                    sy += (y as f32 + 0.5) * wgt;
                    weight += wgt;
                }
            }
            if land * 10 < total * 3 {
                continue;
            }
            let ratio = sum / land as f32;
            let storm = storm * 2 >= land;
            if ratio < CLOUDY && !storm {
                continue;
            }
            let intensity = ((ratio - 1.0) / 0.7).clamp(0.15, 1.0);
            let lat = latitude((sy / weight) as u32, map.height);
            out.push(Cloud {
                x: sx / weight,
                y: sy / weight,
                radius: CELL as f32 * (0.75 + 0.35 * intensity),
                intensity,
                storm,
                wind: if (1.0 / 3.0..2.0 / 3.0).contains(&lat) {
                    1.0
                } else {
                    -1.0
                },
            });
        }
    }
    // Las tormentas tienen algo de preferencia, sin desplazar a todas las lluvias.
    let rank = |c: &Cloud| c.intensity + if c.storm { 0.3 } else { 0.0 };
    out.sort_by(|a, b| rank(b).total_cmp(&rank(a)));
    out.truncate(MAX_CLOUDS);
    out
}

/// La región más lluviosa y la más seca del año, comparadas con lo normal.
fn extremes(world: &World, rain: &[f32]) -> (Option<RegionRain>, Option<RegionRain>) {
    let mut sums = vec![(0.0f32, 0u32); world.regions.len()];
    for (i, tile) in world.map.tiles.iter().enumerate() {
        if let (Some(region), false) = (tile.region, tile.biome.is_water()) {
            let s = &mut sums[region.index()];
            s.0 += rain[i];
            s.1 += 1;
        }
    }
    let regions: Vec<RegionRain> = sums
        .iter()
        .enumerate()
        .filter(|(_, (_, count))| *count >= 20)
        .map(|(i, (sum, count))| RegionRain {
            region: RegionId(i as u32),
            ratio: sum / *count as f32,
        })
        .collect();
    let wettest = regions
        .iter()
        .copied()
        .max_by(|a, b| a.ratio.total_cmp(&b.ratio));
    let driest = regions
        .iter()
        .copied()
        .min_by(|a, b| a.ratio.total_cmp(&b.ratio));
    (wettest, driest)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generate;

    #[test]
    fn climate_is_reproducible_and_varies_by_year() {
        let world = generate(&WorldConfig {
            seed: 5,
            history_years: 0,
            ..Default::default()
        });
        let a = year_climate(&world, 10);
        assert_eq!(a, year_climate(&world, 10));
        assert_ne!(a.rain, year_climate(&world, 11).rain);
    }

    #[test]
    fn rain_averages_near_normal() {
        let world = generate(&WorldConfig {
            seed: 9,
            history_years: 0,
            ..Default::default()
        });
        for year in 0..5 {
            let c = year_climate(&world, year);
            let land: Vec<f32> = world
                .map
                .tiles
                .iter()
                .zip(&c.rain)
                .filter(|(t, _)| t.elevation >= SEA_LEVEL)
                .map(|(_, &r)| r)
                .collect();
            let mean = land.iter().sum::<f32>() / land.len() as f32;
            assert!((0.75..1.25).contains(&mean), "año {year}: media {mean}");
            assert!(c.clouds.iter().all(|cl| {
                (0.0..=1.0).contains(&cl.intensity)
                    && cl.x >= 0.0
                    && cl.y >= 0.0
                    && cl.x <= world.map.width as f32
                    && cl.y <= world.map.height as f32
            }));
            let wet = c.wettest.unwrap();
            let dry = c.driest.unwrap();
            assert!(wet.ratio >= dry.ratio);
        }
    }
}
