use crate::dsp::types::SampleRate;

pub struct AudioSample {
    sample_rate: SampleRate,
    decoded_data: Vec<f32>,
}

impl AudioSample {
    pub fn new(rate: SampleRate, data: Vec<f32>) -> Self {
        Self {
            sample_rate: rate,
            decoded_data: data,
        }
    }

    pub fn sample_rate(&self) -> SampleRate {
        self.sample_rate
    }

    pub fn len(&self) -> usize {
        self.decoded_data.len()
    }

    pub fn sample_at(&self, position: f32) -> f32 {
        if position < 0.0 {
            return 0.0;
        }
        let index: usize = position as usize;
        self.decoded_data.get(index).copied().unwrap_or(0.0)
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
    fn new_stores_rate_and_data() {
        let sample = AudioSample::new(48_000.0, vec![0.1, 0.2, 0.3]);

        assert_approx_eq(sample.sample_rate(), 48_000.0);

        assert_eq!(sample.len(), 3);
    }

    #[test]
    fn len_reflects_decoded_data_length() {
        let sample = AudioSample::new(48_000.0, vec![0.0; 10]);

        assert_eq!(sample.len(), 10);
    }

    #[test]
    fn len_is_zero_for_empty_data() {
        let sample = AudioSample::new(48_000.0, Vec::new());

        assert_eq!(sample.len(), 0);
    }

    #[test]
    fn sample_at_reads_values_in_bounds() {
        let sample = AudioSample::new(48_000.0, vec![0.1, 0.2, 0.3]);

        assert_approx_eq(sample.sample_at(0.0), 0.1);

        assert_approx_eq(sample.sample_at(1.0), 0.2);

        assert_approx_eq(sample.sample_at(2.0), 0.3);
    }

    #[test]
    fn sample_at_truncates_fractional_position() {
        let sample = AudioSample::new(48_000.0, vec![0.1, 0.2, 0.3]);

        assert_approx_eq(sample.sample_at(1.9), 0.2);
    }

    #[test]
    fn sample_at_returns_silence_for_negative_position() {
        let sample = AudioSample::new(48_000.0, vec![0.1, 0.2, 0.3]);

        assert_approx_eq(sample.sample_at(-1.0), 0.0);
    }

    #[test]
    fn sample_at_returns_silence_exactly_at_end() {
        let sample = AudioSample::new(48_000.0, vec![0.1, 0.2, 0.3]);

        assert_approx_eq(sample.sample_at(3.0), 0.0);
    }

    #[test]
    fn sample_at_returns_silence_past_end() {
        let sample = AudioSample::new(48_000.0, vec![0.1, 0.2, 0.3]);

        assert_approx_eq(sample.sample_at(1000.0), 0.0);
    }

    #[test]
    fn sample_at_returns_silence_for_empty_buffer() {
        let sample = AudioSample::new(48_000.0, Vec::new());

        assert_approx_eq(sample.sample_at(0.0), 0.0);
    }
}
