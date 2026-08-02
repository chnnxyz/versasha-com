use crate::dsp::types::{DryWet, Sample, SampleRate, Time};

const MAX_DELAY_SECONDS: Time = 2.0;
const DEFAULT_TIME: Time = 0.3;
const DEFAULT_FEEDBACK: Sample = 0.3;
const DEFAULT_MIX: DryWet = 0.5;
const MAX_FEEDBACK: Sample = 0.98;

pub struct Delay {
    buffer: Vec<Sample>,
    write_index: usize,
    sample_rate: SampleRate,
    time: Time,
    feedback: Sample,
    mix: DryWet,
}

impl Delay {
    pub fn new(sample_rate: SampleRate) -> Self {
        let capacity = ((sample_rate * MAX_DELAY_SECONDS) as usize).max(1);

        Self {
            buffer: vec![0.0; capacity],
            write_index: 0,
            sample_rate,
            time: DEFAULT_TIME,
            feedback: DEFAULT_FEEDBACK,
            mix: DEFAULT_MIX,
        }
    }

    pub fn set_time(&mut self, time: Time) {
        self.time = time.clamp(0.0, MAX_DELAY_SECONDS);
    }

    pub fn time(&self) -> Time {
        self.time
    }

    pub fn set_feedback(&mut self, feedback: Sample) {
        self.feedback = feedback.clamp(0.0, MAX_FEEDBACK);
    }

    pub fn feedback(&self) -> Sample {
        self.feedback
    }

    pub fn set_mix(&mut self, mix: DryWet) {
        self.mix = mix.clamp(0.0, 1.0);
    }

    pub fn mix(&self) -> DryWet {
        self.mix
    }

    pub fn reset(&mut self) {
        self.buffer.iter_mut().for_each(|sample| *sample = 0.0);
        self.write_index = 0;
    }

    fn delay_samples(&self) -> usize {
        ((self.time * self.sample_rate) as usize).min(self.buffer.len() - 1)
    }

    pub fn process(&mut self, input: Sample) -> Sample {
        let delay_samples = self.delay_samples();

        let read_index = (self.write_index + self.buffer.len() - delay_samples) % self.buffer.len();

        let delayed = self.buffer[read_index];

        self.buffer[self.write_index] = input + delayed * self.feedback;

        self.write_index = (self.write_index + 1) % self.buffer.len();

        input * (1.0 - self.mix) + delayed * self.mix
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
    fn new_delay_is_silent() {
        let mut delay = Delay::new(48_000.0);

        assert_approx_eq(delay.process(0.0), 0.0);
    }

    #[test]
    fn dry_only_passes_input_through() {
        let mut delay = Delay::new(48_000.0);

        delay.set_mix(0.0);

        assert_approx_eq(delay.process(0.5), 0.5);
        assert_approx_eq(delay.process(-0.25), -0.25);
    }

    #[test]
    fn wet_only_repeats_input_after_delay_time() {
        let mut delay = Delay::new(48_000.0);

        delay.set_time(0.001);
        delay.set_feedback(0.0);
        delay.set_mix(1.0);

        let delay_samples = (48_000.0 * 0.001) as usize;

        delay.process(1.0);

        for _ in 0..delay_samples - 1 {
            delay.process(0.0);
        }

        assert_approx_eq(delay.process(0.0), 1.0);
    }

    #[test]
    fn feedback_produces_repeated_echoes() {
        let mut delay = Delay::new(48_000.0);

        delay.set_time(0.001);
        delay.set_feedback(0.5);
        delay.set_mix(1.0);

        let delay_samples = (48_000.0 * 0.001) as usize;

        delay.process(1.0);

        for _ in 0..delay_samples - 1 {
            delay.process(0.0);
        }

        let first_echo = delay.process(0.0);
        assert_approx_eq(first_echo, 1.0);

        for _ in 0..delay_samples - 1 {
            delay.process(0.0);
        }

        let second_echo = delay.process(0.0);
        assert_approx_eq(second_echo, 0.5);
    }

    #[test]
    fn feedback_is_clamped() {
        let mut delay = Delay::new(48_000.0);

        delay.set_feedback(5.0);

        assert_approx_eq(delay.feedback(), MAX_FEEDBACK);
    }

    #[test]
    fn mix_is_clamped() {
        let mut delay = Delay::new(48_000.0);

        delay.set_mix(2.0);
        assert_approx_eq(delay.mix(), 1.0);

        delay.set_mix(-1.0);
        assert_approx_eq(delay.mix(), 0.0);
    }

    #[test]
    fn time_is_clamped_to_max() {
        let mut delay = Delay::new(48_000.0);

        delay.set_time(10.0);

        assert_approx_eq(delay.time(), MAX_DELAY_SECONDS);
    }

    #[test]
    fn reset_clears_buffered_echoes() {
        let mut delay = Delay::new(48_000.0);

        delay.set_time(0.001);
        delay.set_feedback(0.0);
        delay.set_mix(1.0);

        let delay_samples = (48_000.0 * 0.001) as usize;

        delay.process(1.0);
        delay.reset();

        for _ in 0..delay_samples {
            assert_approx_eq(delay.process(0.0), 0.0);
        }
    }

    #[test]
    fn output_stays_finite_with_high_feedback() {
        // a feedback comb filter is expected to gain up sustained/correlated
        // input by up to 1/(1-feedback) before settling — that's not a bug,
        // but it must never diverge to NaN/Infinity
        let mut delay = Delay::new(48_000.0);

        delay.set_time(0.05);
        delay.set_feedback(MAX_FEEDBACK);
        delay.set_mix(0.5);

        for i in 0..100_000 {
            let input = if i % 200 < 100 { 1.0 } else { -1.0 };
            let sample = delay.process(input);

            assert!(sample.is_finite(), "non-finite output: {}", sample);
        }
    }
}
