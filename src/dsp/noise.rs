use crate::dsp::types::Sample;

// xorshift32 must never be seeded with 0
const DEFAULT_SEED: u32 = 0x9E3779B9; // arbitrary odd, non-zero (golden ratio constant)

pub struct NoiseGenerator {
    state: u32,
}

impl NoiseGenerator {
    pub fn new(seed: u32) -> Self {
        Self {
            state: if seed == 0 { DEFAULT_SEED } else { seed },
        }
    }

    fn next_u32(&mut self) -> u32 {
        // xorshift32 (Marsaglia)
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.state = x;
        x
    }

    pub fn next_sample(&mut self) -> Sample {
        // normalize and center.
        let unipolar = self.next_u32() as f32 / u32::MAX as f32;
        unipolar * 2.0 - 1.0
    }

    pub fn reset(&mut self, seed: u32) {
        self.state = if seed == 0 { DEFAULT_SEED } else { seed };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_seed_falls_back_to_default() {
        let mut noise = NoiseGenerator::new(0);

        // must not get stuck producing the same value forever
        let a = noise.next_sample();
        let b = noise.next_sample();

        assert_ne!(a, b);
    }

    #[test]
    fn same_seed_produces_deterministic_sequence() {
        let mut a = NoiseGenerator::new(42);
        let mut b = NoiseGenerator::new(42);

        for _ in 0..100 {
            assert_eq!(a.next_sample(), b.next_sample());
        }
    }

    #[test]
    fn different_seeds_diverge() {
        let mut a = NoiseGenerator::new(1);
        let mut b = NoiseGenerator::new(2);

        let sequence_a: Vec<f32> = (0..10).map(|_| a.next_sample()).collect();
        let sequence_b: Vec<f32> = (0..10).map(|_| b.next_sample()).collect();

        assert_ne!(sequence_a, sequence_b);
    }

    #[test]
    fn samples_stay_within_bipolar_range() {
        let mut noise = NoiseGenerator::new(12_345);

        for _ in 0..100_000 {
            let sample = noise.next_sample();

            assert!(
                (-1.0..=1.0).contains(&sample),
                "sample {} outside range",
                sample
            );
        }
    }

    #[test]
    fn reset_restarts_the_sequence_from_the_given_seed() {
        let mut noise = NoiseGenerator::new(7);
        let first_run: Vec<f32> = (0..10).map(|_| noise.next_sample()).collect();

        noise.reset(7);
        let second_run: Vec<f32> = (0..10).map(|_| noise.next_sample()).collect();

        assert_eq!(first_run, second_run);
    }

    #[test]
    fn output_has_roughly_zero_mean_over_many_samples() {
        // a real distribution check, not just a range check -- catches a
        // bug where the bit-mixing skews heavily positive or negative
        // while still technically staying inside [-1, 1]
        let mut noise = NoiseGenerator::new(999);

        let sum: f32 = (0..200_000).map(|_| noise.next_sample()).sum();
        let mean = sum / 200_000.0;

        assert!(mean.abs() < 0.01, "mean {} too far from zero", mean);
    }
}
