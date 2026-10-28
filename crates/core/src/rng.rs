//! Aleatoriedad reproducible.
//!
//! Todo el mundo sale de una sola semilla. Cada etapa de la generación pide su
//! propio generador con [`WorldRng::stream`], de modo que cambiar una etapa
//! (por ejemplo, cuántos personajes se crean) no altera el resultado de las demás.

use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

pub type Rng = ChaCha8Rng;

#[derive(Debug, Clone, Copy)]
pub struct WorldRng {
    seed: u64,
}

impl WorldRng {
    pub fn new(seed: u64) -> Self {
        Self { seed }
    }

    /// Generador independiente para una etapa con nombre.
    pub fn stream(&self, stage: &str) -> Rng {
        Rng::seed_from_u64(self.seed ^ fnv1a(stage.as_bytes()))
    }
}

/// Hash FNV-1a: estable entre plataformas y versiones de Rust, a diferencia de
/// `std::hash`, así que una semilla da siempre el mismo mundo.
pub fn fnv1a(bytes: &[u8]) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for b in bytes {
        hash ^= u64::from(*b);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

/// Convierte un texto (por ejemplo "Arda") en semilla numérica.
pub fn seed_from_text(text: &str) -> u64 {
    match text.trim().parse::<u64>() {
        Ok(n) => n,
        Err(_) => fnv1a(text.trim().as_bytes()),
    }
}
