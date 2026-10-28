//! Modo Leyendas: consultar el mundo en cualquier año de su historia y
//! convertir esa historia en una crónica legible.
//!
//! Como la simulación es determinista año a año, no hace falta guardar el
//! estado de cada año: [`Chronicle`] guarda una instantánea cada pocos años y
//! reconstruye los años intermedios simulando desde la instantánea anterior.

use std::fmt::Write as _;

use crate::generate::found;
use crate::model::*;
use crate::simulate;

/// Años entre instantáneas guardadas.
const INTERVAL: i32 = 10;

/// Máquina del tiempo de un mundo: devuelve su estado en cualquier año.
pub struct Chronicle {
    snapshots: Vec<World>,
}

impl Chronicle {
    /// Prepara la crónica de un mundo a partir de su configuración. Las
    /// instantáneas se calculan a medida que se piden años más tardíos.
    pub fn new(config: &WorldConfig) -> Self {
        Self {
            snapshots: vec![found(config)],
        }
    }

    pub fn start(&self) -> Year {
        self.snapshots[0].year
    }

    /// El mundo tal como era al terminar `year`.
    pub fn at(&mut self, year: Year) -> World {
        let year = year.max(self.start());
        let index = ((year - self.start()) / INTERVAL) as usize;
        while self.snapshots.len() <= index {
            let mut next = self.snapshots.last().unwrap().clone();
            simulate::run(&mut next, INTERVAL as u32);
            self.snapshots.push(next);
        }
        let mut world = self.snapshots[index].clone();
        let remaining = (year - world.year) as u32;
        simulate::run(&mut world, remaining);
        world
    }
}

impl World {
    /// La historia completa del mundo como texto Markdown: geografía, países,
    /// guerras y anales año por año. Pensada como punto de partida para escribir.
    pub fn chronicle_markdown(&self) -> String {
        let mut out = String::new();
        let first = self.events.first().map_or(self.year, |e| e.year);
        let _ = writeln!(out, "# Crónica de {}\n", self.name);
        let _ = writeln!(
            out,
            "_Desde el año {first} hasta el año {}. Semilla {}._\n",
            self.year, self.config.seed
        );

        let _ = writeln!(out, "## El mundo\n");
        let mut regions: Vec<&Region> = self.regions.iter().filter(|r| !r.biome.is_sea()).collect();
        regions.sort_by_key(|r| std::cmp::Reverse(r.area));
        let _ = writeln!(out, "Grandes regiones:\n");
        for r in regions.iter().take(10) {
            let _ = writeln!(
                out,
                "- **{}**: {}, {} casillas.",
                r.name,
                r.biome.name().to_lowercase(),
                r.area
            );
        }
        let mut rivers: Vec<&River> = self
            .rivers
            .iter()
            .filter(|r| r.flows_into.is_none())
            .collect();
        rivers.sort_by_key(|r| std::cmp::Reverse(r.path.len()));
        let _ = writeln!(out, "\nRíos principales:\n");
        for r in rivers.iter().take(6) {
            let tributaries = self
                .rivers
                .iter()
                .filter(|t| t.flows_into == Some(r.id))
                .count();
            let tributaries = match tributaries {
                0 => "sin afluentes".to_string(),
                1 => "1 afluente".to_string(),
                n => format!("{n} afluentes"),
            };
            let _ = writeln!(
                out,
                "- **{}**: {} casillas de largo, {tributaries}.",
                r.name,
                r.path.len()
            );
        }

        let _ = writeln!(out, "\n## Países\n");
        let mut countries: Vec<&Country> = self.countries.iter().collect();
        countries.sort_by_key(|c| (c.founded, c.id));
        for c in countries {
            let end = c.fell.map_or("hoy".to_string(), |y| format!("año {y}"));
            let _ = writeln!(
                out,
                "### {} ({}, año {} – {end})\n",
                c.name,
                c.government.name().to_lowercase(),
                c.founded
            );
            if c.exists() {
                let _ = writeln!(
                    out,
                    "Capital: {}. Territorio: {} casillas. Localidades: {}.\n",
                    self.locality(c.capital).name,
                    self.territory_size(c.id),
                    self.localities_of(c.id)
                        .filter(|l| l.ruined.is_none())
                        .count()
                );
            }
            let rulers = self.rulers_of(c.id);
            if !rulers.is_empty() {
                let list: Vec<String> = rulers
                    .iter()
                    .map(|(year, p)| format!("{} ({year})", self.person(*p).name))
                    .collect();
                let _ = writeln!(out, "Gobernantes: {}.\n", list.join(", "));
            }
        }

        if !self.wars.is_empty() {
            let _ = writeln!(out, "## Guerras\n");
            for w in &self.wars {
                let end = w.ended.map_or("en curso".to_string(), |y| y.to_string());
                let result = match w.winner {
                    Some(c) => format!("Vence {}.", self.country(c).name),
                    None if w.ended.is_some() => "Sin vencedor.".to_string(),
                    None => String::new(),
                };
                let conquests: Vec<&str> = w
                    .conquests
                    .iter()
                    .map(|&l| self.locality(l).name.as_str())
                    .collect();
                let _ = write!(
                    out,
                    "- **{}** ({} – {end}): {} contra {}. {result}",
                    w.name,
                    w.started,
                    self.country(w.attacker).name,
                    self.country(w.defender).name
                );
                if !conquests.is_empty() {
                    let _ = write!(out, " Conquistas: {}.", conquests.join(", "));
                }
                let _ = writeln!(out);
            }
            let _ = writeln!(out);
        }

        // Un párrafo por año (separados por una línea en blanco).
        let _ = write!(out, "## Anales");
        let mut current = None;
        for e in self.events.iter().filter(|e| self.is_major(e)) {
            if current != Some(e.year) {
                current = Some(e.year);
                let _ = write!(out, "\n\n**Año {}.**", e.year);
            }
            let _ = write!(out, " {}.", self.describe(e));
        }
        let _ = writeln!(out);
        out
    }
}
