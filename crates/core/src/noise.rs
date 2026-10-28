//! Ruido de valores fractal, suficiente para un terreno de prueba.
//! El paso 2 (geografía) lo reemplazará por algo más rico: placas, erosión y ríos.

use rand::Rng as _;

use crate::rng::Rng;

pub struct ValueNoise {
    size: usize,
    values: Vec<f32>,
}

impl ValueNoise {
    pub fn new(rng: &mut Rng, size: usize) -> Self {
        let values = (0..size * size).map(|_| rng.gen::<f32>()).collect();
        Self { size, values }
    }

    fn lattice(&self, x: i64, y: i64) -> f32 {
        let s = self.size as i64;
        self.values[(y.rem_euclid(s) * s + x.rem_euclid(s)) as usize]
    }

    /// Valor suavizado en [0, 1].
    pub fn sample(&self, x: f32, y: f32) -> f32 {
        let (x0, y0) = (x.floor(), y.floor());
        let (tx, ty) = (smooth(x - x0), smooth(y - y0));
        let (x0, y0) = (x0 as i64, y0 as i64);
        let a = lerp(self.lattice(x0, y0), self.lattice(x0 + 1, y0), tx);
        let b = lerp(self.lattice(x0, y0 + 1), self.lattice(x0 + 1, y0 + 1), tx);
        lerp(a, b, ty)
    }

    /// Suma de octavas, normalizada a [0, 1].
    pub fn fractal(&self, x: f32, y: f32, octaves: u32) -> f32 {
        let (mut total, mut amplitude, mut frequency, mut norm) = (0.0, 1.0, 1.0, 0.0);
        for _ in 0..octaves {
            total += self.sample(x * frequency, y * frequency) * amplitude;
            norm += amplitude;
            amplitude *= 0.5;
            frequency *= 2.0;
        }
        total / norm
    }
}

fn smooth(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}
