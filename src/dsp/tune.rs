use crate::dsp::types::Sample;

const MAX_SEMITONES: Sample = 24.0; // +/- 2 octaves -- a generous but sane trim range

pub struct Tune {
    semitones: Sample,
}

impl Tune {
    pub fn new() -> Self {
        Self { semitones: 0.0 }
    }

    pub fn semitones(&self) -> Sample {
        self.semitones
    }

    pub fn set_semitones(&mut self, semitones: Sample) {
        self.semitones = semitones.clamp(-MAX_SEMITONES, MAX_SEMITONES);
    }

    //tuning parameter to alter sample pitch
    pub fn ratio(&self) -> Sample {
        2f32.powf(self.semitones / 12.0)
    }
}

impl Default for Tune {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPSILON: f32 = 1e-6;

    fn assert_approx_eq(actual: f32, expected: f32) {
        assert!(
            (actual - expected).abs() < EPSILON,
            "expected {}, got {}",
            expected,
            actual
        );
    }

    #[test]
    fn new_defaults_to_no_shift() {
        let tune = Tune::new();

        assert_approx_eq(tune.semitones(), 0.0);
        assert_approx_eq(tune.ratio(), 1.0);
    }

    #[test]
    fn one_octave_up_doubles_the_rate() {
        let mut tune = Tune::new();

        tune.set_semitones(12.0);

        assert_approx_eq(tune.ratio(), 2.0);
    }

    #[test]
    fn one_octave_down_halves_the_rate() {
        let mut tune = Tune::new();

        tune.set_semitones(-12.0);

        assert_approx_eq(tune.ratio(), 0.5);
    }

    #[test]
    fn set_semitones_clamps_to_max_range() {
        let mut tune = Tune::new();

        tune.set_semitones(100.0);
        assert_approx_eq(tune.semitones(), MAX_SEMITONES);

        tune.set_semitones(-100.0);
        assert_approx_eq(tune.semitones(), -MAX_SEMITONES);
    }
}
