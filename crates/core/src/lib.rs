//! Motor de WorldEngine: genera mundos ficticios a partir de una semilla,
//! simula su historia año a año y permite consultarla como en el modo Leyendas
//! de Dwarf Fortress.
//!
//! ```
//! use worldengine_core::{generate, WorldConfig};
//!
//! let world = generate(&WorldConfig { seed: 42, ..Default::default() });
//! assert!(!world.countries.is_empty());
//! ```

pub mod climate;
pub mod generate;
pub mod geography;
pub mod ids;
pub mod legends;
pub mod model;
mod names;
mod noise;
pub mod query;
pub mod rng;
pub mod simulate;

pub use climate::{year_climate, YearClimate};
pub use generate::{found, generate};
pub use ids::*;
pub use legends::Chronicle;
pub use model::*;
pub use query::Subject;
