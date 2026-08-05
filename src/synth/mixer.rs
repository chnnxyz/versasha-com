use crate::dsp::types::Sample;

pub struct SynthMixer {
    ins: Vec<Sample>,
    gain: Sample,
}

impl SynthMixer {
    pub fn new() -> Self {
        Self {
            ins: Vec::new(),
            gain: 1.0,
        }
    }

    // Add a voice to mixer
    pub fn add(&mut self, sample: Sample) {
        self.ins.push(sample);
    }

    // Output signal
    pub fn output(&self) -> Sample {
        if self.ins.is_empty() {
            return 0.0;
        }
        let sum: Sample = self.ins.iter().sum();

        sum / self.ins.len() as Sample * self.gain
    }

    // Reset to empty
    pub fn reset(&mut self) {
        self.ins.clear();
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
    fn new_mixer_is_empty() {
        let mixer = SynthMixer::new();

        assert_approx_eq(mixer.output(), 0.0);
    }

    #[test]
    fn single_input_is_unchanged() {
        let mut mixer = SynthMixer::new();

        mixer.add(0.5);

        assert_approx_eq(mixer.output(), 0.5);
    }

    #[test]
    fn multiple_inputs_are_averaged() {
        let mut mixer = SynthMixer::new();

        mixer.add(1.0);
        mixer.add(-1.0);

        assert_approx_eq(mixer.output(), 0.0);
    }

    #[test]
    fn clear_removes_inputs() {
        let mut mixer = SynthMixer::new();

        mixer.add(1.0);
        mixer.reset();

        assert_approx_eq(mixer.output(), 0.0);
    }

    #[test]
    fn output_stays_in_range() {
        let mut mixer = SynthMixer::new();

        mixer.add(1.0);
        mixer.add(1.0);

        assert!((-1.0..=1.0).contains(&mixer.output()));
    }
}
