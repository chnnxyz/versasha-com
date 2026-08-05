use crate::dsp::oscillators::phase_generator::PhaseGenerator;
use crate::dsp::oscillators::waveform::Waveform;
use crate::dsp::types::{DEFAULT_PHASE, Frequency, Phase, Sample, SampleRate};

pub mod phase_generator;
pub mod waveform;

pub struct Oscillator {
    generator: PhaseGenerator,
}

impl Oscillator {
    pub fn new(rate: SampleRate) -> Self {
        Self {
            generator: PhaseGenerator::new(rate),
        }
    }

    // Setters

    pub fn set_freq(&mut self, freq: Frequency) {
        self.generator.set_freq(freq);
    }

    pub fn set_waveform(&mut self, waveform: Waveform) {
        self.generator.set_waveform(waveform);
    }

    pub fn set_sample_rate(&mut self, sr: SampleRate) {
        self.generator.set_sample_rate(sr);
    }

    pub fn reset(&mut self) {
        self.generator.set_phase(DEFAULT_PHASE);
    }

    pub fn reset_phase(&mut self) {
        self.generator.set_phase(DEFAULT_PHASE);
    }
    // Getters

    pub fn freq(&self) -> Frequency {
        self.generator.freq()
    }

    pub fn waveform(&self) -> Waveform {
        self.generator.waveform()
    }

    pub fn phase(&self) -> Phase {
        self.generator.phase()
    }

    pub fn sample_rate(&self) -> SampleRate {
        self.generator.sample_rate()
    }

    // Audio generation

    pub fn next_sample(&mut self) -> Sample {
        self.generator.next_sample()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dsp::types::{DEFAULT_FREQ, DEFAULT_SAMPLE_RATE};

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
    fn new_initializes_defaults() {
        let osc = Oscillator::new(DEFAULT_SAMPLE_RATE);

        assert_eq!(osc.waveform(), Waveform::Sine);
        assert_eq!(osc.freq(), DEFAULT_FREQ);
        assert_eq!(osc.phase(), DEFAULT_PHASE);
        assert_eq!(osc.sample_rate(), DEFAULT_SAMPLE_RATE);
    }

    #[test]
    fn set_frequency_changes_frequency() {
        let mut osc = Oscillator::new(DEFAULT_SAMPLE_RATE);

        osc.set_freq(220.0);

        assert_eq!(osc.freq(), 220.0);
    }

    #[test]
    fn set_waveform_changes_waveform() {
        let mut osc = Oscillator::new(DEFAULT_SAMPLE_RATE);

        osc.set_waveform(Waveform::Saw);

        assert_eq!(osc.waveform(), Waveform::Saw);
    }

    #[test]
    fn set_sample_rate_changes_sample_rate() {
        let mut osc = Oscillator::new(DEFAULT_SAMPLE_RATE);

        osc.set_sample_rate(44_100.0);

        assert_eq!(osc.sample_rate(), 44_100.0);
    }

    #[test]
    fn reset_resets_phase_only() {
        let mut osc = Oscillator::new(DEFAULT_SAMPLE_RATE);

        osc.set_freq(1234.0);
        osc.set_waveform(Waveform::Triangle);

        osc.next_sample();
        osc.next_sample();

        osc.reset();

        assert_eq!(osc.phase(), DEFAULT_PHASE);
        assert_eq!(osc.freq(), 1234.0);
        assert_eq!(osc.waveform(), Waveform::Triangle);
    }

    #[test]
    fn phase_advances_correctly() {
        let mut osc = Oscillator::new(4.0);

        osc.set_freq(1.0);

        osc.next_sample();

        assert_approx_eq(osc.phase(), 0.25);

        osc.next_sample();

        assert_approx_eq(osc.phase(), 0.5);

        osc.next_sample();

        assert_approx_eq(osc.phase(), 0.75);
    }

    #[test]
    fn phase_wraps_after_one_cycle() {
        let mut osc = Oscillator::new(4.0);

        osc.set_freq(1.0);

        for _ in 0..4 {
            osc.next_sample();
        }

        assert_approx_eq(osc.phase(), 0.0);
    }

    #[test]
    fn produces_samples() {
        let mut osc = Oscillator::new(DEFAULT_SAMPLE_RATE);

        osc.set_freq(440.0);

        let sample = osc.next_sample();

        assert!(
            (-1.0..=1.0).contains(&sample),
            "sample {} outside range",
            sample
        );
    }

    #[test]
    fn samples_are_within_range() {
        let mut osc = Oscillator::new(DEFAULT_SAMPLE_RATE);

        let waveforms = [
            Waveform::Sine,
            Waveform::Square,
            Waveform::Saw,
            Waveform::Triangle,
        ];

        for waveform in waveforms {
            osc.reset();
            osc.set_waveform(waveform);

            for _ in 0..100_000 {
                let sample = osc.next_sample();

                assert!(
                    (-1.0..=1.0).contains(&sample),
                    "{:?} produced sample {}",
                    waveform,
                    sample
                );
            }
        }
    }
    #[test]
    fn reset_phase_returns_to_zero() {
        let mut osc = Oscillator::new(48_000.0);

        osc.set_freq(440.0);

        osc.next_sample();
        osc.next_sample();

        assert_ne!(osc.phase(), 0.0);

        osc.reset_phase();

        assert_eq!(osc.phase(), 0.0);
    }
}
