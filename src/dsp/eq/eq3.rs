use crate::dsp::filters::{Filter, FilterType};
use crate::dsp::types::{Frequency, Sample, SampleRate};

// Simple 3-band EQ built from crossover filtering
pub struct Eq3 {
    low_filter: Filter,  // LowPass at low_freq -- passes everything below it
    high_filter: Filter, // HighPass at high_freq -- passes everything above it

    low_freq: Frequency,
    high_freq: Frequency,

    low_gain: f32,
    mid_gain: f32,
    high_gain: f32,
}

const LOW_CROSSOVER: Frequency = 200.0;
const HI_CROSSOVER: Frequency = 4000.0;

impl Eq3 {
    pub fn new(rate: SampleRate) -> Self {
        let mut low_filter = Filter::new(rate);
        low_filter.set_filter_type(FilterType::LowPass);
        low_filter.set_cutoff(LOW_CROSSOVER);
        let mut high_filter = Filter::new(rate);
        high_filter.set_filter_type(FilterType::HighPass);
        high_filter.set_cutoff(HI_CROSSOVER);

        Self {
            low_filter,
            high_filter,
            low_freq: LOW_CROSSOVER,
            high_freq: HI_CROSSOVER,
            low_gain: 1.0,
            mid_gain: 1.0,
            high_gain: 1.0,
        }
    }

    // getters
    pub fn low_gain(&self) -> f32 {
        self.low_gain
    }

    pub fn mid_gain(&self) -> f32 {
        self.mid_gain
    }

    pub fn high_gain(&self) -> f32 {
        self.high_gain
    }

    pub fn low_freq(&self) -> Frequency {
        self.low_freq
    }

    pub fn high_freq(&self) -> Frequency {
        self.high_freq
    }

    // setters
    pub fn set_low_gain(&mut self, gain: f32) {
        self.low_gain = gain.clamp(0.0, 2.0);
    }

    pub fn set_mid_gain(&mut self, gain: f32) {
        self.mid_gain = gain.clamp(0.0, 2.0);
    }

    pub fn set_high_gain(&mut self, gain: f32) {
        self.high_gain = gain.clamp(0.0, 2.0);
    }

    pub fn set_low_freq(&mut self, freq: Frequency) {
        self.low_freq = freq;
        self.low_filter.set_cutoff(freq);
    }

    pub fn set_high_freq(&mut self, freq: Frequency) {
        self.high_freq = freq;
        self.high_filter.set_cutoff(freq);
    }

    // mid_band is the remainder after subtracting the low/high bands out
    // of the input -- a cheap crossover instead of a dedicated bandpass.
    // Both filters always run, even at zero gain, so their state never
    // goes stale.
    pub fn process(&mut self, input: Sample) -> Sample {
        let low_band = self.low_filter.process(input);
        let high_band = self.high_filter.process(input);
        let mid_band = input - low_band - high_band;
        low_band * self.low_gain + mid_band * self.mid_gain + high_band * self.high_gain
    }

    pub fn reset(&mut self) {
        self.low_filter.reset();
        self.high_filter.reset();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPSILON: f32 = 1e-3;

    fn assert_approx_eq(actual: f32, expected: f32) {
        assert!(
            (actual - expected).abs() < EPSILON,
            "expected {}, got {}",
            expected,
            actual
        );
    }

    #[test]
    fn new_has_defaults() {
        let eq = Eq3::new(48_000.0);

        assert_approx_eq(eq.low_gain(), 1.0);
        assert_approx_eq(eq.mid_gain(), 1.0);
        assert_approx_eq(eq.high_gain(), 1.0);
        assert_approx_eq(eq.low_freq(), 200.0);
        assert_approx_eq(eq.high_freq(), 4000.0);
    }

    #[test]
    fn set_low_gain_clamps() {
        let mut eq = Eq3::new(48_000.0);

        eq.set_low_gain(5.0);
        assert_approx_eq(eq.low_gain(), 2.0);

        eq.set_low_gain(-1.0);
        assert_approx_eq(eq.low_gain(), 0.0);
    }

    #[test]
    fn set_mid_gain_clamps() {
        let mut eq = Eq3::new(48_000.0);

        eq.set_mid_gain(5.0);
        assert_approx_eq(eq.mid_gain(), 2.0);

        eq.set_mid_gain(-1.0);
        assert_approx_eq(eq.mid_gain(), 0.0);
    }

    #[test]
    fn set_high_gain_clamps() {
        let mut eq = Eq3::new(48_000.0);

        eq.set_high_gain(5.0);
        assert_approx_eq(eq.high_gain(), 2.0);

        eq.set_high_gain(-1.0);
        assert_approx_eq(eq.high_gain(), 0.0);
    }

    #[test]
    fn set_low_freq_updates_getter() {
        let mut eq = Eq3::new(48_000.0);

        eq.set_low_freq(300.0);

        assert_approx_eq(eq.low_freq(), 300.0);
    }

    #[test]
    fn set_high_freq_updates_getter() {
        let mut eq = Eq3::new(48_000.0);

        eq.set_high_freq(5000.0);

        assert_approx_eq(eq.high_freq(), 5000.0);
    }

    #[test]
    fn unity_gain_reconstructs_the_input() {
        // at low_gain = mid_gain = high_gain = 1.0 (the default), the
        // three bands are algebraically guaranteed to recombine to
        // exactly the original input, regardless of what the low/high
        // filters actually computed -- mid_band is defined as
        // `input - low_band - high_band`, so
        // low_band + mid_band + high_band == input always holds at unity
        // gain. This is a self-consistency check on the recombination
        // math itself, independent of the filters' actual cutoffs.
        let mut eq = Eq3::new(48_000.0);

        for i in 0..2000 {
            let input = (i as f32 * 0.01).sin();

            assert_approx_eq(eq.process(input), input);
        }
    }

    #[test]
    fn zero_gain_on_every_band_is_silent() {
        let mut eq = Eq3::new(48_000.0);

        eq.set_low_gain(0.0);
        eq.set_mid_gain(0.0);
        eq.set_high_gain(0.0);

        for i in 0..2000 {
            let input = (i as f32 * 0.01).sin();

            assert_approx_eq(eq.process(input), 0.0);
        }
    }

    #[test]
    fn reset_clears_filter_state() {
        let mut eq = Eq3::new(48_000.0);

        for _ in 0..100 {
            eq.process(1.0);
        }

        eq.reset();

        assert_approx_eq(eq.process(0.0), 0.0);
    }

    #[test]
    fn output_stays_finite_at_extreme_gains() {
        let mut eq = Eq3::new(48_000.0);

        eq.set_low_gain(2.0);
        eq.set_mid_gain(2.0);
        eq.set_high_gain(2.0);

        for i in 0..20_000 {
            let input = if i % 50 < 25 { 1.0 } else { -1.0 };
            let sample = eq.process(input);

            assert!(sample.is_finite(), "non-finite output: {sample}");
        }
    }
}
