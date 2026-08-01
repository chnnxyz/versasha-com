use crate::dsp::modulation::{Modulation, ModulationTarget};
use crate::dsp::phase_generator::PhaseGenerator;
use crate::dsp::types::DEFAULT_LFO_FREQ;

use super::types::{Frequency, Phase, Sample, SampleRate};
use super::waveform::Waveform;

pub struct Lfo {
    generator: PhaseGenerator,
    depth: Sample,
    target: ModulationTarget,
}

impl Lfo {
    pub fn new(rate: SampleRate) -> Self {
        let mut generator = PhaseGenerator::new(rate);

        generator.set_freq(DEFAULT_LFO_FREQ);

        Self {
            generator,
            depth: 1.0,
            target: ModulationTarget::Vibrato,
        }
    }

    // Setters

    pub fn set_freq(&mut self, freq: Frequency) {
        self.generator.set_freq(freq);
    }

    pub fn set_waveform(&mut self, waveform: Waveform) {
        self.generator.set_waveform(waveform);
    }

    pub fn set_depth(&mut self, depth: Sample) {
        self.depth = depth;
    }

    pub fn set_target(&mut self, target: ModulationTarget) {
        self.target = target;
    }

    // Getters

    pub fn freq(&self) -> Frequency {
        self.generator.freq()
    }

    pub fn waveform(&self) -> Waveform {
        self.generator.waveform()
    }

    pub fn depth(&self) -> Sample {
        self.depth
    }

    pub fn phase(&self) -> Phase {
        self.generator.phase()
    }

    pub fn target(&self) -> ModulationTarget {
        self.target
    }

    // Control

    pub fn reset(&mut self) {
        self.generator.reset();
    }

    // Output

    pub fn next_sample(&mut self) -> Sample {
        self.generator.next_sample() * self.depth
    }

    pub fn modulation(&mut self) -> Modulation {
        Modulation::new(self.target, self.next_sample())
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
    fn new_lfo_has_defaults() {
        let lfo = Lfo::new(48_000.0);

        assert_eq!(lfo.waveform(), Waveform::Sine);
        assert_eq!(lfo.freq(), DEFAULT_LFO_FREQ);
        assert_eq!(lfo.phase(), 0.0);
        assert_eq!(lfo.depth(), 1.0);
        assert_eq!(lfo.target(), ModulationTarget::Vibrato);
    }

    #[test]
    fn set_frequency_changes_frequency() {
        let mut lfo = Lfo::new(48_000.0);

        lfo.set_freq(2.0);

        assert_eq!(lfo.freq(), 2.0);
    }

    #[test]
    fn set_waveform_changes_waveform() {
        let mut lfo = Lfo::new(48_000.0);

        lfo.set_waveform(Waveform::Triangle);

        assert_eq!(lfo.waveform(), Waveform::Triangle);
    }

    #[test]
    fn set_depth_changes_depth() {
        let mut lfo = Lfo::new(48_000.0);

        lfo.set_depth(0.25);

        assert_eq!(lfo.depth(), 0.25);
    }

    #[test]
    fn set_target_changes_target() {
        let mut lfo = Lfo::new(48_000.0);

        lfo.set_target(ModulationTarget::Pitch);

        assert_eq!(lfo.target(), ModulationTarget::Pitch);
    }

    #[test]
    fn reset_returns_phase_to_zero() {
        let mut lfo = Lfo::new(48_000.0);

        lfo.set_freq(10.0);

        lfo.next_sample();

        assert_ne!(lfo.phase(), 0.0);

        lfo.reset();

        assert_eq!(lfo.phase(), 0.0);
    }

    #[test]
    fn reset_does_not_change_parameters() {
        let mut lfo = Lfo::new(48_000.0);

        lfo.set_freq(2.0);
        lfo.set_depth(0.5);
        lfo.set_waveform(Waveform::Square);
        lfo.set_target(ModulationTarget::Pitch);

        lfo.next_sample();

        lfo.reset();

        assert_eq!(lfo.freq(), 2.0);
        assert_eq!(lfo.depth(), 0.5);
        assert_eq!(lfo.waveform(), Waveform::Square);
        assert_eq!(lfo.target(), ModulationTarget::Pitch);
    }

    #[test]
    fn phase_advances_correctly() {
        let mut lfo = Lfo::new(4.0);

        lfo.set_freq(1.0);

        lfo.next_sample();

        assert_approx_eq(lfo.phase(), 0.25);

        lfo.next_sample();

        assert_approx_eq(lfo.phase(), 0.5);
    }

    #[test]
    fn phase_wraps_after_cycle() {
        let mut lfo = Lfo::new(4.0);

        lfo.set_freq(1.0);

        for _ in 0..4 {
            lfo.next_sample();
        }

        assert_approx_eq(lfo.phase(), 0.0);
    }

    #[test]
    fn depth_scales_output() {
        let mut lfo = Lfo::new(48_000.0);

        lfo.set_depth(0.5);

        lfo.generator.set_phase(0.25);

        let sample = lfo.next_sample();

        assert_approx_eq(sample, 0.5);
    }

    #[test]
    fn output_respects_depth_range() {
        let mut lfo = Lfo::new(48_000.0);

        lfo.set_depth(0.25);

        for _ in 0..100_000 {
            let sample = lfo.next_sample();

            assert!(
                (-0.25..=0.25).contains(&sample),
                "sample {} outside depth range",
                sample
            );
        }
    }

    #[test]
    fn output_stays_in_range_with_full_depth() {
        let mut lfo = Lfo::new(48_000.0);

        for _ in 0..100_000 {
            let sample = lfo.next_sample();

            assert!(
                (-1.0..=1.0).contains(&sample),
                "sample {} outside range",
                sample
            );
        }
    }

    #[test]
    fn modulation_contains_target_and_value() {
        let mut lfo = Lfo::new(48_000.0);

        lfo.set_target(ModulationTarget::Pitch);

        let modulation = lfo.modulation();

        assert_eq!(modulation.target(), ModulationTarget::Pitch);

        assert!((-1.0..=1.0).contains(&modulation.value()));
    }

    #[test]
    fn target_does_not_affect_generation() {
        let mut pitch_lfo = Lfo::new(48_000.0);
        let mut volume_lfo = Lfo::new(48_000.0);

        pitch_lfo.set_target(ModulationTarget::Pitch);
        volume_lfo.set_target(ModulationTarget::Volume);

        let a = pitch_lfo.next_sample();
        let b = volume_lfo.next_sample();

        assert_approx_eq(a, b);
    }
}
