//! Geografía física del mundo, en el mismo orden que Dwarf Fortress:
//! relieve → clima → agua (cuencas, lagos, ríos, erosión) → biomas → regiones.
//!
//! - **Relieve**: placas tectónicas (continentales u oceánicas) que se mueven;
//!   donde chocan se levantan cordilleras. El ruido añade el detalle fino.
//! - **Clima**: la temperatura depende de la latitud y la altura. La lluvia la
//!   traen vientos dominantes según la latitud: se cargan de humedad sobre el
//!   mar y la descargan al subir montañas, dejando sombra de lluvia detrás.
//! - **Agua**: se rellenan las depresiones (las grandes quedan como lagos), cada
//!   casilla drena hacia una vecina y el caudal acumula la lluvia de la cuenca.
//!   Donde el caudal es grande nace un río.

use std::cmp::Ordering;
use std::collections::{BinaryHeap, VecDeque};

use rand::Rng as _;

use crate::ids::*;
use crate::model::*;
use crate::names;
use crate::noise::ValueNoise;
use crate::rng::{Rng, WorldRng};

/// Elevación que separa el mar de la tierra firme.
pub const SEA_LEVEL: f32 = 0.4;

/// Las regiones más pequeñas que esto se funden con una vecina.
const MIN_REGION_AREA: u32 = 6;

/// Profundidad mínima de una depresión para formar un lago.
const LAKE_DEPTH: f32 = 0.006;

/// Los lagos más pequeños que esto se consideran charcas y se ignoran.
const MIN_LAKE_AREA: usize = 4;

/// Un río nace cuando la cuenca equivale a tantas casillas de lluvia media.
const RIVER_CATCHMENT: f32 = 30.0;

pub struct Geography {
    pub map: WorldMap,
    pub regions: Vec<Region>,
    pub rivers: Vec<River>,
}

pub fn build(config: &WorldConfig, rng: &WorldRng) -> Geography {
    let (w, h) = (config.width, config.height);
    let elevation = relief(config, &mut rng.stream("relief"));
    let temperature = temperature(w, h, &elevation, &mut rng.stream("temperature"));
    let rainfall = rainfall(w, h, &elevation, &temperature, &mut rng.stream("rainfall"));

    let mut map = WorldMap {
        width: w,
        height: h,
        tiles: (0..elevation.len())
            .map(|i| Tile {
                elevation: elevation[i],
                temperature: temperature[i],
                rainfall: rainfall[i],
                biome: Biome::Ocean,
                flow: 0.0,
                river: None,
                region: None,
                owner: None,
            })
            .collect(),
    };

    let drainage = drain(&mut map, &mut rng.stream("drainage"));
    erode(&mut map);
    assign_biomes(&mut map);
    let rivers = trace_rivers(&mut map, &drainage, &mut rng.stream("rivers"));
    let regions = build_regions(&mut map, &mut rng.stream("regions"));
    Geography {
        map,
        regions,
        rivers,
    }
}

// ---------------------------------------------------------------- relieve

struct Plate {
    center: (f32, f32),
    velocity: (f32, f32),
    continental: bool,
}

fn relief(config: &WorldConfig, rng: &mut Rng) -> Vec<f32> {
    let (w, h) = (config.width, config.height);
    let land_share = 1.0 - config.sea_level.clamp(0.05, 0.95) as f64;
    let plates: Vec<Plate> = (0..rng.gen_range(10..16))
        .map(|_| Plate {
            center: (rng.gen_range(0.0..w as f32), rng.gen_range(0.0..h as f32)),
            velocity: (rng.gen_range(-1.0..1.0), rng.gen_range(-1.0..1.0)),
            continental: rng.gen_bool(land_share),
        })
        .collect();

    let warp_x = ValueNoise::new(rng, 64);
    let warp_y = ValueNoise::new(rng, 64);
    let detail = ValueNoise::new(rng, 64);
    let ridges = ValueNoise::new(rng, 64);

    let size = w.max(h) as f32;
    let scale = 6.0 / size;
    let warp = size * 0.12;
    let boundary_width = size * 0.04;

    let mut raw = Vec::with_capacity((w * h) as usize);
    for y in 0..h {
        for x in 0..w {
            let (fx, fy) = (x as f32 * scale, y as f32 * scale);
            // Deformar las coordenadas para que los bordes de placa no sean rectos.
            let px = x as f32 + (warp_x.fractal(fx, fy, 3) - 0.5) * warp * 2.0;
            let py = y as f32 + (warp_y.fractal(fx, fy, 3) - 0.5) * warp * 2.0;

            let distances: Vec<f32> = plates
                .iter()
                .map(|p| ((p.center.0 - px).powi(2) + (p.center.1 - py).powi(2)).sqrt())
                .collect();
            let a = (0..plates.len())
                .min_by(|&i, &j| distances[i].total_cmp(&distances[j]))
                .unwrap();

            // Altura base: media de las placas ponderada por cercanía, para que
            // la transición entre placa continental y oceánica sea suave.
            let base = |p: &Plate| if p.continental { 0.62 } else { 0.28 };
            let (mut sum, mut weights) = (0.0, 0.0);
            for (k, p) in plates.iter().enumerate() {
                let weight = (-(distances[k] - distances[a]) / boundary_width).exp();
                sum += base(p) * weight;
                weights += weight;
            }
            let mut e = sum / weights;

            // Cada borde cercano levanta montañas si las placas chocan o abre una
            // fosa si se separan. Se recorren todos los pares de placas con un
            // peso simétrico, para que no haya saltos donde se cruzan tres placas.
            let nearest = distances[a];
            let ridge = 0.6 + 0.4 * ridges.ridged(fx * 2.0, fy * 2.0, 4);
            for i in 0..plates.len() {
                for j in i + 1..plates.len() {
                    let gap = (distances[i] - nearest) + (distances[j] - nearest);
                    // Campana suave: la cumbre queda redondeada, no en punta.
                    let near = (-(gap * 0.5 / (boundary_width * 1.3)).powi(2)).exp();
                    if near < 0.01 {
                        continue;
                    }
                    let (pi, pj) = (&plates[i], &plates[j]);
                    let normal = normalize((pj.center.0 - pi.center.0, pj.center.1 - pi.center.1));
                    let relative = (pi.velocity.0 - pj.velocity.0, pi.velocity.1 - pj.velocity.1);
                    let convergence = relative.0 * normal.0 + relative.1 * normal.1;
                    if convergence > 0.0 {
                        let strength = if pi.continental || pj.continental {
                            0.55
                        } else {
                            0.2
                        };
                        e += convergence * near * strength * ridge;
                    } else {
                        e += convergence * near * 0.12;
                    }
                }
            }

            e += (detail.fractal(fx * 1.5, fy * 1.5, 6) - 0.5) * 0.35;
            // Serranías sueltas en el interior de los continentes.
            e += ridges.ridged(fx * 1.2 + 17.0, fy * 1.2, 4).powi(3) * 0.12;

            // Hundir los bordes del mapa para que el mundo quede rodeado de mar.
            let dx = x as f32 / w as f32 * 2.0 - 1.0;
            let dy = y as f32 / h as f32 * 2.0 - 1.0;
            let edge = dx.abs().max(dy.abs());
            e -= ((edge - 0.75).max(0.0) * 4.0).powi(2) * 0.4;
            raw.push(e);
        }
    }

    // El nivel del mar es el cuantil pedido, así la proporción de agua es estable.
    let mut sorted = raw.clone();
    sorted.sort_by(f32::total_cmp);
    let sea = sorted[((sorted.len() - 1) as f32 * config.sea_level.clamp(0.0, 1.0)) as usize];
    let (min, max) = (sorted[0], sorted[sorted.len() - 1]);
    raw.iter()
        .map(|&e| {
            if e < sea {
                (e - min) / (sea - min).max(f32::EPSILON) * SEA_LEVEL
            } else {
                // Curva para que haya más llanura que cumbre.
                let t = (e - sea) / (max - sea).max(f32::EPSILON);
                SEA_LEVEL + t.powf(1.15) * (1.0 - SEA_LEVEL)
            }
        })
        .collect()
}

fn normalize(v: (f32, f32)) -> (f32, f32) {
    let len = (v.0 * v.0 + v.1 * v.1).sqrt().max(f32::EPSILON);
    (v.0 / len, v.1 / len)
}

// ---------------------------------------------------------------- clima

/// 0 en el ecuador (centro del mapa), 1 en los polos (bordes superior e inferior).
pub(crate) fn latitude(y: u32, h: u32) -> f32 {
    ((y as f32 + 0.5) / h as f32 - 0.5).abs() * 2.0
}

fn temperature(w: u32, h: u32, elevation: &[f32], rng: &mut Rng) -> Vec<f32> {
    let noise = ValueNoise::new(rng, 64);
    let scale = 5.0 / w.max(h) as f32;
    (0..elevation.len())
        .map(|i| {
            let (x, y) = (i as u32 % w, i as u32 / w);
            let warmth = 1.0 - latitude(y, h);
            let altitude = (elevation[i] - SEA_LEVEL).max(0.0) / (1.0 - SEA_LEVEL);
            let local = (noise.fractal(x as f32 * scale, y as f32 * scale, 3) - 0.5) * 0.2;
            (warmth * 1.05 - altitude * 0.55 + local).clamp(0.0, 1.0)
        })
        .collect()
}

fn rainfall(w: u32, h: u32, elevation: &[f32], temperature: &[f32], rng: &mut Rng) -> Vec<f32> {
    let noise = ValueNoise::new(rng, 64);
    let scale = 6.0 / w.max(h) as f32;
    let mut rain = vec![0.0f32; elevation.len()];

    for y in 0..h {
        let lat = latitude(y, h);
        // Alisios y vientos polares soplan hacia el oeste; los del oeste, hacia el este.
        let eastward = (1.0 / 3.0..2.0 / 3.0).contains(&lat);
        // Ecuador y latitudes medias húmedos; trópicos y polos secos.
        let band = 0.7 + 0.3 * (lat * std::f32::consts::PI * 3.0).cos();
        let xs: Vec<u32> = if eastward {
            (0..w).collect()
        } else {
            (0..w).rev().collect()
        };

        // Dos pasadas: la segunda empieza con la humedad con la que acabó la primera,
        // como si el viento diera la vuelta al mundo.
        let mut moisture = 0.5f32;
        for pass in 0..2 {
            let mut previous = SEA_LEVEL;
            for &x in &xs {
                let i = (y * w + x) as usize;
                let e = elevation[i];
                if e < SEA_LEVEL {
                    let evaporation = 0.04 + 0.08 * temperature[i];
                    moisture = (moisture + evaporation).min(1.0);
                    previous = SEA_LEVEL;
                    continue;
                }
                let climb = (e - previous).max(0.0);
                let fraction = (0.03 + climb * 6.0).min(0.6);
                let fall = moisture * fraction;
                moisture -= fall;
                previous = e;
                if pass == 1 {
                    rain[i] = fall;
                }
            }
        }
        for x in 0..w {
            let i = (y * w + x) as usize;
            let local = noise.fractal(x as f32 * scale, y as f32 * scale, 4);
            rain[i] = rain[i] * band * (0.6 + 0.8 * local);
        }
    }

    // Cada fila se calcula por separado; suavizar entre filas evita franjas.
    let rain = blur(&rain, w as usize, h as usize, 2, 5);

    // Normalizar por rangos sobre tierra firme: el valor dice qué tan lluviosa es
    // la casilla comparada con el resto del mundo.
    let mut land: Vec<usize> = (0..rain.len())
        .filter(|&i| elevation[i] >= SEA_LEVEL)
        .collect();
    land.sort_by(|&a, &b| rain[a].total_cmp(&rain[b]));
    let mut out = vec![0.0; rain.len()];
    let n = land.len().max(1) as f32;
    for (rank, &i) in land.iter().enumerate() {
        out[i] = rank as f32 / n;
    }
    out
}

/// Desenfoque de caja separable con radios distintos en x e y.
fn blur(values: &[f32], w: usize, h: usize, rx: usize, ry: usize) -> Vec<f32> {
    let mut horizontal = vec![0.0; values.len()];
    for y in 0..h {
        for x in 0..w {
            let (lo, hi) = (x.saturating_sub(rx), (x + rx).min(w - 1));
            let sum: f32 = (lo..=hi).map(|k| values[y * w + k]).sum();
            horizontal[y * w + x] = sum / (hi - lo + 1) as f32;
        }
    }
    let mut out = vec![0.0; values.len()];
    for y in 0..h {
        let (lo, hi) = (y.saturating_sub(ry), (y + ry).min(h - 1));
        for x in 0..w {
            let sum: f32 = (lo..=hi).map(|k| horizontal[k * w + x]).sum();
            out[y * w + x] = sum / (hi - lo + 1) as f32;
        }
    }
    out
}

// ---------------------------------------------------------------- agua

/// Resultado del drenaje: hacia qué casilla vierte cada una.
struct Drainage {
    downstream: Vec<Option<usize>>,
}

/// Entrada de la cola de la inundación: nivel, desempate aleatorio y casilla.
#[derive(PartialEq)]
struct Lowest(f32, u64, usize);

impl Eq for Lowest {}

impl Ord for Lowest {
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .0
            .total_cmp(&self.0)
            .then(other.1.cmp(&self.1))
            .then(other.2.cmp(&self.2))
    }
}

impl PartialOrd for Lowest {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Inundación por prioridad (Barnes et al.): avanza desde el mar hacia arriba
/// siempre por la casilla más baja. Así cada casilla de tierra drena hacia la
/// casilla desde la que se alcanzó, y las depresiones quedan rellenas.
///
/// En zonas llanas (incluidas las depresiones rellenas) el orden entre casillas
/// del mismo nivel se decide al azar; si no, los ríos cruzarían las llanuras en
/// línea recta.
fn drain(map: &mut WorldMap, rng: &mut Rng) -> Drainage {
    let n = map.tiles.len();
    let mut filled: Vec<f32> = map.tiles.iter().map(|t| t.elevation).collect();
    let mut downstream = vec![None; n];
    let mut closed = vec![false; n];
    let mut order = Vec::with_capacity(n);
    let mut open = BinaryHeap::new();

    for i in 0..n {
        let pos = map.pos(i);
        let border = pos.x == 0 || pos.y == 0 || pos.x == map.width - 1 || pos.y == map.height - 1;
        if map.tiles[i].elevation < SEA_LEVEL || border {
            closed[i] = true;
            open.push(Lowest(filled[i], rng.gen(), i));
        }
    }
    while let Some(Lowest(level, _, i)) = open.pop() {
        order.push(i);
        for nb in map.neighbors8(map.pos(i)) {
            let j = map.index(nb);
            if !closed[j] {
                closed[j] = true;
                filled[j] = filled[j].max(level);
                downstream[j] = Some(i);
                open.push(Lowest(filled[j], rng.gen(), j));
            }
        }
    }

    // Caudal: cada casilla aporta su lluvia y la pasa aguas abajo. En el orden
    // inverso de la inundación, toda casilla se procesa antes que su destino.
    let mut flow: Vec<f32> = map
        .tiles
        .iter()
        .map(|t| {
            if t.elevation >= SEA_LEVEL {
                t.rainfall
            } else {
                0.0
            }
        })
        .collect();
    for &i in order.iter().rev() {
        if let Some(d) = downstream[i] {
            flow[d] += flow[i];
        }
    }
    let mean_rain = map
        .tiles
        .iter()
        .filter(|t| t.elevation >= SEA_LEVEL)
        .map(|t| t.rainfall)
        .sum::<f32>()
        / map
            .tiles
            .iter()
            .filter(|t| t.elevation >= SEA_LEVEL)
            .count()
            .max(1) as f32;
    for (tile, f) in map.tiles.iter_mut().zip(&flow) {
        // En unidades de "casillas de lluvia media".
        tile.flow = f / mean_rain.max(f32::EPSILON);
    }

    // Lagos: depresiones rellenas lo bastante hondas y extensas.
    let mut is_lake: Vec<bool> = (0..n)
        .map(|i| {
            map.tiles[i].elevation >= SEA_LEVEL && filled[i] - map.tiles[i].elevation > LAKE_DEPTH
        })
        .collect();
    let mut seen = vec![false; n];
    for start in 0..n {
        if !is_lake[start] || seen[start] {
            continue;
        }
        let mut component = vec![start];
        seen[start] = true;
        let mut k = 0;
        while k < component.len() {
            let i = component[k];
            k += 1;
            for nb in map.neighbors(map.pos(i)) {
                let j = map.index(nb);
                if is_lake[j] && !seen[j] {
                    seen[j] = true;
                    component.push(j);
                }
            }
        }
        if component.len() < MIN_LAKE_AREA {
            for i in component {
                is_lake[i] = false;
            }
        }
    }
    for (i, tile) in map.tiles.iter_mut().enumerate() {
        if is_lake[i] {
            tile.biome = Biome::Lake;
            tile.elevation = filled[i];
        }
    }

    Drainage { downstream }
}

/// Erosión fluvial sencilla: los ríos excavan valles proporcionales a su caudal.
fn erode(map: &mut WorldMap) {
    for tile in &mut map.tiles {
        if tile.elevation >= SEA_LEVEL && tile.biome != Biome::Lake {
            let cut = (tile.flow.max(1.0).ln() * 0.006).min(0.05);
            tile.elevation = (tile.elevation - cut).max(SEA_LEVEL + 0.002);
        }
    }
}

fn is_river_tile(tile: &Tile) -> bool {
    tile.flow >= RIVER_CATCHMENT && !tile.biome.is_water() && tile.elevation >= SEA_LEVEL
}

/// Convierte las casillas con mucho caudal en ríos con nombre. Cada río se
/// sigue aguas arriba desde su desembocadura por el brazo de mayor caudal; los
/// demás brazos son afluentes.
fn trace_rivers(map: &mut WorldMap, drainage: &Drainage, rng: &mut Rng) -> Vec<River> {
    let n = map.tiles.len();
    let mut upstream: Vec<Vec<usize>> = vec![Vec::new(); n];
    for i in 0..n {
        if let Some(d) = drainage.downstream[i] {
            if is_river_tile(&map.tiles[i]) {
                upstream[d].push(i);
            }
        }
    }

    // Desembocaduras: casillas de río cuyo destino no es río (mar, lago o borde).
    let mut heads: VecDeque<(usize, Option<RiverId>)> = (0..n)
        .filter(|&i| is_river_tile(&map.tiles[i]))
        .filter(|&i| drainage.downstream[i].is_none_or(|d| !is_river_tile(&map.tiles[d])))
        .map(|i| (i, None))
        .collect();

    let mut rivers = Vec::new();
    while let Some((mouth_tile, parent)) = heads.pop_front() {
        let id = RiverId(rivers.len() as u32);
        let mut path = vec![mouth_tile];
        let mut current = mouth_tile;
        loop {
            let mut branches = upstream[current].clone();
            branches.sort_by(|&a, &b| {
                map.tiles[b]
                    .flow
                    .total_cmp(&map.tiles[a].flow)
                    .then(a.cmp(&b))
            });
            let Some((&main, rest)) = branches.split_first() else {
                break;
            };
            for &tributary in rest {
                heads.push_back((tributary, Some(id)));
            }
            path.push(main);
            current = main;
        }
        path.reverse();
        for &i in &path {
            map.tiles[i].river = Some(id);
        }
        let mouth = drainage.downstream[mouth_tile].unwrap_or(mouth_tile);
        rivers.push(River {
            id,
            name: format!("Río {}", names::word(rng, 2, 3)),
            path: path.iter().map(|&i| map.pos(i)).collect(),
            mouth: map.pos(mouth),
            flows_into: parent,
            discharge: map.tiles[mouth_tile].flow,
        });
    }
    rivers
}

// ---------------------------------------------------------------- biomas

fn assign_biomes(map: &mut WorldMap) {
    for tile in &mut map.tiles {
        if tile.biome == Biome::Lake {
            continue;
        }
        let (e, t, r) = (tile.elevation, tile.temperature, tile.rainfall);
        tile.biome = if e < SEA_LEVEL {
            if e > SEA_LEVEL * 0.88 {
                Biome::Coast
            } else {
                Biome::Ocean
            }
        } else if t < 0.1 {
            Biome::Glacier
        } else if e > 0.8 {
            Biome::Mountain
        } else if e > 0.66 {
            Biome::Hills
        } else if t < 0.22 {
            Biome::Tundra
        } else if t < 0.38 {
            if r > 0.3 {
                Biome::Taiga
            } else {
                Biome::Tundra
            }
        } else if e < SEA_LEVEL + 0.02 && r > 0.8 {
            Biome::Swamp
        } else if r < 0.15 {
            Biome::Desert
        } else if r < 0.3 {
            Biome::Shrubland
        } else if t > 0.68 {
            if r > 0.6 {
                Biome::Jungle
            } else {
                Biome::Savanna
            }
        } else if r > 0.55 {
            Biome::Forest
        } else {
            Biome::Grassland
        };
    }
}

// ---------------------------------------------------------------- regiones

/// Agrupa casillas contiguas del mismo bioma en regiones con nombre.
fn build_regions(map: &mut WorldMap, rng: &mut Rng) -> Vec<Region> {
    let n = map.tiles.len();
    let mut component = vec![u32::MAX; n];
    let mut areas: Vec<u32> = Vec::new();
    for start in 0..n {
        if component[start] != u32::MAX {
            continue;
        }
        let id = areas.len() as u32;
        let biome = map.tiles[start].biome;
        let mut area = 0;
        let mut queue = VecDeque::from([start]);
        component[start] = id;
        while let Some(i) = queue.pop_front() {
            area += 1;
            for nb in map.neighbors(map.pos(i)) {
                let j = map.index(nb);
                if component[j] == u32::MAX && same_region_kind(map.tiles[j].biome, biome) {
                    component[j] = id;
                    queue.push_back(j);
                }
            }
        }
        areas.push(area);
    }

    // Fundir regiones diminutas con la vecina más grande, repitiendo hasta que
    // no quede ninguna (una diminuta puede rodear a otra diminuta).
    let mut parent: Vec<u32> = (0..areas.len() as u32).collect();
    loop {
        let mut changed = false;
        for i in 0..n {
            let c = find(&mut parent, component[i]);
            if areas[c as usize] >= MIN_REGION_AREA {
                continue;
            }
            let best = map
                .neighbors(map.pos(i))
                .map(|nb| find(&mut parent, component[map.index(nb)]))
                .filter(|&o| o != c)
                .max_by_key(|&o| areas[o as usize]);
            if let Some(o) = best {
                parent[c as usize] = o;
                areas[o as usize] += areas[c as usize];
                areas[c as usize] = 0;
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }

    // Asignar ids compactos y tomar el bioma más común como bioma de la región.
    let mut compact = vec![u32::MAX; areas.len()];
    let mut regions: Vec<Region> = Vec::new();
    let mut biome_counts: Vec<Vec<(Biome, u32)>> = Vec::new();
    for (i, &comp) in component.iter().enumerate() {
        let root = find(&mut parent, comp) as usize;
        if compact[root] == u32::MAX {
            compact[root] = regions.len() as u32;
            regions.push(Region {
                id: RegionId(compact[root]),
                name: String::new(),
                biome: map.tiles[i].biome,
                area: 0,
                center: map.pos(i),
            });
            biome_counts.push(Vec::new());
        }
        let r = compact[root] as usize;
        regions[r].area += 1;
        let biome = map.tiles[i].biome;
        match biome_counts[r].iter_mut().find(|(b, _)| *b == biome) {
            Some((_, c)) => *c += 1,
            None => biome_counts[r].push((biome, 1)),
        }
        map.tiles[i].region = Some(RegionId(r as u32));
    }

    // Centro de cada región: la casilla más alejada de sus fronteras, donde
    // cabe mejor la etiqueta (distancia calculada con una BFS desde los bordes).
    let mut depth = vec![u32::MAX; n];
    let mut queue = VecDeque::new();
    for (i, d) in depth.iter_mut().enumerate() {
        let region = map.tiles[i].region;
        if map
            .neighbors(map.pos(i))
            .any(|nb| map.tile(nb).unwrap().region != region)
        {
            *d = 0;
            queue.push_back(i);
        }
    }
    while let Some(i) = queue.pop_front() {
        let region = map.tiles[i].region;
        for nb in map.neighbors(map.pos(i)) {
            let j = map.index(nb);
            if depth[j] == u32::MAX && map.tiles[j].region == region {
                depth[j] = depth[i] + 1;
                queue.push_back(j);
            }
        }
    }
    let mut best = vec![(0u32, 0usize); regions.len()];
    for (i, tile) in map.tiles.iter().enumerate() {
        let r = tile.region.unwrap().index();
        let d = if depth[i] == u32::MAX {
            u32::MAX - 1
        } else {
            depth[i]
        };
        if d + 1 > best[r].0 {
            best[r] = (d + 1, i);
        }
    }

    for (r, region) in regions.iter_mut().enumerate() {
        region.biome = biome_counts[r].iter().max_by_key(|(_, c)| *c).unwrap().0;
        region.name = region_name(region.biome, &names::word(rng, 2, 3));
        region.center = map.pos(best[r].1);
    }
    regions
}

/// El mar abierto y la costa forman una sola región marina.
fn same_region_kind(a: Biome, b: Biome) -> bool {
    a == b || (a.is_sea() && b.is_sea())
}

fn find(parent: &mut [u32], mut x: u32) -> u32 {
    while parent[x as usize] != x {
        parent[x as usize] = parent[parent[x as usize] as usize];
        x = parent[x as usize];
    }
    x
}

fn region_name(biome: Biome, word: &str) -> String {
    let prefix = match biome {
        Biome::Ocean => "Mar de",
        Biome::Coast => "Bahía de",
        Biome::Lake => "Lago",
        Biome::Desert => "Desierto de",
        Biome::Shrubland => "Páramos de",
        Biome::Grassland => "Llanuras de",
        Biome::Savanna => "Sabana de",
        Biome::Forest => "Bosque de",
        Biome::Jungle => "Selva de",
        Biome::Swamp => "Ciénagas de",
        Biome::Taiga => "Pinares de",
        Biome::Tundra => "Tundra de",
        Biome::Hills => "Colinas de",
        Biome::Mountain => "Montes de",
        Biome::Glacier => "Hielos de",
    };
    format!("{prefix} {word}")
}
