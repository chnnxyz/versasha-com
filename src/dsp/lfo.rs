use super::types::{Frequency, Phase, Sample, SampleRate};
use super::waveform::Waveform;

pub struct Lfo {
    waveform: Waveform,
    phase: Phase,
    frequency: Frequency,
    sample_rate: SampleRate,

    depth: Sample,
}

impl Lfo {
    pub fn new(rate: SampleRate) -> Self {
        Self {
            waveform: Waveform::Sine,
            phase: 0.0,
            frequency: 5.0,
            sample_rate: rate,
            depth: 1.0,
        }
    }

    // Setters

    pub fn set_freq(&mut self, freq: Frequency) {
        self.frequency = freq;
    }

    pub fn set_waveform(&mut self, waveform: Waveform) {
        self.waveform = waveform;
    }

    pub fn set_depth(&mut self, depth: Sample) {
        self.depth = depth;
    }

    // Getters

    pub fn freq(&self) -> Frequency {
        self.frequency
    }

    pub fn waveform(&self) -> Waveform {
        self.waveform
    }

    pub fn depth(&self) -> Sample {
        self.depth
    }

    pub fn phase(&self) -> Phase {
        self.phase
    }

    pub fn reset(&mut self) {
        self.phase = 0.0;
    }

    pub fn next_sample(&mut self) -> Sample {
        let sample = match self.waveform {
            Waveform::Sine => self.sine(),
            Waveform::Square => self.square(),
            Waveform::Saw => self.saw(),
            Waveform::Triangle => self.triangle(),
        };

        self.advance_phase();

        sample * self.depth
    }

    // Helpers

    fn advance_phase(&mut self) {
        self.phase += self.frequency / self.sample_rate;

        if self.phase >= 1.0 {
            self.phase -= 1.0;
        }
    }

    fn sine(&self) -> Sample {
        (self.phase * std::f32::consts::TAU).sin()
    }

    fn square(&self) -> Sample {
        if self.phase < 0.5 { 1.0 } else { -1.0 }
    }

    fn saw(&self) -> Sample {
        (self.phase * 2.0) - 1.0
    }

    fn triangle(&self) -> Sample {
        if self.phase < 0.5 {
            (self.phase * 4.0) - 1.0
        } else {
            3.0 - (self.phase * 4.0)
        }
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
        assert_eq!(lfo.freq(), 5.0);
        assert_eq!(lfo.phase(), 0.0);
        assert_eq!(lfo.depth(), 1.0);
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
    fn phase_advances_correctly() {
        let mut lfo = Lfo::new(4.0);

        lfo.set_freq(1.0);

        lfo.next_sample();

        assert_approx_eq(lfo.phase(), 0.25);

        lfo.next_sample();

        assert_approx_eq(lfo.phase(), 0.5);
    }

    #[test]
    fn phase_wraps() {
        let mut lfo = Lfo::new(4.0);

        lfo.set_freq(1.0);

        for _ in 0..4 {
            lfo.next_sample();
        }

        assert_approx_eq(lfo.phase(), 0.0);
    }

    #[test]
    fn sine_wave_known_points() {
        let mut lfo = Lfo::new(48_000.0);

        lfo.phase = 0.0;
        assert_approx_eq(lfo.sine(), 0.0);

        lfo.phase = 0.25;
        assert_approx_eq(lfo.sine(), 1.0);

        lfo.phase = 0.5;
        assert_approx_eq(lfo.sine(), 0.0);

        lfo.phase = 0.75;
        assert_approx_eq(lfo.sine(), -1.0);
    }

    #[test]
    fn depth_scales_output() {
        let mut lfo = Lfo::new(48_000.0);

        lfo.phase = 0.25;
        lfo.set_depth(0.5);

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
}
