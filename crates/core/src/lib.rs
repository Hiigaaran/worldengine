//! Motor de WorldEngine: genera mundos ficticios a partir de una semilla y
//! (en próximos pasos) simula su historia año a año.
//!
//! ```
//! use worldengine_core::{generate, WorldConfig};
//!
//! let world = generate(&WorldConfig { seed: 42, ..Default::default() });
//! assert!(!world.countries.is_empty());
//! ```

pub mod generate;
pub mod geography;
pub mod ids;
pub mod model;
mod names;
mod noise;
pub mod query;
pub mod rng;
pub mod simulate;

pub use generate::generate;
pub use ids::*;
pub use model::*;
pub use query::Subject;
