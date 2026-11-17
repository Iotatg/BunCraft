//! Small deterministic PRNG. BunCraft never uses thread randomness: a seed
//! must reproduce the same artwork on every machine, forever.

#[derive(Clone, Debug)]
pub struct Rng {
    state: u64,
}

impl Rng {
    pub fn new(seed: u64) -> Self {
        Self {
            state: seed.wrapping_add(0x9E37_79B9_7F4A_7C15),
        }
    }

    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// Uniform in [0, 1).
    pub fn f32(&mut self) -> f32 {
        (self.next_u64() >> 40) as f32 / (1u64 << 24) as f32
    }

    pub fn range(&mut self, a: f32, b: f32) -> f32 {
        a + (b - a) * self.f32()
    }

    pub fn range_i(&mut self, a: i32, b: i32) -> i32 {
        if b <= a {
            return a;
        }
        a + (self.next_u64() % ((b - a) as u64)) as i32
    }

    /// Standard normal (Box-Muller).
    pub fn gauss(&mut self) -> f32 {
        let u1 = self.f32().max(1e-7);
        let u2 = self.f32();
        (-2.0 * u1.ln()).sqrt() * (std::f32::consts::TAU * u2).cos()
    }

    pub fn chance(&mut self, p: f32) -> bool {
        self.f32() < p
    }

    pub fn pick<'a, T>(&mut self, items: &'a [T]) -> &'a T {
        let i = (self.next_u64() % items.len() as u64) as usize;
        &items[i]
    }
}

/// Stable hash-based noise in [0, 1), used for pencil grain.
pub fn noise2(x: f32, y: f32, seed: u64) -> f32 {
    let mut h = seed
        ^ (x.to_bits() as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15)
        ^ (y.to_bits() as u64).wrapping_mul(0xC2B2_AE3D_27D4_EB4F);
    h ^= h >> 33;
    h = h.wrapping_mul(0xFF51_AFD7_ED55_8CCD);
    h ^= h >> 33;
    (h >> 40) as f32 / (1u64 << 24) as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deterministic_sequence() {
        let mut a = Rng::new(1234);
        let mut b = Rng::new(1234);
        for _ in 0..100 {
            assert_eq!(a.next_u64(), b.next_u64());
        }
    }

    #[test]
    fn values_in_range() {
        let mut r = Rng::new(9);
        for _ in 0..1000 {
            let v = r.f32();
            assert!((0.0..1.0).contains(&v));
        }
    }

    #[test]
    fn noise_is_stable() {
        assert_eq!(noise2(1.5, 2.5, 7), noise2(1.5, 2.5, 7));
        assert!((0.0..1.0).contains(&noise2(1.5, 2.5, 7)));
    }
}
